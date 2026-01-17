//! SyncEngine Network Debugger MCP Server
//!
//! Start with: `cargo run -p syncengine-mcp`
//! Or with logging: `RUST_LOG=debug cargo run -p syncengine-mcp`

use rmcp::{ServerHandler, ServiceExt, model::ServerInfo, tool};
use std::sync::Arc;
use syncengine_mcp::NetworkDebugger;
use tokio::io::{stdin, stdout};
use tracing_subscriber::EnvFilter;

/// MCP Server handler
#[derive(Clone)]
struct NetworkDebuggerServer {
    debugger: Arc<NetworkDebugger>,
}

impl NetworkDebuggerServer {
    fn new() -> Self {
        Self {
            debugger: Arc::new(NetworkDebugger::new()),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for NetworkDebuggerServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("SyncEngine Network Debugger MCP Server for debugging P2P sync and testing immune system defenses.".into()),
            ..Default::default()
        }
    }
}

#[tool(tool_box)]
impl NetworkDebuggerServer {
    #[tool(description = "Create an ephemeral isolated test node")]
    async fn create_test_node(&self, #[tool(param)] name: Option<String>) -> String {
        match self.debugger.create_test_node(name).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Create a multi-node network with specified topology (full, ring, star, chain, none)")]
    async fn create_mesh(
        &self,
        #[tool(param)] count: i64,
        #[tool(param)] topology: String,
        #[tool(param)] name: Option<String>,
    ) -> String {
        match self.debugger.create_mesh(count as usize, &topology, name).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "List all test nodes in the harness")]
    async fn list_nodes(&self) -> String {
        let result = self.debugger.list_nodes().await;
        serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }

