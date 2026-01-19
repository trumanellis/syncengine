//! Capability Types for Synchronicity Engine
//!
//! Capabilities are unforgeable tokens that grant specific authority.
//! Based on the object-capability model, they:
//! - Cannot be forged (you either have the reference or you don't)
//! - Can be delegated (authority flows through explicit transfer)
//! - Can be attenuated (grant restricted subset of your own authority)
//!
//! ## Capability Types
//!
//! - **ReadProfile**: Authority to fetch and view a peer's profile
//! - **RealmParticipant**: Authority to participate in a realm
//! - **ContactChannel**: Bilateral authority for direct messaging
//! - **BlobAccess**: Authority to fetch specific content-addressed data
//!
//! ## Example
//!
//! ```rust
//! use syncengine_core::types::capability::{Capability, CapabilityId};
//! use syncengine_core::types::RealmId;
//!
//! // A realm capability grants participation rights
//! let cap = Capability::RealmParticipant {
//!     realm_id: RealmId::new(),
//!     realm_key: [0u8; 32],
//!     granted_by: "did:sync:zAlice".to_string(),
//!     can_invite: true,
//! };
//!
//! // Each capability has a unique ID
//! let id = cap.id();
//! ```

use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::types::RealmId;

/// Unique identifier for a capability
///
/// Used for tracking, revocation, and capability references.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityId(pub [u8; 16]);

impl CapabilityId {
    /// Create a new random CapabilityId
    pub fn new() -> Self {
        let mut bytes = [0u8; 16];
        rand::rng().fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Create from raw bytes
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    /// Convert to base58 string for display
    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }

    /// Parse from base58 string
    pub fn from_base58(s: &str) -> Result<Self, bs58::decode::Error> {
        let bytes = bs58::decode(s).into_vec()?;
        if bytes.len() != 16 {
            return Err(bs58::decode::Error::BufferTooSmall);
        }
        let mut arr = [0u8; 16];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }
}

impl Default for CapabilityId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cap_{}", bs58::encode(&self.0[..8]).into_string())
    }
}

/// A Capability is an unforgeable token granting specific authority
///
/// Capabilities follow the object-capability model:
/// - **Unforgeable**: Cryptographically secured, cannot be guessed
/// - **Delegatable**: Can be transferred to others
/// - **Attenuatable**: Can grant restricted versions (e.g., read-only)
///
/// Each variant represents a different type of authority that can be
/// granted and tracked in the capability graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Capability {
    /// Authority to read a peer's profile
    ///
    /// This capability is typically granted implicitly during contact
    /// exchange, but can also be granted explicitly for profile sharing
    /// without full contact establishment.
    ReadProfile {
        /// Unique identifier for this capability instance
        id: CapabilityId,
        /// DID of the profile owner
        target_did: String,
        /// DID of who granted this capability
        granted_by: String,
        /// Optional expiration (Unix timestamp)
        expires_at: Option<i64>,
    },

    /// Authority to participate in a realm
    ///
    /// Grants the ability to:
    /// - Join the realm's gossip topic
    /// - Read and sync realm content (tasks/intentions)
    /// - Create new content in the realm
    /// - Optionally invite others (if `can_invite` is true)
    RealmParticipant {
        /// Unique identifier for this capability instance
        id: CapabilityId,
        /// The realm this capability grants access to
        realm_id: RealmId,
        /// Cryptographic key for realm encryption
        realm_key: [u8; 32],
        /// DID of who granted this capability
        granted_by: String,
        /// Whether this capability allows inviting others
        /// (attenuation: can be false even if granter has true)
        can_invite: bool,
    },

    /// Bilateral authority for direct messaging channel
    ///
    /// This capability is special: it's always created symmetrically
    /// during contact exchange. Both parties derive the same topic
    /// and key from their DIDs, making it a true bilateral contract.
    ContactChannel {
        /// Unique identifier for this capability instance
        id: CapabilityId,
        /// DID of the contact peer
        peer_did: String,
        /// Gossip topic for this contact channel
        topic: [u8; 32],
        /// Encryption key for messages
        key: [u8; 32],
        /// When the channel was established (Unix timestamp)
        established_at: i64,
    },

    /// Authority to fetch specific content-addressed data
    ///
    /// Used for accessing blobs (images, files) that are referenced
    /// by hash. The capability grants both read access and potentially
    /// caching/forwarding rights.
    BlobAccess {
        /// Unique identifier for this capability instance
        id: CapabilityId,
        /// Hash of the blob (content address)
        blob_hash: String,
        /// DID of who granted this capability
        granted_by: String,
    },
}

