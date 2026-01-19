//! Contact Exchange Types
//!
//! Provides types for peer-to-peer contact exchange system where users can
//! share profile invites, mutually accept connections, and maintain a
//! permanent contact list with auto-reconnection.

use rand::RngCore;
use serde::{Deserialize, Serialize};

use crate::invite::NodeAddrBytes;

/// Contact invite for peer-to-peer connection (Version 1 - LEGACY)
///
/// **This is the legacy format with embedded profile snapshot.**
/// Version 2 (HybridContactInvite) is preferred for new invites.
///
/// Contains all information needed to send a contact request to another peer.
/// Includes cryptographic proof of invite authenticity via hybrid signatures.
///
/// # Example
///
/// ```rust
/// use syncengine_core::types::contact::PeerContactInvite;
/// use syncengine_core::identity::{HybridKeypair, Did};
///
/// let keypair = HybridKeypair::generate();
/// let did = Did::from_public_key(&keypair.public_key());
/// // ... create and sign invite
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerContactInvite {
    /// Protocol version (should be 1)
    pub version: u8,
    /// Unique random ID (nonce) for this invite
    pub invite_id: [u8; 16],
    /// DID of the inviter
    pub inviter_did: String,
    /// HybridPublicKey bytes for signature verification
    pub inviter_pubkey: Vec<u8>,
    /// Minimal profile information for preview
    pub profile_snapshot: ProfileSnapshot,
    /// Network address for connection
    pub node_addr: NodeAddrBytes,
    /// Unix timestamp when invite was created
    pub created_at: i64,
    /// Unix timestamp when invite expires (max 7 days)
    pub expires_at: i64,
    /// HybridSignature over all fields
    pub signature: Vec<u8>,
}

/// Hybrid contact invite with on-demand profile fetching (Version 2)
///
/// This is the preferred format for new invites. It embeds only the display name
/// as a fallback, and attempts to fetch the full profile from the inviter's node
/// when they are online.
///
/// **Benefits over Version 1:**
/// - ~4.5x smaller (420-450 chars vs 2000+)
/// - QR code friendly
/// - Always-fresh profiles when inviter is online
/// - Graceful degradation when inviter is offline
///
/// # Example
///
/// ```rust
/// use syncengine_core::types::contact::HybridContactInvite;
/// use syncengine_core::identity::{HybridKeypair, Did};
///
/// let keypair = HybridKeypair::generate();
/// let did = Did::from_public_key(&keypair.public_key());
/// // ... create and sign invite
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridContactInvite {
    /// Protocol version (should be 2)
    pub version: u8,
    /// Unique random ID (nonce) for this invite
    pub invite_id: [u8; 16],
    /// DID of the inviter
    pub inviter_did: String,
    /// Network address for connection
    pub node_addr: NodeAddrBytes,
    /// Display name as fallback (when profile fetch fails)
    pub display_name: String,
    /// Unix timestamp when invite was created
    pub created_at: i64,
    /// Unix timestamp when invite expires (max 7 days)
    pub expires_at: i64,
    /// HybridSignature over all fields
    pub signature: Vec<u8>,
}

impl PeerContactInvite {
    /// Create a new invite ID (random 16 bytes)
    pub fn generate_invite_id() -> [u8; 16] {
        let mut id = [0u8; 16];
        rand::rng().fill_bytes(&mut id);
        id
    }

    /// Check if this invite has expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at
    }

    /// Get remaining validity duration
    pub fn time_until_expiry(&self) -> Option<i64> {
        let remaining = self.expires_at - chrono::Utc::now().timestamp();
        if remaining > 0 {
            Some(remaining)
        } else {
            None
        }
    }
}

impl HybridContactInvite {
    /// Create a new invite ID (random 16 bytes)
    ///
    /// This is a convenience method that wraps the same implementation
    /// as PeerContactInvite for consistency.
    pub fn generate_invite_id() -> [u8; 16] {
        let mut id = [0u8; 16];
        rand::rng().fill_bytes(&mut id);
        id
    }

