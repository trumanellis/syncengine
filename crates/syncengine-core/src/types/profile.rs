//! User Profile Type - Rich identity information for peers
//!
//! Stores profile data including display name, avatar, bio, and featured quests.
//! Also provides types for signed profiles and profile pinning for P2P redundancy.

use serde::{Deserialize, Serialize};

use crate::identity::{HybridPublicKey, HybridSignature};
use crate::types::RealmId;

/// User profile with rich identity information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    /// Peer's public key / node ID
    pub peer_id: String,

    /// Display name shown in UI
    pub display_name: String,

    /// Optional subtitle (e.g., role, tagline)
    pub subtitle: Option<String>,

    /// Custom short link (e.g., "love" → sync.local/love)
    pub profile_link: Option<String>,

    /// Iroh blob hash for avatar image
    pub avatar_blob_id: Option<String>,

    /// Markdown-formatted biography
    pub bio: String,

    /// Quest IDs featured in profile gallery
    pub top_quests: Vec<String>,

    /// Unix timestamp when profile was created
    pub created_at: i64,

    /// Unix timestamp of last update
    pub updated_at: i64,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            peer_id: String::new(),
            display_name: "Anonymous User".to_string(),
            subtitle: None,
            profile_link: None,
            avatar_blob_id: None,
            bio: String::new(),
            top_quests: vec![],
            created_at: 0,
            updated_at: 0,
        }
    }
}

