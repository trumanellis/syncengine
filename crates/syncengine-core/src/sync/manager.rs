//! Background sync manager for coordinating multiple realm synchronizations
//!
//! The `SyncManager` handles concurrent synchronization of multiple realms,
//! tracking status per-realm and emitting events when changes arrive from peers.
//!
//! ## Features
//!
//! - Start/stop sync for individual realms without blocking
//! - Track per-realm sync status (Idle, Connecting, Syncing, Error)
//! - Emit events via broadcast channel for UI updates
//! - Handle reconnection on temporary network failures
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  SyncManager                                                     │
//! │  ├── realm_tasks: HashMap<RealmId, JoinHandle>                  │
//! │  │   └── Background task per syncing realm                      │
//! │  ├── realm_status: HashMap<RealmId, SyncStatus>                 │
//! │  │   └── Current status per realm                               │
//! │  └── event_tx: broadcast::Sender<SyncEvent>                     │
//! │      └── Broadcasts events to all listeners                     │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use super::events::{SyncEvent, SyncStatus};
use super::TopicHandle;
use crate::types::RealmId;

/// Default capacity for the event broadcast channel
const EVENT_CHANNEL_CAPACITY: usize = 256;

/// State for a single realm's sync task
struct RealmSyncState {
    /// Handle to the background sync task
    task_handle: JoinHandle<()>,
    /// Current sync status
    status: SyncStatus,
    /// Handle to the gossip topic
    #[allow(dead_code)]
    topic_handle: Arc<TopicHandle>,
}

/// Manager for coordinating multiple concurrent realm syncs
///
/// The SyncManager runs background tasks for each syncing realm and provides
/// a unified interface for status queries and event subscriptions.
///
/// # Example
///
/// ```ignore
/// let manager = SyncManager::new();
///
/// // Subscribe to events
/// let mut events = manager.subscribe();
///
/// // Start syncing multiple realms
/// manager.start_realm_sync(realm1, topic_handle1).await?;
/// manager.start_realm_sync(realm2, topic_handle2).await?;
///
/// // Check status
/// assert_eq!(manager.status(&realm1).await, SyncStatus::Syncing { peer_count: 0 });
///
/// // Listen for events
/// while let Ok(event) = events.recv().await {
///     match event {
///         SyncEvent::RealmChanged { realm_id, .. } => {
///             println!("Realm {} changed!", realm_id);
///         }
///         _ => {}
///     }
/// }
/// ```
pub struct SyncManager {
    /// Per-realm sync state
    realms: Arc<RwLock<HashMap<RealmId, RealmSyncState>>>,
    /// Event broadcast channel
    event_tx: broadcast::Sender<SyncEvent>,
}