impl Capability {
    /// Get the unique ID of this capability
    pub fn id(&self) -> &CapabilityId {
        match self {
            Capability::ReadProfile { id, .. } => id,
            Capability::RealmParticipant { id, .. } => id,
            Capability::ContactChannel { id, .. } => id,
            Capability::BlobAccess { id, .. } => id,
        }
    }

    /// Get a short type name for this capability
    pub fn type_name(&self) -> &'static str {
        match self {
            Capability::ReadProfile { .. } => "ReadProfile",
            Capability::RealmParticipant { .. } => "RealmParticipant",
            Capability::ContactChannel { .. } => "ContactChannel",
            Capability::BlobAccess { .. } => "BlobAccess",
        }
    }

    /// Get the DID of the party who granted this capability (if applicable)
    pub fn granted_by(&self) -> Option<&str> {
        match self {
            Capability::ReadProfile { granted_by, .. } => Some(granted_by),
            Capability::RealmParticipant { granted_by, .. } => Some(granted_by),
            Capability::ContactChannel { .. } => None, // Bilateral, no single granter
            Capability::BlobAccess { granted_by, .. } => Some(granted_by),
        }
    }

    /// Check if this capability has expired
    pub fn is_expired(&self) -> bool {
        match self {
            Capability::ReadProfile { expires_at, .. } => {
                if let Some(exp) = expires_at {
                    chrono::Utc::now().timestamp() > *exp
                } else {
                    false
                }
            }
            // Other capability types don't currently support expiration
            _ => false,
        }
    }

    /// Get expiration timestamp if this capability can expire
    pub fn expires_at(&self) -> Option<i64> {
        match self {
            Capability::ReadProfile { expires_at, .. } => *expires_at,
            _ => None,
        }
    }

    /// Create a ReadProfile capability
    pub fn read_profile(target_did: String, granted_by: String, expires_at: Option<i64>) -> Self {
        Capability::ReadProfile {
            id: CapabilityId::new(),
            target_did,
            granted_by,
            expires_at,
        }
    }

    /// Create a RealmParticipant capability
    pub fn realm_participant(
        realm_id: RealmId,
        realm_key: [u8; 32],
        granted_by: String,
        can_invite: bool,
    ) -> Self {
        Capability::RealmParticipant {
            id: CapabilityId::new(),
            realm_id,
            realm_key,
            granted_by,
            can_invite,
        }
    }

    /// Create a ContactChannel capability
    pub fn contact_channel(peer_did: String, topic: [u8; 32], key: [u8; 32]) -> Self {
        Capability::ContactChannel {
            id: CapabilityId::new(),
            peer_did,
            topic,
            key,
            established_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Create a BlobAccess capability
    pub fn blob_access(blob_hash: String, granted_by: String) -> Self {
        Capability::BlobAccess {
            id: CapabilityId::new(),
            blob_hash,
            granted_by,
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Capability::ReadProfile { target_did, .. } => {
                write!(f, "ReadProfile({})", &target_did[..20.min(target_did.len())])
            }
            Capability::RealmParticipant { realm_id, can_invite, .. } => {
                let invite_str = if *can_invite { "+invite" } else { "" };
                write!(f, "RealmParticipant({}{})", realm_id, invite_str)
            }
            Capability::ContactChannel { peer_did, .. } => {
                write!(f, "ContactChannel({})", &peer_did[..20.min(peer_did.len())])
            }
            Capability::BlobAccess { blob_hash, .. } => {
                write!(f, "BlobAccess({})", &blob_hash[..16.min(blob_hash.len())])
            }
        }
    }
}

/// A capability with delegation metadata
///
/// Tracks the chain of authority when capabilities are delegated
/// from one party to another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegatedCapability {
    /// The actual capability being delegated
    pub capability: Capability,
    /// DID of who delegated this capability
    pub delegated_from: String,
    /// DID of who received this capability
    pub delegated_to: String,
    /// Restrictions applied during delegation
    pub restrictions: Vec<String>,
    /// Cryptographic proof of valid delegation
    pub proof: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_id_new() {
        let id1 = CapabilityId::new();
        let id2 = CapabilityId::new();
        // Should generate different IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_capability_id_base58_roundtrip() {
        let id = CapabilityId::new();
        let encoded = id.to_base58();
        let decoded = CapabilityId::from_base58(&encoded).expect("Failed to decode");
        assert_eq!(id, decoded);
    }

    #[test]
    fn test_capability_id_display() {
        let id = CapabilityId::new();
        let display = format!("{}", id);
        assert!(display.starts_with("cap_"));
    }

    #[test]
    fn test_read_profile_capability() {
        let cap = Capability::read_profile(
            "did:sync:zBob".to_string(),
            "did:sync:zAlice".to_string(),
            None,
        );

        assert_eq!(cap.type_name(), "ReadProfile");
        assert_eq!(cap.granted_by(), Some("did:sync:zAlice"));
        assert!(!cap.is_expired());
        assert!(cap.expires_at().is_none());
    }

    #[test]
    fn test_read_profile_expiration() {
        // Expired capability
        let expired = Capability::ReadProfile {
            id: CapabilityId::new(),
            target_did: "did:sync:zBob".to_string(),
            granted_by: "did:sync:zAlice".to_string(),
            expires_at: Some(chrono::Utc::now().timestamp() - 100),
        };
        assert!(expired.is_expired());

        // Valid capability
        let valid = Capability::ReadProfile {
            id: CapabilityId::new(),
            target_did: "did:sync:zBob".to_string(),
            granted_by: "did:sync:zAlice".to_string(),
            expires_at: Some(chrono::Utc::now().timestamp() + 3600),
        };
        assert!(!valid.is_expired());
    }

    #[test]
    fn test_realm_participant_capability() {
        let realm_id = RealmId::new();
        let realm_key = [42u8; 32];

        let cap = Capability::realm_participant(
            realm_id.clone(),
            realm_key,
            "did:sync:zAlice".to_string(),
            true,
        );

        assert_eq!(cap.type_name(), "RealmParticipant");
        assert_eq!(cap.granted_by(), Some("did:sync:zAlice"));

        if let Capability::RealmParticipant { realm_id: r, can_invite, .. } = &cap {
            assert_eq!(r, &realm_id);
            assert!(*can_invite);
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn test_contact_channel_capability() {
        let topic = [1u8; 32];
        let key = [2u8; 32];

        let cap = Capability::contact_channel("did:sync:zBob".to_string(), topic, key);

        assert_eq!(cap.type_name(), "ContactChannel");
        assert!(cap.granted_by().is_none()); // Bilateral, no single granter

        if let Capability::ContactChannel {
            peer_did,
            topic: t,
            key: k,
            established_at,
            ..
        } = &cap
        {
            assert_eq!(peer_did, "did:sync:zBob");
            assert_eq!(t, &topic);
            assert_eq!(k, &key);
            assert!(*established_at > 0);
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn test_blob_access_capability() {
        let cap = Capability::blob_access(
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
            "did:sync:zAlice".to_string(),
        );

        assert_eq!(cap.type_name(), "BlobAccess");
        assert_eq!(cap.granted_by(), Some("did:sync:zAlice"));
    }

    #[test]
    fn test_capability_display() {
        let cap = Capability::read_profile(
            "did:sync:zBobVeryLongIdentifier".to_string(),
            "did:sync:zAlice".to_string(),
            None,
        );
        let display = format!("{}", cap);
        assert!(display.contains("ReadProfile"));
        assert!(display.contains("did:sync:zBobVeryLon")); // Truncated
    }

    #[test]
    fn test_capability_serde_roundtrip() {
        let cap = Capability::realm_participant(
            RealmId::new(),
            [0u8; 32],
            "did:sync:zAlice".to_string(),
            true,
        );

        let json = serde_json::to_string(&cap).expect("Should serialize");
        let recovered: Capability = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(cap.id(), recovered.id());
        assert_eq!(cap.type_name(), recovered.type_name());
    }

    #[test]
    fn test_capability_equality() {
        let id = CapabilityId::new();
        let cap1 = Capability::ReadProfile {
            id: id.clone(),
            target_did: "did:sync:zBob".to_string(),
            granted_by: "did:sync:zAlice".to_string(),
            expires_at: None,
        };
        let cap2 = Capability::ReadProfile {
            id: id.clone(),
            target_did: "did:sync:zBob".to_string(),
            granted_by: "did:sync:zAlice".to_string(),
            expires_at: None,
        };

        assert_eq!(cap1, cap2);

        // Different IDs means different capabilities
        let cap3 = Capability::read_profile(
            "did:sync:zBob".to_string(),
            "did:sync:zAlice".to_string(),
            None,
        );
        assert_ne!(cap1, cap3);
    }

    #[test]
    fn test_delegated_capability() {
        let cap = Capability::realm_participant(
            RealmId::new(),
            [0u8; 32],
            "did:sync:zAlice".to_string(),
            true,
        );

        let delegated = DelegatedCapability {
            capability: cap,
            delegated_from: "did:sync:zAlice".to_string(),
            delegated_to: "did:sync:zBob".to_string(),
            restrictions: vec!["no-invite".to_string()],
            proof: vec![1, 2, 3, 4],
        };

        assert_eq!(delegated.delegated_from, "did:sync:zAlice");
        assert_eq!(delegated.delegated_to, "did:sync:zBob");
        assert!(delegated.restrictions.contains(&"no-invite".to_string()));
    }
}