    /// Check if this invite has expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at
    }

    /// Get remaining validity duration
    pub fn time_until_expiry(&self) -> Option<i64> {
        let remaining = self.expires_at - chrono::Utc::now().timestamp();
        if remaining > 0 {
            Some(remaining)
        } else {
            None
        }
    }
}

/// Profile information snapshot
///
/// Contains profile information for display during contact exchange.
/// Can represent either:
/// - Full profile (from PROFILE_ALPN fetch)
/// - Truncated profile (for v1 backward compatibility)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    /// Display name shown in UI
    pub display_name: String,
    /// Optional subtitle (e.g., role, tagline)
    pub subtitle: Option<String>,
    /// Iroh blob hash for avatar image
    pub avatar_blob_id: Option<String>,
    /// Biography text (full or excerpt depending on source)
    pub bio: String,
}

impl ProfileSnapshot {
    /// Create a bio excerpt (up to 200 chars) for invites and previews
    pub fn truncate_bio(bio: &str) -> String {
        if bio.len() <= 200 {
            bio.to_string()
        } else {
            let mut excerpt = bio.chars().take(197).collect::<String>();
            excerpt.push_str("...");
            excerpt
        }
    }
}

/// Contact record in user's contact list
///
/// Represents a mutually accepted peer connection with all information
/// needed for automatic reconnection and synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    /// DID of the contact
    pub peer_did: String,
    /// Iroh endpoint ID (32-byte public key)
    pub peer_endpoint_id: [u8; 32],
    /// Cached profile information
    pub profile: ProfileSnapshot,
    /// Last known network address
    pub node_addr: NodeAddrBytes,
    /// Dedicated 1:1 gossip topic for this contact
    pub contact_topic: [u8; 32],
    /// Shared encryption key for messages
    pub contact_key: [u8; 32],
    /// Unix timestamp of mutual acceptance
    pub accepted_at: i64,
    /// Last time this contact was seen online
    pub last_seen: u64,
    /// Current online/offline status
    pub status: ContactStatus,
    /// Priority for auto-connect on startup
    pub is_favorite: bool,
}

impl ContactInfo {
    /// Update last seen timestamp to now
    pub fn mark_seen(&mut self) {
        self.last_seen = chrono::Utc::now().timestamp() as u64;
    }

    /// Check if contact was recently online (within 5 minutes)
    pub fn is_recently_active(&self) -> bool {
        let now = chrono::Utc::now().timestamp() as u64;
        now.saturating_sub(self.last_seen) < 300 // 5 minutes
    }
}

/// Online/offline status of a contact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactStatus {
    /// Contact is currently connected and reachable
    Online,
    /// Contact is not currently connected
    Offline,
}

impl Default for ContactStatus {
    fn default() -> Self {
        Self::Offline
    }
}

impl std::fmt::Display for ContactStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Online => write!(f, "Online"),
            Self::Offline => write!(f, "Offline"),
        }
    }
}

/// Pending contact request state
///
/// Tracks the lifecycle of a contact request from initial invite
/// through mutual acceptance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingContact {
    /// Unique invite ID for tracking
    pub invite_id: [u8; 16],
    /// DID of the other peer
    pub peer_did: String,
    /// Profile information from invite (for display)
    pub profile: ProfileSnapshot,
    /// Full signed profile from contact protocol (for pinning)
    /// Only present when using the new protocol with SignedProfile exchange
    #[serde(default)]
    pub signed_profile: Option<crate::types::SignedProfile>,
    /// Network address for connection
    pub node_addr: NodeAddrBytes,
    /// Current state in acceptance workflow
    pub state: ContactState,
    /// Unix timestamp when request was created
    pub created_at: i64,
}

impl PendingContact {
    /// Check if this pending contact has been waiting for more than 7 days
    pub fn is_stale(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now - self.created_at > 7 * 24 * 60 * 60 // 7 days
    }
}

/// State machine for contact acceptance workflow
///
/// Represents the current state of a contact request in the
/// mutual acceptance handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactState {
    /// I sent an invite, waiting for them to accept
    OutgoingPending,
    /// They sent me a request, waiting for my decision
    IncomingPending,
    /// I accepted their request, waiting for their acceptance
    WaitingForMutual,
    /// Both parties have accepted, ready to finalize
    MutuallyAccepted,
    /// This peer has been blocked
    Blocked,
}

