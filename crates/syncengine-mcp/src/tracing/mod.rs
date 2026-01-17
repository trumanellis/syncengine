//! Message tracing for debugging P2P sync
//!
//! Enables following messages through the gossip mesh to verify delivery.

mod trace;
mod events;

pub use trace::{MessageTrace, TraceResult, TraceStatus, TraceHop};
pub use events::{MessageEvent, MessageEventType, TraceStore};

use crate::error::{McpError, McpResult};
use crate::harness::TestHarness;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use syncengine_core::RealmId;

/// Trace ID for tracking messages
pub type TraceId = [u8; 16];

/// Generate a new trace ID
pub fn new_trace_id() -> TraceId {
    let mut id = [0u8; 16];
    getrandom::getrandom(&mut id).expect("Failed to generate trace ID");
    id
}

/// Message tracing manager
pub struct MessageTracer {
    /// Active traces
    traces: RwLock<HashMap<TraceId, MessageTrace>>,
    /// Event store for received events
    event_store: Arc<TraceStore>,
}

impl MessageTracer {
    /// Create a new message tracer
    pub fn new() -> Self {
        Self {
            traces: RwLock::new(HashMap::new()),
            event_store: Arc::new(TraceStore::new()),
        }
    }

    /// Send a traced message from a node
    pub async fn send_traced_message(
        &self,
        harness: &TestHarness,
        from_node: &str,
        realm_id: &RealmId,
        content: &str,
    ) -> McpResult<TraceId> {
        let trace_id = new_trace_id();

        // Get the source node
        let node = harness.get_node(from_node)?;

        // Create the trace record
        let trace = MessageTrace::new(
            trace_id,
            from_node.to_string(),
            realm_id.clone(),
            content.to_string(),
        );
        self.traces.write().insert(trace_id, trace);

        // Record the send event
        self.event_store.record_event(MessageEvent {
            trace_id,
            event_type: MessageEventType::Sent,
            node_id: from_node.to_string(),
            timestamp: chrono::Utc::now(),
            peer_id: None,
            details: Some(format!("Content: {}", content)),
        });

        // Add a task with the trace ID embedded in the title
        // This is a simple way to send a traceable message
        let trace_marker = format!("[trace:{}] {}", hex::encode(trace_id), content);
        node.add_task(realm_id, &trace_marker).await?;

        tracing::info!(
            trace_id = %hex::encode(trace_id),
            from = %from_node,
            realm = %hex::encode(realm_id.as_bytes()),
            "Sent traced message"
        );

        Ok(trace_id)
    }

    /// Get trace results for a message
    pub fn get_trace_results(&self, trace_id: &TraceId) -> McpResult<TraceResult> {
        let traces = self.traces.read();
        let trace = traces
            .get(trace_id)
            .ok_or_else(|| McpError::TraceNotFound(hex::encode(trace_id)))?;

        let events = self.event_store.get_events(trace_id);

        Ok(TraceResult::from_trace_and_events(trace, &events))
    }

    /// List all pending traces (not yet fully delivered)
    pub fn list_pending_traces(&self) -> Vec<TraceSummary> {
        self.traces
            .read()
            .iter()
            .filter(|(_, trace)| !trace.is_complete())
            .map(|(id, trace)| TraceSummary {
                trace_id: hex::encode(id),
                source_node: trace.source_node.clone(),
                realm_id: hex::encode(trace.realm_id.as_bytes()),
                status: trace.status(),
                hop_count: trace.hops.len(),
                started_at: trace.started_at.to_rfc3339(),
            })
            .collect()
    }

    /// Record a message event (called when messages are received)
    pub fn record_event(&self, event: MessageEvent) {
        // Update the trace if we have it
        if let Some(trace) = self.traces.write().get_mut(&event.trace_id) {
            trace.add_hop(TraceHop {
                node_id: event.node_id.clone(),
                event_type: event.event_type.clone(),
                timestamp: event.timestamp,
                from_peer: event.peer_id.clone(),
            });
        }

        self.event_store.record_event(event);
    }

    /// Mark a trace as complete
    pub fn complete_trace(&self, trace_id: &TraceId) {
        if let Some(trace) = self.traces.write().get_mut(trace_id) {
            trace.mark_complete();
        }
    }

    /// Clear old traces
    pub fn clear_completed(&self) {
        self.traces.write().retain(|_, trace| !trace.is_complete());
    }

    /// Get the event store for external event recording
    pub fn event_store(&self) -> Arc<TraceStore> {
        Arc::clone(&self.event_store)
    }
}

impl Default for MessageTracer {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of a trace for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSummary {
    pub trace_id: String,
    pub source_node: String,
    pub realm_id: String,
    pub status: TraceStatus,
    pub hop_count: usize,
    pub started_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_trace_id() {
        let id1 = new_trace_id();
        let id2 = new_trace_id();
        assert_ne!(id1, id2);
    }
}
