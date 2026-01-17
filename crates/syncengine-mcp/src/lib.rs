//! SyncEngine Network Debugger MCP Server
//!
//! An MCP (Model Context Protocol) server for debugging P2P sync and testing
//! the immune system of the Synchronicity Engine.
//!
//! ## Features
//!
//! - **Message Tracing**: Follow messages through the gossip mesh
//! - **Delivery Verification**: Confirm messages arrive at expected nodes
//! - **Network Topology**: Visualize peer connections and subscriptions
//! - **Immune System Testing**: Test rate limiting, reputation, anomaly detection
//!
//! ## Usage
//!
//! ```bash
//! # Start the MCP server
//! cargo run -p syncengine-mcp
//!
//! # Or with logging
//! RUST_LOG=debug cargo run -p syncengine-mcp
//! ```
//!
//! ## Safety Model
//!
//! | Mode | Behavior |
//! |------|----------|
//! | test_mode (default) | Ephemeral nodes, isolated storage, no real network |
//! | live_mode | Requires `--allow-live`, monitoring only, injection disabled |
//! | NEVER | Extract private keys, modify production storage |

pub mod error;
pub mod harness;
pub mod immune;
pub mod topology;
pub mod tracing;
pub mod verification;

use error::McpResult;
use harness::{MeshTopology, NodeInfo, TestHarness};
use immune::{BadBehavior, ImmuneTester};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use topology::TopologyInspector;
use tracing::{MessageTracer, TraceSummary};
use verification::DeliveryVerifier;

/// The MCP Network Debugger server
pub struct NetworkDebugger {
    /// Test harness for managing nodes
    harness: Arc<TestHarness>,
    /// Message tracer (async RwLock for async operations)
    tracer: Arc<RwLock<MessageTracer>>,
    /// Immune system tester (async RwLock because simulate_bad_behavior is async)
    immune: Arc<RwLock<ImmuneTester>>,
    /// Whether live mode is enabled
    live_mode: bool,
}

impl NetworkDebugger {
    /// Create a new network debugger in test mode
    pub fn new() -> Self {
        Self {
            harness: Arc::new(TestHarness::new()),
            tracer: Arc::new(RwLock::new(MessageTracer::new())),
            immune: Arc::new(RwLock::new(ImmuneTester::new())),
            live_mode: false,
        }
    }

    /// Create with live mode enabled
    pub fn with_live_mode() -> Self {
        Self {
            harness: Arc::new(TestHarness::new()),
            tracer: Arc::new(RwLock::new(MessageTracer::new())),
            immune: Arc::new(RwLock::new(ImmuneTester::new())),
            live_mode: true,
        }
    }

    /// Check if in live mode
    pub fn is_live_mode(&self) -> bool {
        self.live_mode
    }

    // =========================================================================
    // Test Harness Tools
    // =========================================================================

    /// Create a new test node
    pub async fn create_test_node(&self, name: Option<String>) -> McpResult<NodeInfo> {
        let node = self.harness.create_node(name).await?;
        Ok(node.info().await)
    }

    /// Create a mesh of connected nodes
    pub async fn create_mesh(
        &self,
        count: usize,
        topology: &str,
        name: Option<String>,
    ) -> McpResult<Vec<NodeInfo>> {
        let topology = MeshTopology::from_str(topology)?;
        let nodes = self.harness.create_mesh(count, topology, name).await?;
        let mut infos = Vec::with_capacity(nodes.len());
        for node in nodes {
            infos.push(node.info().await);
        }
        Ok(infos)
    }

    /// List all test nodes
    pub async fn list_nodes(&self) -> Vec<NodeInfo> {
        self.harness.list_nodes().await
    }

    /// Get state of a specific node
    pub async fn get_node_state(&self, node_id: &str) -> McpResult<NodeInfo> {
        let node = self.harness.get_node(node_id)?;
        Ok(node.info().await)
    }

    /// Connect two nodes
    pub async fn connect_nodes(&self, node_a: &str, node_b: &str) -> McpResult<()> {
        self.harness.connect_nodes(node_a, node_b).await
    }

    /// Disconnect two nodes (simulate partition)
    pub async fn disconnect_nodes(&self, node_a: &str, node_b: &str) -> McpResult<()> {
        self.harness.disconnect_nodes(node_a, node_b).await
    }

