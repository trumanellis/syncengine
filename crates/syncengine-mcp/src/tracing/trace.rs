//! Message trace data structures

use super::TraceId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use syncengine_core::RealmId;

use super::events::{MessageEvent, MessageEventType};

/// Status of a message trace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceStatus {
    /// Message is still propagating
    InProgress,
    /// Message reached all expected nodes
    Complete,
    /// Message delivery timed out
    TimedOut,
    /// Message was dropped
    Dropped,
}

/// A hop in the message trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceHop {
    /// Node that processed this hop
    pub node_id: String,
    /// Type of event at this hop
    pub event_type: MessageEventType,
    /// When this hop occurred
    pub timestamp: DateTime<Utc>,
    /// Peer that sent us the message (if received)
    pub from_peer: Option<String>,
}

/// A complete message trace
#[derive(Debug, Clone)]
pub struct MessageTrace {
    /// Unique trace identifier
    pub trace_id: TraceId,
    /// Node that originated the message
    pub source_node: String,
    /// Realm the message belongs to
    pub realm_id: RealmId,
    /// Message content (for debugging)
    pub content: String,
    /// When the trace started
    pub started_at: DateTime<Utc>,
    /// All hops in the trace
    pub hops: Vec<TraceHop>,
    /// Current status
    status: TraceStatus,
    /// Expected recipient nodes
    expected_recipients: Vec<String>,
}

impl MessageTrace {
    /// Create a new trace
    pub fn new(
        trace_id: TraceId,
        source_node: String,
        realm_id: RealmId,
        content: String,
    ) -> Self {
        Self {
            trace_id,
            source_node,
            realm_id,
            content,
            started_at: Utc::now(),
            hops: Vec::new(),
            status: TraceStatus::InProgress,
            expected_recipients: Vec::new(),
        }
    }

    /// Add a hop to the trace
    pub fn add_hop(&mut self, hop: TraceHop) {
        self.hops.push(hop);
    }

    /// Set expected recipients
    pub fn set_expected_recipients(&mut self, recipients: Vec<String>) {
        self.expected_recipients = recipients;
    }

    /// Get current status
    pub fn status(&self) -> TraceStatus {
        self.status
    }

    /// Check if trace is complete
    pub fn is_complete(&self) -> bool {
        self.status == TraceStatus::Complete
    }

    /// Mark as complete
    pub fn mark_complete(&mut self) {
        self.status = TraceStatus::Complete;
    }

    /// Mark as timed out
    pub fn mark_timed_out(&mut self) {
        self.status = TraceStatus::TimedOut;
    }

    /// Get nodes that received the message
    pub fn received_by(&self) -> Vec<String> {
        self.hops
            .iter()
            .filter(|h| h.event_type == MessageEventType::Received)
            .map(|h| h.node_id.clone())
            .collect()
    }

    /// Get nodes that forwarded the message
    pub fn forwarded_by(&self) -> Vec<String> {
        self.hops
            .iter()
            .filter(|h| h.event_type == MessageEventType::Forwarded)
            .map(|h| h.node_id.clone())
            .collect()
    }

    /// Calculate total propagation time
    pub fn propagation_time_ms(&self) -> Option<i64> {
        if self.hops.is_empty() {
            return None;
        }

        let first = self.hops.first()?.timestamp;
        let last = self.hops.last()?.timestamp;
        Some((last - first).num_milliseconds())
    }
}

/// Result of a trace query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceResult {
    /// Trace ID (hex encoded)
    pub trace_id: String,
    /// Source node
    pub source_node: String,
    /// Realm ID (hex encoded)
    pub realm_id: String,
    /// Message content
    pub content: String,
    /// Current status
    pub status: TraceStatus,
    /// When trace started
    pub started_at: String,
    /// All hops
    pub hops: Vec<TraceHopResult>,
    /// Nodes that received the message
    pub delivered_to: Vec<String>,
    /// Expected recipients that didn't receive
    pub missing_from: Vec<String>,
    /// Total propagation time in milliseconds
    pub propagation_time_ms: Option<i64>,
    /// Per-node latency from source
    pub latency_per_node: Vec<NodeLatency>,
}

/// A hop result for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceHopResult {
    pub node_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub from_peer: Option<String>,
    pub latency_ms: i64,
}

/// Latency to a specific node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLatency {
    pub node_id: String,
    pub latency_ms: i64,
}

impl TraceResult {
    /// Build a trace result from a trace and its events
    pub fn from_trace_and_events(trace: &MessageTrace, _events: &[MessageEvent]) -> Self {
        let started_at = trace.started_at;

        let hops: Vec<TraceHopResult> = trace
            .hops
            .iter()
            .map(|h| TraceHopResult {
                node_id: h.node_id.clone(),
                event_type: format!("{:?}", h.event_type),
                timestamp: h.timestamp.to_rfc3339(),
                from_peer: h.from_peer.clone(),
                latency_ms: (h.timestamp - started_at).num_milliseconds(),
            })
            .collect();

        let delivered_to = trace.received_by();
        let missing_from: Vec<String> = trace
            .expected_recipients
            .iter()
            .filter(|r| !delivered_to.contains(r))
            .cloned()
            .collect();

        let latency_per_node: Vec<NodeLatency> = trace
            .hops
            .iter()
            .filter(|h| h.event_type == MessageEventType::Received)
            .map(|h| NodeLatency {
                node_id: h.node_id.clone(),
                latency_ms: (h.timestamp - started_at).num_milliseconds(),
            })
            .collect();

        Self {
            trace_id: hex::encode(trace.trace_id),
            source_node: trace.source_node.clone(),
            realm_id: hex::encode(trace.realm_id.as_bytes()),
            content: trace.content.clone(),
            status: trace.status,
            started_at: trace.started_at.to_rfc3339(),
            hops,
            delivered_to,
            missing_from,
            propagation_time_ms: trace.propagation_time_ms(),
            latency_per_node,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_creation() {
        let trace_id = [0u8; 16];
        let realm_id = RealmId::new();
        let trace = MessageTrace::new(
            trace_id,
            "alice".into(),
            realm_id,
            "Hello".into(),
        );

        assert_eq!(trace.source_node, "alice");
        assert_eq!(trace.status(), TraceStatus::InProgress);
        assert!(!trace.is_complete());
    }

    #[test]
    fn test_add_hop() {
        let trace_id = [0u8; 16];
        let realm_id = RealmId::new();
        let mut trace = MessageTrace::new(
            trace_id,
            "alice".into(),
            realm_id,
            "Hello".into(),
        );

        trace.add_hop(TraceHop {
            node_id: "bob".into(),
            event_type: MessageEventType::Received,
            timestamp: Utc::now(),
            from_peer: Some("alice".into()),
        });

        assert_eq!(trace.hops.len(), 1);
        assert_eq!(trace.received_by(), vec!["bob".to_string()]);
    }

    #[test]
    fn test_mark_complete() {
        let trace_id = [0u8; 16];
        let realm_id = RealmId::new();
        let mut trace = MessageTrace::new(
            trace_id,
            "alice".into(),
            realm_id,
            "Hello".into(),
        );

        trace.mark_complete();
        assert!(trace.is_complete());
        assert_eq!(trace.status(), TraceStatus::Complete);
    }
}
