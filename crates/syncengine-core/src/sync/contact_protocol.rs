//! Contact exchange protocol for peer-to-peer connections
//!
//! This module implements a simplified 2-step mutual acceptance handshake for
//! establishing permanent peer-to-peer contacts. Each contact pair derives a
//! unique gossip topic and encryption key locally from their DIDs.
//!
//! ## Protocol Overview
//!
//! The contact exchange protocol enables users to establish mutual connections:
//!
//! 1. **ContactRequest**: Requester sends request to inviter via QUIC stream
//! 2. **ContactAccept**: Inviter accepts and sends their profile back
//!    (or **ContactDecline** if rejecting)
//!
//! Both parties then derive the shared keys locally - no key transmission needed!
//!
//! ## Message Flow
//!
//! ```text
//! Inviter (Alice)                Requester (Bob)
//!   |                               |
//!   |--- Generate Invite ---------->|
//!   |                               |
//!   |<-- ContactRequest ------------|
//!   |    (profile, node_addr, sig)  |
//!   |                               |
//!   |--- ContactAccept ------------>|
//!   |    (profile, node_addr, sig)  |
//!   |                               |
//!   |    Both derive keys locally:  |
//!   |    BLAKE3(sorted_dids)        |
//!   |                               |
//!   |    Both subscribe to          |
//!   |    contact_topic              |
//! ```
//!
//! ## Deterministic Key Derivation
//!
//! Contact topics and encryption keys are derived deterministically from both DIDs:
//!
//! ```text
//! contact_topic = BLAKE3("sync-contact-topic" || sorted_did1 || sorted_did2)
//! contact_key   = BLAKE3("sync-contact-key" || sorted_did1 || sorted_did2)
//! ```
//!
//! DIDs are sorted lexicographically to ensure both peers derive the same values.
//! This eliminates the need to transmit keys over the network.

use serde::{Deserialize, Serialize};

use crate::types::contact::ProfileSnapshot;

/// ALPN protocol identifier for contact exchange
///
/// Contact messages use a separate ALPN from realm sync to allow
/// independent protocol evolution and resource allocation.
pub const CONTACT_ALPN: &[u8] = b"/sync/contact/1";

/// Contact protocol messages for mutual peer acceptance
///
/// The simplified protocol uses just 3 message types:
/// - `ContactRequest`: Requester → Inviter
/// - `ContactAccept`: Inviter → Requester (acceptance)
/// - `ContactDecline`: Inviter → Requester (rejection)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContactMessage {
    /// Step 1: Requester to inviter (via direct QUIC stream)
    ///
    /// The requester initiates contact by sending this message to the inviter's
    /// network address (obtained from the invite).
    ContactRequest {
        /// Unique invite ID from the original invite
        invite_id: [u8; 16],
        /// DID of the requester
        requester_did: String,
        /// Requester's public key (for signature verification)
        requester_pubkey: Vec<u8>, // HybridPublicKey serialized
        /// Requester's profile snapshot for preview
        requester_profile: ProfileSnapshot,
        /// Requester's network address for future connections
        requester_node_addr: Vec<u8>, // NodeAddrBytes serialized
        /// Signature over all above fields (HybridSignature)
        requester_signature: Vec<u8>,
    },

    /// Step 2a: Inviter to requester (acceptance)
    ///
    /// The inviter accepts the contact request and sends their profile back.
    /// Both parties then derive shared keys locally from their DIDs.
    ContactAccept {
        /// Invite ID from the request
        invite_id: [u8; 16],
        /// DID of the accepter (inviter)
        accepter_did: String,
        /// Accepter's public key (for signature verification)
        accepter_pubkey: Vec<u8>, // HybridPublicKey serialized
        /// Accepter's profile snapshot
        accepter_profile: ProfileSnapshot,
        /// Accepter's network address for future connections
        accepter_node_addr: Vec<u8>, // NodeAddrBytes serialized
        /// Signature over all above fields (HybridSignature)
        signature: Vec<u8>,
        // NOTE: contact_topic and contact_key are derived locally, not transmitted
    },

    /// Step 2b: Inviter to requester (rejection)
    ///
    /// The inviter declines the contact request.
    ContactDecline {
        /// Invite ID from the request
        invite_id: [u8; 16],
    },
}

impl ContactMessage {
    /// Encode message to bytes using postcard
    pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    /// Decode message from bytes using postcard
    pub fn decode(data: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(data)
    }
}

