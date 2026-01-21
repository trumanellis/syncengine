//! Test node wrapper for ephemeral SyncEngine instances

use crate::error::{McpError, McpResult};
use parking_lot::RwLock as SyncRwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use syncengine_core::{RealmId, SyncEngine, SyncEvent};
use tempfile::TempDir;
use tokio::sync::{broadcast, RwLock};

/// Information about a test node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node name/identifier
    pub name: String,
    /// Node's iroh public key (hex encoded)
    pub node_id: Option<String>,
    /// DID of the node's identity
    pub did: Option<String>,
    /// Active realms
    pub realms: Vec<String>,
    /// Connected peers
    pub peers: Vec<String>,
    /// Whether sync is running
    pub sync_active: bool,
}

/// State of a realm on a test node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmState {
    /// Realm ID (hex)
    pub realm_id: String,
    /// Realm name
    pub name: String,
    /// Number of tasks
    pub task_count: usize,
    /// Automerge document heads (hex)
    pub heads: Vec<String>,
    /// Connected peers for this realm
    pub peers: Vec<String>,
}

/// A test node with ephemeral storage
pub struct TestNode {
    /// Node name
    name: String,
    /// Temporary directory for storage
    _temp_dir: TempDir,
    /// Data directory path
    data_dir: PathBuf,
    /// The sync engine (async RwLock for async operations)
    engine: RwLock<SyncEngine>,
    /// Event receiver for sync events
    _event_rx: broadcast::Receiver<SyncEvent>,
    /// Connected peer node IDs (sync RwLock for quick access)
    connected_peers: SyncRwLock<HashMap<String, String>>, // node_name -> iroh_node_id
}

impl TestNode {
    /// Create a new test node with ephemeral storage
    pub async fn new(name: String) -> McpResult<Self> {
        let temp_dir = TempDir::new().map_err(|e| McpError::NodeCreation(e.to_string()))?;
        let data_dir = temp_dir.path().to_path_buf();

        // Create engine with ephemeral storage
        let mut engine = SyncEngine::new(&data_dir)
            .await
            .map_err(|e| McpError::NodeCreation(format!("Engine: {}", e)))?;

        // Initialize identity
        engine
            .init_identity()
            .map_err(|e| McpError::NodeCreation(format!("Identity: {}", e)))?;

        // Subscribe to events
        let event_rx = engine.subscribe_events();

        tracing::debug!(name = %name, path = ?data_dir, "Created test node");

        Ok(Self {
            name,
            _temp_dir: temp_dir,
            data_dir,
            engine: RwLock::new(engine),
            _event_rx: event_rx,
            connected_peers: SyncRwLock::new(HashMap::new()),
        })
    }

    /// Get node name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get data directory
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    /// Get node info
    pub async fn info(&self) -> NodeInfo {
        let engine = self.engine.read().await;
        let realms = engine
            .storage()
            .list_realms()
            .unwrap_or_default()
            .iter()
            .map(|r| hex::encode(r.id.as_bytes()))
            .collect();

        NodeInfo {
            name: self.name.clone(),
            node_id: engine.endpoint_id().map(|id| id.to_string()),
            did: engine.did().map(|d| d.to_string()),
            realms,
            peers: self.connected_peers.read().keys().cloned().collect(),
            sync_active: engine.is_networking_active(),
        }
    }

