//! Delivery verification tools
//!
//! Confirm message arrival and check sync consistency across nodes.

use crate::error::McpResult;
use crate::harness::TestHarness;
use crate::tracing::{MessageTracer, TraceId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use syncengine_core::RealmId;

/// Result of verifying message delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReport {
    /// Trace ID or message ID (hex)
    pub message_id: String,
    /// Nodes that received the message
    pub delivered_to: Vec<String>,
    /// Expected nodes that didn't receive
    pub missing_from: Vec<String>,
    /// Nodes that partially received (had errors)
    pub partial_receives: Vec<PartialReceive>,
    /// Overall delivery success rate
    pub success_rate: f64,
    /// Whether all expected nodes received
    pub complete: bool,
}

/// Information about a partial receive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialReceive {
    /// Node that had partial receive
    pub node_id: String,
    /// Error message
    pub error: String,
    /// What part was received
    pub received_portion: Option<String>,
}

/// Comparison of realm state across nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmStateComparison {
    /// Realm ID (hex)
    pub realm_id: String,
    /// Nodes compared
    pub nodes: Vec<String>,
    /// Head hashes per node
    pub heads_per_node: HashMap<String, Vec<String>>,
    /// Whether all nodes are in sync
    pub in_sync: bool,
    /// Divergence points if not in sync
    pub divergence: Option<DivergenceInfo>,
    /// Task count per node
    pub task_counts: HashMap<String, usize>,
    /// Last modified timestamps per node
    pub last_modified: HashMap<String, String>,
}

/// Information about where nodes diverged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceInfo {
    /// Groups of nodes with same state
    pub state_groups: Vec<StateGroup>,
    /// Common ancestor heads (if any)
    pub common_ancestor: Vec<String>,
    /// Estimated number of divergent changes
    pub divergent_changes: usize,
}

/// Group of nodes with identical state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateGroup {
    /// Nodes in this group
    pub nodes: Vec<String>,
    /// Heads for this group
    pub heads: Vec<String>,
    /// Task count for this group
    pub task_count: usize,
}

/// Missing messages on a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageGaps {
    /// Node being checked
    pub node_id: String,
    /// Realm ID (hex)
    pub realm_id: String,
    /// Message IDs present on peers but not here
    pub missing_messages: Vec<String>,
    /// Peers that have these messages
    pub available_from: HashMap<String, Vec<String>>,
    /// Total missing count
    pub missing_count: usize,
}

/// Delivery verification service
pub struct DeliveryVerifier;

impl DeliveryVerifier {
    /// Verify that a message was delivered to expected nodes
    pub fn verify_delivery(
        tracer: &MessageTracer,
        trace_id: &TraceId,
        expected_nodes: &[&str],
    ) -> McpResult<DeliveryReport> {
        let trace_result = tracer.get_trace_results(trace_id)?;

        let expected: HashSet<&str> = expected_nodes.iter().copied().collect();
        let delivered: HashSet<String> = trace_result.delivered_to.iter().cloned().collect();

        let missing_from: Vec<String> = expected
            .iter()
            .filter(|n| !delivered.contains(**n))
            .map(|n| n.to_string())
            .collect();

        let success_rate = if expected.is_empty() {
            1.0
        } else {
            delivered.len() as f64 / expected.len() as f64
        };

        Ok(DeliveryReport {
            message_id: trace_result.trace_id,
            delivered_to: trace_result.delivered_to,
            missing_from: missing_from.clone(),
            partial_receives: vec![], // Would be populated from trace errors
            success_rate,
            complete: missing_from.is_empty(),
        })
    }

    /// Compare realm state across multiple nodes
    pub async fn compare_realm_state(
        harness: &TestHarness,
        realm_id: &RealmId,
        node_ids: &[&str],
    ) -> McpResult<RealmStateComparison> {
        let realm_hex = hex::encode(realm_id.as_bytes());
        let mut heads_per_node: HashMap<String, Vec<String>> = HashMap::new();
        let mut task_counts: HashMap<String, usize> = HashMap::new();
        let mut last_modified: HashMap<String, String> = HashMap::new();

        for node_id in node_ids {
            let node = harness.get_node(node_id)?;
            let state = node.realm_state(realm_id).await?;

            heads_per_node.insert(node_id.to_string(), state.heads.clone());
            task_counts.insert(node_id.to_string(), state.task_count);
            last_modified.insert(node_id.to_string(), chrono::Utc::now().to_rfc3339());
        }

        // Check if all nodes have the same heads
        let all_heads: Vec<&Vec<String>> = heads_per_node.values().collect();
        let in_sync = if all_heads.is_empty() {
            true
        } else {
            all_heads.iter().all(|h| *h == all_heads[0])
        };

        let divergence = if in_sync {
            None
        } else {
            Some(Self::compute_divergence(&heads_per_node, &task_counts))
        };

        Ok(RealmStateComparison {
            realm_id: realm_hex,
            nodes: node_ids.iter().map(|s| s.to_string()).collect(),
            heads_per_node,
            in_sync,
            divergence,
            task_counts,
            last_modified,
        })
    }

