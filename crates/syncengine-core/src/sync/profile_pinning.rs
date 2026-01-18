//! Profile Pinning Protocol - Gossip-based profile synchronization
//!
//! Implements a P2P profile pinning system where:
//! - Nodes announce their profile updates to a global topic
//! - Interested peers (contacts, realm members) receive and pin profiles
//! - Pinned profiles can be served to others for redundancy
//!
//! # Protocol Messages
//!
//! - `Announce`: Broadcast when profile is updated (includes avatar ticket)
//! - `Request`: Ask for a specific profile by DID
//! - `Response`: Reply to a request with the signed profile
//!
//! # Topic Structure
//!
//! All profile messages use a single global topic derived from a fixed seed.
//! This allows any node to discover profile announcements from the network.

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::identity::Did;
use crate::types::SignedProfile;

/// The seed used to derive the global profile topic ID
const PROFILE_TOPIC_SEED: &[u8] = b"syncengine:profiles:v1";

/// The domain separator for per-peer profile topics
const PROFILE_TOPIC_DOMAIN: &[u8] = b"sync-profile";

/// Get the global profile topic ID.
///
/// All profile announcements and requests use this single topic.
/// The topic ID is deterministic and identical across all nodes.
///
/// **DEPRECATED**: Use `derive_profile_topic()` for per-peer topics instead.
/// This global topic is kept for backwards compatibility during migration.
pub fn global_profile_topic() -> iroh_gossip::proto::TopicId {
    let hash = blake3::hash(PROFILE_TOPIC_SEED);
    iroh_gossip::proto::TopicId::from_bytes(*hash.as_bytes())
}

/// Derive a profile topic for a specific peer's profile broadcasts.
///
/// Each peer has their own profile topic where they broadcast updates.
/// Contacts subscribe to this topic to receive profile changes.
///
/// # Algorithm
///
/// ```text
/// BLAKE3("sync-profile" || peer_did)
/// ```
///
/// # Benefits
///
/// - **Targeted delivery**: Only contacts receive updates
/// - **Scalability**: O(contacts) messages instead of O(all users)
/// - **Privacy**: Profile updates don't go to non-contacts
/// - **Ownership**: Each peer controls their profile topic
///
/// # Example
///
/// ```ignore
/// let topic = derive_profile_topic("did:sync:abc123");
/// gossip.subscribe(topic, bootstrap_peers).await?;
/// ```
pub fn derive_profile_topic(did: &str) -> iroh_gossip::proto::TopicId {
    let mut hasher = blake3::Hasher::new();
    hasher.update(PROFILE_TOPIC_DOMAIN);
    hasher.update(did.as_bytes());
    iroh_gossip::proto::TopicId::from_bytes(*hasher.finalize().as_bytes())
}

/// Messages sent over the profile gossip topic.
///
/// These messages enable profile discovery and redundant storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfileGossipMessage {
    /// Announce a profile update (broadcast when profile changes)
    ///
    /// Sent by a node when their own profile is updated.
    /// Contains the signed profile and optionally a blob ticket for the avatar.
    Announce {
        /// The signed profile data
        signed_profile: SignedProfile,
        /// Base58-encoded BlobTicket for avatar download (if avatar exists)
        avatar_ticket: Option<String>,
    },

    /// Request a profile by DID
    ///
    /// Sent when a node wants to fetch a specific peer's profile.
    /// Any node that has pinned the requested profile may respond.
    Request {
        /// DID of the profile being requested
        target_did: String,
        /// DID of the requester (so responders know who to send to)
        requester_did: String,
    },

    /// Response to a profile request
    ///
    /// Sent by a node that has pinned the requested profile.
    /// Only the original requester should process this response.
    Response {
        /// The signed profile (None if not found)
        signed_profile: Option<SignedProfile>,
        /// Base58-encoded BlobTicket for avatar download (if avatar exists)
        avatar_ticket: Option<String>,
        /// DID of the original requester (for routing)
        requester_did: String,
    },

}

impl ProfileGossipMessage {
    /// Create an announce message for a profile update.
    pub fn announce(signed_profile: SignedProfile, avatar_ticket: Option<String>) -> Self {
        Self::Announce {
            signed_profile,
            avatar_ticket,
        }
    }

