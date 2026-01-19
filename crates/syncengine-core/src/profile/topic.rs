//! Per-profile gossip topics
//!
//! Each profile has its own gossip topic for packet announcements and sync.
//! This enables efficient routing: packets are broadcast on the sender's topic,
//! and peers who care about that profile subscribe to it.
//!
//! ## Topic Derivation
//!
//! Profile topics are derived from the profile's DID using BLAKE3:
//!
//! ```text
//! topic_id = BLAKE3("indra-profile-topic-v1" || did_string)
//! ```
//!
//! ## Routing Strategy
//!
//! 1. **Direct delivery**: Broadcast on sender's profile topic
//! 2. **Realm fallback**: If recipient is offline, route via shared realm topics
//! 3. **Relay storage**: Peers store packets they relay for offline delivery

use crate::identity::Did;
use crate::types::RealmId;
use iroh_gossip::proto::TopicId;

/// Domain separation prefix for profile topics.
const PROFILE_TOPIC_PREFIX: &[u8] = b"indra-profile-topic-v1:";

/// Domain separation prefix for realm packet topics.
const REALM_PACKET_TOPIC_PREFIX: &[u8] = b"indra-realm-packet-topic-v1:";

/// Derive a gossip topic ID for a profile.
///
/// Each profile has a unique topic where their packets are announced.
/// Peers interested in a profile's packets subscribe to this topic.
pub fn derive_profile_packet_topic(did: &Did) -> TopicId {
    let mut input = Vec::with_capacity(PROFILE_TOPIC_PREFIX.len() + did.as_str().len());
    input.extend_from_slice(PROFILE_TOPIC_PREFIX);
    input.extend_from_slice(did.as_str().as_bytes());

    let hash = blake3::hash(&input);
    TopicId::from_bytes(*hash.as_bytes())
}

/// Derive a gossip topic ID for packet sync within a realm.
///
/// This is a separate topic from the realm's Automerge sync topic,
/// used specifically for profile packet announcements.
pub fn derive_realm_packet_topic(realm_id: &RealmId) -> TopicId {
    let mut input = Vec::with_capacity(REALM_PACKET_TOPIC_PREFIX.len() + 32);
    input.extend_from_slice(REALM_PACKET_TOPIC_PREFIX);
    input.extend_from_slice(&realm_id.0);

    let hash = blake3::hash(&input);
    TopicId::from_bytes(*hash.as_bytes())
}

/// Routing decision for packet delivery.
#[derive(Debug, Clone, PartialEq)]
pub enum PacketRoute {
    /// Send directly to recipient's profile topic
    Direct {
        /// Recipient's DID
        recipient: Did,
        /// Derived topic ID
        topic: TopicId,
    },
    /// Send via shared realm topic (fallback)
    RealmFallback {
        /// Realm to route through
        realm_id: RealmId,
        /// Derived topic ID
        topic: TopicId,
    },
    /// Broadcast to multiple topics
    MultiTopic {
        /// List of (recipient_did, topic_id) pairs
        routes: Vec<(Did, TopicId)>,
    },
    /// Global broadcast (for public packets)
    Global {
        /// List of realm topics to broadcast to
        realms: Vec<TopicId>,
    },
}

impl PacketRoute {
    /// Create a direct route to a recipient.
    pub fn direct(recipient: &Did) -> Self {
        let topic = derive_profile_packet_topic(recipient);
        PacketRoute::Direct {
            recipient: recipient.clone(),
            topic,
        }
    }

    /// Create a realm fallback route.
    pub fn realm_fallback(realm_id: &RealmId) -> Self {
        let topic = derive_realm_packet_topic(realm_id);
        PacketRoute::RealmFallback {
            realm_id: realm_id.clone(),
            topic,
        }
    }

    /// Create a multi-topic route for multiple recipients.
    pub fn multi_topic(recipients: &[Did]) -> Self {
        let routes = recipients
            .iter()
            .map(|did| (did.clone(), derive_profile_packet_topic(did)))
            .collect();
        PacketRoute::MultiTopic { routes }
    }

