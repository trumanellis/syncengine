//! Sync event types and status tracking for multi-realm synchronization
//!
//! This module provides types for tracking the synchronization status of individual
//! realms and notifying consumers when changes arrive from peers.
//!
//! ## Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  SyncStatus: Per-realm connection state                         │
//! │  ├── Idle: Not syncing                                          │
//! │  ├── Connecting: Establishing peer connections                  │
//! │  ├── Syncing: Actively exchanging data                          │
//! │  └── Error: Sync failed with error message                      │
//! │                                                                 │
//! │  SyncEvent: Notifications about sync activity                   │
//! │  ├── RealmChanged: Remote peer made changes                     │
//! │  ├── PeerConnected: New peer joined realm                       │
//! │  ├── PeerDisconnected: Peer left realm                          │
//! │  └── SyncError: Error occurred during sync                      │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use std::fmt;

use crate::types::RealmId;

/// Debug information about a single peer connection.
#[derive(Debug, Clone, PartialEq)]
pub struct PeerDebugInfo {
    /// Short peer ID (first 8 chars for display)
    pub peer_id: String,
    /// Full peer ID (for copying)
    pub peer_id_full: String,
    /// Whether this peer is currently connected
    pub is_connected: bool,
    /// How long this peer has been connected (seconds), if connected
    pub connection_duration_secs: Option<u64>,
}

/// Debug information about the network state for a realm.
/// Used by UI to show detailed sync status in a debug dropdown.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NetworkDebugInfo {
    /// Our node's public key (hex string, first 8 chars for display)
    pub node_id: String,
    /// Full node ID (for copying)
    pub node_id_full: String,
    /// Current sync status
    pub status: SyncStatus,
    /// Number of bootstrap peers configured for reconnection
    pub bootstrap_peer_count: usize,
    /// Whether this realm is shared (P2P enabled)
    pub is_shared: bool,
    /// Whether sync is currently active (listener running)
    pub sync_active: bool,
    /// Last error message, if any
    pub last_error: Option<String>,
    /// List of connected peer IDs (short form) - DEPRECATED, use peers instead
    pub connected_peers: Vec<String>,
    /// Detailed peer information
    pub peers: Vec<PeerDebugInfo>,
}

/// Status of synchronization for a realm
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    /// Realm is not currently syncing
    Idle,
    /// Establishing connections to peers
    Connecting,
    /// Actively syncing with peers
    Syncing {
        /// Number of connected peers
        peer_count: usize,
    },
    /// Sync encountered an error
    Error(String),
}

impl Default for SyncStatus {
    fn default() -> Self {
        SyncStatus::Idle
    }
}

impl fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncStatus::Idle => write!(f, "Idle"),
            SyncStatus::Connecting => write!(f, "Connecting"),
            SyncStatus::Syncing { peer_count } => write!(f, "Syncing ({} peers)", peer_count),
            SyncStatus::Error(msg) => write!(f, "Error: {}", msg),
        }
    }
}

/// Events emitted during synchronization
#[derive(Debug, Clone)]
pub enum SyncEvent {
    /// Remote peer made changes to a realm's document
    RealmChanged {
        /// The realm that was changed
        realm_id: RealmId,
        /// Number of changes applied
        changes_applied: usize,
    },
    /// A new peer connected to a realm's sync topic
    PeerConnected {
        /// The realm the peer connected to
        realm_id: RealmId,
        /// The peer's public key (as hex string for now)
        peer_id: String,
    },
    /// A peer disconnected from a realm's sync topic
    PeerDisconnected {
        /// The realm the peer disconnected from
        realm_id: RealmId,
        /// The peer's public key (as hex string for now)
        peer_id: String,
    },
    /// Sync status changed for a realm
    StatusChanged {
        /// The realm whose status changed
        realm_id: RealmId,
        /// The new sync status
        status: SyncStatus,
    },
    /// An error occurred during sync
    SyncError {
        /// The realm where the error occurred (if known)
        realm_id: Option<RealmId>,
        /// Error message
        message: String,
    },
}

impl SyncEvent {
    /// Get the realm ID associated with this event, if any
    pub fn realm_id(&self) -> Option<&RealmId> {
        match self {
            SyncEvent::RealmChanged { realm_id, .. } => Some(realm_id),
            SyncEvent::PeerConnected { realm_id, .. } => Some(realm_id),
            SyncEvent::PeerDisconnected { realm_id, .. } => Some(realm_id),
            SyncEvent::StatusChanged { realm_id, .. } => Some(realm_id),
            SyncEvent::SyncError { realm_id, .. } => realm_id.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_default_is_idle() {
        let status: SyncStatus = Default::default();
        assert_eq!(status, SyncStatus::Idle);
    }

    #[test]
    fn test_sync_status_display() {
        assert_eq!(format!("{}", SyncStatus::Idle), "Idle");
        assert_eq!(format!("{}", SyncStatus::Connecting), "Connecting");
        assert_eq!(
            format!("{}", SyncStatus::Syncing { peer_count: 3 }),
            "Syncing (3 peers)"
        );
        assert_eq!(
            format!("{}", SyncStatus::Error("Connection lost".to_string())),
            "Error: Connection lost"
        );
    }

    #[test]
    fn test_sync_status_equality() {
        assert_eq!(SyncStatus::Idle, SyncStatus::Idle);
        assert_eq!(SyncStatus::Connecting, SyncStatus::Connecting);
        assert_eq!(
            SyncStatus::Syncing { peer_count: 2 },
            SyncStatus::Syncing { peer_count: 2 }
        );
        assert_ne!(
            SyncStatus::Syncing { peer_count: 2 },
            SyncStatus::Syncing { peer_count: 3 }
        );
    }

    #[test]
    fn test_sync_event_realm_id() {
        let realm_id = RealmId::new();

        let event = SyncEvent::RealmChanged {
            realm_id: realm_id.clone(),
            changes_applied: 5,
        };
        assert_eq!(event.realm_id(), Some(&realm_id));

        let event = SyncEvent::SyncError {
            realm_id: None,
            message: "Network down".to_string(),
        };
        assert_eq!(event.realm_id(), None);
    }
}