    /// Get read access to the engine
    pub async fn engine(&self) -> tokio::sync::RwLockReadGuard<'_, SyncEngine> {
        self.engine.read().await
    }

    /// Get write access to the engine
    pub async fn engine_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, SyncEngine> {
        self.engine.write().await
    }

    /// Create a new realm
    pub async fn create_realm(&self, name: &str) -> McpResult<RealmId> {
        let mut engine = self.engine.write().await;
        let realm_id = engine.create_realm(name).await?;
        Ok(realm_id)
    }

    /// Generate an invite for a realm
    pub async fn generate_invite(&self, realm_id: &RealmId) -> McpResult<String> {
        let mut engine = self.engine.write().await;
        // Make sure sync is started for the realm
        engine.start_sync(realm_id).await?;
        let invite = engine.generate_invite(realm_id).await?;
        Ok(invite.encode()?)
    }

    /// Join a realm via invite
    pub async fn join_via_invite(&self, invite: &str) -> McpResult<RealmId> {
        let mut engine = self.engine.write().await;
        let ticket = syncengine_core::InviteTicket::decode(invite)?;
        let realm_id = engine.join_via_invite(&ticket).await?;
        Ok(realm_id)
    }

    /// Start sync for a realm
    pub async fn start_sync(&self, realm_id: &RealmId) -> McpResult<()> {
        let mut engine = self.engine.write().await;
        engine.start_sync(realm_id).await?;
        Ok(())
    }

    /// Stop sync for a realm
    pub async fn stop_sync(&self, realm_id: &RealmId) -> McpResult<()> {
        let mut engine = self.engine.write().await;
        engine.stop_sync(realm_id).await?;
        Ok(())
    }

    /// Connect to another test node
    pub async fn connect_to(&self, other: &TestNode) -> McpResult<()> {
        let other_info = other.info().await;
        // Use node_id if available, otherwise use name as identifier (for testing without full networking)
        let other_node_id = other_info
            .node_id
            .unwrap_or_else(|| format!("test:{}", other.name));

        // In a real implementation, we'd get the NodeAddr from the other node
        // and add it to our address book. For now, we track the connection.
        self.connected_peers
            .write()
            .insert(other.name.clone(), other_node_id.clone());

        let our_info = self.info().await;
        let our_node_id = our_info
            .node_id
            .unwrap_or_else(|| format!("test:{}", self.name));
        other
            .connected_peers
            .write()
            .insert(self.name.clone(), our_node_id);

        tracing::debug!(
            from = %self.name,
            to = %other.name,
            node_id = %other_node_id,
            "Connected to peer"
        );

        Ok(())
    }

    /// Disconnect from another test node
    pub async fn disconnect_from(&self, other: &TestNode) -> McpResult<()> {
        self.connected_peers.write().remove(&other.name);
        other.connected_peers.write().remove(&self.name);

        // Note: In a full implementation, we would call into the core to
        // actually disconnect the iroh connection. For now, we just track
        // the logical connection state.

        tracing::debug!(
            from = %self.name,
            to = %other.name,
            "Disconnected from peer"
        );

        Ok(())
    }

    /// Get realm state
    pub async fn realm_state(&self, realm_id: &RealmId) -> McpResult<RealmState> {
        let engine = self.engine.read().await;

        let realm_info = engine
            .storage()
            .list_realms()
            .map_err(McpError::Core)?
            .into_iter()
            .find(|r| &r.id == realm_id)
            .ok_or_else(|| McpError::RealmNotFound(hex::encode(realm_id.as_bytes())))?;

        let tasks = engine.list_tasks(realm_id).unwrap_or_default();

        // For now, we don't have direct access to heads
        let heads = vec![];

        Ok(RealmState {
            realm_id: hex::encode(realm_id.as_bytes()),
            name: realm_info.name,
            task_count: tasks.len(),
            heads,
            peers: vec![], // Would need to query gossip
        })
    }

    /// Add a task to a realm
    pub async fn add_task(&self, realm_id: &RealmId, title: &str) -> McpResult<syncengine_core::TaskId> {
        let mut engine = self.engine.write().await;
        let task_id = engine.add_task(realm_id, title).await?;
        Ok(task_id)
    }

    /// Get all tasks in a realm
    pub async fn list_tasks(&self, realm_id: &RealmId) -> McpResult<Vec<syncengine_core::Task>> {
        let engine = self.engine.read().await;
        let tasks = engine.list_tasks(realm_id)?;
        Ok(tasks)
    }

    /// Shutdown the node
    /// Note: This doesn't call engine.shutdown() since that consumes the engine.
    /// The temp directory cleanup will happen when TestNode is dropped.
    pub async fn shutdown(&self) -> McpResult<()> {
        // Stop networking if active
        // The temp directory will be cleaned up when the node is dropped
        tracing::debug!(name = %self.name, "Shutting down test node");
        Ok(())
    }

    /// Subscribe to sync events
    pub async fn subscribe(&self) -> broadcast::Receiver<SyncEvent> {
        self.engine.read().await.subscribe_events()
    }

    /// Get connected peer count
    pub fn peer_count(&self) -> usize {
        self.connected_peers.read().len()
    }

    /// Check if connected to a specific peer
    pub fn is_connected_to(&self, peer_name: &str) -> bool {
        self.connected_peers.read().contains_key(peer_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_node() {
        let node = TestNode::new("test_node".to_string()).await.unwrap();
        assert_eq!(node.name(), "test_node");
        let info = node.info().await;
        assert_eq!(info.name, "test_node");
        assert!(info.did.is_some());
    }

    #[tokio::test]
    async fn test_create_realm() {
        let node = TestNode::new("test_node".to_string()).await.unwrap();
        let realm_id = node.create_realm("Test Realm").await.unwrap();

        let state = node.realm_state(&realm_id).await.unwrap();
        assert_eq!(state.name, "Test Realm");
        assert_eq!(state.task_count, 0);
    }

    #[tokio::test]
    async fn test_connect_nodes() {
        let node_a = TestNode::new("love".to_string()).await.unwrap();
        let node_b = TestNode::new("joy".to_string()).await.unwrap();

        node_a.connect_to(&node_b).await.unwrap();

        assert!(node_a.is_connected_to("joy"));
        assert!(node_b.is_connected_to("love"));
    }
}