impl UserProfile {
    /// Create a new profile with just peer ID and display name
    pub fn new(peer_id: String, display_name: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            peer_id,
            display_name,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    /// Update the profile's timestamp
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile() {
        let profile = UserProfile::default();
        assert_eq!(profile.display_name, "Anonymous User");
        assert!(profile.bio.is_empty());
        assert!(profile.avatar_blob_id.is_none());
    }

    #[test]
    fn test_new_profile() {
        let profile = UserProfile::new("test-peer-id".to_string(), "Love".to_string());
        assert_eq!(profile.peer_id, "test-peer-id");
        assert_eq!(profile.display_name, "Love");
        assert!(profile.created_at > 0);
        assert_eq!(profile.created_at, profile.updated_at);
    }

    #[test]
    fn test_touch_updates_timestamp() {
        let mut profile = UserProfile::new("test".to_string(), "Test".to_string());
        let original_time = profile.updated_at;

        // Sleep for >1 second since Unix timestamps have 1-second granularity
        std::thread::sleep(std::time::Duration::from_millis(1001));
        profile.touch();

        assert!(profile.updated_at > original_time);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Signed Profile - Cryptographically authenticated profile data
// ═══════════════════════════════════════════════════════════════════════════════

/// A signed profile that can be verified for authenticity.
///
/// Contains the profile data along with a hybrid signature (Ed25519 + ML-DSA-65)
/// that proves the profile was created by the owner of the corresponding keypair.
/// The public key is included to allow verification without needing to look up
/// the key separately.
///
/// # Security Properties
///
/// - **Authenticity**: Signature proves the profile came from the claimed identity
/// - **Integrity**: Any modification to the profile invalidates the signature
/// - **Quantum-resistant**: ML-DSA-65 provides post-quantum security
///
/// # Example
///
/// ```ignore
/// use syncengine_core::identity::HybridKeypair;
/// use syncengine_core::types::{UserProfile, SignedProfile};
///
/// let keypair = HybridKeypair::generate();
/// let profile = UserProfile::new("peer123".to_string(), "Love".to_string());
/// let signed = SignedProfile::sign(&profile, &keypair);
///
/// // Later, verify the profile
/// assert!(signed.verify());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedProfile {
    /// The profile data being signed
    pub profile: UserProfile,
    /// Hybrid signature over the serialized profile
    pub signature: HybridSignature,
    /// Public key for verification (allows standalone verification)
    pub public_key: HybridPublicKey,
}

impl PartialEq for SignedProfile {
    fn eq(&self, other: &Self) -> bool {
        // Compare profiles directly (UserProfile implements PartialEq)
        // and compare signature/public_key by their serialized bytes
        self.profile == other.profile
            && self.signature.to_bytes() == other.signature.to_bytes()
            && self.public_key.to_bytes() == other.public_key.to_bytes()
    }
}

impl Eq for SignedProfile {}

impl SignedProfile {
    /// Sign a profile with the given keypair.
    ///
    /// The profile is serialized using postcard before signing to ensure
    /// consistent byte representation across platforms.
    pub fn sign(profile: &UserProfile, keypair: &crate::identity::HybridKeypair) -> Self {
        let profile_bytes = postcard::to_allocvec(profile)
            .expect("Profile serialization should never fail");
        let signature = keypair.sign(&profile_bytes);
        let public_key = keypair.public_key();

        Self {
            profile: profile.clone(),
            signature,
            public_key,
        }
    }

    /// Verify that the signature is valid for this profile.
    ///
    /// Returns `true` if both the Ed25519 and ML-DSA-65 signatures verify.
    pub fn verify(&self) -> bool {
        let profile_bytes = match postcard::to_allocvec(&self.profile) {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };
        self.public_key.verify(&profile_bytes, &self.signature)
    }

    /// Get the DID (Decentralized Identifier) for this profile's signer.
    ///
    /// The DID is derived from the public key and can be used as a unique
    /// identifier for the profile owner.
    pub fn did(&self) -> crate::identity::Did {
        crate::identity::Did::from_public_key(&self.public_key)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Profile Pin - Tracking pinned profiles for P2P redundancy
// ═══════════════════════════════════════════════════════════════════════════════

/// Relationship that caused us to pin this profile.
///
/// We auto-pin profiles based on our relationship with the peer,
/// which helps determine eviction priority when storage limits are reached.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinRelationship {
    /// This is our own profile (always pinned, never evicted)
    Own,
    /// This is a direct contact we've mutually accepted
    Contact,
    /// This peer is a member of a realm we're in
    RealmMember {
        /// The realm where we share membership
        realm_id: RealmId,
    },
    /// Manually pinned by the user (explicit request)
    Manual,
}

impl std::fmt::Display for PinRelationship {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Own => write!(f, "Own"),
            Self::Contact => write!(f, "Contact"),
            Self::RealmMember { realm_id } => write!(f, "Realm Member ({})", realm_id),
            Self::Manual => write!(f, "Manual"),
        }
    }
}

/// A pinned profile with metadata about why and when we pinned it.
///
/// Profile pinning provides redundancy in the P2P network - if a peer goes offline,
/// their profile can still be served by peers who have pinned it. This is essential
/// for a censorship-resistant system where no central server stores profiles.
///
/// # Pinning Strategy
///
/// - **Self-pinning**: Each node pins and serves their own profile
/// - **Contact-pinning**: Nodes auto-pin profiles of accepted contacts
/// - **Realm-pinning**: Nodes pin profiles of realm members
/// - **Manual-pinning**: Users can explicitly pin any profile
///
/// # Storage Limits
///
/// To prevent unbounded storage growth:
/// - Max 100 pinned profiles (configurable)
/// - Max 5MB total avatar storage
/// - Eviction priority: Manual < RealmMember < Contact < Own
///
/// # Example
///
/// ```ignore
/// use syncengine_core::types::{ProfilePin, PinRelationship, SignedProfile};
///
/// // When accepting a contact
/// let pin = ProfilePin::new(
///     "did:sync:abc123".to_string(),
///     signed_profile,
///     PinRelationship::Contact,
/// );
/// storage.save_pinned_profile(&pin)?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilePin {
    /// DID of the profile owner (used as storage key)
    pub did: String,
    /// The signed profile data
    pub signed_profile: SignedProfile,
    /// Unix timestamp when we pinned this profile
    pub pinned_at: i64,
    /// Why we pinned this profile
    pub relationship: PinRelationship,
    /// BLAKE3 hash of the avatar blob (if avatar is pinned)
    pub avatar_hash: Option<[u8; 32]>,
    /// Unix timestamp of last profile update we received
    pub last_updated: i64,
}

impl ProfilePin {
    /// Create a new profile pin.
    pub fn new(did: String, signed_profile: SignedProfile, relationship: PinRelationship) -> Self {
        let now = chrono::Utc::now().timestamp();
        let avatar_hash = signed_profile
            .profile
            .avatar_blob_id
            .as_ref()
            .and_then(|hash_str| {
                // Try to decode hex hash
                hex::decode(hash_str)
                    .ok()
                    .and_then(|bytes| bytes.try_into().ok())
            });

        Self {
            did,
            signed_profile,
            pinned_at: now,
            relationship,
            avatar_hash,
            last_updated: now,
        }
    }