    /// Create a shared realm across nodes
    pub async fn create_shared_realm(
        &self,
        node_ids: &[&str],
        name: &str,
    ) -> McpResult<String> {
        let realm_id = self.harness.create_shared_realm(node_ids, name).await?;
        Ok(hex::encode(realm_id.as_bytes()))
    }

    /// Get realm state on a node
    pub async fn get_realm_state(
        &self,
        node_id: &str,
        realm_id: &str,
    ) -> McpResult<harness::RealmState> {
        let node = self.harness.get_node(node_id)?;
        let realm_bytes = hex::decode(realm_id)
            .map_err(|e| error::McpError::InvalidOperation(format!("Invalid realm ID: {}", e)))?;
        let realm_id = syncengine_core::RealmId::from_bytes(
            realm_bytes
                .try_into()
                .map_err(|_| error::McpError::InvalidOperation("Invalid realm ID length".into()))?,
        );
        node.realm_state(&realm_id).await
    }

    /// Clean up all test nodes
    pub async fn cleanup(&self) -> McpResult<()> {
        self.harness.cleanup().await?;
        self.tracer.write().await.clear_completed();
        self.immune.write().await.reset();
        Ok(())
    }

    // =========================================================================
    // Message Tracing Tools
    // =========================================================================

    /// Send a traced message from a node
    pub async fn send_traced_message(
        &self,
        from_node: &str,
        realm_id: &str,
        content: &str,
    ) -> McpResult<String> {
        let realm_bytes = hex::decode(realm_id)
            .map_err(|e| error::McpError::InvalidOperation(format!("Invalid realm ID: {}", e)))?;
        let realm_id = syncengine_core::RealmId::from_bytes(
            realm_bytes
                .try_into()
                .map_err(|_| error::McpError::InvalidOperation("Invalid realm ID length".into()))?,
        );

        let trace_id = self
            .tracer
            .write()
            .await
            .send_traced_message(&self.harness, from_node, &realm_id, content)
            .await?;

        Ok(hex::encode(trace_id))
    }

    /// Get trace results
    pub async fn get_trace_results(&self, trace_id: &str) -> McpResult<tracing::TraceResult> {
        let trace_bytes = hex::decode(trace_id)
            .map_err(|e| error::McpError::InvalidOperation(format!("Invalid trace ID: {}", e)))?;
        let trace_id: [u8; 16] = trace_bytes
            .try_into()
            .map_err(|_| error::McpError::InvalidOperation("Invalid trace ID length".into()))?;

        self.tracer.read().await.get_trace_results(&trace_id)
    }

    /// List pending traces
    pub async fn list_pending_traces(&self) -> Vec<TraceSummary> {
        self.tracer.read().await.list_pending_traces()
    }

    // =========================================================================
    // Network Topology Tools
    // =========================================================================

    /// Get full connection graph
    pub async fn get_connection_graph(&self) -> topology::ConnectionGraph {
        TopologyInspector::get_connection_graph(&self.harness).await
    }

    /// Get gossip topology for a realm
    pub async fn get_gossip_topology(&self, realm_id: &str) -> McpResult<topology::TopicInfo> {
        let realm_bytes = hex::decode(realm_id)
            .map_err(|e| error::McpError::InvalidOperation(format!("Invalid realm ID: {}", e)))?;
        let realm_id = syncengine_core::RealmId::from_bytes(
            realm_bytes
                .try_into()
                .map_err(|_| error::McpError::InvalidOperation("Invalid realm ID length".into()))?,
        );

        TopologyInspector::get_gossip_topology(&self.harness, &realm_id).await
    }

    /// Get what a node sees
    pub async fn get_node_view(&self, node_id: &str) -> McpResult<topology::NodeView> {
        TopologyInspector::get_node_view(&self.harness, node_id).await
    }

    /// Ping between nodes
    pub async fn ping_peer(
        &self,
        from_node: &str,
        to_node: &str,
    ) -> McpResult<topology::PingResult> {
        TopologyInspector::ping_peer(&self.harness, from_node, to_node).await
    }

    // =========================================================================
    // Delivery Verification Tools
    // =========================================================================

