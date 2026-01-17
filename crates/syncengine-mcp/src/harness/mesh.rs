//! Mesh topology management for test networks

use super::TestNode;
use crate::error::{McpError, McpResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Mesh topology patterns
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeshTopology {
    /// Every node connected to every other node
    Full,
    /// Nodes connected in a ring (1->2->3->...->n->1)
    Ring,
    /// All nodes connected to a central hub (node 0)
    Star,
    /// Linear chain (1->2->3->...->n)
    Chain,
    /// No automatic connections (manual connection required)
    None,
}

impl MeshTopology {
    /// Parse topology from string
    pub fn from_str(s: &str) -> McpResult<Self> {
        match s.to_lowercase().as_str() {
            "full" => Ok(MeshTopology::Full),
            "ring" => Ok(MeshTopology::Ring),
            "star" => Ok(MeshTopology::Star),
            "chain" | "linear" => Ok(MeshTopology::Chain),
            "none" | "manual" => Ok(MeshTopology::None),
            _ => Err(McpError::InvalidTopology(format!(
                "Unknown topology: {}. Valid options: full, ring, star, chain, none",
                s
            ))),
        }
    }
}

/// A configured mesh of test nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMesh {
    /// Mesh name
    pub name: String,
    /// Node IDs in this mesh
    pub node_ids: Vec<String>,
    /// Topology type
    pub topology: MeshTopology,
    /// Connection edges (node_a, node_b)
    pub edges: Vec<(String, String)>,
}

impl TestMesh {
    /// Create a new mesh configuration
    pub fn new(name: String, node_ids: Vec<String>, topology: MeshTopology) -> Self {
        let edges = Self::compute_edges(&node_ids, &topology);
        Self {
            name,
            node_ids,
            topology,
            edges,
        }
    }

    /// Compute edges for a given topology
    fn compute_edges(node_ids: &[String], topology: &MeshTopology) -> Vec<(String, String)> {
        let n = node_ids.len();
        if n < 2 {
            return vec![];
        }

        match topology {
            MeshTopology::Full => {
                // Every pair connected
                let mut edges = Vec::new();
                for i in 0..n {
                    for j in (i + 1)..n {
                        edges.push((node_ids[i].clone(), node_ids[j].clone()));
                    }
                }
                edges
            }
            MeshTopology::Ring => {
                // i -> i+1, with wrap-around
                let mut edges = Vec::new();
                for i in 0..n {
                    let next = (i + 1) % n;
                    edges.push((node_ids[i].clone(), node_ids[next].clone()));
                }
                edges
            }
            MeshTopology::Star => {
                // Node 0 is the hub, connected to all others
                let hub = &node_ids[0];
                node_ids
                    .iter()
                    .skip(1)
                    .map(|node| (hub.clone(), node.clone()))
                    .collect()
            }
            MeshTopology::Chain => {
                // Linear: 0 -> 1 -> 2 -> ... -> n-1
                let mut edges = Vec::new();
                for i in 0..(n - 1) {
                    edges.push((node_ids[i].clone(), node_ids[i + 1].clone()));
                }
                edges
            }
            MeshTopology::None => vec![],
        }
    }

    /// Connect nodes according to a topology
    pub async fn connect_topology(
        nodes: &[Arc<TestNode>],
        topology: &MeshTopology,
    ) -> McpResult<()> {
        let n = nodes.len();
        if n < 2 {
            return Ok(());
        }

        match topology {
            MeshTopology::Full => {
                // Connect every pair
                for i in 0..n {
                    for j in (i + 1)..n {
                        nodes[i].connect_to(&nodes[j]).await?;
                    }
                }
            }
            MeshTopology::Ring => {
                // Connect in a ring
                for i in 0..n {
                    let next = (i + 1) % n;
                    nodes[i].connect_to(&nodes[next]).await?;
                }
            }
            MeshTopology::Star => {
                // Node 0 is hub
                for node in nodes.iter().skip(1) {
                    nodes[0].connect_to(node).await?;
                }
            }
            MeshTopology::Chain => {
                // Linear chain
                for i in 0..(n - 1) {
                    nodes[i].connect_to(&nodes[i + 1]).await?;
                }
            }
            MeshTopology::None => {
                // No automatic connections
            }
        }

        tracing::info!(
            topology = ?topology,
            nodes = n,
            "Connected mesh"
        );

        Ok(())
    }

    /// Get node count
    pub fn node_count(&self) -> usize {
        self.node_ids.len()
    }

    /// Get edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check if two nodes are connected in this topology
    pub fn are_connected(&self, a: &str, b: &str) -> bool {
        self.edges.iter().any(|(x, y)| {
            (x == a && y == b) || (x == b && y == a)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_topology() {
        let nodes: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let edges = TestMesh::compute_edges(&nodes, &MeshTopology::Full);

        // 3 nodes should have 3 edges in full mesh
        assert_eq!(edges.len(), 3);
        assert!(edges.contains(&("a".into(), "b".into())));
        assert!(edges.contains(&("a".into(), "c".into())));
        assert!(edges.contains(&("b".into(), "c".into())));
    }

    #[test]
    fn test_ring_topology() {
        let nodes: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let edges = TestMesh::compute_edges(&nodes, &MeshTopology::Ring);

        // 3 nodes in ring = 3 edges
        assert_eq!(edges.len(), 3);
        assert!(edges.contains(&("a".into(), "b".into())));
        assert!(edges.contains(&("b".into(), "c".into())));
        assert!(edges.contains(&("c".into(), "a".into())));
    }

    #[test]
    fn test_star_topology() {
        let nodes: Vec<String> = vec!["hub".into(), "a".into(), "b".into(), "c".into()];
        let edges = TestMesh::compute_edges(&nodes, &MeshTopology::Star);

        // Hub connected to 3 others = 3 edges
        assert_eq!(edges.len(), 3);
        assert!(edges.iter().all(|(h, _)| h == "hub"));
    }

    #[test]
    fn test_chain_topology() {
        let nodes: Vec<String> = vec!["a".into(), "b".into(), "c".into(), "d".into()];
        let edges = TestMesh::compute_edges(&nodes, &MeshTopology::Chain);

        // 4 nodes in chain = 3 edges
        assert_eq!(edges.len(), 3);
        assert!(edges.contains(&("a".into(), "b".into())));
        assert!(edges.contains(&("b".into(), "c".into())));
        assert!(edges.contains(&("c".into(), "d".into())));
    }

    #[test]
    fn test_mesh_connectivity_check() {
        let nodes: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let mesh = TestMesh::new("test".into(), nodes, MeshTopology::Chain);

        assert!(mesh.are_connected("a", "b"));
        assert!(mesh.are_connected("b", "c"));
        assert!(!mesh.are_connected("a", "c")); // Not directly connected in chain
    }

    #[test]
    fn test_topology_from_str() {
        assert_eq!(MeshTopology::from_str("full").unwrap(), MeshTopology::Full);
        assert_eq!(MeshTopology::from_str("RING").unwrap(), MeshTopology::Ring);
        assert_eq!(MeshTopology::from_str("Star").unwrap(), MeshTopology::Star);
        assert!(MeshTopology::from_str("invalid").is_err());
    }
}
