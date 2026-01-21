//! Packet event buffer for Indra's Network visualization.
//!
//! This module provides an in-memory circular buffer for tracking packet events,
//! enabling the UI to visualize packet flow and relay behavior.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  PacketEventBuffer                                              │
//! │  ├── events: HashMap<peer_did, VecDeque<PacketEvent>>          │
//! │  │   └── Circular buffer per peer (default 50 events)          │
//! │  │                                                              │
//! │  ├── broadcast_tx: broadcast::Sender<PacketEvent>              │
//! │  │   └── Real-time updates for UI subscriptions                │
//! │  │                                                              │
//! │  └── config                                                    │
//! │      └── max_events_per_peer: usize                            │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::broadcast;

use super::events::PacketEvent;

/// Default maximum events to keep per peer.
const DEFAULT_MAX_EVENTS_PER_PEER: usize = 50;

/// Buffer size for the broadcast channel.
const BROADCAST_CHANNEL_SIZE: usize = 256;

/// Configuration for the packet event buffer.
#[derive(Debug, Clone)]
pub struct PacketEventBufferConfig {
    /// Maximum events to keep per peer (oldest are evicted).
    pub max_events_per_peer: usize,
}

impl Default for PacketEventBufferConfig {
    fn default() -> Self {
        Self {
            max_events_per_peer: DEFAULT_MAX_EVENTS_PER_PEER,
        }
    }
}

/// Inner state for the packet event buffer.
struct PacketEventBufferInner {
    /// Events organized by peer DID → circular buffer of events.
    events: HashMap<String, VecDeque<PacketEvent>>,
    /// Configuration.
    config: PacketEventBufferConfig,
}

/// In-memory circular buffer for packet events.
///
/// Provides:
/// - Per-peer event storage with configurable limits
/// - Real-time broadcast channel for UI subscriptions
/// - Thread-safe access via RwLock
///
/// # Example
///
/// ```ignore
/// let buffer = PacketEventBuffer::new(PacketEventBufferConfig::default());
///
/// // Record an event
/// buffer.record(event);
///
/// // Get events for a specific peer
/// let events = buffer.get_events_for_peer("did:sync:joy");
///
/// // Subscribe to real-time updates
/// let mut rx = buffer.subscribe();
/// while let Ok(event) = rx.recv().await {
///     println!("New event: {:?}", event);
/// }
/// ```
pub struct PacketEventBuffer {
    inner: RwLock<PacketEventBufferInner>,
    broadcast_tx: broadcast::Sender<PacketEvent>,
}

impl PacketEventBuffer {
    /// Create a new packet event buffer with the given configuration.
    pub fn new(config: PacketEventBufferConfig) -> Arc<Self> {
        let (broadcast_tx, _) = broadcast::channel(BROADCAST_CHANNEL_SIZE);

        Arc::new(Self {
            inner: RwLock::new(PacketEventBufferInner {
                events: HashMap::new(),
                config,
            }),
            broadcast_tx,
        })
    }

    /// Create a new buffer with default configuration.
    pub fn with_defaults() -> Arc<Self> {
        Self::new(PacketEventBufferConfig::default())
    }

    /// Record a new packet event.
    ///
    /// The event is stored in the buffer for the associated peer and
    /// broadcast to all subscribers.
    pub fn record(&self, event: PacketEvent) {
        let peer_did = event.peer_did.clone();

        // Store in buffer
        {
            let mut inner = self.inner.write();
            let max_events = inner.config.max_events_per_peer;
            let peer_events = inner.events.entry(peer_did).or_insert_with(VecDeque::new);

            // Evict oldest if at capacity
            if peer_events.len() >= max_events {
                peer_events.pop_front();
            }

            peer_events.push_back(event.clone());
        }

        // Broadcast to subscribers (ignore errors if no subscribers)
        let _ = self.broadcast_tx.send(event);
    }