    /// Create a request message for a specific DID.
    pub fn request(target_did: impl Into<String>, requester_did: impl Into<String>) -> Self {
        Self::Request {
            target_did: target_did.into(),
            requester_did: requester_did.into(),
        }
    }

    /// Create a response message.
    pub fn response(
        signed_profile: Option<SignedProfile>,
        avatar_ticket: Option<String>,
        requester_did: impl Into<String>,
    ) -> Self {
        Self::Response {
            signed_profile,
            avatar_ticket,
            requester_did: requester_did.into(),
        }
    }

    /// Serialize the message to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, crate::SyncError> {
        postcard::to_allocvec(self).map_err(|e| crate::SyncError::Serialization(e.to_string()))
    }

    /// Deserialize a message from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, crate::SyncError> {
        postcard::from_bytes(bytes).map_err(|e| crate::SyncError::Serialization(e.to_string()))
    }

    /// Get the DID of the signer if this is an Announce message.
    pub fn signer_did(&self) -> Option<Did> {
        match self {
            Self::Announce { signed_profile, .. } => Some(signed_profile.did()),
            _ => None,
        }
    }

    /// Check if this message is for us (based on our DID).
    ///
    /// - Announce: Always relevant (we might want to pin)
    /// - Request: Only if we might have the profile
    /// - Response: Only if we're the requester
    pub fn is_relevant_to(&self, our_did: &str) -> bool {
        match self {
            Self::Announce { .. } => true, // Always process announcements
            Self::Request { .. } => true,  // We might have the profile
            Self::Response { requester_did, .. } => requester_did == our_did,
        }
    }
}

/// Handler for processing incoming profile gossip messages.
///
/// This struct provides methods to process each message type and
/// determine appropriate actions (pin, respond, ignore).
pub struct ProfileMessageHandler {
    /// Our own DID for filtering responses
    our_did: String,
    /// DIDs we're interested in pinning (contacts, realm members)
    pin_interests: std::collections::HashSet<String>,
}

impl ProfileMessageHandler {
    /// Create a new message handler.
    pub fn new(our_did: impl Into<String>) -> Self {
        Self {
            our_did: our_did.into(),
            pin_interests: std::collections::HashSet::new(),
        }
    }

    /// Add a DID to the set of profiles we're interested in pinning.
    pub fn add_pin_interest(&mut self, did: impl Into<String>) {
        self.pin_interests.insert(did.into());
    }

    /// Remove a DID from the set of profiles we're interested in.
    pub fn remove_pin_interest(&mut self, did: &str) {
        self.pin_interests.remove(did);
    }

    /// Check if we're interested in pinning a specific DID.
    pub fn is_interested_in(&self, did: &str) -> bool {
        self.pin_interests.contains(did)
    }

    /// Process an incoming message and return the recommended action.
    pub fn process_message(&self, msg: &ProfileGossipMessage) -> ProfileAction {
        match msg {
            ProfileGossipMessage::Announce {
                signed_profile,
                avatar_ticket,
            } => {
                let signer_did = signed_profile.did().to_string();

                // Always verify signature
                if !signed_profile.verify() {
                    warn!(did = %signer_did, "Received announce with invalid signature");
                    return ProfileAction::Ignore;
                }

                // Check if we should pin this profile
                if self.is_interested_in(&signer_did) {
                    debug!(did = %signer_did, "Received announce from pinned peer");
                    ProfileAction::UpdatePin {
                        signed_profile: signed_profile.clone(),
                        avatar_ticket: avatar_ticket.clone(),
                    }
                } else {
                    debug!(did = %signer_did, "Received announce from unknown peer");
                    ProfileAction::Ignore
                }
            }

            ProfileGossipMessage::Request {
                target_did,
                requester_did,
            } => {
                // Don't respond to our own requests
                if requester_did == &self.our_did {
                    return ProfileAction::Ignore;
                }

                debug!(target = %target_did, requester = %requester_did, "Received profile request");
                ProfileAction::CheckAndRespond {
                    target_did: target_did.clone(),
                    requester_did: requester_did.clone(),
                }
            }

            ProfileGossipMessage::Response {
                signed_profile,
                avatar_ticket,
                requester_did,
            } => {
                // Only process responses meant for us
                if requester_did != &self.our_did {
                    return ProfileAction::Ignore;
                }

                if let Some(profile) = signed_profile {
                    // Verify signature
                    if !profile.verify() {
                        warn!("Received response with invalid signature");
                        return ProfileAction::Ignore;
                    }

                    let signer_did = profile.did().to_string();
                    debug!(did = %signer_did, "Received valid profile response");

                    ProfileAction::PinResponse {
                        signed_profile: profile.clone(),
                        avatar_ticket: avatar_ticket.clone(),
                    }
                } else {
                    debug!("Received empty profile response");
                    ProfileAction::Ignore
                }
            }

        }
    }
}