    /// Verify message delivery
    pub async fn verify_delivery(
        &self,
        trace_id: &str,
        expected_nodes: &[&str],
    ) -> McpResult<verification::DeliveryReport> {
        let trace_bytes = hex::decode(trace_id)
            .map_err(|e| error::McpError::InvalidOperation(format!("Invalid trace ID: {}", e)))?;
        let trace_id: [u8; 16] = trace_bytes
            .try_into()
            .map_err(|_| error::McpError::InvalidOperation("Invalid trace ID length".into()))?;

        DeliveryVerifier::verify_delivery(&*self.tracer.read().await, &trace_id, expected_nodes)
    }

    /// Compare realm state across nodes
    pub async fn compare_realm_state(
        &self,
        realm_id: &str,
        node_ids: &[&str],
    ) -> McpResult<verification::RealmStateComparison> {
        let realm_bytes = hex::decode(realm_id)
            .map_err(|e| error::McpError::InvalidOperation(format!("Invalid realm ID: {}", e)))?;
        let realm_id = syncengine_core::RealmId::from_bytes(
            realm_bytes
                .try_into()
                .map_err(|_| error::McpError::InvalidOperation("Invalid realm ID length".into()))?,
        );

        DeliveryVerifier::compare_realm_state(&self.harness, &realm_id, node_ids).await
    }

    /// Find message gaps on a node
    pub async fn find_message_gaps(
        &self,
        realm_id: &str,
        node_id: &str,
    ) -> McpResult<verification::MessageGaps> {
        let realm_bytes = hex::decode(realm_id)
            .map_err(|e| error::McpError::InvalidOperation(format!("Invalid realm ID: {}", e)))?;
        let realm_id = syncengine_core::RealmId::from_bytes(
            realm_bytes
                .try_into()
                .map_err(|_| error::McpError::InvalidOperation("Invalid realm ID length".into()))?,
        );

        DeliveryVerifier::find_message_gaps(&self.harness, &realm_id, node_id).await
    }

    // =========================================================================
    // Immune System Tools
    // =========================================================================

    /// Trigger rate limit test
    pub async fn trigger_rate_limit(
        &self,
        node_id: &str,
        peer_id: &str,
        message_count: u32,
    ) -> immune::RateLimitResult {
        self.immune
            .write()
            .await
            .trigger_rate_limit(node_id, peer_id, message_count)
    }

    /// Get peer reputation
    pub async fn get_peer_reputation(&self, node_id: &str, peer_id: &str) -> immune::PeerReputation {
        self.immune.read().await.get_peer_reputation(node_id, peer_id)
    }

    /// Simulate bad behavior
    pub async fn simulate_bad_behavior(
        &self,
        peer_id: &str,
        behavior: &str,
    ) -> McpResult<immune::BehaviorTestResult> {
        let behavior = BadBehavior::from_str(behavior)?;
        self.immune
            .write()
            .await
            .simulate_bad_behavior(&self.harness, peer_id, behavior)
            .await
    }

    /// Get quarantine list
    pub async fn get_quarantine_list(&self, node_id: &str) -> Vec<immune::QuarantineEntry> {
        self.immune.read().await.get_quarantine_list(node_id)
    }

    /// Test anomaly detection
    pub async fn test_anomaly_detection(&self, pattern: &str) -> immune::AnomalyDetectionResult {
        self.immune.read().await.test_anomaly_detection(pattern)
    }

    // =========================================================================
    // Profile Pinning Tools (for offline peer scenario testing)
    // =========================================================================

    /// Set a node's profile (display name, bio)
    pub async fn set_node_profile(
        &self,
        node_id: &str,
        display_name: &str,
        bio: Option<String>,
    ) -> McpResult<ProfileInfo> {
        let node = self.harness.get_node(node_id)?;
        let mut engine = node.engine_mut().await;

        // Get or create the profile
        let peer_id = engine.did().map(|d| d.to_string()).unwrap_or_else(|| node_id.to_string());
        let mut profile = syncengine_core::types::UserProfile::new(peer_id.clone(), display_name.to_string());
        if let Some(bio_text) = bio {
            profile.bio = bio_text;
        }

        // Save the profile
        engine.storage().save_profile(&profile)?;

        Ok(ProfileInfo {
            node_id: node_id.to_string(),
            did: engine.did().map(|d| d.to_string()),
            display_name: profile.display_name.clone(),
            bio: profile.bio.clone(),
        })
    }

