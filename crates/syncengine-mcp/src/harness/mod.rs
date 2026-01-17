//! Test harness for managing ephemeral test nodes
//!
//! Provides tools to create isolated test networks for debugging P2P sync.

mod node;
mod mesh;

pub use node::{TestNode, NodeInfo, RealmState};
pub use mesh::{TestMesh, MeshTopology};

use crate::error::{McpError, McpResult};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Unique identifier for a test node
pub type NodeId = String;

/// Test harness managing all test nodes
pub struct TestHarness {
    /// All active test nodes
    nodes: RwLock<HashMap<NodeId, Arc<TestNode>>>,
    /// Test mesh configurations
    meshes: RwLock<HashMap<String, TestMesh>>,
    /// Counter for generating unique node names
    node_counter: RwLock<u32>,
}

impl TestHarness {
    /// Create a new test harness
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            meshes: RwLock::new(HashMap::new()),
            node_counter: RwLock::new(0),
        }
    }

    /// Create a new test node with ephemeral storage
    pub async fn create_node(&self, name: Option<String>) -> McpResult<Arc<TestNode>> {
        let name = name.unwrap_or_else(|| {
            let mut counter = self.node_counter.write();
            *counter += 1;
            format!("node_{}", *counter)
        });

        if self.nodes.read().contains_key(&name) {
            return Err(McpError::InvalidOperation(format!(
                "Node with name '{}' already exists",
                name
            )));
        }

        let node = TestNode::new(name.clone()).await?;
        let node = Arc::new(node);
        self.nodes.write().insert(name.clone(), Arc::clone(&node));

        tracing::info!(node = %name, "Created test node");
        Ok(node)
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> McpResult<Arc<TestNode>> {
        self.nodes
            .read()
            .get(id)
            .cloned()
            .ok_or_else(|| McpError::NodeNotFound(id.to_string()))
    }

    /// List all active nodes
    pub async fn list_nodes(&self) -> Vec<NodeInfo> {
        let nodes: Vec<_> = self.nodes.read().values().cloned().collect();
        let mut infos = Vec::with_capacity(nodes.len());
        for node in nodes {
            infos.push(node.info().await);
        }
        infos
    }

    /// Create a mesh of connected test nodes
    pub async fn create_mesh(
        &self,
        count: usize,
        topology: MeshTopology,
        name: Option<String>,
    ) -> McpResult<Vec<Arc<TestNode>>> {
        if count < 2 {
            return Err(McpError::InvalidTopology(
                "Mesh requires at least 2 nodes".into(),
            ));
        }

        // Create nodes
        let mut nodes = Vec::with_capacity(count);
        for i in 0..count {
            let node_name = format!("mesh_{}_{}", name.as_deref().unwrap_or("default"), i);
            let node = self.create_node(Some(node_name)).await?;
            nodes.push(node);
        }

        // Connect according to topology
        TestMesh::connect_topology(&nodes, &topology).await?;

        // Store mesh configuration
        let mesh_name = name.unwrap_or_else(|| format!("mesh_{}", self.meshes.read().len()));
        let mesh = TestMesh::new(
            mesh_name.clone(),
            nodes.iter().map(|n| n.name().to_string()).collect(),
            topology,
        );
        self.meshes.write().insert(mesh_name, mesh);

        Ok(nodes)
    }

    /// Connect two specific nodes
    pub async fn connect_nodes(&self, node_a: &str, node_b: &str) -> McpResult<()> {
        let a = self.get_node(node_a)?;
        let b = self.get_node(node_b)?;

        a.connect_to(&b).await?;
        tracing::info!(from = %node_a, to = %node_b, "Connected nodes");
        Ok(())
    }

    /// Disconnect two nodes (simulate network partition)
    pub async fn disconnect_nodes(&self, node_a: &str, node_b: &str) -> McpResult<()> {
        let a = self.get_node(node_a)?;
        let b = self.get_node(node_b)?;

        a.disconnect_from(&b).await?;
        tracing::info!(from = %node_a, to = %node_b, "Disconnected nodes");
        Ok(())
    }

    /// Create a shared realm visible to multiple nodes
    pub async fn create_shared_realm(
        &self,
        node_ids: &[&str],
        name: &str,
    ) -> McpResult<syncengine_core::RealmId> {
        if node_ids.is_empty() {
            return Err(McpError::InvalidOperation(
                "At least one node required".into(),
            ));
        }

        // Get all nodes
        let nodes: Vec<Arc<TestNode>> = node_ids
            .iter()
            .map(|id| self.get_node(id))
            .collect::<McpResult<Vec<_>>>()?;

        // Create realm on first node
        let realm_id = nodes[0].create_realm(name).await?;

        // Join realm on other nodes using invite
        if nodes.len() > 1 {
            let invite = nodes[0].generate_invite(&realm_id).await?;

            for node in nodes.iter().skip(1) {
                node.join_via_invite(&invite).await?;
            }
        }

        tracing::info!(
            realm = %hex::encode(realm_id.as_bytes()),
            nodes = ?node_ids,
            "Created shared realm"
        );
        Ok(realm_id)
    }

    /// Remove a node from the harness
    pub async fn remove_node(&self, id: &str) -> McpResult<()> {
        let node = self.nodes.write().remove(id);
        if let Some(node) = node {
            node.shutdown().await?;
            tracing::info!(node = %id, "Removed test node");
            Ok(())
        } else {
            Err(McpError::NodeNotFound(id.to_string()))
        }
    }

    /// Clean up all test nodes
    pub async fn cleanup(&self) -> McpResult<()> {
        let node_ids: Vec<String> = self.nodes.read().keys().cloned().collect();

        for id in node_ids {
            if let Err(e) = self.remove_node(&id).await {
                tracing::warn!(node = %id, error = %e, "Failed to remove node during cleanup");
            }
        }

        self.meshes.write().clear();
        tracing::info!("Test harness cleanup complete");
        Ok(())
    }

    /// Get mesh information
    pub fn get_mesh(&self, name: &str) -> Option<TestMesh> {
        self.meshes.read().get(name).cloned()
    }

    /// List all meshes
    pub fn list_meshes(&self) -> Vec<TestMesh> {
        self.meshes.read().values().cloned().collect()
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        // Note: async cleanup in drop is tricky
        // The harness should be explicitly cleaned up via cleanup()
        if !self.nodes.read().is_empty() {
            tracing::warn!(
                "TestHarness dropped with {} active nodes. Call cleanup() explicitly.",
                self.nodes.read().len()
            );
        }
    }
}