impl SyncManager {
    /// Create a new SyncManager
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self {
            realms: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Subscribe to sync events
    ///
    /// Returns a receiver that will receive all sync events (realm changes,
    /// peer connections, status changes, errors).
    ///
    /// Multiple subscribers can exist; events are broadcast to all.
    pub fn subscribe(&self) -> broadcast::Receiver<SyncEvent> {
        self.event_tx.subscribe()
    }

    /// Get the current sync status for a realm
    ///
    /// Returns `SyncStatus::Idle` if the realm is not currently syncing.
    pub async fn status(&self, realm_id: &RealmId) -> SyncStatus {
        let realms = self.realms.read().await;
        realms
            .get(realm_id)
            .map(|s| s.status.clone())
            .unwrap_or(SyncStatus::Idle)
    }

    /// Check if a realm is currently syncing
    pub async fn is_syncing(&self, realm_id: &RealmId) -> bool {
        let realms = self.realms.read().await;
        realms.contains_key(realm_id)
    }

    /// Get the number of realms currently syncing
    pub async fn syncing_count(&self) -> usize {
        let realms = self.realms.read().await;
        realms.len()
    }

    /// List all currently syncing realm IDs
    pub async fn syncing_realms(&self) -> Vec<RealmId> {
        let realms = self.realms.read().await;
        realms.keys().cloned().collect()
    }

    /// Start syncing a realm
    ///
    /// Spawns a background task to handle incoming messages on the topic.
    /// If the realm is already syncing, this is a no-op.
    ///
    /// # Arguments
    ///
    /// * `realm_id` - The realm to start syncing
    /// * `topic_handle` - Handle to the gossip topic for this realm
    ///
    /// # Returns
    ///
    /// `true` if sync was started, `false` if already syncing
    pub async fn start_realm_sync(&self, realm_id: RealmId, topic_handle: TopicHandle) -> bool {
        // Check if already syncing
        {
            let realms = self.realms.read().await;
            if realms.contains_key(&realm_id) {
                debug!(%realm_id, "Realm already syncing");
                return false;
            }
        }

        info!(%realm_id, "Starting realm sync");

        let topic_handle = Arc::new(topic_handle);
        let topic_handle_for_task = topic_handle.clone();
        let event_tx = self.event_tx.clone();
        let realms_ref = self.realms.clone();
        let realm_id_clone = realm_id.clone();

        // Emit status change event
        let _ = self.event_tx.send(SyncEvent::StatusChanged {
            realm_id: realm_id.clone(),
            status: SyncStatus::Connecting,
        });

        // Spawn background task for this realm
        let task_handle = tokio::spawn(async move {
            Self::realm_sync_task(realm_id_clone, topic_handle_for_task, event_tx, realms_ref)
                .await;
        });

        // Store state
        {
            let mut realms = self.realms.write().await;
            realms.insert(
                realm_id.clone(),
                RealmSyncState {
                    task_handle,
                    status: SyncStatus::Connecting,
                    topic_handle,
                },
            );
        }

        true
    }

    /// Stop syncing a realm
    ///
    /// Cancels the background sync task and removes the realm from tracking.
    /// If the realm is not syncing, this is a no-op.
    ///
    /// # Returns
    ///
    /// `true` if sync was stopped, `false` if not syncing
    pub async fn stop_realm_sync(&self, realm_id: &RealmId) -> bool {
        let state = {
            let mut realms = self.realms.write().await;
            realms.remove(realm_id)
        };

        if let Some(state) = state {
            info!(%realm_id, "Stopping realm sync");
            state.task_handle.abort();

            // Emit status change event
            let _ = self.event_tx.send(SyncEvent::StatusChanged {
                realm_id: realm_id.clone(),
                status: SyncStatus::Idle,
            });

            true
        } else {
            debug!(%realm_id, "Realm not syncing");
            false
        }
    }

    /// Update the sync status for a realm
    async fn update_status(
        realms: &Arc<RwLock<HashMap<RealmId, RealmSyncState>>>,
        realm_id: &RealmId,
        status: SyncStatus,
        event_tx: &broadcast::Sender<SyncEvent>,
    ) {
        let mut realms = realms.write().await;
        if let Some(state) = realms.get_mut(realm_id) {
            if state.status != status {
                state.status = status.clone();
                let _ = event_tx.send(SyncEvent::StatusChanged {
                    realm_id: realm_id.clone(),
                    status,
                });
            }
        }
    }

    /// Background task for handling a single realm's sync
    async fn realm_sync_task(
        realm_id: RealmId,
        topic_handle: Arc<TopicHandle>,
        event_tx: broadcast::Sender<SyncEvent>,
        realms: Arc<RwLock<HashMap<RealmId, RealmSyncState>>>,
    ) {
        debug!(%realm_id, "Realm sync task started");

        let peer_count: usize = 0;

        // Update to syncing status once we start receiving
        Self::update_status(
            &realms,
            &realm_id,
            SyncStatus::Syncing { peer_count },
            &event_tx,
        )
        .await;

        // Main message loop
        loop {
            tokio::select! {
                // Poll for incoming messages
                msg = topic_handle.recv() => {
                    match msg {
                        Some(gossip_msg) => {
                            debug!(
                                %realm_id,
                                from = ?gossip_msg.from,
                                bytes = gossip_msg.content.len(),
                                "Received gossip message"
                            );

                            // Emit realm changed event
                            // Note: The actual message processing (decryption, applying changes)
                            // is done by the SyncEngine, not here. We just notify that data arrived.
                            let _ = event_tx.send(SyncEvent::RealmChanged {
                                realm_id: realm_id.clone(),
                                changes_applied: 1, // Placeholder - actual count determined by caller
                            });
                        }
                        None => {
                            // Topic subscription closed
                            warn!(%realm_id, "Topic subscription closed");
                            let _ = event_tx.send(SyncEvent::SyncError {
                                realm_id: Some(realm_id.clone()),
                                message: "Topic subscription closed unexpectedly".to_string(),
                            });
                            Self::update_status(
                                &realms,
                                &realm_id,
                                SyncStatus::Error("Connection lost".to_string()),
                                &event_tx,
                            )
                            .await;
                            break;
                        }
                    }
                }
            }
        }

        debug!(%realm_id, "Realm sync task ended");
    }

    /// Gracefully shutdown all realm syncs
    pub async fn shutdown(self) {
        info!("Shutting down SyncManager");
        let realms = self.realms.write().await;
        for (realm_id, state) in realms.iter() {
            debug!(%realm_id, "Aborting sync task");
            state.task_handle.abort();
        }
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_manager_creates() {
        let manager = SyncManager::new();
        assert_eq!(manager.syncing_count().await, 0);
    }

    #[tokio::test]
    async fn test_status_returns_idle_for_unknown_realm() {
        let manager = SyncManager::new();
        let realm_id = RealmId::new();
        assert_eq!(manager.status(&realm_id).await, SyncStatus::Idle);
    }

    #[tokio::test]
    async fn test_is_syncing_returns_false_for_unknown_realm() {
        let manager = SyncManager::new();
        let realm_id = RealmId::new();
        assert!(!manager.is_syncing(&realm_id).await);
    }

    #[tokio::test]
    async fn test_syncing_realms_returns_empty_initially() {
        let manager = SyncManager::new();
        let realms = manager.syncing_realms().await;
        assert!(realms.is_empty());
    }

    #[tokio::test]
    async fn test_subscribe_returns_receiver() {
        let manager = SyncManager::new();
        let _rx = manager.subscribe();
        // Just verify we can create a receiver
    }

    #[tokio::test]
    async fn test_stop_unknown_realm_returns_false() {
        let manager = SyncManager::new();
        let realm_id = RealmId::new();
        assert!(!manager.stop_realm_sync(&realm_id).await);
    }

    // Note: Tests that involve actual TopicHandle require gossip infrastructure
    // and are tested in the integration tests (multi_realm_sync.rs)
}