    /// Sign and pin the node's own profile (required before it can be served to others)
    pub async fn sign_own_profile(&self, node_id: &str) -> McpResult<SignedProfileInfo> {
        let node = self.harness.get_node(node_id)?;
        let mut engine = node.engine_mut().await;

        let signed = engine.sign_and_pin_own_profile()?;
        let did = signed.did().to_string();

        Ok(SignedProfileInfo {
            did,
            display_name: signed.profile.display_name.clone(),
            bio: signed.profile.bio.clone(),
            is_pinned: true,
        })
    }

    /// Pin a peer's profile on a node (simulates receiving and storing a profile)
    pub async fn pin_peer_profile(
        &self,
        node_id: &str,
        signed_profile_json: &str,
        relationship: &str,
    ) -> McpResult<PinResult> {
        let node = self.harness.get_node(node_id)?;
        let mut engine = node.engine_mut().await;

        // Parse the signed profile
        let signed_profile: syncengine_core::types::SignedProfile =
            serde_json::from_str(signed_profile_json)
                .map_err(|e| error::McpError::InvalidOperation(format!("Invalid profile JSON: {}", e)))?;

        let did = signed_profile.did().to_string();

        // Parse relationship
        let rel = match relationship.to_lowercase().as_str() {
            "contact" => syncengine_core::types::PinRelationship::Contact,
            "manual" => syncengine_core::types::PinRelationship::Manual,
            _ => syncengine_core::types::PinRelationship::Manual,
        };

        engine.pin_profile(signed_profile.clone(), rel)?;

        Ok(PinResult {
            did,
            display_name: signed_profile.profile.display_name,
            pinned_by: node_id.to_string(),
            relationship: relationship.to_string(),
        })
    }

    /// Get a pinned profile from a node (simulates serving a cached profile)
    pub async fn get_pinned_profile(
        &self,
        node_id: &str,
        target_did: &str,
    ) -> McpResult<Option<PinnedProfileInfo>> {
        let node = self.harness.get_node(node_id)?;
        let engine = node.engine().await;

        if let Some(pin) = engine.get_pinned_profile(target_did)? {
            let pinned_at = chrono::DateTime::from_timestamp(pin.pinned_at, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| pin.pinned_at.to_string());
            Ok(Some(PinnedProfileInfo {
                did: pin.did.clone(),
                display_name: pin.signed_profile.profile.display_name.clone(),
                bio: pin.signed_profile.profile.bio.clone(),
                relationship: format!("{:?}", pin.relationship),
                pinned_at,
                signed_profile_json: serde_json::to_string(&pin.signed_profile)
                    .unwrap_or_else(|_| "{}".to_string()),
            }))
        } else {
            Ok(None)
        }
    }