    /// Get all events for a specific peer.
    ///
    /// Returns events in chronological order (oldest first).
    pub fn get_events_for_peer(&self, peer_did: &str) -> Vec<PacketEvent> {
        let inner = self.inner.read();
        inner
            .events
            .get(peer_did)
            .map(|deque| deque.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get events for all peers.
    ///
    /// Returns a map of peer DID → events (chronological order).
    pub fn get_all_events(&self) -> HashMap<String, Vec<PacketEvent>> {
        let inner = self.inner.read();
        inner
            .events
            .iter()
            .map(|(k, v)| (k.clone(), v.iter().cloned().collect()))
            .collect()
    }

    /// Subscribe to real-time packet events.
    ///
    /// Returns a receiver that will receive all new events as they are recorded.
    /// If the receiver falls behind, older events will be dropped.
    pub fn subscribe(&self) -> broadcast::Receiver<PacketEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Get the number of events stored for a peer.
    pub fn event_count_for_peer(&self, peer_did: &str) -> usize {
        let inner = self.inner.read();
        inner.events.get(peer_did).map(|v| v.len()).unwrap_or(0)
    }

    /// Get total number of events across all peers.
    pub fn total_event_count(&self) -> usize {
        let inner = self.inner.read();
        inner.events.values().map(|v| v.len()).sum()
    }

    /// Clear all events for a specific peer.
    pub fn clear_peer(&self, peer_did: &str) {
        let mut inner = self.inner.write();
        inner.events.remove(peer_did);
    }

    /// Clear all events.
    pub fn clear_all(&self) {
        let mut inner = self.inner.write();
        inner.events.clear();
    }

    /// Mark a packet as delivered (sets is_delivered = true).
    ///
    /// This is used to show strikethrough in the UI when a relayed packet
    /// has reached its final destination.
    pub fn mark_delivered(&self, peer_did: &str, event_id: &str) {
        let mut inner = self.inner.write();
        if let Some(events) = inner.events.get_mut(peer_did) {
            for event in events.iter_mut() {
                if event.id == event_id {
                    event.is_delivered = true;
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::events::{DecryptionStatus, PacketDirection};

    fn make_test_event(peer_did: &str, sequence: u64) -> PacketEvent {
        PacketEvent {
            id: PacketEvent::make_id("did:sync:author", sequence),
            timestamp: chrono::Utc::now().timestamp_millis(),
            direction: PacketDirection::Incoming,
            sequence,
            author_did: "did:sync:author".to_string(),
            author_name: "Author".to_string(),
            relay_did: None,
            relay_name: None,
            destination_did: "did:sync:me".to_string(),
            destination_name: "Me".to_string(),
            decryption_status: DecryptionStatus::Decrypted,
            content_preview: format!("Message {}", sequence),
            is_delivered: false,
            peer_did: peer_did.to_string(),
        }
    }

    #[test]
    fn test_packet_event_buffer_record_and_retrieve() {
        let buffer = PacketEventBuffer::with_defaults();

        let event1 = make_test_event("peer1", 1);
        let event2 = make_test_event("peer1", 2);
        let event3 = make_test_event("peer2", 1);

        buffer.record(event1);
        buffer.record(event2);
        buffer.record(event3);

        // Check peer1 events
        let peer1_events = buffer.get_events_for_peer("peer1");
        assert_eq!(peer1_events.len(), 2);
        assert_eq!(peer1_events[0].sequence, 1);
        assert_eq!(peer1_events[1].sequence, 2);

        // Check peer2 events
        let peer2_events = buffer.get_events_for_peer("peer2");
        assert_eq!(peer2_events.len(), 1);
        assert_eq!(peer2_events[0].sequence, 1);

        // Check unknown peer
        let unknown = buffer.get_events_for_peer("unknown");
        assert!(unknown.is_empty());
    }

    #[test]
    fn test_packet_event_buffer_circular() {
        let config = PacketEventBufferConfig {
            max_events_per_peer: 3,
        };
        let buffer = PacketEventBuffer::new(config);

        // Add 5 events (should evict first 2)
        for seq in 1..=5 {
            buffer.record(make_test_event("peer1", seq));
        }

        let events = buffer.get_events_for_peer("peer1");
        assert_eq!(events.len(), 3);
        // Should have events 3, 4, 5 (oldest evicted)
        assert_eq!(events[0].sequence, 3);
        assert_eq!(events[1].sequence, 4);
        assert_eq!(events[2].sequence, 5);
    }

    #[test]
    fn test_packet_event_buffer_per_peer_limits() {
        let config = PacketEventBufferConfig {
            max_events_per_peer: 2,
        };
        let buffer = PacketEventBuffer::new(config);

        // Add 3 events to peer1
        for seq in 1..=3 {
            buffer.record(make_test_event("peer1", seq));
        }

        // Add 3 events to peer2
        for seq in 1..=3 {
            buffer.record(make_test_event("peer2", seq));
        }

        // Each peer should have only 2 events
        assert_eq!(buffer.event_count_for_peer("peer1"), 2);
        assert_eq!(buffer.event_count_for_peer("peer2"), 2);
        assert_eq!(buffer.total_event_count(), 4);

        // Peer1 should have events 2 and 3
        let peer1_events = buffer.get_events_for_peer("peer1");
        assert_eq!(peer1_events[0].sequence, 2);
        assert_eq!(peer1_events[1].sequence, 3);
    }

    #[tokio::test]
    async fn test_packet_event_buffer_subscribe() {
        let buffer = PacketEventBuffer::with_defaults();
        let mut rx = buffer.subscribe();

        // Record an event
        let event = make_test_event("peer1", 42);
        buffer.record(event.clone());

        // Should receive it
        let received = rx.recv().await.expect("Should receive event");
        assert_eq!(received.sequence, 42);
        assert_eq!(received.peer_did, "peer1");
    }

    #[test]
    fn test_packet_event_buffer_mark_delivered() {
        let buffer = PacketEventBuffer::with_defaults();

        let event = make_test_event("peer1", 1);
        let event_id = event.id.clone();
        buffer.record(event);

        // Should not be delivered initially
        let events = buffer.get_events_for_peer("peer1");
        assert!(!events[0].is_delivered);

        // Mark as delivered
        buffer.mark_delivered("peer1", &event_id);

        // Should be delivered now
        let events = buffer.get_events_for_peer("peer1");
        assert!(events[0].is_delivered);
    }

    #[test]
    fn test_packet_event_buffer_clear() {
        let buffer = PacketEventBuffer::with_defaults();

        buffer.record(make_test_event("peer1", 1));
        buffer.record(make_test_event("peer2", 1));

        assert_eq!(buffer.total_event_count(), 2);

        // Clear peer1
        buffer.clear_peer("peer1");
        assert_eq!(buffer.event_count_for_peer("peer1"), 0);
        assert_eq!(buffer.event_count_for_peer("peer2"), 1);

        // Clear all
        buffer.clear_all();
        assert_eq!(buffer.total_event_count(), 0);
    }

    #[test]
    fn test_packet_event_buffer_get_all_events() {
        let buffer = PacketEventBuffer::with_defaults();

        buffer.record(make_test_event("peer1", 1));
        buffer.record(make_test_event("peer1", 2));
        buffer.record(make_test_event("peer2", 1));

        let all = buffer.get_all_events();
        assert_eq!(all.len(), 2);
        assert_eq!(all.get("peer1").unwrap().len(), 2);
        assert_eq!(all.get("peer2").unwrap().len(), 1);
    }
}
