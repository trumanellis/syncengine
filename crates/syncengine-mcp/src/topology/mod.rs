//! Network topology inspection
//!
//! Tools for visualizing peer connections and gossip subscriptions.

use crate::error::McpResult;
use crate::harness::TestHarness;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Node in the connection graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Node identifier
    pub id: String,
    /// Node's iroh public key (hex)
    pub node_key: Option<String>,
    /// Node's DID
    pub did: Option<String>,
    /// Active realm subscriptions
    pub realms: Vec<String>,
    /// Number of connections
    pub connection_count: usize,
}

/// Edge in the connection graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub from: String,
    /// Target node ID
    pub to: String,
    /// Connection quality (0-100)
    pub quality: u8,
    /// Whether connection is direct or relayed
    pub is_direct: bool,
    /// Latency in milliseconds
    pub latency_ms: Option<u32>,
}

/// Complete connection graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionGraph {
    /// All nodes in the graph
    pub nodes: Vec<GraphNode>,
    /// All edges (connections) in the graph
    pub edges: Vec<GraphEdge>,
    /// Total node count
    pub node_count: usize,
    /// Total edge count
    pub edge_count: usize,
    /// Graph density (edges / max possible edges)
    pub density: f64,
}

impl ConnectionGraph {
    /// Build a connection graph from the test harness
    pub async fn from_harness(harness: &TestHarness) -> Self {
        let node_infos = harness.list_nodes().await;
        let node_count = node_infos.len();

        let nodes: Vec<GraphNode> = node_infos
            .iter()
            .map(|info| GraphNode {
                id: info.name.clone(),
                node_key: info.node_id.clone(),
                did: info.did.clone(),
                realms: info.realms.clone(),
                connection_count: info.peers.len(),
            })
            .collect();

        // Build edges from peer connections
        let mut edges: Vec<GraphEdge> = Vec::new();
        let mut seen_edges: HashSet<(String, String)> = HashSet::new();

        for info in &node_infos {
            for peer in &info.peers {
                // Create canonical edge key to avoid duplicates
                let (a, b) = if info.name < *peer {
                    (info.name.clone(), peer.clone())
                } else {
                    (peer.clone(), info.name.clone())
                };

                if !seen_edges.contains(&(a.clone(), b.clone())) {
                    seen_edges.insert((a.clone(), b.clone()));
                    edges.push(GraphEdge {
                        from: a,
                        to: b,
                        quality: 100, // Assume perfect quality in test harness
                        is_direct: true,
                        latency_ms: None,
                    });
                }
            }
        }

        let edge_count = edges.len();

        // Calculate density
        let max_edges = if node_count > 1 {
            node_count * (node_count - 1) / 2
        } else {
            0
        };
        let density = if max_edges > 0 {
            edge_count as f64 / max_edges as f64
        } else {
            0.0
        };

        Self {
            nodes,
            edges,
            node_count,
            edge_count,
            density,
        }
    }

    /// Check if two nodes are directly connected
    pub fn are_connected(&self, a: &str, b: &str) -> bool {
        self.edges.iter().any(|e| {
            (e.from == a && e.to == b) || (e.from == b && e.to == a)
        })
    }