    /// Update the signed profile while preserving pin metadata.
    ///
    /// Returns `false` if the new profile's signature is invalid.
    pub fn update_profile(&mut self, new_signed_profile: SignedProfile) -> bool {
        if !new_signed_profile.verify() {
            return false;
        }

        // Update avatar hash if changed
        self.avatar_hash = new_signed_profile
            .profile
            .avatar_blob_id
            .as_ref()
            .and_then(|hash_str| {
                hex::decode(hash_str)
                    .ok()
                    .and_then(|bytes| bytes.try_into().ok())
            });

        self.signed_profile = new_signed_profile;
        self.last_updated = chrono::Utc::now().timestamp();
        true
    }

    /// Check if this pin should be kept over another based on relationship priority.
    ///
    /// Higher priority pins should be kept when storage limits are reached.
    /// Priority order (highest to lowest): Own > Contact > RealmMember > Manual
    pub fn priority(&self) -> u8 {
        match &self.relationship {
            PinRelationship::Own => 255,     // Never evict own profile
            PinRelationship::Contact => 100, // High priority
            PinRelationship::RealmMember { .. } => 50, // Medium priority
            PinRelationship::Manual => 25,   // Lowest priority (user can re-pin)
        }
    }

    /// Check if this is our own profile.
    pub fn is_own(&self) -> bool {
        matches!(self.relationship, PinRelationship::Own)
    }
}

#[cfg(test)]
mod signed_profile_tests {
    use super::*;
    use crate::identity::HybridKeypair;

    #[test]
    fn test_sign_and_verify_profile() {
        let keypair = HybridKeypair::generate();
        let profile = UserProfile::new("peer123".to_string(), "Love".to_string());

        let signed = SignedProfile::sign(&profile, &keypair);

        // Verify should succeed
        assert!(signed.verify());
        assert_eq!(signed.profile.display_name, "Love");
    }

    #[test]
    fn test_modified_profile_fails_verification() {
        let keypair = HybridKeypair::generate();
        let profile = UserProfile::new("peer123".to_string(), "Love".to_string());

        let mut signed = SignedProfile::sign(&profile, &keypair);

        // Modify the profile after signing
        signed.profile.display_name = "Eve".to_string();

        // Verify should fail
        assert!(!signed.verify());
    }

    #[test]
    fn test_wrong_key_fails_verification() {
        let keypair1 = HybridKeypair::generate();
        let keypair2 = HybridKeypair::generate();
        let profile = UserProfile::new("peer123".to_string(), "Love".to_string());

        // Sign with keypair1 but replace public key with keypair2's
        let mut signed = SignedProfile::sign(&profile, &keypair1);
        signed.public_key = keypair2.public_key();

        // Verify should fail because signature was made with different key
        assert!(!signed.verify());
    }

    #[test]
    fn test_signed_profile_serialization_roundtrip() {
        let keypair = HybridKeypair::generate();
        let profile = UserProfile::new("peer123".to_string(), "Love".to_string());
        let signed = SignedProfile::sign(&profile, &keypair);

        // Serialize and deserialize
        let bytes = postcard::to_allocvec(&signed).expect("Serialization failed");
        let recovered: SignedProfile =
            postcard::from_bytes(&bytes).expect("Deserialization failed");

        // Should still verify after roundtrip
        assert!(recovered.verify());
        assert_eq!(recovered.profile.display_name, "Love");
    }

    #[test]
    fn test_signed_profile_did() {
        let keypair = HybridKeypair::generate();
        let profile = UserProfile::new("peer123".to_string(), "Love".to_string());
        let signed = SignedProfile::sign(&profile, &keypair);

        let did = signed.did();
        assert!(did.to_string().starts_with("did:sync:"));
    }
}

#[cfg(test)]
mod profile_pin_tests {
    use super::*;
    use crate::identity::HybridKeypair;