    /// Create a global broadcast route.
    pub fn global(realm_ids: &[RealmId]) -> Self {
        let realms = realm_ids
            .iter()
            .map(derive_realm_packet_topic)
            .collect();
        PacketRoute::Global { realms }
    }

    /// Get all topic IDs in this route.
    pub fn topics(&self) -> Vec<TopicId> {
        match self {
            PacketRoute::Direct { topic, .. } => vec![*topic],
            PacketRoute::RealmFallback { topic, .. } => vec![*topic],
            PacketRoute::MultiTopic { routes } => {
                routes.iter().map(|(_, t)| *t).collect()
            }
            PacketRoute::Global { realms } => realms.clone(),
        }
    }

    /// Check if this route includes a specific topic.
    pub fn includes_topic(&self, topic: &TopicId) -> bool {
        self.topics().contains(topic)
    }
}

/// Subscription manager for profile topics.
///
/// Tracks which profiles we're interested in and manages subscriptions.
#[derive(Debug, Default)]
pub struct ProfileTopicTracker {
    /// Profiles we're subscribed to
    subscribed: Vec<(Did, TopicId)>,
    /// Realms we're in (for packet routing)
    realm_topics: Vec<(RealmId, TopicId)>,
}

impl ProfileTopicTracker {
    /// Create a new tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to a profile's packet topic.
    pub fn subscribe_profile(&mut self, did: &Did) -> TopicId {
        let topic = derive_profile_packet_topic(did);
        if !self.subscribed.iter().any(|(d, _)| d == did) {
            self.subscribed.push((did.clone(), topic));
        }
        topic
    }

    /// Unsubscribe from a profile's packet topic.
    pub fn unsubscribe_profile(&mut self, did: &Did) -> Option<TopicId> {
        if let Some(pos) = self.subscribed.iter().position(|(d, _)| d == did) {
            Some(self.subscribed.remove(pos).1)
        } else {
            None
        }
    }

    /// Check if we're subscribed to a profile.
    pub fn is_subscribed(&self, did: &Did) -> bool {
        self.subscribed.iter().any(|(d, _)| d == did)
    }

    /// Get the topic ID for a subscribed profile.
    pub fn get_topic(&self, did: &Did) -> Option<TopicId> {
        self.subscribed
            .iter()
            .find(|(d, _)| d == did)
            .map(|(_, t)| *t)
    }

    /// Add a realm for packet routing.
    pub fn add_realm(&mut self, realm_id: &RealmId) -> TopicId {
        let topic = derive_realm_packet_topic(realm_id);
        if !self.realm_topics.iter().any(|(r, _)| r == realm_id) {
            self.realm_topics.push((realm_id.clone(), topic));
        }
        topic
    }

    /// Remove a realm.
    pub fn remove_realm(&mut self, realm_id: &RealmId) -> Option<TopicId> {
        if let Some(pos) = self.realm_topics.iter().position(|(r, _)| r == realm_id) {
            Some(self.realm_topics.remove(pos).1)
        } else {
            None
        }
    }

    /// Get all subscribed profile topics.
    pub fn profile_topics(&self) -> Vec<TopicId> {
        self.subscribed.iter().map(|(_, t)| *t).collect()
    }

    /// Get all realm packet topics.
    pub fn realm_packet_topics(&self) -> Vec<TopicId> {
        self.realm_topics.iter().map(|(_, t)| *t).collect()
    }

    /// Get all topics (profiles + realms).
    pub fn all_topics(&self) -> Vec<TopicId> {
        let mut topics = self.profile_topics();
        topics.extend(self.realm_packet_topics());
        topics
    }

    /// List subscribed profiles.
    pub fn subscribed_profiles(&self) -> Vec<&Did> {
        self.subscribed.iter().map(|(d, _)| d).collect()
    }