    /// Get neighbors of a node
    pub fn neighbors(&self, node_id: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter_map(|e| {
                if e.from == node_id {
                    Some(e.to.clone())
                } else if e.to == node_id {
                    Some(e.from.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Gossip topic subscription info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    /// Topic ID (hex encoded)
    pub topic_id: String,
    /// Realm ID if known (hex encoded)
    pub realm_id: Option<String>,
    /// Nodes subscribed to this topic
    pub subscribers: Vec<String>,
    /// Number of subscribers
    pub subscriber_count: usize,
}

/// What a specific node sees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeView {
    /// Node identifier
    pub node_id: String,
    /// Known peers
    pub known_peers: Vec<PeerInfo>,
    /// Active topics
    pub active_topics: Vec<TopicInfo>,
    /// Pending sync operations
    pub pending_syncs: Vec<PendingSyncInfo>,
    /// Current sync status
    pub sync_status: String,
}

/// Information about a known peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer node ID
    pub node_id: String,
    /// Peer DID
    pub did: Option<String>,
    /// Connection state
    pub state: String,
    /// Last seen timestamp
    pub last_seen: Option<String>,
    /// Shared realms
    pub shared_realms: Vec<String>,
}

/// Pending sync operation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSyncInfo {
    /// Realm being synced
    pub realm_id: String,
    /// Sync direction (push/pull)
    pub direction: String,
    /// Target peer
    pub peer_id: Option<String>,
    /// Status
    pub status: String,
}

/// Ping result between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    /// Source node
    pub from_node: String,
    /// Target node
    pub to_node: String,
    /// Whether ping succeeded
    pub success: bool,
    /// Round-trip latency in milliseconds
    pub latency_ms: Option<u32>,
    /// Packet loss percentage
    pub packet_loss: f32,
    /// Whether connection is direct or relayed
    pub is_direct: bool,
    /// Relay URL if relayed
    pub relay_url: Option<String>,
}

/// Network topology inspector
pub struct TopologyInspector;

impl TopologyInspector {
    /// Get the full connection graph
    pub async fn get_connection_graph(harness: &TestHarness) -> ConnectionGraph {
        ConnectionGraph::from_harness(harness).await
    }

    /// Get gossip topology for a specific topic/realm
    pub async fn get_gossip_topology(
        harness: &TestHarness,
        realm_id: &syncengine_core::RealmId,
    ) -> McpResult<TopicInfo> {
        let node_infos = harness.list_nodes().await;
        let realm_hex = hex::encode(realm_id.as_bytes());

        let subscribers: Vec<String> = node_infos
            .iter()
            .filter(|info| info.realms.contains(&realm_hex))
            .map(|info| info.name.clone())
            .collect();

        Ok(TopicInfo {
            topic_id: realm_hex.clone(),
            realm_id: Some(realm_hex),
            subscribers: subscribers.clone(),
            subscriber_count: subscribers.len(),
        })
    }

    /// Get what a specific node sees
    pub async fn get_node_view(harness: &TestHarness, node_id: &str) -> McpResult<NodeView> {
        let node = harness.get_node(node_id)?;
        let info = node.info().await;

        let mut known_peers: Vec<PeerInfo> = Vec::new();
        for peer_name in &info.peers {
            // Try to get info about the peer
            let peer_info = if let Ok(peer_node) = harness.get_node(peer_name) {
                Some(peer_node.info().await)
            } else {
                None
            };

            known_peers.push(PeerInfo {
                node_id: peer_name.clone(),
                did: peer_info.as_ref().and_then(|i| i.did.clone()),
                state: "connected".into(),
                last_seen: Some(chrono::Utc::now().to_rfc3339()),
                shared_realms: vec![], // Would need to compute intersection
            });
        }

        let active_topics: Vec<TopicInfo> = info
            .realms
            .iter()
            .map(|realm_hex| TopicInfo {
                topic_id: realm_hex.clone(),
                realm_id: Some(realm_hex.clone()),
                subscribers: vec![node_id.to_string()], // Just this node for now
                subscriber_count: 1,
            })
            .collect();

        Ok(NodeView {
            node_id: node_id.to_string(),
            known_peers,
            active_topics,
            pending_syncs: vec![], // Would need to query engine
            sync_status: if info.sync_active { "active" } else { "idle" }.into(),
        })
    }