/// Action to take after processing a profile message.
#[derive(Debug, Clone)]
pub enum ProfileAction {
    /// Ignore the message (not relevant or invalid)
    Ignore,

    /// Update or create a pin for this profile
    UpdatePin {
        signed_profile: SignedProfile,
        avatar_ticket: Option<String>,
    },

    /// Check if we have the profile and respond if so
    CheckAndRespond {
        target_did: String,
        requester_did: String,
    },

    /// Pin the profile from a response we requested
    PinResponse {
        signed_profile: SignedProfile,
        avatar_ticket: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::HybridKeypair;
    use crate::types::UserProfile;

    fn create_test_signed_profile(name: &str) -> SignedProfile {
        let keypair = HybridKeypair::generate();
        let profile = UserProfile::new(format!("peer_{}", name), name.to_string());
        SignedProfile::sign(&profile, &keypair)
    }

    #[test]
    fn test_global_profile_topic_is_deterministic() {
        let topic1 = global_profile_topic();
        let topic2 = global_profile_topic();
        assert_eq!(topic1, topic2);
    }

    #[test]
    fn test_derive_profile_topic_is_deterministic() {
        let did = "did:sync:abc123";
        let topic1 = derive_profile_topic(did);
        let topic2 = derive_profile_topic(did);
        assert_eq!(topic1, topic2, "Same DID should produce same topic");
    }

    #[test]
    fn test_derive_profile_topic_unique_per_peer() {
        let alice_topic = derive_profile_topic("did:sync:alice");
        let bob_topic = derive_profile_topic("did:sync:bob");
        assert_ne!(alice_topic, bob_topic, "Different DIDs should produce different topics");
    }

    #[test]
    fn test_derive_profile_topic_differs_from_global() {
        let per_peer = derive_profile_topic("did:sync:test");
        let global = global_profile_topic();
        assert_ne!(per_peer, global, "Per-peer topic should differ from global topic");
    }

    #[test]
    fn test_announce_message_serialization() {
        let signed = create_test_signed_profile("Alice");
        let msg = ProfileGossipMessage::announce(signed, Some("ticket123".to_string()));

        let bytes = msg.to_bytes().unwrap();
        let recovered = ProfileGossipMessage::from_bytes(&bytes).unwrap();

        match recovered {
            ProfileGossipMessage::Announce {
                signed_profile,
                avatar_ticket,
            } => {
                assert_eq!(signed_profile.profile.display_name, "Alice");
                assert_eq!(avatar_ticket, Some("ticket123".to_string()));
            }
            _ => panic!("Expected Announce message"),
        }
    }

    #[test]
    fn test_request_message_serialization() {
        let msg = ProfileGossipMessage::request("did:sync:target", "did:sync:requester");

        let bytes = msg.to_bytes().unwrap();
        let recovered = ProfileGossipMessage::from_bytes(&bytes).unwrap();

        match recovered {
            ProfileGossipMessage::Request {
                target_did,
                requester_did,
            } => {
                assert_eq!(target_did, "did:sync:target");
                assert_eq!(requester_did, "did:sync:requester");
            }
            _ => panic!("Expected Request message"),
        }
    }

    #[test]
    fn test_response_message_serialization() {
        let signed = create_test_signed_profile("Bob");
        let msg =
            ProfileGossipMessage::response(Some(signed), Some("ticket456".to_string()), "did:sync:requester");

        let bytes = msg.to_bytes().unwrap();
        let recovered = ProfileGossipMessage::from_bytes(&bytes).unwrap();

        match recovered {
            ProfileGossipMessage::Response {
                signed_profile,
                avatar_ticket,
                requester_did,
            } => {
                assert!(signed_profile.is_some());
                assert_eq!(
                    signed_profile.unwrap().profile.display_name,
                    "Bob"
                );
                assert_eq!(avatar_ticket, Some("ticket456".to_string()));
                assert_eq!(requester_did, "did:sync:requester");
            }
            _ => panic!("Expected Response message"),
        }
    }

    #[test]
    fn test_message_relevance() {
        let signed = create_test_signed_profile("Alice");
        let announce = ProfileGossipMessage::announce(signed, None);
        let request = ProfileGossipMessage::request("did:sync:target", "did:sync:other");
        let response = ProfileGossipMessage::response(None, None, "did:sync:me");

        // Announces are always relevant
        assert!(announce.is_relevant_to("did:sync:me"));

        // Requests are always relevant (we might have the profile)
        assert!(request.is_relevant_to("did:sync:me"));

        // Responses are only relevant to the requester
        assert!(response.is_relevant_to("did:sync:me"));
        assert!(!response.is_relevant_to("did:sync:other"));
    }

    #[test]
    fn test_handler_processes_announce_for_interested_peer() {
        let mut handler = ProfileMessageHandler::new("did:sync:me");

        let keypair = HybridKeypair::generate();
        let profile = UserProfile::new("peer123".to_string(), "Alice".to_string());
        let signed = SignedProfile::sign(&profile, &keypair);
        let peer_did = signed.did().to_string();

        // Add interest in this peer
        handler.add_pin_interest(&peer_did);

        let msg = ProfileGossipMessage::announce(signed, None);
        let action = handler.process_message(&msg);

        match action {
            ProfileAction::UpdatePin { signed_profile, .. } => {
                assert_eq!(signed_profile.profile.display_name, "Alice");
            }
            _ => panic!("Expected UpdatePin action"),
        }
    }

    #[test]
    fn test_handler_ignores_announce_for_unknown_peer() {
        let handler = ProfileMessageHandler::new("did:sync:me");

        let signed = create_test_signed_profile("Unknown");
        let msg = ProfileGossipMessage::announce(signed, None);
        let action = handler.process_message(&msg);

        assert!(matches!(action, ProfileAction::Ignore));
    }

    #[test]
    fn test_handler_ignores_invalid_signature() {
        let mut handler = ProfileMessageHandler::new("did:sync:me");

        let keypair = HybridKeypair::generate();
        let profile = UserProfile::new("peer123".to_string(), "Alice".to_string());
        let mut signed = SignedProfile::sign(&profile, &keypair);

        // Tamper with the profile
        signed.profile.display_name = "Tampered".to_string();

        let peer_did = signed.did().to_string();
        handler.add_pin_interest(&peer_did);

        let msg = ProfileGossipMessage::announce(signed, None);
        let action = handler.process_message(&msg);

        assert!(matches!(action, ProfileAction::Ignore));
    }

    #[test]
    fn test_handler_processes_request() {
        let handler = ProfileMessageHandler::new("did:sync:me");

        let msg = ProfileGossipMessage::request("did:sync:target", "did:sync:other");
        let action = handler.process_message(&msg);

        match action {
            ProfileAction::CheckAndRespond {
                target_did,
                requester_did,
            } => {
                assert_eq!(target_did, "did:sync:target");
                assert_eq!(requester_did, "did:sync:other");
            }
            _ => panic!("Expected CheckAndRespond action"),
        }
    }

    #[test]
    fn test_handler_ignores_own_request() {
        let handler = ProfileMessageHandler::new("did:sync:me");

        // Our own request
        let msg = ProfileGossipMessage::request("did:sync:target", "did:sync:me");
        let action = handler.process_message(&msg);

        assert!(matches!(action, ProfileAction::Ignore));
    }

    #[test]
    fn test_handler_processes_response_for_us() {
        let handler = ProfileMessageHandler::new("did:sync:me");

        let signed = create_test_signed_profile("Bob");
        let msg = ProfileGossipMessage::response(Some(signed), None, "did:sync:me");
        let action = handler.process_message(&msg);

        match action {
            ProfileAction::PinResponse { signed_profile, .. } => {
                assert_eq!(signed_profile.profile.display_name, "Bob");
            }
            _ => panic!("Expected PinResponse action"),
        }
    }

    #[test]
    fn test_handler_ignores_response_for_others() {
        let handler = ProfileMessageHandler::new("did:sync:me");

        let signed = create_test_signed_profile("Bob");
        let msg = ProfileGossipMessage::response(Some(signed), None, "did:sync:other");
        let action = handler.process_message(&msg);

        assert!(matches!(action, ProfileAction::Ignore));
    }

}