    /// List tracked realms.
    pub fn tracked_realms(&self) -> Vec<&RealmId> {
        self.realm_topics.iter().map(|(r, _)| r).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::ProfileKeys;

    #[test]
    fn test_derive_profile_packet_topic_deterministic() {
        let keys = ProfileKeys::generate();
        let did = keys.did();

        let topic1 = derive_profile_packet_topic(&did);
        let topic2 = derive_profile_packet_topic(&did);

        assert_eq!(topic1, topic2);
    }

    #[test]
    fn test_derive_profile_packet_topic_unique() {
        let keys1 = ProfileKeys::generate();
        let keys2 = ProfileKeys::generate();

        let topic1 = derive_profile_packet_topic(&keys1.did());
        let topic2 = derive_profile_packet_topic(&keys2.did());

        assert_ne!(topic1, topic2);
    }

    #[test]
    fn test_derive_realm_packet_topic_deterministic() {
        let realm_id = RealmId::new();

        let topic1 = derive_realm_packet_topic(&realm_id);
        let topic2 = derive_realm_packet_topic(&realm_id);

        assert_eq!(topic1, topic2);
    }

    #[test]
    fn test_derive_realm_packet_topic_unique() {
        let realm_id1 = RealmId::new();
        let realm_id2 = RealmId::new();

        let topic1 = derive_realm_packet_topic(&realm_id1);
        let topic2 = derive_realm_packet_topic(&realm_id2);

        assert_ne!(topic1, topic2);
    }

    #[test]
    fn test_packet_route_direct() {
        let keys = ProfileKeys::generate();
        let did = keys.did();

        let route = PacketRoute::direct(&did);
        let expected_topic = derive_profile_packet_topic(&did);

        match route {
            PacketRoute::Direct { recipient, topic } => {
                assert_eq!(recipient, did);
                assert_eq!(topic, expected_topic);
            }
            _ => panic!("Expected Direct route"),
        }
    }

    #[test]
    fn test_packet_route_topics() {
        let keys1 = ProfileKeys::generate();
        let keys2 = ProfileKeys::generate();

        let route = PacketRoute::multi_topic(&[keys1.did(), keys2.did()]);
        let topics = route.topics();

        assert_eq!(topics.len(), 2);
        assert!(route.includes_topic(&derive_profile_packet_topic(&keys1.did())));
        assert!(route.includes_topic(&derive_profile_packet_topic(&keys2.did())));
    }

    #[test]
    fn test_profile_topic_tracker_subscribe() {
        let mut tracker = ProfileTopicTracker::new();
        let keys = ProfileKeys::generate();
        let did = keys.did();

        let topic = tracker.subscribe_profile(&did);
        assert!(tracker.is_subscribed(&did));
        assert_eq!(tracker.get_topic(&did), Some(topic));
    }

    #[test]
    fn test_profile_topic_tracker_unsubscribe() {
        let mut tracker = ProfileTopicTracker::new();
        let keys = ProfileKeys::generate();
        let did = keys.did();

        tracker.subscribe_profile(&did);
        assert!(tracker.is_subscribed(&did));

        tracker.unsubscribe_profile(&did);
        assert!(!tracker.is_subscribed(&did));
    }

    #[test]
    fn test_profile_topic_tracker_no_duplicates() {
        let mut tracker = ProfileTopicTracker::new();
        let keys = ProfileKeys::generate();
        let did = keys.did();

        tracker.subscribe_profile(&did);
        tracker.subscribe_profile(&did);
        tracker.subscribe_profile(&did);

        assert_eq!(tracker.subscribed_profiles().len(), 1);
    }

    #[test]
    fn test_profile_topic_tracker_realms() {
        let mut tracker = ProfileTopicTracker::new();
        let realm_id = RealmId::new();

        let topic = tracker.add_realm(&realm_id);
        assert_eq!(tracker.tracked_realms().len(), 1);
        assert!(tracker.realm_packet_topics().contains(&topic));

        tracker.remove_realm(&realm_id);
        assert_eq!(tracker.tracked_realms().len(), 0);
    }

    #[test]
    fn test_profile_topic_tracker_all_topics() {
        let mut tracker = ProfileTopicTracker::new();
        let keys = ProfileKeys::generate();
        let realm_id = RealmId::new();

        tracker.subscribe_profile(&keys.did());
        tracker.add_realm(&realm_id);

        let all = tracker.all_topics();
        assert_eq!(all.len(), 2);
    }
}