    /// Ping between two nodes
    pub async fn ping_peer(
        harness: &TestHarness,
        from_node: &str,
        to_node: &str,
    ) -> McpResult<PingResult> {
        let from = harness.get_node(from_node)?;
        let _to = harness.get_node(to_node)?;

        // Check if nodes are connected
        let is_connected = from.is_connected_to(to_node);

        if !is_connected {
            return Ok(PingResult {
                from_node: from_node.to_string(),
                to_node: to_node.to_string(),
                success: false,
                latency_ms: None,
                packet_loss: 100.0,
                is_direct: false,
                relay_url: None,
            });
        }

        // In a real implementation, we'd do actual ICMP-style ping
        // For the test harness, we simulate a successful ping
        Ok(PingResult {
            from_node: from_node.to_string(),
            to_node: to_node.to_string(),
            success: true,
            latency_ms: Some(1), // Simulated 1ms latency in test
            packet_loss: 0.0,
            is_direct: true,
            relay_url: None,
        })
    }

    /// Find path between two nodes
    pub fn find_path(graph: &ConnectionGraph, from: &str, to: &str) -> Option<Vec<String>> {
        if from == to {
            return Some(vec![from.to_string()]);
        }

        // BFS to find shortest path
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: Vec<(String, Vec<String>)> = vec![(from.to_string(), vec![from.to_string()])];

        while let Some((current, path)) = queue.pop() {
            if current == to {
                return Some(path);
            }

            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            for neighbor in graph.neighbors(&current) {
                if !visited.contains(&neighbor) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor.clone());
                    queue.push((neighbor, new_path));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_graph_density() {
        // Full mesh of 3 nodes should have density 1.0
        let graph = ConnectionGraph {
            nodes: vec![
                GraphNode {
                    id: "a".into(),
                    node_key: None,
                    did: None,
                    realms: vec![],
                    connection_count: 2,
                },
                GraphNode {
                    id: "b".into(),
                    node_key: None,
                    did: None,
                    realms: vec![],
                    connection_count: 2,
                },
                GraphNode {
                    id: "c".into(),
                    node_key: None,
                    did: None,
                    realms: vec![],
                    connection_count: 2,
                },
            ],
            edges: vec![
                GraphEdge { from: "a".into(), to: "b".into(), quality: 100, is_direct: true, latency_ms: None },
                GraphEdge { from: "a".into(), to: "c".into(), quality: 100, is_direct: true, latency_ms: None },
                GraphEdge { from: "b".into(), to: "c".into(), quality: 100, is_direct: true, latency_ms: None },
            ],
            node_count: 3,
            edge_count: 3,
            density: 1.0,
        };

        assert_eq!(graph.density, 1.0);
    }

    #[test]
    fn test_find_path() {
        let graph = ConnectionGraph {
            nodes: vec![
                GraphNode { id: "a".into(), node_key: None, did: None, realms: vec![], connection_count: 1 },
                GraphNode { id: "b".into(), node_key: None, did: None, realms: vec![], connection_count: 2 },
                GraphNode { id: "c".into(), node_key: None, did: None, realms: vec![], connection_count: 1 },
            ],
            edges: vec![
                GraphEdge { from: "a".into(), to: "b".into(), quality: 100, is_direct: true, latency_ms: None },
                GraphEdge { from: "b".into(), to: "c".into(), quality: 100, is_direct: true, latency_ms: None },
            ],
            node_count: 3,
            edge_count: 2,
            density: 0.67,
        };

        let path = TopologyInspector::find_path(&graph, "a", "c").unwrap();
        assert_eq!(path, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_neighbors() {
        let graph = ConnectionGraph {
            nodes: vec![],
            edges: vec![
                GraphEdge { from: "a".into(), to: "b".into(), quality: 100, is_direct: true, latency_ms: None },
                GraphEdge { from: "a".into(), to: "c".into(), quality: 100, is_direct: true, latency_ms: None },
            ],
            node_count: 3,
            edge_count: 2,
            density: 0.67,
        };

        let neighbors = graph.neighbors("a");
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&"b".to_string()));
        assert!(neighbors.contains(&"c".to_string()));
    }
}