/// Derive 1:1 contact topic from two DIDs (deterministic)
///
/// The topic is derived by hashing both DIDs in sorted order. This ensures
/// both peers independently derive the same topic ID without coordination.
///
/// # Algorithm
///
/// ```text
/// 1. Sort DIDs lexicographically
/// 2. Hash: BLAKE3("sync-contact-topic" || did_a || did_b)
/// 3. Return first 32 bytes as topic ID
/// ```
///
/// # Example
///
/// ```ignore
/// let topic = derive_contact_topic("did:alice", "did:bob");
/// // Both Alice and Bob will derive the same topic
/// ```
pub fn derive_contact_topic(did1: &str, did2: &str) -> [u8; 32] {
    // Sort DIDs lexicographically for deterministic order
    let (a, b) = if did1 < did2 {
        (did1, did2)
    } else {
        (did2, did1)
    };

    // BLAKE3("sync-contact-topic" || a || b)
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"sync-contact-topic");
    hasher.update(a.as_bytes());
    hasher.update(b.as_bytes());
    *hasher.finalize().as_bytes()
}

/// Derive shared encryption key from two DIDs (deterministic)
///
/// The encryption key is derived similarly to the topic, but with a different
/// domain separator to ensure the key and topic are independent.
///
/// # Algorithm
///
/// ```text
/// 1. Sort DIDs lexicographically
/// 2. Hash: BLAKE3("sync-contact-key" || did_a || did_b)
/// 3. Return first 32 bytes as encryption key
/// ```
///
/// # Example
///
/// ```ignore
/// let key = derive_contact_key("did:alice", "did:bob");
/// // Both Alice and Bob will derive the same key
/// ```
pub fn derive_contact_key(did1: &str, did2: &str) -> [u8; 32] {
    // Sort DIDs lexicographically for deterministic order
    let (a, b) = if did1 < did2 {
        (did1, did2)
    } else {
        (did2, did1)
    };

    // BLAKE3("sync-contact-key" || a || b)
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"sync-contact-key");
    hasher.update(a.as_bytes());
    hasher.update(b.as_bytes());
    *hasher.finalize().as_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_contact_topic_deterministic() {
        let alice_did = "did:key:alice123";
        let bob_did = "did:key:bob456";

        // Derive topic in both orders
        let topic_ab = derive_contact_topic(alice_did, bob_did);
        let topic_ba = derive_contact_topic(bob_did, alice_did);

        // Should be identical regardless of order
        assert_eq!(
            topic_ab, topic_ba,
            "Topic derivation must be order-independent"
        );

        // Should be exactly 32 bytes
        assert_eq!(topic_ab.len(), 32);
    }

    #[test]
    fn test_derive_contact_topic_different_peers() {
        let alice_did = "did:key:alice123";
        let bob_did = "did:key:bob456";
        let charlie_did = "did:key:charlie789";

        let topic_alice_bob = derive_contact_topic(alice_did, bob_did);
        let topic_alice_charlie = derive_contact_topic(alice_did, charlie_did);

        // Different peer pairs should produce different topics
        assert_ne!(
            topic_alice_bob, topic_alice_charlie,
            "Different peer pairs must produce unique topics"
        );
    }

    #[test]
    fn test_derive_contact_key_deterministic() {
        let alice_did = "did:key:alice123";
        let bob_did = "did:key:bob456";

        // Derive key in both orders
        let key_ab = derive_contact_key(alice_did, bob_did);
        let key_ba = derive_contact_key(bob_did, alice_did);

        // Should be identical regardless of order
        assert_eq!(key_ab, key_ba, "Key derivation must be order-independent");

        // Should be exactly 32 bytes
        assert_eq!(key_ab.len(), 32);
    }

    #[test]
    fn test_derive_contact_key_different_peers() {
        let alice_did = "did:key:alice123";
        let bob_did = "did:key:bob456";
        let charlie_did = "did:key:charlie789";

        let key_alice_bob = derive_contact_key(alice_did, bob_did);
        let key_alice_charlie = derive_contact_key(alice_did, charlie_did);

        // Different peer pairs should produce different keys
        assert_ne!(
            key_alice_bob, key_alice_charlie,
            "Different peer pairs must produce unique keys"
        );
    }

    #[test]
    fn test_topic_and_key_are_different() {
        let alice_did = "did:key:alice123";
        let bob_did = "did:key:bob456";

        let topic = derive_contact_topic(alice_did, bob_did);
        let key = derive_contact_key(alice_did, bob_did);

        // Topic and key should be different (different domain separators)
        assert_ne!(
            topic, key,
            "Topic and key must be independent (use different domain separators)"
        );
    }

    #[test]
    fn test_contact_request_serialization() {
        let profile = ProfileSnapshot {
            display_name: "Alice".to_string(),
            subtitle: Some("Software Engineer".to_string()),
            avatar_blob_id: Some("QmXXXXX".to_string()),
            bio: "Building the future".to_string(),
        };

        let msg = ContactMessage::ContactRequest {
            invite_id: [42u8; 16],
            requester_did: "did:key:alice123".to_string(),
            requester_pubkey: vec![9, 10, 11, 12],
            requester_profile: profile.clone(),
            requester_node_addr: vec![1, 2, 3, 4],
            requester_signature: vec![5, 6, 7, 8],
        };

        // Encode
        let encoded = msg.encode().expect("Failed to encode");

        // Decode
        let decoded = ContactMessage::decode(&encoded).expect("Failed to decode");

        // Verify round-trip
        assert_eq!(msg, decoded, "Message should round-trip correctly");
    }

    #[test]
    fn test_contact_accept_serialization() {
        let profile = ProfileSnapshot {
            display_name: "Alice".to_string(),
            subtitle: Some("Engineer".to_string()),
            avatar_blob_id: None,
            bio: "Hello!".to_string(),
        };

        let msg = ContactMessage::ContactAccept {
            invite_id: [99u8; 16],
            accepter_did: "did:sync:zAlice123".to_string(),
            accepter_pubkey: vec![1, 2, 3, 4],
            accepter_profile: profile.clone(),
            accepter_node_addr: vec![5, 6, 7, 8],
            signature: vec![9, 10, 11, 12],
        };

        let encoded = msg.encode().expect("Failed to encode");
        let decoded = ContactMessage::decode(&encoded).expect("Failed to decode");

        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_contact_decline_serialization() {
        let msg = ContactMessage::ContactDecline {
            invite_id: [88u8; 16],
        };

        let encoded = msg.encode().expect("Failed to encode");
        let decoded = ContactMessage::decode(&encoded).expect("Failed to decode");

        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_profile_snapshot_serialization() {
        let profile = ProfileSnapshot {
            display_name: "Test User".to_string(),
            subtitle: Some("Testing".to_string()),
            avatar_blob_id: Some("QmTest".to_string()),
            bio: "A test bio excerpt that is exactly 200 characters long. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad min.".to_string(),
        };

        let encoded = postcard::to_allocvec(&profile).expect("Failed to encode");
        let decoded: ProfileSnapshot = postcard::from_bytes(&encoded).expect("Failed to decode");

        assert_eq!(profile, decoded);
    }

    #[test]
    fn test_empty_profile_serialization() {
        let profile = ProfileSnapshot {
            display_name: String::new(),
            subtitle: None,
            avatar_blob_id: None,
            bio: String::new(),
        };

        let encoded = postcard::to_allocvec(&profile).expect("Failed to encode");
        let decoded: ProfileSnapshot = postcard::from_bytes(&encoded).expect("Failed to decode");

        assert_eq!(profile, decoded);
    }

    #[test]
    fn test_deterministic_derivation_with_same_dids() {
        let did = "did:key:same123";

        // Even with the same DID twice, should produce deterministic results
        let topic1 = derive_contact_topic(did, did);
        let topic2 = derive_contact_topic(did, did);
        assert_eq!(topic1, topic2);

        let key1 = derive_contact_key(did, did);
        let key2 = derive_contact_key(did, did);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_with_empty_dids() {
        // Edge case: empty DIDs should still produce valid hashes
        let topic = derive_contact_topic("", "");
        assert_eq!(topic.len(), 32);

        let key = derive_contact_key("", "");
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_derive_with_one_empty_did() {
        let did = "did:key:alice123";

        let topic1 = derive_contact_topic(did, "");
        let topic2 = derive_contact_topic("", did);
        assert_eq!(topic1, topic2, "Should be order-independent");

        let key1 = derive_contact_key(did, "");
        let key2 = derive_contact_key("", did);
        assert_eq!(key1, key2, "Should be order-independent");
    }

    #[test]
    fn test_contact_alpn_constant() {
        assert_eq!(CONTACT_ALPN, b"/sync/contact/1");
        assert!(!CONTACT_ALPN.is_empty());
    }

    #[test]
    fn test_contact_message_variants_are_distinct() {
        let profile = ProfileSnapshot {
            display_name: "Test".to_string(),
            subtitle: None,
            avatar_blob_id: None,
            bio: "Test".to_string(),
        };

        let request = ContactMessage::ContactRequest {
            invite_id: [1u8; 16],
            requester_did: "did:test".to_string(),
            requester_pubkey: vec![],
            requester_profile: profile.clone(),
            requester_node_addr: vec![],
            requester_signature: vec![],
        };

        let accept = ContactMessage::ContactAccept {
            invite_id: [1u8; 16],
            accepter_did: "did:test".to_string(),
            accepter_pubkey: vec![],
            accepter_profile: profile,
            accepter_node_addr: vec![],
            signature: vec![],
        };

        let decline = ContactMessage::ContactDecline {
            invite_id: [1u8; 16],
        };

        // Ensure all variants encode to different bytes
        let request_bytes = request.encode().unwrap();
        let accept_bytes = accept.encode().unwrap();
        let decline_bytes = decline.encode().unwrap();

        assert_ne!(request_bytes, accept_bytes);
        assert_ne!(request_bytes, decline_bytes);
        assert_ne!(accept_bytes, decline_bytes);
    }
}