    #[tool(description = "Get full state dump of a node")]
    async fn get_node_state(&self, #[tool(param)] node_id: String) -> String {
        match self.debugger.get_node_state(&node_id).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Add connection between two nodes")]
    async fn connect_nodes(
        &self,
        #[tool(param)] node_a: String,
        #[tool(param)] node_b: String,
    ) -> String {
        match self.debugger.connect_nodes(&node_a, &node_b).await {
            Ok(_) => r#"{"success": true}"#.to_string(),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Remove connection between two nodes (simulate partition)")]
    async fn disconnect_nodes(
        &self,
        #[tool(param)] node_a: String,
        #[tool(param)] node_b: String,
    ) -> String {
        match self.debugger.disconnect_nodes(&node_a, &node_b).await {
            Ok(_) => r#"{"success": true}"#.to_string(),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Create a realm visible to multiple nodes")]
    async fn create_shared_realm(
        &self,
        #[tool(param)] node_ids: Vec<String>,
        #[tool(param)] name: String,
    ) -> String {
        let node_refs: Vec<&str> = node_ids.iter().map(|s| s.as_str()).collect();
        match self.debugger.create_shared_realm(&node_refs, &name).await {
            Ok(result) => format!(r#"{{"realm_id": "{}"}}"#, result),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Tear down all test nodes")]
    async fn cleanup(&self) -> String {
        match self.debugger.cleanup().await {
            Ok(_) => r#"{"success": true}"#.to_string(),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Send a message with tracing enabled")]
    async fn send_traced_message(
        &self,
        #[tool(param)] from_node: String,
        #[tool(param)] realm_id: String,
        #[tool(param)] content: String,
    ) -> String {
        match self.debugger.send_traced_message(&from_node, &realm_id, &content).await {
            Ok(result) => format!(r#"{{"trace_id": "{}"}}"#, result),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Get delivery report for a traced message")]
    async fn get_trace_results(&self, #[tool(param)] trace_id: String) -> String {
        match self.debugger.get_trace_results(&trace_id).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "List messages still propagating")]
    async fn list_pending_traces(&self) -> String {
        let result = self.debugger.list_pending_traces().await;
        serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }

    #[tool(description = "Get full peer connectivity map")]
    async fn get_connection_graph(&self) -> String {
        let result = self.debugger.get_connection_graph().await;
        serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }

    #[tool(description = "Get subscribers for a topic/realm")]
    async fn get_gossip_topology(&self, #[tool(param)] realm_id: String) -> String {
        match self.debugger.get_gossip_topology(&realm_id).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Get what a specific node sees")]
    async fn get_node_view(&self, #[tool(param)] node_id: String) -> String {
        match self.debugger.get_node_view(&node_id).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Test connectivity between nodes")]
    async fn ping_peer(
        &self,
        #[tool(param)] from_node: String,
        #[tool(param)] to_node: String,
    ) -> String {
        match self.debugger.ping_peer(&from_node, &to_node).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Confirm message arrived at expected nodes")]
    async fn verify_delivery(
        &self,
        #[tool(param)] trace_id: String,
        #[tool(param)] expected_nodes: Vec<String>,
    ) -> String {
        let node_refs: Vec<&str> = expected_nodes.iter().map(|s| s.as_str()).collect();
        match self.debugger.verify_delivery(&trace_id, &node_refs).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Check if all nodes are in sync for a realm")]
    async fn compare_realm_state(
        &self,
        #[tool(param)] realm_id: String,
        #[tool(param)] node_ids: Vec<String>,
    ) -> String {
        let node_refs: Vec<&str> = node_ids.iter().map(|s| s.as_str()).collect();
        match self.debugger.compare_realm_state(&realm_id, &node_refs).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Find messages present on peers but not on a node")]
    async fn find_message_gaps(
        &self,
        #[tool(param)] realm_id: String,
        #[tool(param)] node_id: String,
    ) -> String {
        match self.debugger.find_message_gaps(&realm_id, &node_id).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Force rate limit to test defensive mechanisms")]
    async fn trigger_rate_limit(
        &self,
        #[tool(param)] node_id: String,
        #[tool(param)] peer_id: String,
        #[tool(param)] message_count: i64,
    ) -> String {
        let result = self.debugger.trigger_rate_limit(&node_id, &peer_id, message_count as u32).await;
        serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }

    #[tool(description = "Get trust score and violation history for a peer")]
    async fn get_peer_reputation(
        &self,
        #[tool(param)] node_id: String,
        #[tool(param)] peer_id: String,
    ) -> String {
        let result = self.debugger.get_peer_reputation(&node_id, &peer_id).await;
        serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }

    #[tool(description = "Simulate spam, invalid signatures, etc. to test defenses")]
    async fn simulate_bad_behavior(
        &self,
        #[tool(param)] peer_id: String,
        #[tool(param)] behavior: String,
    ) -> String {
        match self.debugger.simulate_bad_behavior(&peer_id, &behavior).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Get blocked peers for a node")]
    async fn get_quarantine_list(&self, #[tool(param)] node_id: String) -> String {
        let result = self.debugger.get_quarantine_list(&node_id).await;
        serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }

    #[tool(description = "Feed a suspicious pattern to test anomaly detection")]
    async fn test_anomaly_detection(&self, #[tool(param)] pattern: String) -> String {
        let result = self.debugger.test_anomaly_detection(&pattern).await;
        serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
    }

    // =========================================================================
    // Profile Pinning Tools
    // =========================================================================

    #[tool(description = "Set a node's profile (display name, optional bio)")]
    async fn set_node_profile(
        &self,
        #[tool(param)] node_id: String,
        #[tool(param)] display_name: String,
        #[tool(param)] bio: Option<String>,
    ) -> String {
        match self.debugger.set_node_profile(&node_id, &display_name, bio).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Sign and pin a node's own profile (required before sharing)")]
    async fn sign_own_profile(&self, #[tool(param)] node_id: String) -> String {
        match self.debugger.sign_own_profile(&node_id).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Get a node's own signed profile JSON (for sharing with others)")]
    async fn get_own_signed_profile(&self, #[tool(param)] node_id: String) -> String {
        match self.debugger.get_own_signed_profile(&node_id).await {
            Ok(Some(json)) => format!("{{\"signed_profile\": {}}}", json),
            Ok(None) => r#"{"error": "Node has not signed their profile yet. Call sign_own_profile first."}"#.to_string(),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Pin a peer's signed profile on a node (relationship: contact, manual)")]
    async fn pin_peer_profile(
        &self,
        #[tool(param)] node_id: String,
        #[tool(param)] signed_profile_json: String,
        #[tool(param)] relationship: String,
    ) -> String {
        match self.debugger.pin_peer_profile(&node_id, &signed_profile_json, &relationship).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Get a pinned profile from a node by DID")]
    async fn get_pinned_profile(
        &self,
        #[tool(param)] node_id: String,
        #[tool(param)] target_did: String,
    ) -> String {
        match self.debugger.get_pinned_profile(&node_id, &target_did).await {
            Ok(Some(result)) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Ok(None) => format!("{{\"found\": false, \"did\": \"{}\"}}", target_did),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "List all pinned profiles on a node")]
    async fn list_pinned_profiles(&self, #[tool(param)] node_id: String) -> String {
        match self.debugger.list_pinned_profiles(&node_id).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }

    #[tool(description = "Request a profile from a node that has it pinned (simulates offline peer scenario)")]
    async fn request_pinned_profile(
        &self,
        #[tool(param)] requester_node: String,
        #[tool(param)] responder_node: String,
        #[tool(param)] target_did: String,
        #[tool(param)] pin_on_receive: bool,
    ) -> String {
        match self.debugger.request_pinned_profile(&requester_node, &responder_node, &target_did, pin_on_receive).await {
            Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e)),
            Err(e) => format!("{{\"error\": \"{}\"}}", e),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging to stderr (not stdout, as stdout is for MCP communication)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting SyncEngine Network Debugger MCP Server");

    // Create the server
    let server = NetworkDebuggerServer::new();

    // Create stdio transport
    let transport = (stdin(), stdout());

    // Serve via stdio
    let service = server.serve(transport).await?;

    // Wait for shutdown
    service.waiting().await?;

    Ok(())
}