    /// List all pinned profiles on a node
    pub async fn list_pinned_profiles(&self, node_id: &str) -> McpResult<Vec<PinnedProfileInfo>> {
        let node = self.harness.get_node(node_id)?;
        let engine = node.engine().await;

        let pins = engine.list_pinned_profiles()?;
        Ok(pins
            .into_iter()
            .map(|pin| {
                let pinned_at = chrono::DateTime::from_timestamp(pin.pinned_at, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_else(|| pin.pinned_at.to_string());
                PinnedProfileInfo {
                    did: pin.did.clone(),
                    display_name: pin.signed_profile.profile.display_name.clone(),
                    bio: pin.signed_profile.profile.bio.clone(),
                    relationship: format!("{:?}", pin.relationship),
                    pinned_at,
                    signed_profile_json: serde_json::to_string(&pin.signed_profile)
                        .unwrap_or_else(|_| "{}".to_string()),
                }
            })
            .collect())
    }

    /// Simulate profile request/response between nodes
    ///
    /// This simulates:
    /// 1. Node A requests profile of offline node B
    /// 2. Node C (who has B's profile pinned) serves it to A
    /// 3. Node A receives and optionally pins the profile
    pub async fn request_pinned_profile(
        &self,
        requester_node: &str,
        responder_node: &str,
        target_did: &str,
        pin_on_receive: bool,
    ) -> McpResult<ProfileRequestResult> {
        // Check if responder has the profile pinned
        let pinned = self.get_pinned_profile(responder_node, target_did).await?;

        if let Some(profile_info) = pinned {
            // Responder has the profile - serve it to requester
            if pin_on_receive {
                // Pin the profile on the requester's node
                self.pin_peer_profile(
                    requester_node,
                    &profile_info.signed_profile_json,
                    "manual",
                )
                .await?;
            }

            Ok(ProfileRequestResult {
                success: true,
                target_did: target_did.to_string(),
                served_by: responder_node.to_string(),
                received_by: requester_node.to_string(),
                display_name: Some(profile_info.display_name),
                pinned_by_requester: pin_on_receive,
                error: None,
            })
        } else {
            Ok(ProfileRequestResult {
                success: false,
                target_did: target_did.to_string(),
                served_by: responder_node.to_string(),
                received_by: requester_node.to_string(),
                display_name: None,
                pinned_by_requester: false,
                error: Some(format!("{} does not have profile for {} pinned", responder_node, target_did)),
            })
        }
    }

    /// Get a node's own signed profile (for sharing with other nodes)
    pub async fn get_own_signed_profile(&self, node_id: &str) -> McpResult<Option<String>> {
        let node = self.harness.get_node(node_id)?;
        let engine = node.engine().await;

        if let Some(pin) = engine.get_own_pinned_profile()? {
            Ok(Some(serde_json::to_string(&pin.signed_profile)
                .map_err(|e| error::McpError::InvalidOperation(format!("Serialization error: {}", e)))?))
        } else {
            Ok(None)
        }
    }
}

/// Profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub node_id: String,
    pub did: Option<String>,
    pub display_name: String,
    pub bio: String,
}

/// Signed profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedProfileInfo {
    pub did: String,
    pub display_name: String,
    pub bio: String,
    pub is_pinned: bool,
}

/// Pin result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinResult {
    pub did: String,
    pub display_name: String,
    pub pinned_by: String,
    pub relationship: String,
}

/// Pinned profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedProfileInfo {
    pub did: String,
    pub display_name: String,
    pub bio: String,
    pub relationship: String,
    pub pinned_at: String,
    pub signed_profile_json: String,
}

/// Result of a profile request operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRequestResult {
    pub success: bool,
    pub target_did: String,
    pub served_by: String,
    pub received_by: String,
    pub display_name: Option<String>,
    pub pinned_by_requester: bool,
    pub error: Option<String>,
}