    fn create_test_signed_profile(name: &str) -> SignedProfile {
        let keypair = HybridKeypair::generate();
        let profile = UserProfile::new("peer123".to_string(), name.to_string());
        SignedProfile::sign(&profile, &keypair)
    }

    #[test]
    fn test_profile_pin_creation() {
        let signed = create_test_signed_profile("Love");
        let pin = ProfilePin::new(
            "did:sync:test123".to_string(),
            signed,
            PinRelationship::Contact,
        );

        assert_eq!(pin.did, "did:sync:test123");
        assert!(pin.pinned_at > 0);
        assert_eq!(pin.relationship, PinRelationship::Contact);
    }

    #[test]
    fn test_profile_pin_priority() {
        let signed = create_test_signed_profile("Test");
        let realm_id = RealmId::new();

        let own_pin = ProfilePin::new("did1".to_string(), signed.clone(), PinRelationship::Own);
        let contact_pin =
            ProfilePin::new("did2".to_string(), signed.clone(), PinRelationship::Contact);
        let realm_pin = ProfilePin::new(
            "did3".to_string(),
            signed.clone(),
            PinRelationship::RealmMember { realm_id },
        );
        let manual_pin = ProfilePin::new("did4".to_string(), signed, PinRelationship::Manual);

        // Own has highest priority
        assert!(own_pin.priority() > contact_pin.priority());
        assert!(contact_pin.priority() > realm_pin.priority());
        assert!(realm_pin.priority() > manual_pin.priority());
    }

    #[test]
    fn test_profile_pin_update() {
        let keypair = HybridKeypair::generate();
        let profile1 = UserProfile::new("peer123".to_string(), "Love".to_string());
        let signed1 = SignedProfile::sign(&profile1, &keypair);

        let mut pin = ProfilePin::new(
            "did:sync:test".to_string(),
            signed1,
            PinRelationship::Contact,
        );

        let original_updated = pin.last_updated;

        // Wait a tiny bit to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Update with new profile
        let mut profile2 = UserProfile::new("peer123".to_string(), "Love Updated".to_string());
        profile2.touch();
        let signed2 = SignedProfile::sign(&profile2, &keypair);

        assert!(pin.update_profile(signed2));
        assert_eq!(pin.signed_profile.profile.display_name, "Love Updated");
        assert!(pin.last_updated >= original_updated);
    }

    #[test]
    fn test_profile_pin_rejects_invalid_signature() {
        let keypair = HybridKeypair::generate();
        let profile = UserProfile::new("peer123".to_string(), "Love".to_string());
        let signed = SignedProfile::sign(&profile, &keypair);

        let mut pin = ProfilePin::new(
            "did:sync:test".to_string(),
            signed,
            PinRelationship::Contact,
        );

        // Create an invalid signed profile (modified after signing)
        let keypair2 = HybridKeypair::generate();
        let profile2 = UserProfile::new("peer456".to_string(), "Joy".to_string());
        let mut invalid_signed = SignedProfile::sign(&profile2, &keypair2);
        invalid_signed.profile.display_name = "Tampered".to_string();

        // Update should fail
        assert!(!pin.update_profile(invalid_signed));
        // Original profile should be preserved
        assert_eq!(pin.signed_profile.profile.display_name, "Love");
    }

    #[test]
    fn test_pin_relationship_display() {
        assert_eq!(PinRelationship::Own.to_string(), "Own");
        assert_eq!(PinRelationship::Contact.to_string(), "Contact");
        assert_eq!(PinRelationship::Manual.to_string(), "Manual");

        let realm_id = RealmId::new();
        let realm_rel = PinRelationship::RealmMember { realm_id };
        assert!(realm_rel.to_string().starts_with("Realm Member"));
    }

    #[test]
    fn test_profile_pin_serialization_roundtrip() {
        let signed = create_test_signed_profile("Love");
        let pin = ProfilePin::new(
            "did:sync:test".to_string(),
            signed,
            PinRelationship::Contact,
        );

        // Serialize and deserialize
        let bytes = postcard::to_allocvec(&pin).expect("Serialization failed");
        let recovered: ProfilePin = postcard::from_bytes(&bytes).expect("Deserialization failed");

        assert_eq!(recovered.did, pin.did);
        assert_eq!(recovered.relationship, pin.relationship);
        assert!(recovered.signed_profile.verify());
    }
}
