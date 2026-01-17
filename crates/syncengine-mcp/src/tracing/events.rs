//! Message event tracking for tracing

use super::TraceId;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of message event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageEventType {
    /// Message was sent by this node
    Sent,
    /// Message was received by this node
    Received,
    /// Message was forwarded to other peers
    Forwarded,
    /// Message was dropped (invalid, rate limited, etc.)
    Dropped,
    /// Message was acknowledged
    Acknowledged,
}

/// A message event for tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEvent {
    /// Trace ID this event belongs to
    pub trace_id: TraceId,
    /// Type of event
    pub event_type: MessageEventType,
    /// Node that generated this event
    pub node_id: String,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Peer involved (sender for received, receiver for sent)
    pub peer_id: Option<String>,
    /// Additional details
    pub details: Option<String>,
}

impl MessageEvent {
    /// Create a new sent event
    pub fn sent(trace_id: TraceId, node_id: String, content: &str) -> Self {
        Self {
            trace_id,
            event_type: MessageEventType::Sent,
            node_id,
            timestamp: Utc::now(),
            peer_id: None,
            details: Some(format!("Content: {}", content)),
        }
    }

    /// Create a new received event
    pub fn received(trace_id: TraceId, node_id: String, from_peer: String) -> Self {
        Self {
            trace_id,
            event_type: MessageEventType::Received,
            node_id,
            timestamp: Utc::now(),
            peer_id: Some(from_peer),
            details: None,
        }
    }

    /// Create a new forwarded event
    pub fn forwarded(trace_id: TraceId, node_id: String, to_peer: String) -> Self {
        Self {
            trace_id,
            event_type: MessageEventType::Forwarded,
            node_id,
            timestamp: Utc::now(),
            peer_id: Some(to_peer),
            details: None,
        }
    }

    /// Create a new dropped event
    pub fn dropped(trace_id: TraceId, node_id: String, reason: String) -> Self {
        Self {
            trace_id,
            event_type: MessageEventType::Dropped,
            node_id,
            timestamp: Utc::now(),
            peer_id: None,
            details: Some(reason),
        }
    }
}

/// Store for message events
pub struct TraceStore {
    /// Events indexed by trace ID
    events: RwLock<HashMap<TraceId, Vec<MessageEvent>>>,
    /// Maximum events per trace
    max_events_per_trace: usize,
    /// Maximum traces to keep
    max_traces: usize,
}

impl TraceStore {
    /// Create a new trace store
    pub fn new() -> Self {
        Self {
            events: RwLock::new(HashMap::new()),
            max_events_per_trace: 1000,
            max_traces: 10000,
        }
    }

    /// Create with custom limits
    pub fn with_limits(max_events_per_trace: usize, max_traces: usize) -> Self {
        Self {
            events: RwLock::new(HashMap::new()),
            max_events_per_trace,
            max_traces,
        }
    }

    /// Record an event
    pub fn record_event(&self, event: MessageEvent) {
        let mut events = self.events.write();

        // Enforce max traces limit
        if events.len() >= self.max_traces && !events.contains_key(&event.trace_id) {
            // Remove oldest trace (by first event timestamp)
            if let Some((oldest_id, _)) = events
                .iter()
                .filter_map(|(id, evts)| evts.first().map(|e| (*id, e.timestamp)))
                .min_by_key(|(_, ts)| *ts)
            {
                events.remove(&oldest_id);
            }
        }

        let trace_events = events.entry(event.trace_id).or_insert_with(Vec::new);

        // Enforce max events per trace
        if trace_events.len() >= self.max_events_per_trace {
            trace_events.remove(0);
        }

        trace_events.push(event);
    }

    /// Get all events for a trace
    pub fn get_events(&self, trace_id: &TraceId) -> Vec<MessageEvent> {
        self.events
            .read()
            .get(trace_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get events for a trace filtered by event type
    pub fn get_events_by_type(
        &self,
        trace_id: &TraceId,
        event_type: MessageEventType,
    ) -> Vec<MessageEvent> {
        self.get_events(trace_id)
            .into_iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// Get all events for a node
    pub fn get_events_for_node(&self, node_id: &str) -> Vec<MessageEvent> {
        self.events
            .read()
            .values()
            .flatten()
            .filter(|e| e.node_id == node_id)
            .cloned()
            .collect()
    }

    /// Clear events for a trace
    pub fn clear_trace(&self, trace_id: &TraceId) {
        self.events.write().remove(trace_id);
    }

    /// Clear all events
    pub fn clear_all(&self) {
        self.events.write().clear();
    }

    /// Get trace count
    pub fn trace_count(&self) -> usize {
        self.events.read().len()
    }

    /// Get total event count
    pub fn event_count(&self) -> usize {
        self.events.read().values().map(|v| v.len()).sum()
    }

    /// List all trace IDs
    pub fn list_trace_ids(&self) -> Vec<TraceId> {
        self.events.read().keys().copied().collect()
    }
}

impl Default for TraceStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_get_events() {
        let store = TraceStore::new();
        let trace_id = [1u8; 16];

        let event = MessageEvent::sent(trace_id, "alice".into(), "Hello");
        store.record_event(event);

        let events = store.get_events(&trace_id);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].node_id, "alice");
    }

    #[test]
    fn test_get_events_by_type() {
        let store = TraceStore::new();
        let trace_id = [1u8; 16];

        store.record_event(MessageEvent::sent(trace_id, "alice".into(), "Hello"));
        store.record_event(MessageEvent::received(trace_id, "bob".into(), "alice".into()));
        store.record_event(MessageEvent::received(trace_id, "carol".into(), "alice".into()));

        let received = store.get_events_by_type(&trace_id, MessageEventType::Received);
        assert_eq!(received.len(), 2);

        let sent = store.get_events_by_type(&trace_id, MessageEventType::Sent);
        assert_eq!(sent.len(), 1);
    }

    #[test]
    fn test_max_events_limit() {
        let store = TraceStore::with_limits(3, 100);
        let trace_id = [1u8; 16];

        for i in 0..5 {
            store.record_event(MessageEvent::sent(
                trace_id,
                format!("node_{}", i),
                "Hello",
            ));
        }

        let events = store.get_events(&trace_id);
        assert_eq!(events.len(), 3);
        // Should have kept the last 3
        assert_eq!(events[0].node_id, "node_2");
    }

    #[test]
    fn test_clear_trace() {
        let store = TraceStore::new();
        let trace_id = [1u8; 16];

        store.record_event(MessageEvent::sent(trace_id, "alice".into(), "Hello"));
        assert_eq!(store.trace_count(), 1);

        store.clear_trace(&trace_id);
        assert_eq!(store.trace_count(), 0);
    }
}