impl Default for NetworkDebugger {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool definitions for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Get all tool definitions for MCP registration
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        // Test Harness Tools
        ToolDefinition {
            name: "create_test_node".into(),
            description: "Create an ephemeral isolated test node".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Optional name for the node"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "create_mesh".into(),
            description: "Create a multi-node network with specified topology".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "count": {
                        "type": "integer",
                        "description": "Number of nodes to create"
                    },
                    "topology": {
                        "type": "string",
                        "enum": ["full", "ring", "star", "chain", "none"],
                        "description": "Connection topology"
                    },
                    "name": {
                        "type": "string",
                        "description": "Optional mesh name"
                    }
                },
                "required": ["count", "topology"]
            }),
        },
        ToolDefinition {
            name: "list_nodes".into(),
            description: "List all test nodes in the harness".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "get_node_state".into(),
            description: "Get full state dump of a node".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "node_id": {
                        "type": "string",
                        "description": "Node identifier"
                    }
                },
                "required": ["node_id"]
            }),
        },
        ToolDefinition {
            name: "connect_nodes".into(),
            description: "Add connection between two nodes".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "node_a": { "type": "string" },
                    "node_b": { "type": "string" }
                },
                "required": ["node_a", "node_b"]
            }),
        },
        ToolDefinition {
            name: "disconnect_nodes".into(),
            description: "Remove connection between two nodes (simulate partition)".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "node_a": { "type": "string" },
                    "node_b": { "type": "string" }
                },
                "required": ["node_a", "node_b"]
            }),
        },
        ToolDefinition {
            name: "create_shared_realm".into(),
            description: "Create a realm visible to multiple nodes".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "node_ids": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Node IDs to share the realm with"
                    },
                    "name": {
                        "type": "string",
                        "description": "Realm name"
                    }
                },
                "required": ["node_ids", "name"]
            }),
        },
        ToolDefinition {
            name: "cleanup".into(),
            description: "Tear down all test nodes".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        // Message Tracing Tools
        ToolDefinition {
            name: "send_traced_message".into(),
            description: "Send a message with tracing enabled".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "from_node": { "type": "string" },
                    "realm_id": { "type": "string", "description": "Hex-encoded realm ID" },
                    "content": { "type": "string" }
                },
                "required": ["from_node", "realm_id", "content"]
            }),
        },
        ToolDefinition {
            name: "get_trace_results".into(),
            description: "Get delivery report for a traced message".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "trace_id": { "type": "string", "description": "Hex-encoded trace ID" }
                },
                "required": ["trace_id"]
            }),
        },
        ToolDefinition {
            name: "list_pending_traces".into(),
            description: "List messages still propagating".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        // Network Topology Tools
        ToolDefinition {
            name: "get_connection_graph".into(),
            description: "Get full peer connectivity map".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "get_gossip_topology".into(),
            description: "Get subscribers for a topic/realm".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "realm_id": { "type": "string" }
                },
                "required": ["realm_id"]
            }),
        },
        ToolDefinition {
            name: "get_node_view".into(),
            description: "Get what a specific node sees".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "node_id": { "type": "string" }
                },
                "required": ["node_id"]
            }),
        },
        ToolDefinition {
            name: "ping_peer".into(),
            description: "Test connectivity between nodes".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "from_node": { "type": "string" },
                    "to_node": { "type": "string" }
                },
                "required": ["from_node", "to_node"]
            }),
        },
        // Delivery Verification Tools
        ToolDefinition {
            name: "verify_delivery".into(),
            description: "Confirm message arrived at expected nodes".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "trace_id": { "type": "string" },
                    "expected_nodes": {
                        "type": "array",
                        "items": { "type": "string" }
                    }
                },
                "required": ["trace_id", "expected_nodes"]
            }),
        },
        ToolDefinition {
            name: "compare_realm_state".into(),
            description: "Check if all nodes are in sync for a realm".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "realm_id": { "type": "string" },
                    "node_ids": {
                        "type": "array",
                        "items": { "type": "string" }
                    }
                },
                "required": ["realm_id", "node_ids"]
            }),
        },
        ToolDefinition {
            name: "find_message_gaps".into(),
            description: "Find messages present on peers but not on a node".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "realm_id": { "type": "string" },
                    "node_id": { "type": "string" }
                },
                "required": ["realm_id", "node_id"]
            }),
        },
        // Immune System Tools
        ToolDefinition {
            name: "trigger_rate_limit".into(),
            description: "Force rate limit to test defensive mechanisms".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "node_id": { "type": "string" },
                    "peer_id": { "type": "string" },
                    "message_count": { "type": "integer" }
                },
                "required": ["node_id", "peer_id", "message_count"]
            }),
        },
        ToolDefinition {
            name: "get_peer_reputation".into(),
            description: "Get trust score and violation history for a peer".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "node_id": { "type": "string" },
                    "peer_id": { "type": "string" }
                },
                "required": ["node_id", "peer_id"]
            }),
        },
        ToolDefinition {
            name: "simulate_bad_behavior".into(),
            description: "Simulate spam, invalid signatures, etc. to test defenses".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "peer_id": { "type": "string" },
                    "behavior": {
                        "type": "string",
                        "enum": ["message_spam", "invalid_signatures", "malformed_messages",
                                 "replay_attack", "document_bomb", "connection_churn",
                                 "fake_peer_announcement", "invite_spam"]
                    }
                },
                "required": ["peer_id", "behavior"]
            }),
        },
        ToolDefinition {
            name: "get_quarantine_list".into(),
            description: "Get blocked peers for a node".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "node_id": { "type": "string" }
                },
                "required": ["node_id"]
            }),
        },
        ToolDefinition {
            name: "test_anomaly_detection".into(),
            description: "Feed a suspicious pattern to test anomaly detection".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string" }
                },
                "required": ["pattern"]
            }),
        },
    ]
}