impl std::fmt::Display for ContactState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutgoingPending => write!(f, "Outgoing Pending"),
            Self::IncomingPending => write!(f, "Incoming Pending"),
            Self::WaitingForMutual => write!(f, "Waiting for Mutual"),
            Self::MutuallyAccepted => write!(f, "Mutually Accepted"),
            Self::Blocked => write!(f, "Blocked"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_invite_id() {
        let id1 = PeerContactInvite::generate_invite_id();
        let id2 = PeerContactInvite::generate_invite_id();
        // Should generate different IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_bio_truncation() {
        let short_bio = "This is a short bio.";
        assert_eq!(
            ProfileSnapshot::truncate_bio(short_bio),
            "This is a short bio."
        );

        let long_bio = "a".repeat(300);
        let truncated = ProfileSnapshot::truncate_bio(&long_bio);
        assert_eq!(truncated.len(), 200);
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn test_invite_expiry() {
        let mut invite = PeerContactInvite {
            version: 1,
            invite_id: [0u8; 16],
            inviter_did: "did:sync:test".to_string(),
            inviter_pubkey: vec![],
            profile_snapshot: ProfileSnapshot {
                display_name: "Test".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: NodeAddrBytes::new([0u8; 32]),
            created_at: chrono::Utc::now().timestamp(),
            expires_at: chrono::Utc::now().timestamp() - 100, // Already expired
            signature: vec![],
        };

        assert!(invite.is_expired());
        assert!(invite.time_until_expiry().is_none());

        // Make it valid
        invite.expires_at = chrono::Utc::now().timestamp() + 3600; // 1 hour
        assert!(!invite.is_expired());
        assert!(invite.time_until_expiry().is_some());
    }

    #[test]
    fn test_contact_status_display() {
        assert_eq!(ContactStatus::Online.to_string(), "Online");
        assert_eq!(ContactStatus::Offline.to_string(), "Offline");
    }

    #[test]
    fn test_contact_state_display() {
        assert_eq!(
            ContactState::OutgoingPending.to_string(),
            "Outgoing Pending"
        );
        assert_eq!(
            ContactState::IncomingPending.to_string(),
            "Incoming Pending"
        );
        assert_eq!(
            ContactState::WaitingForMutual.to_string(),
            "Waiting for Mutual"
        );
        assert_eq!(
            ContactState::MutuallyAccepted.to_string(),
            "Mutually Accepted"
        );
        assert_eq!(ContactState::Blocked.to_string(), "Blocked");
    }

    #[test]
    fn test_contact_recently_active() {
        let mut contact = ContactInfo {
            peer_did: "did:sync:test".to_string(),
            peer_endpoint_id: [0u8; 32],
            profile: ProfileSnapshot {
                display_name: "Test".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: NodeAddrBytes::new([0u8; 32]),
            contact_topic: [0u8; 32],
            contact_key: [0u8; 32],
            accepted_at: 0,
            last_seen: (chrono::Utc::now().timestamp() - 60) as u64, // 1 minute ago
            status: ContactStatus::Online,
            is_favorite: false,
        };

        assert!(contact.is_recently_active());

        // Make it old
        contact.last_seen = (chrono::Utc::now().timestamp() - 600) as u64; // 10 minutes ago
        assert!(!contact.is_recently_active());
    }

    #[test]
    fn test_pending_contact_staleness() {
        let mut pending = PendingContact {
            invite_id: [0u8; 16],
            peer_did: "did:sync:test".to_string(),
            profile: ProfileSnapshot {
                display_name: "Test".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            signed_profile: None,
            node_addr: NodeAddrBytes::new([0u8; 32]),
            state: ContactState::OutgoingPending,
            created_at: chrono::Utc::now().timestamp(),
        };

        assert!(!pending.is_stale());

        // Make it old (8 days ago)
        pending.created_at = chrono::Utc::now().timestamp() - (8 * 24 * 60 * 60);
        assert!(pending.is_stale());
    }
}
