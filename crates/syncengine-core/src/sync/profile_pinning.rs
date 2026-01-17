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

/// Get the global profile topic ID.
///
/// All profile announcements and requests use this single topic.
/// The topic ID is deterministic and identical across all nodes.
pub fn global_profile_topic() -> iroh_gossip::proto::TopicId {
    let hash = blake3::hash(PROFILE_TOPIC_SEED);
    iroh_gossip::proto::TopicId::from_bytes(*hash.as_bytes())
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

    /// Acknowledgment that a peer has pinned our profile
    ///
    /// Sent when a peer decides to pin our profile (e.g., after becoming a contact
    /// or joining a shared realm). This enables bidirectional awareness - we know
    /// who is carrying our data in the P2P network.
    PinAcknowledgment {
        /// DID of the peer who is pinning (the sender)
        pinner_did: String,
        /// DID of the profile being pinned (should be our DID)
        target_did: String,
        /// Unix timestamp when the pin was created
        pinned_at: i64,
        /// Relationship type: "contact", "realm_member", "manual"
        relationship: String,
    },

    /// Notification that a peer has unpinned our profile
    ///
    /// Sent when a peer removes their pin (e.g., contact removed, left realm).
    /// Allows us to update our "who pins me" list.
    PinRemoval {
        /// DID of the peer who unpinned (the sender)
        pinner_did: String,
        /// DID of the profile that was unpinned (should be our DID)
        target_did: String,
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

    /// Create a pin acknowledgment message.
    ///
    /// Sent to notify a peer that we have pinned their profile.
    pub fn pin_acknowledgment(
        pinner_did: impl Into<String>,
        target_did: impl Into<String>,
        pinned_at: i64,
        relationship: impl Into<String>,
    ) -> Self {
        Self::PinAcknowledgment {
            pinner_did: pinner_did.into(),
            target_did: target_did.into(),
            pinned_at,
            relationship: relationship.into(),
        }
    }

    /// Create a pin removal message.
    ///
    /// Sent to notify a peer that we have unpinned their profile.
    pub fn pin_removal(pinner_did: impl Into<String>, target_did: impl Into<String>) -> Self {
        Self::PinRemoval {
            pinner_did: pinner_did.into(),
            target_did: target_did.into(),
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
    /// - PinAcknowledgment: Only if we're the target (someone pinned our profile)
    /// - PinRemoval: Only if we're the target (someone unpinned our profile)
    pub fn is_relevant_to(&self, our_did: &str) -> bool {
        match self {
            Self::Announce { .. } => true, // Always process announcements
            Self::Request { .. } => true,  // We might have the profile
            Self::Response { requester_did, .. } => requester_did == our_did,
            Self::PinAcknowledgment { target_did, .. } => target_did == our_did,
            Self::PinRemoval { target_did, .. } => target_did == our_did,
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

            ProfileGossipMessage::PinAcknowledgment {
                pinner_did,
                target_did,
                pinned_at,
                relationship,
            } => {
                // Only process if we're the target (someone pinned our profile)
                if target_did != &self.our_did {
                    debug!(target = %target_did, our_did = %self.our_did, "Ignoring PinAck for different target");
                    return ProfileAction::Ignore;
                }

                debug!(pinner = %pinner_did, relationship = %relationship, "Received pin acknowledgment");
                ProfileAction::RecordPinner {
                    pinner_did: pinner_did.clone(),
                    pinned_at: *pinned_at,
                    relationship: relationship.clone(),
                }
            }

            ProfileGossipMessage::PinRemoval {
                pinner_did,
                target_did,
            } => {
                // Only process if we're the target (someone unpinned our profile)
                if target_did != &self.our_did {
                    debug!(target = %target_did, our_did = %self.our_did, "Ignoring PinRemoval for different target");
                    return ProfileAction::Ignore;
                }

                debug!(pinner = %pinner_did, "Received pin removal notification");
                ProfileAction::RemovePinner {
                    pinner_did: pinner_did.clone(),
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

    /// Record that a peer has pinned our profile (for "Souls Carrying Your Light")
    RecordPinner {
        pinner_did: String,
        pinned_at: i64,
        relationship: String,
    },

    /// Remove a pinner record (peer unpinned our profile)
    RemovePinner {
        pinner_did: String,
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

    #[test]
    fn test_pin_acknowledgment_message_serialization() {
        let msg = ProfileGossipMessage::pin_acknowledgment(
            "did:sync:pinner",
            "did:sync:target",
            1234567890,
            "contact",
        );

        let bytes = msg.to_bytes().unwrap();
        let recovered = ProfileGossipMessage::from_bytes(&bytes).unwrap();

        match recovered {
            ProfileGossipMessage::PinAcknowledgment {
                pinner_did,
                target_did,
                pinned_at,
                relationship,
            } => {
                assert_eq!(pinner_did, "did:sync:pinner");
                assert_eq!(target_did, "did:sync:target");
                assert_eq!(pinned_at, 1234567890);
                assert_eq!(relationship, "contact");
            }
            _ => panic!("Expected PinAcknowledgment message"),
        }
    }

    #[test]
    fn test_pin_removal_message_serialization() {
        let msg = ProfileGossipMessage::pin_removal("did:sync:pinner", "did:sync:target");

        let bytes = msg.to_bytes().unwrap();
        let recovered = ProfileGossipMessage::from_bytes(&bytes).unwrap();

        match recovered {
            ProfileGossipMessage::PinRemoval {
                pinner_did,
                target_did,
            } => {
                assert_eq!(pinner_did, "did:sync:pinner");
                assert_eq!(target_did, "did:sync:target");
            }
            _ => panic!("Expected PinRemoval message"),
        }
    }

    #[test]
    fn test_pin_acknowledgment_relevance() {
        let pin_ack =
            ProfileGossipMessage::pin_acknowledgment("did:sync:pinner", "did:sync:me", 123, "contact");

        // Relevant to the target
        assert!(pin_ack.is_relevant_to("did:sync:me"));
        // Not relevant to others
        assert!(!pin_ack.is_relevant_to("did:sync:other"));
    }

    #[test]
    fn test_pin_removal_relevance() {
        let pin_removal = ProfileGossipMessage::pin_removal("did:sync:pinner", "did:sync:me");

        // Relevant to the target
        assert!(pin_removal.is_relevant_to("did:sync:me"));
        // Not relevant to others
        assert!(!pin_removal.is_relevant_to("did:sync:other"));
    }

    #[test]
    fn test_handler_processes_pin_acknowledgment_for_us() {
        let handler = ProfileMessageHandler::new("did:sync:me");

        let msg =
            ProfileGossipMessage::pin_acknowledgment("did:sync:alice", "did:sync:me", 1234567890, "contact");
        let action = handler.process_message(&msg);

        match action {
            ProfileAction::RecordPinner {
                pinner_did,
                pinned_at,
                relationship,
            } => {
                assert_eq!(pinner_did, "did:sync:alice");
                assert_eq!(pinned_at, 1234567890);
                assert_eq!(relationship, "contact");
            }
            _ => panic!("Expected RecordPinner action"),
        }
    }

    #[test]
    fn test_handler_ignores_pin_acknowledgment_for_others() {
        let handler = ProfileMessageHandler::new("did:sync:me");

        let msg =
            ProfileGossipMessage::pin_acknowledgment("did:sync:alice", "did:sync:other", 1234567890, "contact");
        let action = handler.process_message(&msg);

        assert!(matches!(action, ProfileAction::Ignore));
    }

    #[test]
    fn test_handler_processes_pin_removal_for_us() {
        let handler = ProfileMessageHandler::new("did:sync:me");

        let msg = ProfileGossipMessage::pin_removal("did:sync:alice", "did:sync:me");
        let action = handler.process_message(&msg);

        match action {
            ProfileAction::RemovePinner { pinner_did } => {
                assert_eq!(pinner_did, "did:sync:alice");
            }
            _ => panic!("Expected RemovePinner action"),
        }
    }

    #[test]
    fn test_handler_ignores_pin_removal_for_others() {
        let handler = ProfileMessageHandler::new("did:sync:me");

        let msg = ProfileGossipMessage::pin_removal("did:sync:alice", "did:sync:other");
        let action = handler.process_message(&msg);

        assert!(matches!(action, ProfileAction::Ignore));
    }
}
