//! Error types for the MCP network debugger

use thiserror::Error;

/// Errors that can occur in the MCP network debugger
#[derive(Error, Debug)]
pub enum McpError {
    /// Node not found in test harness
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Realm not found on node
    #[error("Realm not found: {0}")]
    RealmNotFound(String),

    /// Trace not found
    #[error("Trace not found: {0}")]
    TraceNotFound(String),

    /// Failed to create test node
    #[error("Failed to create test node: {0}")]
    NodeCreation(String),

    /// Failed to connect nodes
    #[error("Failed to connect nodes: {0}")]
    ConnectionFailed(String),

    /// Core engine error
    #[error("Core engine error: {0}")]
    Core(#[from] syncengine_core::SyncError),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Harness not initialized
    #[error("Test harness not initialized")]
    HarnessNotInitialized,

    /// Invalid mesh topology
    #[error("Invalid mesh topology: {0}")]
    InvalidTopology(String),
}

/// Result type for MCP operations
pub type McpResult<T> = Result<T, McpError>;