    /// Find messages present on peers but missing from a node
    pub async fn find_message_gaps(
        harness: &TestHarness,
        realm_id: &RealmId,
        node_id: &str,
    ) -> McpResult<MessageGaps> {
        let realm_hex = hex::encode(realm_id.as_bytes());
        let node = harness.get_node(node_id)?;
        let node_tasks = node.list_tasks(realm_id).await?;
        let node_task_ids: HashSet<String> = node_tasks
            .iter()
            .map(|t| t.id.to_string())
            .collect();

        let mut missing_messages: Vec<String> = Vec::new();
        let mut available_from: HashMap<String, Vec<String>> = HashMap::new();

        // Check all connected peers
        let info = node.info().await;
        for peer_name in &info.peers {
            if let Ok(peer_node) = harness.get_node(peer_name) {
                if let Ok(peer_tasks) = peer_node.list_tasks(realm_id).await {
                    for task in peer_tasks {
                        let task_id = task.id.to_string();
                        if !node_task_ids.contains(&task_id) {
                            if !missing_messages.contains(&task_id) {
                                missing_messages.push(task_id.clone());
                            }
                            available_from
                                .entry(task_id)
                                .or_insert_with(Vec::new)
                                .push(peer_name.clone());
                        }
                    }
                }
            }
        }

        Ok(MessageGaps {
            node_id: node_id.to_string(),
            realm_id: realm_hex,
            missing_count: missing_messages.len(),
            missing_messages,
            available_from,
        })
    }

    /// Compute divergence information from heads
    fn compute_divergence(
        heads_per_node: &HashMap<String, Vec<String>>,
        task_counts: &HashMap<String, usize>,
    ) -> DivergenceInfo {
        // Group nodes by their heads
        let mut groups: HashMap<Vec<String>, Vec<String>> = HashMap::new();

        for (node, heads) in heads_per_node {
            let mut sorted_heads = heads.clone();
            sorted_heads.sort();
            groups
                .entry(sorted_heads)
                .or_insert_with(Vec::new)
                .push(node.clone());
        }

        let state_groups: Vec<StateGroup> = groups
            .into_iter()
            .map(|(heads, nodes)| {
                let task_count = nodes
                    .first()
                    .and_then(|n| task_counts.get(n))
                    .copied()
                    .unwrap_or(0);

                StateGroup {
                    nodes,
                    heads,
                    task_count,
                }
            })
            .collect();

        // Find common ancestor (intersection of all heads)
        let all_heads: Vec<HashSet<String>> = heads_per_node
            .values()
            .map(|v| v.iter().cloned().collect())
            .collect();

        let common_ancestor: Vec<String> = if all_heads.is_empty() {
            vec![]
        } else {
            all_heads
                .iter()
                .skip(1)
                .fold(all_heads[0].clone(), |acc, set| {
                    acc.intersection(set).cloned().collect()
                })
                .into_iter()
                .collect()
        };

        // Estimate divergent changes (simplified)
        let max_tasks = task_counts.values().max().copied().unwrap_or(0);
        let min_tasks = task_counts.values().min().copied().unwrap_or(0);
        let divergent_changes = max_tasks.saturating_sub(min_tasks);

        DivergenceInfo {
            state_groups,
            common_ancestor,
            divergent_changes,
        }
    }

    /// Wait for sync to complete between nodes
    pub async fn wait_for_sync(
        harness: &TestHarness,
        realm_id: &RealmId,
        node_ids: &[&str],
        timeout_ms: u64,
    ) -> McpResult<bool> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms);

        loop {
            let comparison = Self::compare_realm_state(harness, realm_id, node_ids).await?;

            if comparison.in_sync {
                return Ok(true);
            }

            if start.elapsed() >= timeout {
                return Ok(false);
            }

            // Poll every 50ms
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divergence_detection() {
        let mut heads_per_node = HashMap::new();
        heads_per_node.insert("love".into(), vec!["head1".into(), "head2".into()]);
        heads_per_node.insert("joy".into(), vec!["head1".into(), "head3".into()]);
        heads_per_node.insert("peace".into(), vec!["head1".into(), "head2".into()]);

        let mut task_counts = HashMap::new();
        task_counts.insert("love".into(), 5);
        task_counts.insert("joy".into(), 4);
        task_counts.insert("peace".into(), 5);

        let divergence = DeliveryVerifier::compute_divergence(&heads_per_node, &task_counts);

        // Love and Peace should be in one group, Joy in another
        assert_eq!(divergence.state_groups.len(), 2);

        // Common ancestor should be head1
        assert!(divergence.common_ancestor.contains(&"head1".to_string()));
    }

    #[test]
    fn test_in_sync_detection() {
        let mut heads_per_node: HashMap<String, Vec<String>> = HashMap::new();
        heads_per_node.insert("love".to_string(), vec!["head1".to_string()]);
        heads_per_node.insert("joy".to_string(), vec!["head1".to_string()]);

        let all_heads: Vec<&Vec<String>> = heads_per_node.values().collect();
        let in_sync = all_heads.iter().all(|h| *h == all_heads[0]);

        assert!(in_sync);
    }
}
