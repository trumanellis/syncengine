//! Main SyncEngine - the primary entry point for Synchronicity Engine
//!
//! SyncEngine coordinates Storage, GossipSync, and RealmDoc for:
//! - Persistent storage of realms and tasks
//! - Automerge CRDT documents for each realm
//! - P2P synchronization via iroh-gossip
//!
//! # Example
//!
//! ```ignore
//! use syncengine_core::SyncEngine;
//!
//! let mut engine = SyncEngine::new("~/.syncengine/data").await?;
//!
//! // Create a realm
//! let realm_id = engine.create_realm("My Tasks").await?;
//!
//! // Add tasks
//! engine.add_task(&realm_id, "Build solar dehydrator").await?;
//!
//! // Start syncing with peers
//! engine.start_sync(&realm_id).await?;
//!
//! // Generate an invite for others
//! let invite = engine.generate_invite(&realm_id).await?;
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use iroh_gossip::proto::TopicId;
use rand::RngCore;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::error::SyncError;
use crate::identity::{Did, HybridKeypair, HybridPublicKey};
use crate::invite::{InviteTicket, NodeAddrBytes};
use crate::peers::{PeerInfo, PeerRegistry, PeerSource, PeerStatus};
use crate::realm::RealmDoc;
use crate::storage::Storage;
use crate::sync::{
    GossipSync, NetworkDebugInfo, SyncEnvelope, SyncEvent, SyncMessage, SyncStatus, TopicEvent,
    TopicSender,
};
use crate::types::{RealmId, RealmInfo, Task, TaskId};

/// Internal state for an open realm
struct RealmState {
    /// The Automerge document containing tasks
    doc: RealmDoc,
    /// Sender for the gossip topic (if syncing) - used for broadcasting
    topic_sender: Option<TopicSender>,
    /// Encryption key for the realm (32 bytes for ChaCha20-Poly1305)
    realm_key: [u8; 32],
}

/// Main entry point for Synchronicity Engine
///
/// SyncEngine manages:
/// - Persistent storage of realms and tasks
/// - Automerge documents for each realm
/// - P2P synchronization via iroh-gossip
///
/// # Example
///
/// ```ignore
/// use syncengine_core::SyncEngine;
///
/// let mut engine = SyncEngine::new("~/.syncengine/data").await?;
/// let realm_id = engine.create_realm("My Tasks").await?;
/// engine.add_task(&realm_id, "Build solar dehydrator").await?;
/// engine.start_sync(&realm_id).await?;
/// ```
/// Default capacity for event broadcast channel
const EVENT_CHANNEL_CAPACITY: usize = 256;

/// Incoming sync data from background listener tasks
/// Internal messages for sync coordination between listener tasks and main engine
enum SyncChannelMessage {
    /// Incoming sync data from a peer (envelope bytes to decrypt and apply)
    IncomingData {
        realm_id: RealmId,
        /// Raw envelope bytes received from gossip (not yet decrypted)
        envelope_bytes: Vec<u8>,
    },
    /// Request to broadcast our full document to peers (triggered on NeighborUp)
    BroadcastRequest { realm_id: RealmId },
}

pub struct SyncEngine {
    /// Persistent storage for realms, documents, and keys
    storage: Storage,
    /// Peer registry for tracking discovered peers
    peer_registry: Arc<PeerRegistry>,
    /// Gossip-based P2P networking (lazy-initialized)
    gossip: Option<Arc<GossipSync>>,
    /// Currently open realms with their in-memory state
    realms: HashMap<RealmId, RealmState>,
    /// Data directory path
    data_dir: PathBuf,
    /// Identity keypair (lazy-initialized)
    identity: Option<HybridKeypair>,
    /// Per-realm sync status tracking (Arc<Mutex> for thread-safe access from listener tasks)
    sync_status: Arc<Mutex<HashMap<RealmId, SyncStatus>>>,
    /// Event broadcast channel for notifying listeners of realm changes
    event_tx: broadcast::Sender<SyncEvent>,
    /// Receiver for sync messages from background listener tasks
    sync_rx: tokio::sync::mpsc::UnboundedReceiver<SyncChannelMessage>,
    /// Sender for sync messages (cloned to background tasks)
    sync_tx: tokio::sync::mpsc::UnboundedSender<SyncChannelMessage>,
}

impl SyncEngine {
    /// Create a new SyncEngine with the given data directory
    ///
    /// This will:
    /// - Create the data directory if it doesn't exist
    /// - Initialize the storage database
    /// - The gossip network is lazily initialized when first needed
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Io` if the directory cannot be created.
    /// Returns `SyncError::Database` if storage initialization fails.
    pub async fn new(data_dir: impl AsRef<Path>) -> Result<Self, SyncError> {
        let data_dir = data_dir.as_ref().to_path_buf();
        info!(?data_dir, "Initializing SyncEngine");

        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("syncengine.redb");
        let storage = Storage::new(&db_path)?;

        // Initialize peer registry using the same database connection
        let peer_registry = Arc::new(PeerRegistry::new(storage.db_handle())?);

        let (event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);

        let (sync_tx, sync_rx) = tokio::sync::mpsc::unbounded_channel();

        Ok(Self {
            storage,
            peer_registry,
            gossip: None,
            realms: HashMap::new(),
            data_dir,
            identity: None,
            sync_status: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            sync_rx,
            sync_tx,
        })
    }

    /// Get the data directory path
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Get a reference to the storage layer
    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    /// Get a reference to the peer registry
    pub fn peer_registry(&self) -> &Arc<PeerRegistry> {
        &self.peer_registry
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Identity Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Initialize identity, loading from storage or generating a new one.
    ///
    /// This should be called early in the application lifecycle to ensure
    /// the DID is available for display and signing operations.
    ///
    /// If identity already exists in storage, it is loaded.
    /// Otherwise, a new identity is generated and persisted.
    pub fn init_identity(&mut self) -> Result<(), SyncError> {
        if self.identity.is_some() {
            return Ok(());
        }

        if let Some(keypair) = self.storage.load_identity()? {
            info!("Loaded existing identity");
            self.identity = Some(keypair);
        } else {
            info!("Generating new identity");
            let keypair = HybridKeypair::generate();
            self.storage.save_identity(&keypair)?;
            self.identity = Some(keypair);
        }

        Ok(())
    }

    /// Get the DID for this node.
    ///
    /// Returns `None` if identity has not been initialized.
    /// Call `init_identity()` first to ensure identity is available.
    pub fn did(&self) -> Option<Did> {
        self.identity
            .as_ref()
            .map(|kp| Did::from_public_key(&kp.public_key()))
    }

    /// Get the public key for this node.
    ///
    /// Returns `None` if identity has not been initialized.
    pub fn public_key(&self) -> Option<HybridPublicKey> {
        self.identity.as_ref().map(|kp| kp.public_key())
    }

    /// Check if identity has been initialized.
    pub fn has_identity(&self) -> bool {
        self.identity.is_some()
    }

    /// Regenerate identity (WARNING: irreversible).
    ///
    /// This will generate a new keypair and replace the existing one.
    /// Any data signed with the old identity will no longer verify.
    ///
    /// # Errors
    ///
    /// Returns an error if storage fails.
    pub fn regenerate_identity(&mut self) -> Result<(), SyncError> {
        warn!("Regenerating identity - this is irreversible!");
        let keypair = HybridKeypair::generate();
        self.storage.save_identity(&keypair)?;
        self.identity = Some(keypair);
        info!("New identity generated");
        Ok(())
    }

    /// Export the public key in different formats.
    ///
    /// # Arguments
    ///
    /// * `format` - One of "base58", "hex", or "json"
    ///
    /// # Returns
    ///
    /// The public key encoded in the requested format, or `None` if
    /// identity has not been initialized.
    pub fn export_public_key(&self, format: &str) -> Option<String> {
        let pk = self.public_key()?;
        let bytes = pk.to_bytes();

        match format {
            "hex" => Some(hex::encode(&bytes)),
            "base58" => Some(bs58::encode(&bytes).into_string()),
            "json" => {
                let did = self.did()?;
                let json = serde_json::json!({
                    "did": did.as_str(),
                    "public_key_base58": bs58::encode(&bytes).into_string(),
                    "ed25519_fingerprint": hex::encode(&bytes[..8]),
                });
                Some(json.to_string())
            }
            _ => Some(bs58::encode(&bytes).into_string()), // default to base58
        }
    }

    /// Sign data with our identity.
    ///
    /// Uses hybrid signatures (Ed25519 + ML-DSA-65) for quantum-secure signing.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Identity` if identity has not been initialized.
    /// Call `init_identity()` first.
    pub fn sign(&self, data: &[u8]) -> Result<crate::identity::HybridSignature, SyncError> {
        let keypair = self.identity.as_ref().ok_or_else(|| {
            SyncError::Identity("Identity not initialized. Call init_identity() first.".to_string())
        })?;

        Ok(keypair.sign(data))
    }

    /// Verify a signature from a peer.
    ///
    /// Verifies that the given data was signed by the owner of the provided public key.
    ///
    /// Returns `true` if the signature is valid, `false` otherwise.
    pub fn verify(
        &self,
        public_key: &HybridPublicKey,
        data: &[u8],
        signature: &crate::identity::HybridSignature,
    ) -> bool {
        public_key.verify(data, signature)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Realm Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Create a new realm with the given name
    ///
    /// Creates the realm in storage with a new encryption key and
    /// opens it for immediate use.
    ///
    /// # Returns
    ///
    /// The ID of the newly created realm.
    pub async fn create_realm(&mut self, name: &str) -> Result<RealmId, SyncError> {
        info!(name, "Creating new realm");

        let realm_info = RealmInfo::new(name);
        let realm_id = realm_info.id.clone();

        // Generate encryption key
        let mut realm_key = [0u8; 32];
        rand::rng().fill_bytes(&mut realm_key);

        // Create empty document
        let mut doc = RealmDoc::new();

        // Save to storage
        self.storage.save_realm(&realm_info)?;
        self.storage.save_realm_key(&realm_id, &realm_key)?;
        self.storage.save_document(&realm_id, &doc.save())?;

        // Add to open realms
        self.realms.insert(
            realm_id.clone(),
            RealmState {
                doc,
                topic_sender: None,
                realm_key,
            },
        );

        debug!(%realm_id, "Realm created and opened");
        Ok(realm_id)
    }

    /// List all realms from storage
    pub async fn list_realms(&self) -> Result<Vec<RealmInfo>, SyncError> {
        self.storage.list_realms()
    }

    /// Get a realm by ID from storage
    pub async fn get_realm(&self, realm_id: &RealmId) -> Result<Option<RealmInfo>, SyncError> {
        self.storage.load_realm(realm_id)
    }

    /// Open a realm from storage for use
    ///
    /// Loads the realm's document and encryption key into memory.
    /// If the realm is already open, this is a no-op.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm doesn't exist.
    pub async fn open_realm(&mut self, realm_id: &RealmId) -> Result<(), SyncError> {
        if self.realms.contains_key(realm_id) {
            debug!(%realm_id, "Realm already open");
            return Ok(());
        }

        info!(%realm_id, "Opening realm");

        // Load realm info (needed to check if shared)
        let info = self
            .storage
            .load_realm(realm_id)?
            .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

        // Load document
        let doc_bytes = self.storage.load_document(realm_id)?;
        let doc = match doc_bytes {
            Some(bytes) => RealmDoc::load(&bytes)?,
            None => {
                // Create new document if none exists
                let mut doc = RealmDoc::new();
                self.storage.save_document(realm_id, &doc.save())?;
                doc
            }
        };

        // Load or create realm key
        let realm_key = match self.storage.load_realm_key(realm_id)? {
            Some(key) => key,
            None => {
                let mut key = [0u8; 32];
                rand::rng().fill_bytes(&mut key);
                self.storage.save_realm_key(realm_id, &key)?;
                key
            }
        };

        self.realms.insert(
            realm_id.clone(),
            RealmState {
                doc,
                topic_sender: None,
                realm_key,
            },
        );

        debug!(%realm_id, "Realm opened");

        // AUTO-START SYNC: If this is a shared realm, start P2P sync automatically
        // This ensures sync resumes after app restart for shared realms
        // Note: We call start_sync_internal to avoid circular recursion with start_sync
        if info.is_shared {
            info!(
                %realm_id,
                bootstrap_peers = info.bootstrap_peers.len(),
                "Shared realm detected, auto-starting sync"
            );
            if let Err(e) = self.start_sync_internal(realm_id).await {
                warn!(
                    %realm_id,
                    error = ?e,
                    "Failed to auto-start sync for shared realm (will retry on next open)"
                );
                // Don't fail the open_realm call - the realm is still usable locally
            }
        }

        Ok(())
    }

    /// Save a realm's document to storage
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    pub async fn save_realm(&mut self, realm_id: &RealmId) -> Result<(), SyncError> {
        let state = self
            .realms
            .get_mut(realm_id)
            .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

        let doc_bytes = state.doc.save();
        self.storage.save_document(realm_id, &doc_bytes)?;

        debug!(%realm_id, bytes = doc_bytes.len(), "Realm saved");
        Ok(())
    }

    /// Delete a realm and all its data
    pub async fn delete_realm(&mut self, realm_id: &RealmId) -> Result<(), SyncError> {
        // Remove from open realms
        self.realms.remove(realm_id);

        // Delete from storage
        self.storage.delete_realm(realm_id)?;
        info!(%realm_id, "Deleted realm");
        Ok(())
    }

    /// Check if a realm is currently open
    pub fn is_realm_open(&self, realm_id: &RealmId) -> bool {
        self.realms.contains_key(realm_id)
    }

    /// Check if a realm is currently syncing
    pub fn is_realm_syncing(&self, realm_id: &RealmId) -> bool {
        self.realms
            .get(realm_id)
            .map(|s| s.topic_sender.is_some())
            .unwrap_or(false)
    }

    /// Process any pending sync messages from background listener tasks
    ///
    /// This should be called periodically or before reading realm state
    /// to ensure all received sync data has been applied.
    ///
    /// # Returns
    ///
    /// The number of messages processed.
    pub fn process_pending_sync(&mut self) -> usize {
        let mut processed = 0;
        // Collect broadcast requests to handle after draining messages
        // (we can't broadcast while iterating because broadcast_changes_with_data is async)
        let mut broadcast_requests: Vec<RealmId> = Vec::new();

        // Drain all pending messages from the channel
        loop {
            match self.sync_rx.try_recv() {
                Ok(SyncChannelMessage::IncomingData {
                    realm_id,
                    envelope_bytes,
                }) => {
                    debug!(
                        %realm_id,
                        envelope_bytes = envelope_bytes.len(),
                        "Pulled IncomingData from channel"
                    );
                    // Try to process this incoming message
                    match self.handle_incoming(&realm_id, &envelope_bytes) {
                        Ok(Some(SyncMessage::SyncResponse { document, .. })) => {
                            // Apply the full document
                            if let Err(e) = self.apply_sync_changes(&realm_id, &document, true) {
                                warn!(%realm_id, error = ?e, "Failed to apply sync response");
                            } else {
                                debug!(%realm_id, "Applied sync response (full doc)");
                                processed += 1;
                            }
                        }
                        Ok(Some(SyncMessage::Changes { data: changes, .. })) => {
                            // Apply incremental changes
                            if let Err(e) = self.apply_sync_changes(&realm_id, &changes, false) {
                                warn!(%realm_id, error = ?e, "Failed to apply incremental changes");
                            } else {
                                debug!(%realm_id, "Applied incremental changes");
                                processed += 1;
                            }
                        }
                        Ok(Some(SyncMessage::SyncRequest {
                            realm_id: req_realm_id,
                        })) => {
                            // Peer is requesting our state - queue a broadcast
                            info!(
                                %realm_id,
                                "Received sync request from peer for realm {}",
                                req_realm_id
                            );
                            // Queue a broadcast response
                            broadcast_requests.push(req_realm_id);
                        }
                        Ok(Some(SyncMessage::Announce { sender_addr, .. })) => {
                            // Peer is announcing their state - we could compare and request sync if needed
                            debug!(%realm_id, "Received announce");

                            // If sender included their address, add it to our discovery
                            // This enables bidirectional communication when joining via invite
                            if let Some(ref addr) = sender_addr {
                                if let Ok(endpoint_addr) = addr.to_endpoint_addr() {
                                    if let Some(gossip) = self.gossip.as_ref() {
                                        debug!(
                                            %realm_id,
                                            peer = %endpoint_addr.id,
                                            "Adding peer address from announce"
                                        );
                                        gossip.add_peer_addr(endpoint_addr);
                                    }
                                }
                            }

                            // CRITICAL: Persist learned peer address to storage for reconnection after restart
                            // This fixes the asymmetry where joiners saved creator's address but creator
                            // never saved joiners' addresses, causing sync to break after restart
                            if let Some(addr) = sender_addr {
                                if let Ok(Some(mut realm_info)) = self.storage.load_realm(&realm_id)
                                {
                                    // Check if we already have this peer
                                    let already_has_peer = realm_info
                                        .bootstrap_peers
                                        .iter()
                                        .any(|p| p.node_id == addr.node_id);

                                    if !already_has_peer {
                                        info!(
                                            %realm_id,
                                            peer_node_id = ?&addr.node_id[..8],
                                            "Persisting new peer address from announce for future reconnection"
                                        );
                                        realm_info.bootstrap_peers.push(addr);
                                        if let Err(e) = self.storage.save_realm(&realm_info) {
                                            warn!(
                                                %realm_id,
                                                error = ?e,
                                                "Failed to persist peer address to storage"
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        Ok(None) => {
                            // Message failed verification - ignore
                            debug!(%realm_id, "Incoming message failed verification");
                        }
                        Err(e) => {
                            warn!(%realm_id, error = ?e, "Failed to handle incoming message");
                        }
                    }
                }
                Ok(SyncChannelMessage::BroadcastRequest { realm_id }) => {
                    // Queue broadcast request (will process after draining)
                    debug!(%realm_id, "Received broadcast request from listener (peer connected)");
                    if !broadcast_requests.contains(&realm_id) {
                        broadcast_requests.push(realm_id);
                    }
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                    // No more messages
                    break;
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                    // Channel closed
                    break;
                }
            }
        }

        // Process queued broadcast requests synchronously
        // Note: We can't call async broadcast_changes_with_data from here,
        // so we'll do a simpler synchronous broadcast via the topic sender
        for realm_id in broadcast_requests {
            if let Some(state) = self.realms.get_mut(&realm_id) {
                if let Some(ref sender) = state.topic_sender {
                    // Get the full document to broadcast
                    let doc_bytes = state.doc.save();

                    // Create signed+encrypted envelope
                    let identity = match &self.identity {
                        Some(id) => id,
                        None => {
                            warn!(%realm_id, "Cannot broadcast: no identity");
                            continue;
                        }
                    };

                    let sender_did = Did::from_public_key(&identity.public_key()).to_string();
                    let sign_fn = |data: &[u8]| identity.sign(data).to_bytes().to_vec();

                    let message = SyncMessage::SyncResponse {
                        realm_id: realm_id.clone(),
                        document: doc_bytes,
                    };

                    match SyncEnvelope::seal(&message, &sender_did, &state.realm_key, sign_fn) {
                        Ok(envelope) => {
                            match envelope.to_bytes() {
                                Ok(bytes) => {
                                    // Use blocking broadcast (sender.broadcast is async but we need sync)
                                    // Create a simple oneshot to handle this
                                    let sender_clone = sender.clone();
                                    let realm_id_clone = realm_id.clone();
                                    tokio::spawn(async move {
                                        if let Err(e) =
                                            sender_clone.broadcast(bytes::Bytes::from(bytes)).await
                                        {
                                            warn!(%realm_id_clone, error = ?e, "Failed to broadcast document on peer connect");
                                        } else {
                                            info!(%realm_id_clone, "Broadcast full document to newly connected peer");
                                        }
                                    });
                                }
                                Err(e) => {
                                    warn!(%realm_id, error = ?e, "Failed to serialize envelope");
                                }
                            }
                        }
                        Err(e) => {
                            warn!(%realm_id, error = ?e, "Failed to create envelope");
                        }
                    }
                }
            }
        }

        processed
    }

    /// Apply sync changes to a realm document (internal sync version)
    fn apply_sync_changes(
        &mut self,
        realm_id: &RealmId,
        data: &[u8],
        is_full_doc: bool,
    ) -> Result<(), SyncError> {
        let state = self
            .realms
            .get_mut(realm_id)
            .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

        if is_full_doc {
            // Full document sync handling
            let mut remote_doc = RealmDoc::load(data)?;

            // Debug: Check task counts before merge
            let remote_task_count = remote_doc.list_tasks().map(|t| t.len()).unwrap_or(0);
            let local_task_count_before = state.doc.list_tasks().map(|t| t.len()).unwrap_or(0);

            // CRITICAL: When local document has no tasks and remote has tasks,
            // we REPLACE instead of MERGE. This avoids Automerge's actor-ID-based
            // conflict resolution which can discard the remote's tasks when two
            // documents with no shared history both created the "tasks" map independently.
            //
            // This is safe because:
            // - If local is empty, there's nothing to preserve locally
            // - If remote has content, we want all of it
            // - Once we have a shared base, future merges work correctly
            if local_task_count_before == 0 && remote_task_count > 0 {
                debug!(
                    %realm_id,
                    remote_task_count,
                    "Replacing empty local doc with remote doc (no shared history)"
                );
                // Replace local document entirely with remote
                state.doc = remote_doc;
            } else {
                // Normal merge - both have history, or remote is empty
                // Automerge CRDT merge preserves all changes when documents share history
                state.doc.merge(&mut remote_doc)?;
            }

            let local_task_count_after = state.doc.list_tasks().map(|t| t.len()).unwrap_or(0);
            debug!(
                %realm_id,
                remote_task_count,
                local_task_count_before,
                local_task_count_after,
                "Merge details"
            );
        } else {
            // Incremental changes - apply sync message
            state.doc.apply_sync_message(data)?;
        }

        // Save the updated document to disk
        // This ensures sync changes persist across app restarts
        let doc_bytes = state.doc.save();
        self.storage.save_document(realm_id, &doc_bytes)?;

        // Debug: log task count after merge
        let task_count = state.doc.list_tasks().map(|t| t.len()).unwrap_or(0);
        debug!(
            %realm_id,
            bytes = data.len(),
            is_full_doc,
            saved_bytes = doc_bytes.len(),
            task_count,
            "Applied and saved sync changes"
        );
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Task Operations (with auto-save)
    // ═══════════════════════════════════════════════════════════════════════

    /// Add a task to a realm
    ///
    /// Auto-saves the realm after adding the task.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    pub async fn add_task(&mut self, realm_id: &RealmId, title: &str) -> Result<TaskId, SyncError> {
        // First, ensure realm is open (load from storage if needed)
        if !self.realms.contains_key(realm_id) {
            self.open_realm(realm_id).await?;
        }

        let (task_id, sync_data) = {
            let state = self
                .realms
                .get_mut(realm_id)
                .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

            let task_id = state.doc.add_task(title)?;

            // Capture incremental changes BEFORE save (save resets the checkpoint)
            let sync_data = state.doc.generate_sync_message();

            (task_id, sync_data)
        };

        // Auto-save
        self.save_realm(realm_id).await?;

        // Broadcast changes to peers if syncing
        if !sync_data.is_empty() {
            if let Err(e) = self.broadcast_changes_with_data(realm_id, sync_data).await {
                debug!(%realm_id, error = %e, "Failed to broadcast task addition (may not be syncing)");
            }
        }

        debug!(%realm_id, %task_id, title, "Task added");
        Ok(task_id)
    }

    /// List all tasks in a realm
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    pub fn list_tasks(&self, realm_id: &RealmId) -> Result<Vec<Task>, SyncError> {
        let state = self
            .realms
            .get(realm_id)
            .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

        state.doc.list_tasks()
    }

    /// Get a specific task
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    pub fn get_task(
        &self,
        realm_id: &RealmId,
        task_id: &TaskId,
    ) -> Result<Option<Task>, SyncError> {
        let state = self
            .realms
            .get(realm_id)
            .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

        state.doc.get_task(task_id)
    }

    /// Toggle a task's completion state
    ///
    /// Auto-opens the realm if not already open.
    /// Auto-saves the realm after toggling.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm doesn't exist.
    /// Returns `SyncError::TaskNotFound` if the task doesn't exist.
    pub async fn toggle_task(
        &mut self,
        realm_id: &RealmId,
        task_id: &TaskId,
    ) -> Result<(), SyncError> {
        // Ensure realm is open
        if !self.realms.contains_key(realm_id) {
            self.open_realm(realm_id).await?;
        }

        let sync_data = {
            let state = self
                .realms
                .get_mut(realm_id)
                .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

            state.doc.toggle_task(task_id)?;

            // Capture incremental changes BEFORE save
            state.doc.generate_sync_message()
        };

        // Auto-save
        self.save_realm(realm_id).await?;

        // Broadcast changes to peers if syncing
        if !sync_data.is_empty() {
            if let Err(e) = self.broadcast_changes_with_data(realm_id, sync_data).await {
                debug!(%realm_id, error = %e, "Failed to broadcast task toggle (may not be syncing)");
            }
        }

        debug!(%realm_id, %task_id, "Task toggled");
        Ok(())
    }

    /// Delete a task from a realm
    ///
    /// Auto-opens the realm if not already open.
    /// Auto-saves the realm after deleting.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm doesn't exist.
    pub async fn delete_task(
        &mut self,
        realm_id: &RealmId,
        task_id: &TaskId,
    ) -> Result<(), SyncError> {
        // Ensure realm is open
        if !self.realms.contains_key(realm_id) {
            self.open_realm(realm_id).await?;
        }

        let sync_data = {
            let state = self
                .realms
                .get_mut(realm_id)
                .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

            state.doc.delete_task(task_id)?;

            // Capture incremental changes BEFORE save
            state.doc.generate_sync_message()
        };

        // Auto-save
        self.save_realm(realm_id).await?;

        // Broadcast changes to peers if syncing
        if !sync_data.is_empty() {
            if let Err(e) = self.broadcast_changes_with_data(realm_id, sync_data).await {
                debug!(%realm_id, error = %e, "Failed to broadcast task deletion (may not be syncing)");
            }
        }

        debug!(%realm_id, %task_id, "Task deleted");
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // P2P Sync Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Start the P2P networking layer.
    ///
    /// This initializes the iroh-gossip networking stack, including:
    /// - QUIC endpoint for NAT traversal
    /// - Gossip protocol for topic-based pub/sub
    /// - Connection to default relay servers
    ///
    /// After calling this, the node can:
    /// - Generate invites with its own address as a bootstrap peer
    /// - Subscribe to realm topics for sync
    /// - Accept incoming connections from peers
    ///
    /// This is a no-op if networking is already started.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Network` if the gossip layer fails to initialize.
    pub async fn start_networking(&mut self) -> Result<(), SyncError> {
        self.ensure_gossip().await?;
        info!("P2P networking started");
        Ok(())
    }

    /// Check if the P2P networking layer is active.
    pub fn is_networking_active(&self) -> bool {
        self.gossip.is_some()
    }

    /// Get effective bootstrap peers by combining static peers from storage
    /// with online peers from the peer registry.
    ///
    /// This enables dynamic peer discovery - peers discovered in previous sessions
    /// automatically become bootstrap peers for reconnection.
    fn get_effective_bootstrap_peers(&self, realm_id: &RealmId) -> Result<Vec<iroh::PublicKey>, SyncError> {
        let mut peer_ids = std::collections::HashSet::new();

        // 1. Add static peers from invite/storage
        if let Some(realm_info) = self.storage.load_realm(realm_id)? {
            for peer_bytes in &realm_info.bootstrap_peers {
                match peer_bytes.to_endpoint_addr() {
                    Ok(endpoint_addr) => {
                        peer_ids.insert(endpoint_addr.id);
                    }
                    Err(e) => {
                        warn!(%realm_id, error = ?e, "Failed to parse saved peer address");
                    }
                }
            }
        }

        // 2. Add online peers from registry that share this realm
        match self.peer_registry.list_by_status(PeerStatus::Online) {
            Ok(online_peers) => {
                for peer_info in online_peers {
                    if peer_info.shared_realms.contains(realm_id) {
                        match iroh::PublicKey::from_bytes(&peer_info.endpoint_id) {
                            Ok(peer_id) => {
                                peer_ids.insert(peer_id);
                            }
                            Err(e) => {
                                warn!(error = ?e, "Failed to parse peer endpoint ID from registry");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!(error = ?e, "Failed to query online peers from registry");
            }
        }

        let peers: Vec<iroh::PublicKey> = peer_ids.into_iter().collect();
        if !peers.is_empty() {
            debug!(%realm_id, peer_count = peers.len(), "Effective bootstrap peers (static + registry)");
        }
        Ok(peers)
    }

    /// Ensure gossip networking is initialized
    async fn ensure_gossip(&mut self) -> Result<Arc<GossipSync>, SyncError> {
        if let Some(ref gossip) = self.gossip {
            return Ok(gossip.clone());
        }

        info!("Initializing gossip networking");

        // Load or generate persistent endpoint secret key
        let secret_key = match self.storage.load_endpoint_secret_key()? {
            Some(key_bytes) => {
                info!("Loaded persistent endpoint secret key from storage");
                iroh::SecretKey::from(key_bytes)
            }
            None => {
                info!("No endpoint secret key found, generating new one");
                let secret_key = iroh::SecretKey::generate(&mut rand::rng());
                let key_bytes: [u8; 32] = secret_key.to_bytes();
                self.storage.save_endpoint_secret_key(&key_bytes)?;
                info!("Saved new endpoint secret key to storage");
                secret_key
            }
        };

        let gossip = Arc::new(GossipSync::with_secret_key(Some(secret_key)).await?);
        self.gossip = Some(gossip.clone());
        Ok(gossip)
    }

    /// Start the peer reconnection background task
    ///
    /// This spawns a background task that runs every 5 minutes and attempts
    /// to reconnect to all inactive peers (offline or unknown status).
    ///
    /// The task will continue running until the engine is dropped.
    pub fn start_peer_reconnection_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                if let Err(e) = self.attempt_reconnect_inactive_peers().await {
                    tracing::warn!("Peer reconnection error: {}", e);
                }
            }
        });
        info!("Peer reconnection task started");
    }

    /// Attempt to reconnect to all inactive peers
    ///
    /// This iterates through all peers with status Offline or Unknown and
    /// attempts to establish a connection using exponential backoff.
    ///
    /// Peers are only retried if enough time has passed according to their
    /// backoff delay (5min * 2^failures, capped at 1 hour). Connection attempts
    /// and results are tracked to calculate success rates and adjust backoff.
    ///
    /// This is called automatically by the background reconnection task.
    pub async fn attempt_reconnect_inactive_peers(&self) -> Result<(), SyncError> {
        let inactive = self.peer_registry.list_inactive()?;

        if inactive.is_empty() {
            debug!("No inactive peers to reconnect");
            return Ok(());
        }

        // Skip if gossip not initialized
        let Some(ref gossip) = self.gossip else {
            debug!("Gossip not initialized, skipping peer reconnection");
            return Ok(());
        };

        let mut attempted = 0;
        let mut succeeded = 0;
        let mut skipped = 0;

        info!(
            "Checking {} inactive peers for reconnection (with exponential backoff)",
            inactive.len()
        );

        for mut peer_info in inactive {
            let peer_id = peer_info.public_key();

            // Check if enough time has passed for retry (exponential backoff)
            if !peer_info.should_retry_now() {
                skipped += 1;
                debug!(
                    ?peer_id,
                    backoff_secs = peer_info.backoff_delay(),
                    "Skipping peer - backoff not elapsed"
                );
                continue;
            }

            attempted += 1;
            peer_info.record_attempt();
            debug!(?peer_id, attempt_number = peer_info.connection_attempts, "Attempting to reconnect");

            // Try to connect using the endpoint
            match gossip
                .endpoint()
                .connect(peer_id, iroh_gossip::net::GOSSIP_ALPN)
                .await
            {
                Ok(_conn) => {
                    peer_info.record_success();
                    succeeded += 1;
                    info!(
                        ?peer_id,
                        success_rate = format!("{:.1}%", peer_info.success_rate() * 100.0),
                        "Successfully reconnected to peer"
                    );
                }
                Err(e) => {
                    peer_info.record_failure();
                    debug!(
                        ?peer_id,
                        error = ?e,
                        next_retry_in = format!("{:?}", peer_info.backoff_delay()),
                        "Failed to reconnect, will retry after backoff"
                    );
                }
            }

            // Save updated peer info (with metrics and status)
            self.peer_registry.add_or_update(&peer_info)?;
        }

        if attempted > 0 || skipped > 0 {
            info!(
                "Reconnection summary: attempted={}, succeeded={}, skipped={} (backoff)",
                attempted, succeeded, skipped
            );
        }

        Ok(())
    }

    /// Start syncing a realm with peers via gossip
    ///
    /// Subscribes to the realm's gossip topic and begins sending/receiving
    /// sync messages. This method can be called for multiple realms concurrently
    /// without blocking.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    pub async fn start_sync(&mut self, realm_id: &RealmId) -> Result<(), SyncError> {
        // Ensure realm is open
        if !self.realms.contains_key(realm_id) {
            self.open_realm(realm_id).await?;
        }

        self.start_sync_internal(realm_id).await
    }

    /// Internal sync starter that assumes the realm is already open.
    /// Used by open_realm to avoid circular recursion.
    async fn start_sync_internal(&mut self, realm_id: &RealmId) -> Result<(), SyncError> {
        // Check if already syncing
        if self.is_realm_syncing(realm_id) {
            debug!(%realm_id, "Already syncing");
            return Ok(());
        }

        info!(%realm_id, "Starting sync");

        // Update status to Connecting
        self.sync_status
            .lock()
            .unwrap()
            .insert(realm_id.clone(), SyncStatus::Connecting);
        let _ = self.event_tx.send(SyncEvent::StatusChanged {
            realm_id: realm_id.clone(),
            status: SyncStatus::Connecting,
        });

        // Initialize gossip
        let gossip = self.ensure_gossip().await?;

        // Get effective bootstrap peers (static from storage + online from registry)
        let bootstrap_node_ids = self.get_effective_bootstrap_peers(realm_id)?;

        // Add full address info to static discovery for faster reconnection
        if let Some(realm_info) = self.storage.load_realm(realm_id)? {
            for peer_bytes in &realm_info.bootstrap_peers {
                match peer_bytes.to_endpoint_addr() {
                    Ok(endpoint_addr) => {
                        debug!(
                            %realm_id,
                            peer = %endpoint_addr.id,
                            relay = ?peer_bytes.relay_url,
                            addrs = peer_bytes.direct_addresses.len(),
                            "Adding saved bootstrap peer to static discovery"
                        );
                        gossip.add_peer_addr(endpoint_addr.clone());
                    }
                    Err(e) => {
                        warn!(%realm_id, error = ?e, "Failed to convert saved peer address");
                    }
                }
            }
        }

        if !bootstrap_node_ids.is_empty() {
            info!(%realm_id, peer_count = bootstrap_node_ids.len(), "Using bootstrap peers (static + registry) for reconnection");
        }

        // Subscribe to topic using split API (receiver not wrapped in mutex)
        let topic_id = TopicId::from_bytes(*realm_id.as_bytes());
        let (sender, mut receiver) = gossip.subscribe_split(topic_id, bootstrap_node_ids).await?;

        // Store sender for broadcasting
        if let Some(state) = self.realms.get_mut(realm_id) {
            state.topic_sender = Some(sender);
        }

        // Spawn background listener task that owns the receiver directly
        let listener_realm_id = realm_id.clone();
        let sync_tx = self.sync_tx.clone();
        let event_tx = self.event_tx.clone();
        // Clone sync_status Arc for thread-safe peer counting in listener task
        let sync_status = self.sync_status.clone();
        // Clone peer_registry for tracking discovered peers
        let peer_registry = self.peer_registry.clone();

        tokio::spawn(async move {
            debug!(%listener_realm_id, "Sync listener task started");
            let mut event_count = 0u64;
            loop {
                debug!(%listener_realm_id, event_count, "Listener waiting for next event...");
                match receiver.recv_event().await {
                    Some(TopicEvent::Message(msg)) => {
                        event_count += 1;
                        debug!(
                            %listener_realm_id,
                            event_count,
                            from = ?msg.from,
                            bytes = msg.content.len(),
                            "Received sync message"
                        );
                        // Send to channel for processing by main engine
                        if sync_tx
                            .send(SyncChannelMessage::IncomingData {
                                realm_id: listener_realm_id.clone(),
                                envelope_bytes: msg.content,
                            })
                            .is_err()
                        {
                            debug!(%listener_realm_id, "Sync channel closed, stopping listener");
                            break;
                        }
                        // Notify listeners that data arrived
                        let _ = event_tx.send(SyncEvent::RealmChanged {
                            realm_id: listener_realm_id.clone(),
                            changes_applied: 1,
                        });
                    }
                    Some(TopicEvent::NeighborUp(peer)) => {
                        event_count += 1;
                        debug!(%listener_realm_id, event_count, ?peer, "Peer connected");

                        // Record peer in registry
                        let peer_info =
                            PeerInfo::new(peer, PeerSource::FromRealm(listener_realm_id.clone()))
                                .with_status(PeerStatus::Online);
                        if let Err(e) = peer_registry.add_or_update(&peer_info) {
                            warn!(?peer, error = ?e, "Failed to record peer in registry");
                        } else {
                            // Also record the realm for this peer
                            let _ = peer_registry.add_peer_realm(&peer, &listener_realm_id);
                            debug!(?peer, "Recorded peer in registry");

                            // Note: We don't automatically add gossip-discovered peers to
                            // bootstrap_peers in storage, as we don't have their full EndpointAddr
                            // at this point. The peer_registry tracks them for reconnection purposes.
                        }

                        // Update peer count in sync_status (thread-safe)
                        let new_count = {
                            let mut status_map = sync_status.lock().unwrap();
                            if let Some(SyncStatus::Syncing { peer_count }) =
                                status_map.get_mut(&listener_realm_id)
                            {
                                *peer_count += 1;
                                *peer_count
                            } else {
                                // Initialize to 1 if not in Syncing state
                                status_map.insert(
                                    listener_realm_id.clone(),
                                    SyncStatus::Syncing { peer_count: 1 },
                                );
                                1
                            }
                        };

                        // Request broadcast of our full document to the newly connected peer
                        // This ensures offline changes are shared when peers reconnect
                        let _ = sync_tx.send(SyncChannelMessage::BroadcastRequest {
                            realm_id: listener_realm_id.clone(),
                        });

                        // Emit events
                        let _ = event_tx.send(SyncEvent::PeerConnected {
                            realm_id: listener_realm_id.clone(),
                            peer_id: peer.to_string(),
                        });
                        let _ = event_tx.send(SyncEvent::StatusChanged {
                            realm_id: listener_realm_id.clone(),
                            status: SyncStatus::Syncing {
                                peer_count: new_count,
                            },
                        });
                    }
                    Some(TopicEvent::NeighborDown(peer)) => {
                        event_count += 1;
                        debug!(%listener_realm_id, event_count, ?peer, "Peer disconnected");

                        // Mark peer as offline in registry
                        if let Err(e) = peer_registry.update_status(&peer, PeerStatus::Offline) {
                            warn!(?peer, error = ?e, "Failed to update peer status to offline");
                        } else {
                            debug!(?peer, "Marked peer as offline in registry");
                        }

                        // Update peer count in sync_status (thread-safe)
                        let new_count = {
                            let mut status_map = sync_status.lock().unwrap();
                            if let Some(SyncStatus::Syncing { peer_count }) =
                                status_map.get_mut(&listener_realm_id)
                            {
                                *peer_count = peer_count.saturating_sub(1);
                                *peer_count
                            } else {
                                0
                            }
                        };

                        // Emit events
                        let _ = event_tx.send(SyncEvent::PeerDisconnected {
                            realm_id: listener_realm_id.clone(),
                            peer_id: peer.to_string(),
                        });
                        let _ = event_tx.send(SyncEvent::StatusChanged {
                            realm_id: listener_realm_id.clone(),
                            status: SyncStatus::Syncing {
                                peer_count: new_count,
                            },
                        });
                    }
                    None => {
                        debug!(%listener_realm_id, event_count, "Topic subscription closed");
                        break;
                    }
                }
            }
            debug!(%listener_realm_id, event_count, "Sync listener task ended");
        });

        // Spawn periodic bootstrap reconnection task
        // This handles the case where both peers start at the same time - the initial
        // add_peer_addr calls fail because neither peer is ready yet. This task
        // periodically re-tries the bootstrap peers until we connect.
        if let Some(realm_info) = self.storage.load_realm(realm_id)? {
            if !realm_info.bootstrap_peers.is_empty() {
                let reconnect_realm_id = realm_id.clone();
                let reconnect_gossip = gossip.clone();
                let reconnect_sync_status = self.sync_status.clone();
                let bootstrap_peers = realm_info.bootstrap_peers.clone();

                tokio::spawn(async move {
                    use std::time::Duration;

                    let mut attempt = 0;
                    const MAX_ATTEMPTS: u32 = 24; // 2 minutes of retries (5s * 24)
                    const RETRY_INTERVAL: Duration = Duration::from_secs(5);

                    loop {
                        // Wait before retry (skip on first iteration to allow initial connect attempt)
                        if attempt > 0 {
                            tokio::time::sleep(RETRY_INTERVAL).await;
                        }
                        attempt += 1;

                        // Check if we already have peers - if so, we're done
                        let current_peer_count = {
                            let status_map = reconnect_sync_status.lock().unwrap();
                            match status_map.get(&reconnect_realm_id) {
                                Some(SyncStatus::Syncing { peer_count }) => *peer_count,
                                _ => 0,
                            }
                        };

                        if current_peer_count > 0 {
                            debug!(
                                %reconnect_realm_id,
                                peer_count = current_peer_count,
                                "Bootstrap reconnection task: peers connected, stopping"
                            );
                            break;
                        }

                        if attempt > MAX_ATTEMPTS {
                            debug!(
                                %reconnect_realm_id,
                                attempt,
                                "Bootstrap reconnection task: max attempts reached, stopping"
                            );
                            break;
                        }

                        // Re-add all bootstrap peer addresses
                        for peer_bytes in &bootstrap_peers {
                            if let Ok(endpoint_addr) = peer_bytes.to_endpoint_addr() {
                                debug!(
                                    %reconnect_realm_id,
                                    attempt,
                                    peer = %endpoint_addr.id,
                                    "Bootstrap reconnection: re-adding peer address"
                                );
                                reconnect_gossip.add_peer_addr(endpoint_addr);
                            }
                        }
                    }
                });
            }
        }

        // Update status to Syncing
        self.sync_status
            .lock()
            .unwrap()
            .insert(realm_id.clone(), SyncStatus::Syncing { peer_count: 0 });
        let _ = self.event_tx.send(SyncEvent::StatusChanged {
            realm_id: realm_id.clone(),
            status: SyncStatus::Syncing { peer_count: 0 },
        });

        debug!(%realm_id, "Sync started");

        // CRITICAL: Broadcast our FULL DOCUMENT when sync starts.
        // This ensures that when peers reconnect after being offline, they exchange
        // their complete document states and Automerge merges them automatically.
        // Without this, peers would just sit waiting and never sync offline changes.
        if let Err(e) = self.broadcast_changes_with_data(realm_id, vec![]).await {
            warn!(
                %realm_id,
                error = ?e,
                "Failed to broadcast initial document (will retry on next sync activity)"
            );
        } else {
            info!(%realm_id, "Broadcast initial document to peers");
        }

        Ok(())
    }

    /// Stop syncing a realm
    ///
    /// Removes the realm from the gossip topic. The realm remains open.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    pub async fn stop_sync(&mut self, realm_id: &RealmId) -> Result<(), SyncError> {
        let state = self
            .realms
            .get_mut(realm_id)
            .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

        if state.topic_sender.is_some() {
            state.topic_sender = None;

            // Update status to Idle
            self.sync_status
                .lock()
                .unwrap()
                .insert(realm_id.clone(), SyncStatus::Idle);
            let _ = self.event_tx.send(SyncEvent::StatusChanged {
                realm_id: realm_id.clone(),
                status: SyncStatus::Idle,
            });

            info!(%realm_id, "Sync stopped");
        } else {
            debug!(%realm_id, "Not syncing");
        }

        Ok(())
    }

    /// Get the sync status for a realm
    ///
    /// Returns `SyncStatus::Idle` if the realm is not syncing or not found.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let status = engine.sync_status(&realm_id);
    /// match status {
    ///     SyncStatus::Syncing { peer_count } => println!("Syncing with {} peers", peer_count),
    ///     SyncStatus::Idle => println!("Not syncing"),
    ///     _ => {}
    /// }
    /// ```
    pub fn sync_status(&self, realm_id: &RealmId) -> SyncStatus {
        self.sync_status
            .lock()
            .unwrap()
            .get(realm_id)
            .cloned()
            .unwrap_or(SyncStatus::Idle)
    }

    /// Get detailed network debug information for a realm.
    ///
    /// Returns information useful for debugging sync issues:
    /// - Our node ID
    /// - Current sync status with peer count
    /// - Whether sync is active
    /// - Bootstrap peer count
    pub fn network_debug_info(&self, realm_id: &RealmId) -> NetworkDebugInfo {
        // Get node ID from gossip if available
        let (node_id, node_id_full) = if let Some(ref gossip) = self.gossip {
            let pk = gossip.public_key();
            let full = format!("{:?}", pk);
            // Extract just the hex part, e.g., "PublicKey(abc...)" -> "abc..."
            let short = full
                .strip_prefix("PublicKey(")
                .and_then(|s| s.strip_suffix(")"))
                .map(|s| s.chars().take(8).collect::<String>())
                .unwrap_or_else(|| "unknown".to_string());
            (short, full)
        } else {
            ("offline".to_string(), "offline".to_string())
        };

        // Get sync status
        let status = self.sync_status(realm_id);

        // Get realm info for bootstrap peers
        let (bootstrap_peer_count, is_shared) = self
            .storage
            .load_realm(realm_id)
            .ok()
            .flatten()
            .map(|info| (info.bootstrap_peers.len(), info.is_shared))
            .unwrap_or((0, false));

        // Check if sync is active (we have a gossip and status is not Idle)
        let sync_active = self.gossip.is_some() && !matches!(status, SyncStatus::Idle);

        // Extract error if status is Error
        let last_error = match &status {
            SyncStatus::Error(msg) => Some(msg.clone()),
            _ => None,
        };

        // Get connected peer IDs (currently we only have count, not IDs)
        // TODO: Track actual peer IDs in sync_status for display
        let connected_peers = Vec::new();

        // Get detailed peer information from the peer registry
        // Filter peers by those that share this realm
        let peers = self
            .peer_registry
            .list_all()
            .unwrap_or_default()
            .into_iter()
            .filter(|peer| peer.shared_realms.contains(realm_id))
            .map(|peer| {
                use crate::sync::events::PeerDebugInfo;
                let pk = peer.public_key();
                let full = format!("{:?}", pk);
                let short = full
                    .strip_prefix("PublicKey(")
                    .and_then(|s| s.strip_suffix(")"))
                    .map(|s| s.chars().take(8).collect::<String>())
                    .unwrap_or_else(|| "unknown".to_string());

                // Calculate connection duration (simple: how long since last_seen)
                let connection_duration_secs = if peer.status == crate::peers::PeerStatus::Online {
                    // For online peers, calculate time since last_seen
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    Some(now.saturating_sub(peer.last_seen))
                } else {
                    None
                };

                PeerDebugInfo {
                    peer_id: short,
                    peer_id_full: full,
                    is_connected: peer.status == crate::peers::PeerStatus::Online,
                    connection_duration_secs,
                }
            })
            .collect();

        NetworkDebugInfo {
            node_id,
            node_id_full,
            status,
            bootstrap_peer_count,
            is_shared,
            sync_active,
            last_error,
            connected_peers,
            peers,
        }
    }

    /// Subscribe to sync events
    ///
    /// Returns a receiver that will receive events when:
    /// - Remote changes arrive for a realm
    /// - Peer connects or disconnects
    /// - Sync status changes
    /// - Errors occur
    ///
    /// Multiple subscribers can exist; events are broadcast to all.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut events = engine.subscribe_events();
    ///
    /// tokio::spawn(async move {
    ///     while let Ok(event) = events.recv().await {
    ///         match event {
    ///             SyncEvent::RealmChanged { realm_id, .. } => {
    ///                 println!("Realm {} changed!", realm_id);
    ///             }
    ///             SyncEvent::StatusChanged { realm_id, status } => {
    ///                 println!("Realm {} status: {:?}", realm_id, status);
    ///             }
    ///             _ => {}
    ///         }
    ///     }
    /// });
    /// ```
    pub fn subscribe_events(&self) -> broadcast::Receiver<SyncEvent> {
        self.event_tx.subscribe()
    }

    /// Register a callback for realm changes
    ///
    /// This is a convenience method that spawns a background task to listen
    /// for `SyncEvent::RealmChanged` events and calls the provided callback.
    ///
    /// # Arguments
    ///
    /// * `callback` - Function to call when remote changes arrive for any realm
    ///
    /// # Example
    ///
    /// ```ignore
    /// engine.on_realm_change(|realm_id| {
    ///     println!("Realm {} was updated by a peer!", realm_id);
    /// });
    /// ```
    pub fn on_realm_change<F>(&self, callback: F)
    where
        F: Fn(RealmId) + Send + 'static,
    {
        let mut receiver = self.event_tx.subscribe();

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                if let SyncEvent::RealmChanged { realm_id, .. } = event {
                    callback(realm_id);
                }
            }
        });
    }

    /// Get the list of realms currently syncing
    pub fn syncing_realms(&self) -> Vec<RealmId> {
        self.sync_status
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(realm_id, status)| {
                if !matches!(status, SyncStatus::Idle) {
                    Some(realm_id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the count of realms currently syncing
    pub fn syncing_count(&self) -> usize {
        self.sync_status
            .lock()
            .unwrap()
            .values()
            .filter(|s| !matches!(s, SyncStatus::Idle))
            .count()
    }

    /// Emit a sync event (internal helper)
    #[allow(dead_code)]
    fn emit_event(&self, event: SyncEvent) {
        let _ = self.event_tx.send(event);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Sync Envelope Operations (Signed + Encrypted)
    // ═══════════════════════════════════════════════════════════════════════

    /// Broadcast a sync message to all peers in a realm (signed + encrypted)
    ///
    /// The message is wrapped in a `SyncEnvelope` which provides:
    /// - Encryption with the realm's symmetric key (ChaCha20-Poly1305)
    /// - Signature from our identity for authentication
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open or not syncing.
    /// Returns `SyncError::Identity` if identity has not been initialized.
    pub async fn broadcast_sync(
        &self,
        realm_id: &RealmId,
        message: SyncMessage,
    ) -> Result<(), SyncError> {
        // Get realm state (must be open and syncing)
        let state = self
            .realms
            .get(realm_id)
            .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

        let topic_sender = state
            .topic_sender
            .as_ref()
            .ok_or_else(|| SyncError::Gossip("Realm is not syncing".to_string()))?;

        // Get identity (must be initialized)
        let keypair = self.identity.as_ref().ok_or_else(|| {
            SyncError::Identity("Identity not initialized. Call init_identity() first.".to_string())
        })?;

        let sender_did = Did::from_public_key(&keypair.public_key()).to_string();

        // Create signing function
        let sign_fn = |data: &[u8]| -> Vec<u8> { keypair.sign(data).to_bytes() };

        // Seal the message (encrypt + sign)
        let envelope = SyncEnvelope::seal(&message, &sender_did, &state.realm_key, sign_fn)?;

        // Serialize envelope
        let envelope_bytes = envelope.to_bytes()?;

        debug!(
            %realm_id,
            sender = %sender_did,
            bytes = envelope_bytes.len(),
            "Broadcasting sync envelope"
        );

        // Broadcast via topic sender
        topic_sender.broadcast(envelope_bytes).await?;

        Ok(())
    }

    /// Process incoming sync messages from gossip
    ///
    /// Verifies the signature and decrypts the envelope, returning the
    /// inner `SyncMessage` for processing.
    ///
    /// # Arguments
    ///
    /// * `realm_id` - The realm this message belongs to
    /// * `envelope_bytes` - The raw bytes received from gossip
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(message))` if the envelope is valid.
    /// Returns `Ok(None)` if signature verification or decryption failed.
    /// Returns `Err` for other errors (realm not found, etc).
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    pub fn handle_incoming(
        &self,
        realm_id: &RealmId,
        envelope_bytes: &[u8],
    ) -> Result<Option<SyncMessage>, SyncError> {
        // Get realm state
        let state = self
            .realms
            .get(realm_id)
            .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

        // Deserialize envelope
        let envelope = match SyncEnvelope::from_bytes(envelope_bytes) {
            Ok(env) => env,
            Err(e) => {
                warn!(error = ?e, "Failed to deserialize envelope");
                return Ok(None);
            }
        };

        debug!(
            %realm_id,
            sender = %envelope.sender(),
            version = envelope.version(),
            "Processing incoming envelope"
        );

        // Create verification function
        // For now, we accept all valid signatures since we don't have a DID registry
        // In the future, this would lookup the sender's public key from their DID
        let verify_fn = Self::make_verify_fn();

        // Open the envelope (verify signature + decrypt)
        match envelope.open(&state.realm_key, verify_fn) {
            Ok(message) => {
                debug!(
                    %realm_id,
                    message_type = ?std::mem::discriminant(&message),
                    "Successfully opened envelope"
                );
                Ok(Some(message))
            }
            Err(SyncError::SignatureInvalid(msg)) => {
                warn!(%realm_id, error = %msg, "Signature verification failed");
                Ok(None)
            }
            Err(SyncError::DecryptionFailed(msg)) => {
                warn!(%realm_id, error = %msg, "Decryption failed");
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    /// Sync a realm's current state to peers
    ///
    /// Broadcasts an `Announce` message with the current document heads,
    /// allowing peers to determine if they need to sync.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    /// Returns `SyncError::Gossip` if the realm is not syncing.
    pub async fn sync_realm_state(&mut self, realm_id: &RealmId) -> Result<(), SyncError> {
        // Get the document heads
        let heads = {
            let state = self
                .realms
                .get_mut(realm_id)
                .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

            state
                .doc
                .heads()
                .into_iter()
                .map(|h| h.0.to_vec())
                .collect::<Vec<_>>()
        };

        // Create announce message with our address for peer discovery
        let sender_addr = self
            .gossip
            .as_ref()
            .map(|g| NodeAddrBytes::from_endpoint_addr(&g.endpoint_addr()));

        let message = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads,
            sender_addr,
        };

        // Broadcast it
        self.broadcast_sync(realm_id, message).await
    }

    /// Broadcast incremental changes for a realm
    ///
    /// Generates an incremental sync message from the document and broadcasts
    /// it to all peers. This is more efficient than sending the full document.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    /// Returns `SyncError::Gossip` if the realm is not syncing.
    pub async fn broadcast_changes(&mut self, realm_id: &RealmId) -> Result<(), SyncError> {
        // Generate incremental changes
        let data = {
            let state = self
                .realms
                .get_mut(realm_id)
                .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

            state.doc.generate_sync_message()
        };

        // Only broadcast if there are changes
        if data.is_empty() {
            debug!(%realm_id, "No changes to broadcast");
            return Ok(());
        }

        // Create changes message
        let message = SyncMessage::Changes {
            realm_id: realm_id.clone(),
            data,
        };

        // Broadcast it
        self.broadcast_sync(realm_id, message).await
    }

    /// Broadcast pre-captured changes to peers
    ///
    /// Use this when you've already captured incremental changes before saving
    /// (since save() resets the incremental checkpoint).
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    /// Returns `SyncError::Gossip` if the realm is not syncing.
    pub async fn broadcast_changes_with_data(
        &mut self,
        realm_id: &RealmId,
        _data: Vec<u8>,
    ) -> Result<(), SyncError> {
        // Always broadcast the FULL document instead of incremental changes.
        // This ensures peers with empty/different docs can properly sync.
        // Incremental changes only work when docs share a common history.
        let state = self
            .realms
            .get_mut(realm_id)
            .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

        let full_doc = state.doc.save();

        // Create sync response with full document
        let message = SyncMessage::SyncResponse {
            realm_id: realm_id.clone(),
            document: full_doc,
        };

        debug!(%realm_id, "Broadcasting full document for sync");

        // Broadcast it
        self.broadcast_sync(realm_id, message).await
    }

    /// Apply incoming changes from a peer
    ///
    /// Applies the changes from a `SyncMessage::Changes` to the local document
    /// and saves the realm.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    /// Returns `SyncError::Serialization` if the changes are invalid.
    pub async fn apply_incoming_changes(
        &mut self,
        realm_id: &RealmId,
        data: &[u8],
    ) -> Result<(), SyncError> {
        {
            let state = self
                .realms
                .get_mut(realm_id)
                .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

            state.doc.apply_sync_message(data)?;
        }

        // Save the updated document
        self.save_realm(realm_id).await?;

        debug!(%realm_id, bytes = data.len(), "Applied incoming changes");
        Ok(())
    }

    /// Apply a full document from a peer
    ///
    /// Loads the full document state and merges it with the local document.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    /// Returns `SyncError::Serialization` if the document is invalid.
    pub async fn apply_full_document(
        &mut self,
        realm_id: &RealmId,
        document_bytes: &[u8],
    ) -> Result<(), SyncError> {
        {
            let state = self
                .realms
                .get_mut(realm_id)
                .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

            // Load the remote document
            let mut remote_doc = RealmDoc::load(document_bytes)?;

            // Merge into our document
            state.doc.merge(&mut remote_doc)?;
        }

        // Save the merged document
        self.save_realm(realm_id).await?;

        debug!(%realm_id, bytes = document_bytes.len(), "Applied full document");
        Ok(())
    }

    /// Create a verification function for envelope signatures
    ///
    /// Currently accepts all valid signature formats since we don't have
    /// a DID registry to lookup public keys. In the future, this would
    /// verify signatures against the sender's registered public key.
    fn make_verify_fn() -> impl Fn(&str, &[u8], &[u8]) -> bool {
        |_sender: &str, _data: &[u8], sig: &[u8]| -> bool {
            // For now, we accept any non-empty signature
            // This is a placeholder until we implement proper DID-based verification
            // with a registry that maps DIDs to public keys
            //
            // A proper implementation would:
            // 1. Parse the sender DID to extract the public key
            // 2. Verify the signature using that public key
            // 3. Check that the DID is a member of the realm
            !sig.is_empty()
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Invite Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Generate an invite ticket for a realm
    ///
    /// Creates an invite containing the realm's encryption key and this
    /// node as a bootstrap peer.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    pub async fn generate_invite(&mut self, realm_id: &RealmId) -> Result<InviteTicket, SyncError> {
        // Ensure realm is open
        if !self.realms.contains_key(realm_id) {
            self.open_realm(realm_id).await?;
        }

        // Get the realm key (copy it to avoid borrow issues)
        let realm_key = {
            let state = self
                .realms
                .get(realm_id)
                .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;
            state.realm_key
        };

        // Get realm name for the invite
        let realm_name = self.storage.load_realm(realm_id)?.map(|info| info.name);

        // Ensure gossip is initialized
        let gossip = self.ensure_gossip().await?;

        // Generate invite (include realm name so joiners see it)
        let invite = gossip.generate_invite(realm_id, realm_key, realm_name.as_deref())?;

        // Mark realm as shared
        if let Ok(Some(mut info)) = self.storage.load_realm(realm_id) {
            if !info.is_shared {
                info.is_shared = true;
                self.storage.save_realm(&info)?;
            }
        }

        // Auto-start sync when sharing a realm
        // This ensures the creator can send/receive sync messages
        if !self.is_realm_syncing(realm_id) {
            self.start_sync(realm_id).await?;
        }

        info!(%realm_id, "Invite generated and sync started");
        Ok(invite)
    }

    /// Join a realm via invite ticket
    ///
    /// Connects to bootstrap peers, subscribes to the realm's gossip topic,
    /// and saves the realm to storage.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::InvalidInvite` if the invite is expired.
    pub async fn join_via_invite(&mut self, invite: &InviteTicket) -> Result<RealmId, SyncError> {
        let realm_id = invite.realm_id();
        info!(%realm_id, "Joining realm via invite");

        // Debug: Log existing realms for troubleshooting
        let existing_realms = self.storage.list_realms()?;
        debug!(
            joining_realm = %realm_id,
            joining_base58 = %realm_id.to_base58(),
            existing_count = existing_realms.len(),
            existing_ids = ?existing_realms.iter().map(|r| r.id.to_base58()).collect::<Vec<_>>(),
            "Checking membership before join"
        );

        // Check if we already have this realm
        if self.storage.load_realm(&realm_id)?.is_some() {
            warn!(
                %realm_id,
                "Join rejected: realm already exists in storage"
            );
            return Err(SyncError::InvalidInvite(
                "Already a member of this realm".into(),
            ));
        }

        // Initialize gossip
        let gossip = self.ensure_gossip().await?;

        // Join via invite using split API (receiver not wrapped in mutex)
        let (sender, mut receiver) = gossip.join_via_invite_split(invite).await?;

        // Spawn background listener task that owns the receiver directly
        let listener_realm_id = realm_id.clone();
        let sync_tx = self.sync_tx.clone();
        let event_tx = self.event_tx.clone();
        // Clone sync_status Arc for thread-safe peer counting in listener task
        let sync_status = self.sync_status.clone();
        // Clone peer_registry for tracking discovered peers
        let peer_registry = self.peer_registry.clone();

        tokio::spawn(async move {
            debug!(%listener_realm_id, "Join sync listener task started");
            let mut event_count = 0u64;
            loop {
                debug!(%listener_realm_id, event_count, "Join listener waiting for next event...");
                match receiver.recv_event().await {
                    Some(TopicEvent::Message(msg)) => {
                        event_count += 1;
                        let msg_bytes = msg.content.len();
                        debug!(
                            %listener_realm_id,
                            event_count,
                            from = ?msg.from,
                            bytes = msg_bytes,
                            "Received sync message (joined)"
                        );
                        // Send to channel for processing by main engine
                        let send_result = sync_tx.send(SyncChannelMessage::IncomingData {
                            realm_id: listener_realm_id.clone(),
                            envelope_bytes: msg.content,
                        });
                        if send_result.is_err() {
                            debug!(%listener_realm_id, "Sync channel closed, stopping listener");
                            break;
                        }
                        debug!(
                            %listener_realm_id,
                            bytes = msg_bytes,
                            "Sent message to sync channel (joined)"
                        );
                        // Notify listeners that data arrived
                        let _ = event_tx.send(SyncEvent::RealmChanged {
                            realm_id: listener_realm_id.clone(),
                            changes_applied: 1,
                        });
                    }
                    Some(TopicEvent::NeighborUp(peer)) => {
                        event_count += 1;
                        debug!(%listener_realm_id, event_count, ?peer, "Peer connected (joined)");

                        // Record peer in registry
                        let peer_info =
                            PeerInfo::new(peer, PeerSource::FromRealm(listener_realm_id.clone()))
                                .with_status(PeerStatus::Online);
                        if let Err(e) = peer_registry.add_or_update(&peer_info) {
                            warn!(?peer, error = ?e, "Failed to record peer in registry (joined)");
                        } else {
                            // Also record the realm for this peer
                            let _ = peer_registry.add_peer_realm(&peer, &listener_realm_id);
                            debug!(?peer, "Recorded peer in registry (joined)");
                        }

                        // Update peer count in sync_status (thread-safe)
                        let new_count = {
                            let mut status_map = sync_status.lock().unwrap();
                            if let Some(SyncStatus::Syncing { peer_count }) =
                                status_map.get_mut(&listener_realm_id)
                            {
                                *peer_count += 1;
                                *peer_count
                            } else {
                                // Initialize to 1 if not in Syncing state
                                status_map.insert(
                                    listener_realm_id.clone(),
                                    SyncStatus::Syncing { peer_count: 1 },
                                );
                                1
                            }
                        };

                        // Request broadcast of our full document to the newly connected peer
                        // This ensures offline changes are shared when peers reconnect
                        let _ = sync_tx.send(SyncChannelMessage::BroadcastRequest {
                            realm_id: listener_realm_id.clone(),
                        });

                        // Emit events
                        let _ = event_tx.send(SyncEvent::PeerConnected {
                            realm_id: listener_realm_id.clone(),
                            peer_id: peer.to_string(),
                        });
                        let _ = event_tx.send(SyncEvent::StatusChanged {
                            realm_id: listener_realm_id.clone(),
                            status: SyncStatus::Syncing {
                                peer_count: new_count,
                            },
                        });
                    }
                    Some(TopicEvent::NeighborDown(peer)) => {
                        event_count += 1;
                        debug!(%listener_realm_id, event_count, ?peer, "Peer disconnected (joined)");

                        // Mark peer as offline in registry
                        if let Err(e) = peer_registry.update_status(&peer, PeerStatus::Offline) {
                            warn!(?peer, error = ?e, "Failed to update peer status to offline (joined)");
                        } else {
                            debug!(?peer, "Marked peer as offline in registry (joined)");
                        }

                        // Update peer count in sync_status (thread-safe)
                        let new_count = {
                            let mut status_map = sync_status.lock().unwrap();
                            if let Some(SyncStatus::Syncing { peer_count }) =
                                status_map.get_mut(&listener_realm_id)
                            {
                                *peer_count = peer_count.saturating_sub(1);
                                *peer_count
                            } else {
                                0
                            }
                        };

                        // Emit events
                        let _ = event_tx.send(SyncEvent::PeerDisconnected {
                            realm_id: listener_realm_id.clone(),
                            peer_id: peer.to_string(),
                        });
                        let _ = event_tx.send(SyncEvent::StatusChanged {
                            realm_id: listener_realm_id.clone(),
                            status: SyncStatus::Syncing {
                                peer_count: new_count,
                            },
                        });
                    }
                    None => {
                        debug!(%listener_realm_id, "Topic subscription closed");
                        break;
                    }
                }
            }
            debug!(%listener_realm_id, "Join sync listener task ended");
        });

        // Create realm info with bootstrap peers for reconnection after restart
        let info = RealmInfo {
            id: realm_id.clone(),
            name: invite
                .realm_name
                .clone()
                .unwrap_or_else(|| "Shared Realm".to_string()),
            is_shared: true,
            created_at: chrono::Utc::now().timestamp(),
            bootstrap_peers: invite.bootstrap_peers.clone(),
        };

        // Create document
        let mut doc = RealmDoc::new();

        // Save to storage
        self.storage.save_realm(&info)?;
        self.storage.save_realm_key(&realm_id, &invite.realm_key)?;
        self.storage.save_document(&realm_id, &doc.save())?;

        // Add to open realms
        self.realms.insert(
            realm_id.clone(),
            RealmState {
                doc,
                topic_sender: Some(sender),
                realm_key: invite.realm_key,
            },
        );

        // Update sync status
        self.sync_status
            .lock()
            .unwrap()
            .insert(realm_id.clone(), SyncStatus::Syncing { peer_count: 0 });
        let _ = self.event_tx.send(SyncEvent::StatusChanged {
            realm_id: realm_id.clone(),
            status: SyncStatus::Syncing { peer_count: 0 },
        });

        debug!(%realm_id, "Joined realm and started sync");

        // Wait for connection to establish before announcing
        // The gossip connection typically takes ~20-50ms to establish
        // This ensures we have a neighbor to receive our announce
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        debug!(%realm_id, "Connection delay complete, sending announce");

        // Send an announce message to establish bidirectional communication
        // This allows the inviter to learn our address and send messages back to us
        // by forcing Bob to send a message first, which establishes the QUIC connection
        let heads = if let Some(state) = self.realms.get_mut(&realm_id) {
            state
                .doc
                .heads()
                .into_iter()
                .map(|h| h.0.to_vec())
                .collect()
        } else {
            vec![]
        };

        // Include our endpoint address so the receiver can add us to their discovery
        let sender_addr = self
            .gossip
            .as_ref()
            .map(|g| NodeAddrBytes::from_endpoint_addr(&g.endpoint_addr()));

        let announce = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads,
            sender_addr,
        };
        if let Err(e) = self.broadcast_sync(&realm_id, announce).await {
            debug!(%realm_id, error = ?e, "Failed to send announce (non-fatal)");
        } else {
            debug!(%realm_id, "Sent announce to establish bidirectional connection");
        }

        Ok(realm_id)
    }

    /// Create an invite string for a realm (convenience method)
    ///
    /// Same as `generate_invite` but returns the encoded string.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm doesn't exist.
    pub async fn create_invite(&mut self, realm_id: &RealmId) -> Result<String, SyncError> {
        let ticket = self.generate_invite(realm_id).await?;
        ticket.encode()
    }

    /// Join a realm via invite string (convenience method)
    ///
    /// Decodes the invite ticket and joins the realm.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::InvalidInvite` if the ticket is invalid.
    pub async fn join_realm(&mut self, ticket_str: &str) -> Result<RealmId, SyncError> {
        let ticket = InviteTicket::decode(ticket_str)?;
        self.join_via_invite(&ticket).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Node Info
    // ═══════════════════════════════════════════════════════════════════════

    /// Get information about this node
    pub async fn node_info(&self) -> Result<NodeInfo, SyncError> {
        let realms = self.storage.list_realms()?;

        let (node_id, relay_url) = if let Some(ref gossip) = self.gossip {
            let addr = gossip.endpoint_addr();
            let id = Some(addr.id.to_string());
            let relay = addr.relay_urls().next().map(|u| u.to_string());
            (id, relay)
        } else {
            (None, None)
        };

        let did = self.did().map(|d| d.to_string());

        Ok(NodeInfo {
            data_dir: self.data_dir.clone(),
            realm_count: realms.len(),
            node_id,
            relay_url,
            did,
        })
    }

    /// Get this node's endpoint address
    ///
    /// Returns the EndpointAddr which can be used by other nodes to connect.
    /// Returns None if networking is not active.
    #[cfg(test)]
    pub fn endpoint_addr(&self) -> Option<iroh::EndpointAddr> {
        self.gossip.as_ref().map(|g| g.endpoint_addr())
    }

    /// Add a peer's address to the discovery system
    ///
    /// This allows sending messages to the peer directly.
    #[cfg(test)]
    pub fn add_peer_addr(&self, addr: iroh::EndpointAddr) {
        if let Some(ref gossip) = self.gossip {
            gossip.add_peer_addr(addr);
        }
    }

    /// Wait for a peer to connect to a realm's sync topic
    ///
    /// Returns true if a peer connected within the timeout, false otherwise.
    #[cfg(test)]
    pub async fn wait_for_peer_connection(
        &self,
        realm_id: &RealmId,
        wait_duration: std::time::Duration,
    ) -> bool {
        let mut events = self.event_tx.subscribe();
        let start = std::time::Instant::now();

        loop {
            let remaining = wait_duration.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                debug!(%realm_id, "Timeout waiting for peer connection");
                return false;
            }

            match tokio::time::timeout(remaining, events.recv()).await {
                Ok(Ok(SyncEvent::PeerConnected {
                    realm_id: event_realm,
                    peer_id,
                })) => {
                    if &event_realm == realm_id {
                        debug!(%realm_id, %peer_id, "Peer connected");
                        return true;
                    }
                }
                Ok(Ok(_)) => {
                    // Other event, keep waiting
                    continue;
                }
                Ok(Err(_)) => {
                    // Channel lagged, keep trying
                    continue;
                }
                Err(_) => {
                    // Timeout
                    debug!(%realm_id, "Timeout waiting for peer connection");
                    return false;
                }
            }
        }
    }

    /// Gracefully shutdown the engine
    ///
    /// Saves all open realms and shuts down the gossip network.
    pub async fn shutdown(mut self) -> Result<(), SyncError> {
        info!("Shutting down SyncEngine");

        // Save all open realms
        let realm_ids: Vec<_> = self.realms.keys().cloned().collect();
        for realm_id in realm_ids {
            if let Err(e) = self.save_realm(&realm_id).await {
                warn!(%realm_id, error = ?e, "Failed to save realm during shutdown");
            }
        }

        // Shutdown gossip
        if let Some(gossip) = self.gossip.take() {
            if let Ok(gossip) = Arc::try_unwrap(gossip) {
                if let Err(e) = gossip.shutdown().await {
                    warn!(error = ?e, "Failed to shutdown gossip cleanly");
                }
            }
        }

        info!("SyncEngine shutdown complete");
        Ok(())
    }
}

/// Information about this node
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// Directory where data is stored
    pub data_dir: PathBuf,
    /// Number of realms
    pub realm_count: usize,
    /// Node's public key (when P2P is active)
    pub node_id: Option<String>,
    /// Relay URL (when P2P is active)
    pub relay_url: Option<String>,
    /// Decentralized identifier (when identity is initialized)
    pub did: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_engine() -> (SyncEngine, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let engine = SyncEngine::new(temp_dir.path()).await.unwrap();
        (engine, temp_dir)
    }

    #[tokio::test]
    async fn test_engine_creates() {
        let (engine, _temp) = create_test_engine().await;
        let info = engine.node_info().await.unwrap();
        assert_eq!(info.realm_count, 0);
    }

    #[tokio::test]
    async fn test_engine_create_realm_persists() {
        let (mut engine, _temp) = create_test_engine().await;

        // Create a realm
        let realm_id = engine.create_realm("Test Realm").await.unwrap();

        // Verify it's in storage
        let realms = engine.list_realms().await.unwrap();
        assert_eq!(realms.len(), 1);
        assert_eq!(realms[0].name, "Test Realm");
        assert_eq!(realms[0].id, realm_id);

        // Verify document exists
        let doc_bytes = engine.storage.load_document(&realm_id).unwrap();
        assert!(doc_bytes.is_some());

        // Verify key exists
        let key = engine.storage.load_realm_key(&realm_id).unwrap();
        assert!(key.is_some());
    }

    #[tokio::test]
    async fn test_engine_open_realm_loads() {
        let temp_dir = TempDir::new().unwrap();

        // Create realm in first engine instance
        let realm_id = {
            let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
            let id = engine.create_realm("Persisted Realm").await.unwrap();

            // Add a task
            engine.add_task(&id, "Persisted task").await.unwrap();

            id
        };

        // Open realm in second engine instance
        let mut engine2 = SyncEngine::new(temp_dir.path()).await.unwrap();
        assert!(!engine2.is_realm_open(&realm_id));

        engine2.open_realm(&realm_id).await.unwrap();
        assert!(engine2.is_realm_open(&realm_id));

        // Verify task was persisted
        let tasks = engine2.list_tasks(&realm_id).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Persisted task");
    }

    #[tokio::test]
    async fn test_engine_task_operations() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm_id = engine.create_realm("Task Test").await.unwrap();

        // Add tasks
        let task1 = engine.add_task(&realm_id, "Task 1").await.unwrap();
        let task2 = engine.add_task(&realm_id, "Task 2").await.unwrap();

        // List tasks
        let tasks = engine.list_tasks(&realm_id).unwrap();
        assert_eq!(tasks.len(), 2);

        // Toggle task
        engine.toggle_task(&realm_id, &task1).await.unwrap();
        let tasks = engine.list_tasks(&realm_id).unwrap();
        let toggled_task = tasks.iter().find(|t| t.id == task1).unwrap();
        assert!(toggled_task.completed);

        // Delete task
        engine.delete_task(&realm_id, &task2).await.unwrap();
        let tasks = engine.list_tasks(&realm_id).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, task1);
    }

    #[tokio::test]
    async fn test_engine_generate_invite() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm_id = engine.create_realm("Invite Test").await.unwrap();

        // Generate invite (starts gossip)
        let invite = engine.generate_invite(&realm_id).await.unwrap();

        // Verify invite contains our realm info
        assert_eq!(invite.realm_id(), realm_id);
        assert!(invite.bootstrap_peers.len() >= 1);

        // Verify realm is marked as shared
        let info = engine.storage.load_realm(&realm_id).unwrap().unwrap();
        assert!(info.is_shared);

        // Clean shutdown
        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_engine_realm_not_found() {
        let (mut engine, _temp) = create_test_engine().await;

        let fake_id = RealmId::new();

        // open_realm should fail
        let result = engine.open_realm(&fake_id).await;
        assert!(matches!(result, Err(SyncError::RealmNotFound(_))));

        // list_tasks should fail (realm not open)
        let result = engine.list_tasks(&fake_id);
        assert!(matches!(result, Err(SyncError::RealmNotFound(_))));
    }

    #[tokio::test]
    async fn test_engine_sync_lifecycle() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm_id = engine.create_realm("Sync Test").await.unwrap();

        // Not syncing initially
        assert!(!engine.is_realm_syncing(&realm_id));

        // Start sync
        engine.start_sync(&realm_id).await.unwrap();
        assert!(engine.is_realm_syncing(&realm_id));

        // Starting again is a no-op
        engine.start_sync(&realm_id).await.unwrap();
        assert!(engine.is_realm_syncing(&realm_id));

        // Stop sync
        engine.stop_sync(&realm_id).await.unwrap();
        assert!(!engine.is_realm_syncing(&realm_id));

        // Stopping again is a no-op
        engine.stop_sync(&realm_id).await.unwrap();
        assert!(!engine.is_realm_syncing(&realm_id));

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_engine_auto_save() {
        let temp_dir = TempDir::new().unwrap();

        let realm_id = {
            let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
            let id = engine.create_realm("Auto Save Test").await.unwrap();

            // Add task (should auto-save)
            engine.add_task(&id, "Auto saved task").await.unwrap();

            id
        };

        // Reload and verify
        let mut engine2 = SyncEngine::new(temp_dir.path()).await.unwrap();
        engine2.open_realm(&realm_id).await.unwrap();

        let tasks = engine2.list_tasks(&realm_id).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Auto saved task");
    }

    #[tokio::test]
    async fn test_create_and_decode_invite() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm_id = engine.create_realm("Shared Realm").await.unwrap();
        let ticket_str = engine.create_invite(&realm_id).await.unwrap();

        // Verify ticket is valid
        assert!(ticket_str.starts_with("sync-invite:"));

        // Realm should now be marked as shared
        let realm = engine.get_realm(&realm_id).await.unwrap().unwrap();
        assert!(realm.is_shared);

        // Decode and verify
        let ticket = InviteTicket::decode(&ticket_str).unwrap();
        assert_eq!(ticket.realm_id(), realm_id);

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_join_realm_already_member() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm_id = engine.create_realm("Shared").await.unwrap();
        let ticket = engine.generate_invite(&realm_id).await.unwrap();

        // Try to join our own realm
        let result = engine.join_via_invite(&ticket).await;
        assert!(matches!(result, Err(SyncError::InvalidInvite(_))));

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_data_persists_across_restarts() {
        let temp_dir = TempDir::new().unwrap();

        // Create engine and data
        {
            let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
            let realm_id = engine.create_realm("Persistent").await.unwrap();
            engine.add_task(&realm_id, "Persisted task").await.unwrap();
        }

        // Reopen and verify
        {
            let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
            let realms = engine.list_realms().await.unwrap();
            assert_eq!(realms.len(), 1);
            assert_eq!(realms[0].name, "Persistent");

            // Need to open the realm to access tasks
            engine.open_realm(&realms[0].id).await.unwrap();
            let tasks = engine.list_tasks(&realms[0].id).unwrap();
            assert_eq!(tasks.len(), 1);
            assert_eq!(tasks[0].title, "Persisted task");
        }
    }

    #[tokio::test]
    async fn test_multiple_realms() {
        let (mut engine, _temp) = create_test_engine().await;

        // Create multiple realms
        let realm1 = engine.create_realm("Realm 1").await.unwrap();
        let realm2 = engine.create_realm("Realm 2").await.unwrap();
        let realm3 = engine.create_realm("Realm 3").await.unwrap();

        // Add tasks to each
        engine.add_task(&realm1, "Task in Realm 1").await.unwrap();
        engine.add_task(&realm2, "Task in Realm 2").await.unwrap();
        engine.add_task(&realm3, "Task in Realm 3").await.unwrap();

        // Verify list_realms
        let realms = engine.list_realms().await.unwrap();
        assert_eq!(realms.len(), 3);

        // Verify tasks are isolated
        assert_eq!(engine.list_tasks(&realm1).unwrap().len(), 1);
        assert_eq!(engine.list_tasks(&realm2).unwrap().len(), 1);
        assert_eq!(engine.list_tasks(&realm3).unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_delete_realm() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm_id = engine.create_realm("To Delete").await.unwrap();
        engine.add_task(&realm_id, "Task").await.unwrap();

        engine.delete_realm(&realm_id).await.unwrap();

        let realm = engine.get_realm(&realm_id).await.unwrap();
        assert!(realm.is_none());

        // Should no longer be open
        assert!(!engine.is_realm_open(&realm_id));
    }

    #[tokio::test]
    async fn test_toggle_task() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm_id = engine.create_realm("Tasks").await.unwrap();
        let task_id = engine.add_task(&realm_id, "Toggle me").await.unwrap();

        // Initially not completed
        let task = engine.get_task(&realm_id, &task_id).unwrap().unwrap();
        assert!(!task.completed);

        // Toggle to completed
        engine.toggle_task(&realm_id, &task_id).await.unwrap();
        let task = engine.get_task(&realm_id, &task_id).unwrap().unwrap();
        assert!(task.completed);

        // Toggle back
        engine.toggle_task(&realm_id, &task_id).await.unwrap();
        let task = engine.get_task(&realm_id, &task_id).unwrap().unwrap();
        assert!(!task.completed);
    }

    #[tokio::test]
    async fn test_init_identity() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initially no identity
        assert!(!engine.has_identity());
        assert!(engine.did().is_none());

        // Initialize identity
        engine.init_identity().unwrap();

        // Now identity exists
        assert!(engine.has_identity());
        let did = engine.did();
        assert!(did.is_some());
        assert!(did.unwrap().as_str().starts_with("did:sync:z"));
    }

    #[tokio::test]
    async fn test_identity_persists() {
        let temp_dir = TempDir::new().unwrap();

        // Create identity in first engine
        let original_did = {
            let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
            engine.init_identity().unwrap();
            engine.did().unwrap().to_string()
        };

        // Load identity in second engine
        {
            let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
            engine.init_identity().unwrap();
            let loaded_did = engine.did().unwrap().to_string();
            assert_eq!(original_did, loaded_did);
        }
    }

    #[tokio::test]
    async fn test_regenerate_identity() {
        let (mut engine, _temp) = create_test_engine().await;

        engine.init_identity().unwrap();
        let original_did = engine.did().unwrap().to_string();

        // Regenerate
        engine.regenerate_identity().unwrap();
        let new_did = engine.did().unwrap().to_string();

        // Should be different
        assert_ne!(original_did, new_did);
    }

    #[tokio::test]
    async fn test_export_public_key_formats() {
        let (mut engine, _temp) = create_test_engine().await;
        engine.init_identity().unwrap();

        // Test base58 format
        let base58 = engine.export_public_key("base58");
        assert!(base58.is_some());

        // Test hex format
        let hex = engine.export_public_key("hex");
        assert!(hex.is_some());

        // Test json format
        let json = engine.export_public_key("json");
        assert!(json.is_some());
        let json_str = json.unwrap();
        assert!(json_str.contains("did"));
        assert!(json_str.contains("public_key_base58"));
    }

    #[tokio::test]
    async fn test_node_info_includes_did() {
        let (mut engine, _temp) = create_test_engine().await;

        // Before init_identity, DID should be None
        let info = engine.node_info().await.unwrap();
        assert!(info.did.is_none());

        // After init_identity, DID should be Some
        engine.init_identity().unwrap();
        let info = engine.node_info().await.unwrap();
        assert!(info.did.is_some());
        assert!(info.did.unwrap().starts_with("did:sync:z"));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Identity Tests (required by task)
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_engine_init_identity_creates_new() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initially no identity
        assert!(!engine.has_identity());
        assert!(engine.did().is_none());
        assert!(engine.public_key().is_none());

        // Initialize identity
        engine.init_identity().unwrap();

        // Identity should now exist
        assert!(engine.has_identity());
        let did = engine.did().unwrap();
        assert!(did.as_str().starts_with("did:sync:z"));

        // Public key should also be available
        let public_key = engine.public_key();
        assert!(public_key.is_some());
    }

    #[tokio::test]
    async fn test_engine_init_identity_loads_existing() {
        let temp_dir = TempDir::new().unwrap();

        // Create identity in first engine instance
        let (original_did, original_pk_bytes) = {
            let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
            engine.init_identity().unwrap();
            let did = engine.did().unwrap();
            let pk = engine.public_key().unwrap();
            (did.to_string(), pk.to_bytes())
        };

        // Load identity in new engine instance
        let mut engine2 = SyncEngine::new(temp_dir.path()).await.unwrap();

        // Before init, no identity in memory
        assert!(!engine2.has_identity());

        // Init should load existing identity
        engine2.init_identity().unwrap();

        // Verify it's the same identity
        let loaded_did = engine2.did().unwrap().to_string();
        let loaded_pk = engine2.public_key().unwrap();

        assert_eq!(original_did, loaded_did);
        assert_eq!(original_pk_bytes, loaded_pk.to_bytes());
    }

    #[tokio::test]
    async fn test_engine_sign_and_verify() {
        let (mut engine, _temp) = create_test_engine().await;

        // Sign without identity should fail
        let message = b"Test message to sign";
        let result = engine.sign(message);
        assert!(result.is_err());

        // Initialize identity
        engine.init_identity().unwrap();

        // Sign should now succeed
        let signature = engine.sign(message).unwrap();

        // Get our public key for verification
        let public_key = engine.public_key().unwrap();

        // Verify should succeed with correct message
        assert!(engine.verify(&public_key, message, &signature));

        // Verify should fail with different message
        let wrong_message = b"Different message";
        assert!(!engine.verify(&public_key, wrong_message, &signature));

        // Verify should fail with different public key
        let other_keypair = crate::identity::HybridKeypair::generate();
        let other_pk = other_keypair.public_key();
        assert!(!engine.verify(&other_pk, message, &signature));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Networking Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_start_networking() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initially not active
        assert!(!engine.is_networking_active());

        // Start networking
        engine.start_networking().await.unwrap();

        // Now active
        assert!(engine.is_networking_active());

        // Node info should have node_id
        let info = engine.node_info().await.unwrap();
        assert!(info.node_id.is_some());

        // Starting again is a no-op (doesn't error)
        engine.start_networking().await.unwrap();
        assert!(engine.is_networking_active());

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_start_networking_then_sync() {
        let (mut engine, _temp) = create_test_engine().await;

        // Create a realm
        let realm_id = engine.create_realm("Sync Test").await.unwrap();

        // Start networking first
        engine.start_networking().await.unwrap();
        assert!(engine.is_networking_active());

        // Now start syncing the realm
        engine.start_sync(&realm_id).await.unwrap();
        assert!(engine.is_realm_syncing(&realm_id));

        engine.shutdown().await.unwrap();
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Sync Envelope Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_broadcast_sync_sends_envelope() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity (required for signing)
        engine.init_identity().unwrap();

        // Create realm and start syncing
        let realm_id = engine.create_realm("Envelope Test").await.unwrap();
        engine.start_sync(&realm_id).await.unwrap();

        // Create a sync message
        let message = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads: vec![vec![0u8; 32]],
            sender_addr: None,
        };

        // Broadcast should succeed (message goes to empty topic, but no error)
        let result = engine.broadcast_sync(&realm_id, message).await;
        assert!(result.is_ok());

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_broadcast_sync_requires_identity() {
        let (mut engine, _temp) = create_test_engine().await;

        // Create realm and start syncing WITHOUT initializing identity
        let realm_id = engine.create_realm("No Identity Test").await.unwrap();
        engine.start_sync(&realm_id).await.unwrap();

        // Create a sync message
        let message = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads: vec![],
            sender_addr: None,
        };

        // Broadcast should fail because identity is not initialized
        let result = engine.broadcast_sync(&realm_id, message).await;
        assert!(matches!(result, Err(SyncError::Identity(_))));

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_broadcast_sync_requires_syncing() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity
        engine.init_identity().unwrap();

        // Create realm but DON'T start syncing
        let realm_id = engine.create_realm("Not Syncing Test").await.unwrap();

        // Create a sync message
        let message = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads: vec![],
            sender_addr: None,
        };

        // Broadcast should fail because realm is not syncing
        let result = engine.broadcast_sync(&realm_id, message).await;
        assert!(matches!(result, Err(SyncError::Gossip(_))));
    }

    #[tokio::test]
    async fn test_handle_incoming_valid_envelope() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity
        engine.init_identity().unwrap();

        // Create realm
        let realm_id = engine.create_realm("Incoming Test").await.unwrap();

        // Get the realm key to create a valid envelope
        let realm_key = engine.storage.load_realm_key(&realm_id).unwrap().unwrap();

        // Create a valid envelope
        let message = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads: vec![vec![1, 2, 3]],
            sender_addr: None,
        };

        let keypair = engine.identity.as_ref().unwrap();
        let sender_did = crate::identity::Did::from_public_key(&keypair.public_key()).to_string();
        let sign_fn = |data: &[u8]| keypair.sign(data).to_bytes();

        let envelope = SyncEnvelope::seal(&message, &sender_did, &realm_key, sign_fn).unwrap();
        let envelope_bytes = envelope.to_bytes().unwrap();

        // Process the envelope
        let result = engine.handle_incoming(&realm_id, &envelope_bytes);
        assert!(result.is_ok());

        let maybe_message = result.unwrap();
        assert!(maybe_message.is_some());

        // Verify the message content
        match maybe_message.unwrap() {
            SyncMessage::Announce { heads, .. } => {
                assert_eq!(heads.len(), 1);
                assert_eq!(heads[0], vec![1, 2, 3]);
            }
            _ => panic!("Expected Announce message"),
        }
    }

    #[tokio::test]
    async fn test_handle_incoming_invalid_signature() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity
        engine.init_identity().unwrap();

        // Create realm
        let realm_id = engine.create_realm("Invalid Sig Test").await.unwrap();

        // Get the realm key
        let realm_key = engine.storage.load_realm_key(&realm_id).unwrap().unwrap();

        // Create an envelope with an empty signature (invalid)
        let message = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads: vec![],
            sender_addr: None,
        };

        let mut envelope = SyncEnvelope::seal(&message, "did:example:test", &realm_key, |_| {
            vec![0x51, 0x9E, 1]
        })
        .unwrap();

        // Tamper with the signature to make it invalid (empty)
        envelope.signature = vec![];

        let envelope_bytes = envelope.to_bytes().unwrap();

        // Process the envelope - should return None because signature is invalid
        let result = engine.handle_incoming(&realm_id, &envelope_bytes);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Returns None for invalid signature
    }

    #[tokio::test]
    async fn test_handle_incoming_wrong_key() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity
        engine.init_identity().unwrap();

        // Create realm
        let realm_id = engine.create_realm("Wrong Key Test").await.unwrap();

        // Create an envelope with a DIFFERENT key
        let wrong_key = [99u8; 32];
        let message = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads: vec![],
            sender_addr: None,
        };

        let keypair = engine.identity.as_ref().unwrap();
        let sender_did = crate::identity::Did::from_public_key(&keypair.public_key()).to_string();
        let sign_fn = |data: &[u8]| keypair.sign(data).to_bytes();

        let envelope = SyncEnvelope::seal(&message, &sender_did, &wrong_key, sign_fn).unwrap();
        let envelope_bytes = envelope.to_bytes().unwrap();

        // Process the envelope - should return None because decryption fails
        let result = engine.handle_incoming(&realm_id, &envelope_bytes);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Returns None for decryption failure
    }

    #[tokio::test]
    async fn test_sync_after_task_add() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity
        engine.init_identity().unwrap();

        // Create realm and start syncing
        let realm_id = engine.create_realm("Auto Sync Test").await.unwrap();
        engine.start_sync(&realm_id).await.unwrap();

        // Add a task
        let task_id = engine.add_task(&realm_id, "Test task").await.unwrap();
        assert!(!task_id.to_string().is_empty());

        // Sync realm state - should broadcast Announce
        let result = engine.sync_realm_state(&realm_id).await;
        assert!(result.is_ok());

        // Broadcast changes - should broadcast Changes message
        let result = engine.broadcast_changes(&realm_id).await;
        assert!(result.is_ok());

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_apply_incoming_changes() {
        let (mut engine, _temp) = create_test_engine().await;

        // Create realm
        let realm_id = engine.create_realm("Apply Changes Test").await.unwrap();

        // Get initial task count
        let initial_tasks = engine.list_tasks(&realm_id).unwrap();
        assert_eq!(initial_tasks.len(), 0);

        // Create a fork with changes
        let state = engine.realms.get_mut(&realm_id).unwrap();
        let mut forked_doc = state.doc.fork();
        forked_doc.add_task("Synced task from peer").unwrap();
        let changes = forked_doc.generate_sync_message();

        // Apply the changes
        engine
            .apply_incoming_changes(&realm_id, &changes)
            .await
            .unwrap();

        // Verify the task was added
        let tasks = engine.list_tasks(&realm_id).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Synced task from peer");
    }

    #[tokio::test]
    async fn test_apply_full_document() {
        let (mut engine, _temp) = create_test_engine().await;

        // Create realm with a task
        let realm_id = engine.create_realm("Apply Doc Test").await.unwrap();
        engine.add_task(&realm_id, "Local task").await.unwrap();

        // Fork our own document to create a "remote" document with shared history
        let remote_bytes = {
            let state = engine.realms.get_mut(&realm_id).unwrap();
            let mut remote_doc = state.doc.fork();
            remote_doc.add_task("Remote task 1").unwrap();
            remote_doc.add_task("Remote task 2").unwrap();
            remote_doc.save()
        };

        // Apply the full document
        engine
            .apply_full_document(&realm_id, &remote_bytes)
            .await
            .unwrap();

        // Verify merge - should have all three tasks
        let tasks = engine.list_tasks(&realm_id).unwrap();
        assert_eq!(tasks.len(), 3);

        let titles: Vec<_> = tasks.iter().map(|t| t.title.as_str()).collect();
        assert!(titles.contains(&"Local task"));
        assert!(titles.contains(&"Remote task 1"));
        assert!(titles.contains(&"Remote task 2"));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Multi-Realm Sync Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_sync_status_returns_idle_for_unsynced_realm() {
        let (engine, _temp) = create_test_engine().await;
        let realm_id = RealmId::new();

        assert_eq!(engine.sync_status(&realm_id), SyncStatus::Idle);
    }

    #[tokio::test]
    async fn test_sync_status_changes_when_sync_starts() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm_id = engine.create_realm("Status Test").await.unwrap();

        // Initially idle
        assert_eq!(engine.sync_status(&realm_id), SyncStatus::Idle);

        // Start sync
        engine.start_sync(&realm_id).await.unwrap();

        // Should be syncing now
        let status = engine.sync_status(&realm_id);
        assert!(matches!(status, SyncStatus::Syncing { .. }));

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_sync_status_returns_idle_after_stop() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm_id = engine.create_realm("Stop Status Test").await.unwrap();

        // Start and stop sync
        engine.start_sync(&realm_id).await.unwrap();
        assert!(matches!(
            engine.sync_status(&realm_id),
            SyncStatus::Syncing { .. }
        ));

        engine.stop_sync(&realm_id).await.unwrap();
        assert_eq!(engine.sync_status(&realm_id), SyncStatus::Idle);

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple_realms_can_sync_concurrently() {
        let (mut engine, _temp) = create_test_engine().await;

        // Create multiple realms
        let realm1 = engine.create_realm("Realm 1").await.unwrap();
        let realm2 = engine.create_realm("Realm 2").await.unwrap();
        let realm3 = engine.create_realm("Realm 3").await.unwrap();

        // Start syncing all three
        engine.start_sync(&realm1).await.unwrap();
        engine.start_sync(&realm2).await.unwrap();
        engine.start_sync(&realm3).await.unwrap();

        // All should be syncing
        assert!(engine.is_realm_syncing(&realm1));
        assert!(engine.is_realm_syncing(&realm2));
        assert!(engine.is_realm_syncing(&realm3));

        // Check status for each
        assert!(matches!(
            engine.sync_status(&realm1),
            SyncStatus::Syncing { .. }
        ));
        assert!(matches!(
            engine.sync_status(&realm2),
            SyncStatus::Syncing { .. }
        ));
        assert!(matches!(
            engine.sync_status(&realm3),
            SyncStatus::Syncing { .. }
        ));

        // Verify syncing_count
        assert_eq!(engine.syncing_count(), 3);

        // Verify syncing_realms
        let syncing = engine.syncing_realms();
        assert!(syncing.contains(&realm1));
        assert!(syncing.contains(&realm2));
        assert!(syncing.contains(&realm3));

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_start_sync_multiple_realms_without_blocking() {
        let (mut engine, _temp) = create_test_engine().await;

        // Create realms
        let realm1 = engine.create_realm("Realm A").await.unwrap();
        let realm2 = engine.create_realm("Realm B").await.unwrap();

        // Measure time for starting both syncs
        let start = std::time::Instant::now();

        engine.start_sync(&realm1).await.unwrap();
        engine.start_sync(&realm2).await.unwrap();

        let elapsed = start.elapsed();

        // Should complete quickly (not blocking)
        assert!(
            elapsed.as_millis() < 5000,
            "start_sync took too long: {:?}",
            elapsed
        );

        // Both should be syncing
        assert!(engine.is_realm_syncing(&realm1));
        assert!(engine.is_realm_syncing(&realm2));

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_stop_one_realm_does_not_affect_others() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm1 = engine.create_realm("Keep Syncing").await.unwrap();
        let realm2 = engine.create_realm("Stop Syncing").await.unwrap();

        engine.start_sync(&realm1).await.unwrap();
        engine.start_sync(&realm2).await.unwrap();

        // Stop only realm2
        engine.stop_sync(&realm2).await.unwrap();

        // realm1 should still be syncing
        assert!(engine.is_realm_syncing(&realm1));
        assert!(!engine.is_realm_syncing(&realm2));

        assert!(matches!(
            engine.sync_status(&realm1),
            SyncStatus::Syncing { .. }
        ));
        assert_eq!(engine.sync_status(&realm2), SyncStatus::Idle);

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_subscribe_events_receives_status_changes() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm_id = engine.create_realm("Event Test").await.unwrap();

        // Subscribe before starting sync
        let mut events = engine.subscribe_events();

        // Start sync - should emit events
        engine.start_sync(&realm_id).await.unwrap();

        // Try to receive events (with timeout to avoid hanging)
        let mut found_connecting = false;
        let mut found_syncing = false;

        // Use a short timeout for testing
        for _ in 0..10 {
            match tokio::time::timeout(std::time::Duration::from_millis(100), events.recv()).await {
                Ok(Ok(SyncEvent::StatusChanged {
                    status: SyncStatus::Connecting,
                    ..
                })) => {
                    found_connecting = true;
                }
                Ok(Ok(SyncEvent::StatusChanged {
                    status: SyncStatus::Syncing { .. },
                    ..
                })) => {
                    found_syncing = true;
                }
                _ => break,
            }
        }

        // Should have received at least the connecting or syncing event
        assert!(
            found_connecting || found_syncing,
            "Should receive status change events"
        );

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_syncing_realms_returns_only_active() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm1 = engine.create_realm("Active 1").await.unwrap();
        let realm2 = engine.create_realm("Active 2").await.unwrap();
        let realm3 = engine.create_realm("Inactive").await.unwrap();

        // Start syncing only realm1 and realm2
        engine.start_sync(&realm1).await.unwrap();
        engine.start_sync(&realm2).await.unwrap();

        let syncing = engine.syncing_realms();
        assert_eq!(syncing.len(), 2);
        assert!(syncing.contains(&realm1));
        assert!(syncing.contains(&realm2));
        assert!(!syncing.contains(&realm3));

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_syncing_count_tracks_active_syncs() {
        let (mut engine, _temp) = create_test_engine().await;

        assert_eq!(engine.syncing_count(), 0);

        let realm1 = engine.create_realm("Count 1").await.unwrap();
        let realm2 = engine.create_realm("Count 2").await.unwrap();

        engine.start_sync(&realm1).await.unwrap();
        assert_eq!(engine.syncing_count(), 1);

        engine.start_sync(&realm2).await.unwrap();
        assert_eq!(engine.syncing_count(), 2);

        engine.stop_sync(&realm1).await.unwrap();
        assert_eq!(engine.syncing_count(), 1);

        engine.stop_sync(&realm2).await.unwrap();
        assert_eq!(engine.syncing_count(), 0);

        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_can_add_tasks_while_syncing_multiple_realms() {
        let (mut engine, _temp) = create_test_engine().await;

        let realm1 = engine.create_realm("Tasks Realm 1").await.unwrap();
        let realm2 = engine.create_realm("Tasks Realm 2").await.unwrap();

        // Start syncing both
        engine.start_sync(&realm1).await.unwrap();
        engine.start_sync(&realm2).await.unwrap();

        // Add tasks to both realms while syncing
        let task1 = engine.add_task(&realm1, "Task in Realm 1").await.unwrap();
        let task2 = engine.add_task(&realm2, "Task in Realm 2").await.unwrap();

        // Verify tasks were added
        let tasks1 = engine.list_tasks(&realm1).unwrap();
        let tasks2 = engine.list_tasks(&realm2).unwrap();

        assert_eq!(tasks1.len(), 1);
        assert_eq!(tasks2.len(), 1);
        assert_eq!(tasks1[0].id, task1);
        assert_eq!(tasks2[0].id, task2);

        engine.shutdown().await.unwrap();
    }

    /// Test that two engines can sync tasks via invite flow
    ///
    /// This is the critical user flow:
    /// 1. Alice creates a realm and generates an invite
    /// 2. Bob joins via the invite
    /// 3. Alice adds a task
    /// 4. Bob should see Alice's task (after sync propagates)
    #[tokio::test]
    async fn test_two_engines_sync_tasks_via_invite() {
        use std::time::Duration;

        let _ = tracing_subscriber::fmt::try_init();

        // Create Alice's engine
        let temp_dir_alice = TempDir::new().unwrap();
        let mut alice = SyncEngine::new(temp_dir_alice.path()).await.unwrap();
        alice.init_identity().unwrap(); // Required for signing sync messages

        // Create Bob's engine
        let temp_dir_bob = TempDir::new().unwrap();
        let mut bob = SyncEngine::new(temp_dir_bob.path()).await.unwrap();
        bob.init_identity().unwrap(); // Required for signing sync messages

        // CRITICAL: Start networking on BOTH engines and exchange addresses BEFORE
        // subscribing to any gossip topics. This matches the pattern used in the
        // working p2p_integration tests. The iroh-gossip layer seems to require
        // peer addresses to be in the static discovery BEFORE topic subscription
        // for message delivery to work properly.
        alice.start_networking().await.unwrap();
        bob.start_networking().await.unwrap();

        // Exchange peer addresses bidirectionally
        if let (Some(alice_addr), Some(bob_addr)) = (alice.endpoint_addr(), bob.endpoint_addr()) {
            debug!("Adding bidirectional peer addresses before gossip subscription");
            alice.add_peer_addr(bob_addr);
            bob.add_peer_addr(alice_addr);
        }

        // Small delay to let discovery propagate
        tokio::time::sleep(Duration::from_millis(50)).await;

        // CRITICAL: Subscribe to events BEFORE any sync operations start.
        // This avoids the race condition where PeerConnected events fire
        // before we start listening for them.
        let mut alice_events = alice.subscribe_events();
        let mut bob_events = bob.subscribe_events();

        // Alice creates a realm
        let realm_id = alice.create_realm("Shared Tasks").await.unwrap();

        // Alice generates an invite (this should auto-start sync!)
        let invite_str = alice.create_invite(&realm_id).await.unwrap();
        debug!(
            "Alice generated invite: {}...",
            &invite_str[..50.min(invite_str.len())]
        );

        // Verify Alice is now syncing
        assert!(
            alice.is_realm_syncing(&realm_id),
            "Alice should be syncing after generating invite"
        );

        // Bob joins via invite
        let joined_realm_id = bob.join_realm(&invite_str).await.unwrap();
        assert_eq!(joined_realm_id, realm_id, "Bob should join the same realm");

        // Verify Bob is syncing
        assert!(
            bob.is_realm_syncing(&realm_id),
            "Bob should be syncing after joining"
        );

        // CRITICAL: Wait for peers to connect before sending any messages.
        // The gossip mesh takes time to establish, and messages sent before
        // neighbors are connected won't be delivered.
        debug!("Waiting for peer connections to establish...");

        // Helper to wait for PeerConnected event on an existing subscription
        async fn wait_for_peer_connected(
            events: &mut broadcast::Receiver<SyncEvent>,
            target_realm: &RealmId,
            timeout_duration: Duration,
        ) -> bool {
            let start = std::time::Instant::now();
            loop {
                let remaining = timeout_duration.saturating_sub(start.elapsed());
                if remaining.is_zero() {
                    return false;
                }
                match tokio::time::timeout(remaining, events.recv()).await {
                    Ok(Ok(SyncEvent::PeerConnected { realm_id, peer_id })) => {
                        if &realm_id == target_realm {
                            debug!(%realm_id, %peer_id, "Peer connected event received");
                            return true;
                        }
                    }
                    Ok(Ok(_)) => continue, // Other event
                    Ok(Err(broadcast::error::RecvError::Lagged(_))) => continue, // Lagged, keep trying
                    Ok(Err(_)) => return false,                                  // Channel closed
                    Err(_) => return false,                                      // Timeout
                }
            }
        }

        // Wait for connections using the pre-made subscriptions
        let alice_connected =
            wait_for_peer_connected(&mut alice_events, &realm_id, Duration::from_secs(10)).await;
        let bob_connected =
            wait_for_peer_connected(&mut bob_events, &realm_id, Duration::from_secs(10)).await;

        debug!(alice_connected, bob_connected, "Peer connection status");

        // At least one side should see a connection (typically Bob sees Alice first since he used Alice as bootstrap)
        assert!(
            alice_connected || bob_connected,
            "At least one peer should have connected within 10 seconds"
        );

        // Give time for gossip mesh to stabilize after peer connection.
        // The iroh-gossip layer needs time to establish reliable message routing.
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Process any pending BroadcastRequests from NeighborUp events
        alice.process_pending_sync();
        bob.process_pending_sync();

        // Additional stabilization - let any triggered broadcasts complete
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Alice adds a task (this should broadcast via sync)
        debug!("Alice adding task...");
        let _task_id = alice.add_task(&realm_id, "Alice's task").await.unwrap();
        debug!("Alice added task, waiting for sync to Bob...");

        // Wait for sync to propagate to Bob (up to 5 seconds)
        let mut synced = false;
        for i in 0..50 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            // Process any pending sync messages
            let processed = bob.process_pending_sync();
            if processed > 0 {
                debug!(
                    "Bob processed {} sync messages at iteration {}",
                    processed, i
                );
            }
            let bob_tasks = bob.list_tasks(&realm_id).unwrap();
            debug!(
                iteration = i,
                processed,
                task_count = bob_tasks.len(),
                "Checking Bob's tasks"
            );
            if !bob_tasks.is_empty() {
                debug!("Bob received task after {}ms", (i + 1) * 100);
                assert_eq!(bob_tasks[0].title, "Alice's task");
                synced = true;
                break;
            }
        }

        assert!(
            synced,
            "Bob should have received Alice's task within 5 seconds"
        );

        // Cleanup
        alice.shutdown().await.unwrap();
        bob.shutdown().await.unwrap();
    }

    /// Regression test for sync persistence bug
    ///
    /// Verifies that sync changes (from gossip messages) are saved to disk
    /// and persist across engine restarts. This test simulates:
    /// 1. Receiving a full document sync message
    /// 2. Applying it to memory
    /// 3. Restarting the engine
    /// 4. Verifying the synced data persisted to disk
    #[tokio::test]
    async fn test_sync_changes_persist_across_restart() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        // Phase 1: Create engine, create realm, simulate receiving sync data
        let realm_id = {
            let mut engine = SyncEngine::new(data_dir).await.unwrap();
            engine.init_identity().unwrap();

            // Create a realm
            let realm_id = engine.create_realm("Test Realm").await.unwrap();

            // Create a separate document with a task (simulating what we'd receive from a peer)
            let mut remote_doc = RealmDoc::new();
            let _ = remote_doc.add_task("Synced Task from Peer");

            // Simulate receiving a full document sync (SyncResponse message)
            let remote_doc_bytes = remote_doc.save();
            let result = engine.apply_sync_changes(&realm_id, &remote_doc_bytes, true);
            assert!(result.is_ok(), "Should apply sync changes successfully");

            // Verify the task exists in memory
            let tasks = engine.list_tasks(&realm_id).unwrap();
            assert_eq!(tasks.len(), 1, "Should have 1 task in memory after sync");
            assert_eq!(tasks[0].title, "Synced Task from Peer");

            realm_id
        }; // Engine drops here, releasing all resources

        // Phase 2: Create new engine instance with same data directory
        {
            let mut engine2 = SyncEngine::new(data_dir).await.unwrap();
            engine2.init_identity().unwrap();

            // Open the realm (loads from disk)
            engine2.open_realm(&realm_id).await.unwrap();

            // Verify the synced task persisted to disk
            let tasks = engine2.list_tasks(&realm_id).unwrap();
            assert_eq!(
                tasks.len(),
                1,
                "Should have 1 task after restart (loaded from disk)"
            );
            assert_eq!(
                tasks[0].title, "Synced Task from Peer",
                "Synced task should persist across restart"
            );
        }
    }

    /// Test that incremental sync changes also persist
    #[tokio::test]
    async fn test_incremental_sync_changes_persist() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        let realm_id = {
            let mut engine = SyncEngine::new(data_dir).await.unwrap();
            engine.init_identity().unwrap();

            // Create a realm with an initial task
            let realm_id = engine.create_realm("Test Realm").await.unwrap();
            engine.add_task(&realm_id, "Initial Task").await.unwrap();

            // Get the current document state
            let state = engine.realms.get_mut(&realm_id).unwrap();
            let initial_doc_bytes = state.doc.save();

            // Create a remote doc with the same initial state
            let mut remote_doc = RealmDoc::load(&initial_doc_bytes).unwrap();
            remote_doc.add_task("Synced Task").unwrap();

            // Generate incremental changes using the public API
            let changes = remote_doc.generate_sync_message();

            // Apply incremental sync changes
            let result = engine.apply_sync_changes(&realm_id, &changes, false);
            assert!(result.is_ok(), "Should apply incremental changes");

            // Verify both tasks exist in memory
            let tasks = engine.list_tasks(&realm_id).unwrap();
            assert_eq!(tasks.len(), 2, "Should have 2 tasks after incremental sync");

            realm_id
        };

        // Restart and verify
        {
            let mut engine2 = SyncEngine::new(data_dir).await.unwrap();
            engine2.init_identity().unwrap();
            engine2.open_realm(&realm_id).await.unwrap();

            let tasks = engine2.list_tasks(&realm_id).unwrap();
            assert_eq!(
                tasks.len(),
                2,
                "Both tasks should persist after restart (incremental sync)"
            );
        }
    }

    /// Test that bootstrap peers are saved during join_via_invite and loaded during start_sync
    ///
    /// This regression test verifies the fix for the "sync lost after restart" bug where:
    /// - Bootstrap peers from the invite were used once during initial join
    /// - After restart, start_sync() passed empty peers, creating isolated nodes
    ///
    /// The fix saves bootstrap_peers in RealmInfo and loads them in start_sync().
    #[tokio::test]
    async fn test_bootstrap_peers_persist_for_reconnection() {
        use crate::invite::NodeAddrBytes;

        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        // Create a mock bootstrap peer address
        let mock_peer = NodeAddrBytes {
            node_id: [42u8; 32],
            relay_url: Some("https://relay.example.com".to_string()),
            direct_addresses: vec!["192.168.1.100:4433".to_string()],
        };

        // Phase 1: Create engine, create a realm and manually set up RealmInfo with peers
        // (simulating what happens after join_via_invite)
        let realm_id = {
            let mut engine = SyncEngine::new(data_dir).await.unwrap();
            engine.init_identity().unwrap();

            // Create a realm
            let realm_id = engine.create_realm("Shared Test Realm").await.unwrap();

            // Simulate what join_via_invite does: update the realm with bootstrap peers
            let mut info = engine.storage.load_realm(&realm_id).unwrap().unwrap();
            info.is_shared = true;
            info.bootstrap_peers = vec![mock_peer.clone()];
            engine.storage.save_realm(&info).unwrap();

            realm_id
        }; // Engine drops here

        // Phase 2: Create new engine instance and verify peers are loaded from storage
        {
            let engine2 = SyncEngine::new(data_dir).await.unwrap();

            // Load the realm info from storage
            let loaded_info = engine2.storage.load_realm(&realm_id).unwrap().unwrap();

            // Verify bootstrap peers were persisted
            assert_eq!(
                loaded_info.bootstrap_peers.len(),
                1,
                "Bootstrap peers should persist to storage"
            );
            assert_eq!(
                loaded_info.bootstrap_peers[0].node_id, mock_peer.node_id,
                "Peer node_id should match"
            );
            assert_eq!(
                loaded_info.bootstrap_peers[0].relay_url, mock_peer.relay_url,
                "Peer relay_url should match"
            );
            assert_eq!(
                loaded_info.bootstrap_peers[0].direct_addresses, mock_peer.direct_addresses,
                "Peer direct_addresses should match"
            );
        }
    }

    /// Test that RealmInfo can roundtrip through storage with and without bootstrap_peers
    /// (backwards compatibility - old realms without the field should still load)
    #[tokio::test]
    async fn test_realm_info_backwards_compatible() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        let mut engine = SyncEngine::new(data_dir).await.unwrap();
        engine.init_identity().unwrap();

        // Create a realm using the constructor (which sets bootstrap_peers to empty vec)
        let realm_id = engine.create_realm("Legacy Realm").await.unwrap();

        // Load and verify it has an empty bootstrap_peers vec
        let info = engine.storage.load_realm(&realm_id).unwrap().unwrap();
        assert!(
            info.bootstrap_peers.is_empty(),
            "New realms should have empty bootstrap_peers by default"
        );

        // The #[serde(default)] attribute ensures old stored realms without
        // the bootstrap_peers field will deserialize with an empty vec
    }

    /// Test that peer addresses learned from Announce messages are persisted to storage
    ///
    /// This regression test verifies the fix for the "creator can't reconnect after restart" bug:
    /// - Creator creates realm and has EMPTY bootstrap_peers
    /// - Joiner joins and sends Announce with their address
    /// - Creator MUST persist that address to bootstrap_peers
    /// - After restart, creator can use saved peers to reconnect
    #[tokio::test]
    async fn test_creator_learns_and_persists_peer_from_announce() {
        use crate::invite::NodeAddrBytes;
        use crate::sync::{SyncEnvelope, SyncMessage};

        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path();

        let mut engine = SyncEngine::new(data_dir).await.unwrap();
        engine.init_identity().unwrap();

        // Create a realm (simulating the "love" instance)
        let realm_id = engine.create_realm("Creator Realm").await.unwrap();

        // Verify creator starts with NO bootstrap peers
        let info_before = engine.storage.load_realm(&realm_id).unwrap().unwrap();
        assert!(
            info_before.bootstrap_peers.is_empty(),
            "Creator should start with empty bootstrap_peers"
        );

        // Mark realm as shared (happens during generate_invite)
        let mut info = info_before.clone();
        info.is_shared = true;
        engine.storage.save_realm(&info).unwrap();

        // Save a realm key (required for envelope handling)
        let realm_key = [42u8; 32];
        engine
            .storage
            .save_realm_key(&realm_id, &realm_key)
            .unwrap();

        // Open the realm so it's in memory
        engine.open_realm(&realm_id).await.unwrap();

        // Update the in-memory state with the realm key
        if let Some(state) = engine.realms.get_mut(&realm_id) {
            state.realm_key = realm_key;
        }

        // Simulate receiving an Announce from a joiner with their address
        let joiner_addr = NodeAddrBytes {
            node_id: [99u8; 32], // Different node
            relay_url: Some("https://relay.joiner.com".to_string()),
            direct_addresses: vec!["10.0.0.50:4433".to_string()],
        };

        let announce = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads: vec![],
            sender_addr: Some(joiner_addr.clone()),
        };

        // Create signed envelopes in separate scopes to avoid borrow conflicts
        let envelope_bytes = {
            let identity = engine.identity.as_ref().unwrap();
            let sender_did =
                crate::identity::Did::from_public_key(&identity.public_key()).to_string();
            let sign_fn = |data: &[u8]| identity.sign(data).to_bytes().to_vec();
            let envelope = SyncEnvelope::seal(&announce, &sender_did, &realm_key, sign_fn)
                .expect("Should create envelope");
            envelope.to_bytes().expect("Should serialize")
        };

        // Send to the sync channel (simulating incoming gossip message)
        engine
            .sync_tx
            .send(SyncChannelMessage::IncomingData {
                realm_id: realm_id.clone(),
                envelope_bytes,
            })
            .expect("Should send");

        // Process pending sync messages (this should persist the peer)
        engine.process_pending_sync();

        // Verify the joiner's address was persisted
        let info_after = engine.storage.load_realm(&realm_id).unwrap().unwrap();
        assert_eq!(
            info_after.bootstrap_peers.len(),
            1,
            "Creator should have learned 1 peer from announce"
        );
        assert_eq!(
            info_after.bootstrap_peers[0].node_id, joiner_addr.node_id,
            "Saved peer should match joiner's node_id"
        );
        assert_eq!(
            info_after.bootstrap_peers[0].relay_url, joiner_addr.relay_url,
            "Saved peer should match joiner's relay_url"
        );

        // Verify idempotency - processing the same announce again shouldn't duplicate
        let envelope_bytes2 = {
            let identity = engine.identity.as_ref().unwrap();
            let sender_did =
                crate::identity::Did::from_public_key(&identity.public_key()).to_string();
            let sign_fn = |data: &[u8]| identity.sign(data).to_bytes().to_vec();
            let envelope = SyncEnvelope::seal(&announce, &sender_did, &realm_key, sign_fn)
                .expect("Should create envelope");
            envelope.to_bytes().expect("Should serialize")
        };

        engine
            .sync_tx
            .send(SyncChannelMessage::IncomingData {
                realm_id: realm_id.clone(),
                envelope_bytes: envelope_bytes2,
            })
            .expect("Should send");
        engine.process_pending_sync();

        let info_final = engine.storage.load_realm(&realm_id).unwrap().unwrap();
        assert_eq!(
            info_final.bootstrap_peers.len(),
            1,
            "Should not duplicate peer on repeated announce"
        );
    }

    /// Test that sync_status() returns updated peer count when peers connect
    ///
    /// This is a TDD test that verifies the fix for the peer counting bug:
    /// - Two engines connect via gossip
    /// - Both should report `Syncing { peer_count: 1 }` (or more) after connection
    ///
    /// ROOT CAUSE BEING TESTED: The listener task spawned in start_sync_internal
    /// receives NeighborUp/NeighborDown events but the sync_status HashMap is
    /// not accessible from that task (it's not thread-safe).
    #[tokio::test]
    async fn test_sync_status_updates_peer_count() {
        use std::time::Duration;

        let _ = tracing_subscriber::fmt::try_init();

        // Create Alice's engine
        let temp_dir_alice = TempDir::new().unwrap();
        let mut alice = SyncEngine::new(temp_dir_alice.path()).await.unwrap();
        alice.init_identity().unwrap();

        // Create Bob's engine
        let temp_dir_bob = TempDir::new().unwrap();
        let mut bob = SyncEngine::new(temp_dir_bob.path()).await.unwrap();
        bob.init_identity().unwrap();

        // Start networking on both
        alice.start_networking().await.unwrap();
        bob.start_networking().await.unwrap();

        // Exchange peer addresses bidirectionally
        if let (Some(alice_addr), Some(bob_addr)) = (alice.endpoint_addr(), bob.endpoint_addr()) {
            alice.add_peer_addr(bob_addr);
            bob.add_peer_addr(alice_addr);
        }

        // Small delay to let discovery propagate
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Subscribe to events before sync starts
        let mut alice_events = alice.subscribe_events();
        let mut bob_events = bob.subscribe_events();

        // Alice creates a realm and generates an invite
        let realm_id = alice.create_realm("Peer Count Test").await.unwrap();
        let invite_str = alice.create_invite(&realm_id).await.unwrap();

        // Verify Alice is syncing (but peer_count should be 0 initially)
        let alice_status_initial = alice.sync_status(&realm_id);
        info!(?alice_status_initial, "Alice initial sync status");
        assert!(
            matches!(alice_status_initial, SyncStatus::Syncing { .. }),
            "Alice should be in Syncing state"
        );

        // Bob joins via invite
        let _joined_realm_id = bob.join_realm(&invite_str).await.unwrap();

        // Wait for PeerConnected events (using existing helper pattern)
        async fn wait_for_peer(
            events: &mut broadcast::Receiver<SyncEvent>,
            target_realm: &RealmId,
        ) -> bool {
            let timeout = Duration::from_secs(10);
            let start = std::time::Instant::now();
            loop {
                let remaining = timeout.saturating_sub(start.elapsed());
                if remaining.is_zero() {
                    return false;
                }
                match tokio::time::timeout(remaining, events.recv()).await {
                    Ok(Ok(SyncEvent::PeerConnected { realm_id, .. })) => {
                        if &realm_id == target_realm {
                            return true;
                        }
                    }
                    Ok(Ok(_)) => continue,
                    Ok(Err(broadcast::error::RecvError::Lagged(_))) => continue,
                    Ok(Err(_)) => return false,
                    Err(_) => return false,
                }
            }
        }

        // Wait for at least one peer connection
        let alice_connected = wait_for_peer(&mut alice_events, &realm_id).await;
        let bob_connected = wait_for_peer(&mut bob_events, &realm_id).await;

        debug!(
            alice_connected,
            bob_connected, "Peer connection events received"
        );

        assert!(
            alice_connected || bob_connected,
            "At least one peer should have connected"
        );

        // Give a moment for status to settle
        tokio::time::sleep(Duration::from_millis(200)).await;

        // THE CRITICAL ASSERTION: After peers connect, sync_status should report peer_count >= 1
        let alice_status = alice.sync_status(&realm_id);
        let bob_status = bob.sync_status(&realm_id);

        info!(
            ?alice_status,
            ?bob_status,
            "Final sync status after peer connection"
        );

        // Check Alice's peer count
        match alice_status {
            SyncStatus::Syncing { peer_count } => {
                assert!(
                    peer_count >= 1,
                    "Alice should report at least 1 peer, but got peer_count={}",
                    peer_count
                );
            }
            other => panic!("Alice should be Syncing, but got {:?}", other),
        }

        // Check Bob's peer count
        match bob_status {
            SyncStatus::Syncing { peer_count } => {
                assert!(
                    peer_count >= 1,
                    "Bob should report at least 1 peer, but got peer_count={}",
                    peer_count
                );
            }
            other => panic!("Bob should be Syncing, but got {:?}", other),
        }

        // Cleanup
        alice.shutdown().await.unwrap();
        bob.shutdown().await.unwrap();
    }

    /// Test that offline changes sync correctly after restart
    ///
    /// This verifies the complete offline-to-online sync flow:
    /// 1. Two engines sync initially
    /// 2. Both shut down
    /// 3. Each adds tasks while offline (different tasks)
    /// 4. Both restart and reconnect
    /// 5. After sync, both should have ALL tasks from BOTH peers
    ///
    /// This tests Automerge's CRDT merge behavior for offline changes.
    #[tokio::test]
    async fn test_offline_changes_sync_after_restart() {
        use std::time::Duration;

        let _ = tracing_subscriber::fmt::try_init();

        // Use persistent directories
        let temp_dir_alice = TempDir::new().unwrap();
        let temp_dir_bob = TempDir::new().unwrap();
        let alice_path = temp_dir_alice.path().to_path_buf();
        let bob_path = temp_dir_bob.path().to_path_buf();

        // === Phase 1: Initial sync setup ===
        let realm_id = {
            let mut alice = SyncEngine::new(&alice_path).await.unwrap();
            alice.init_identity().unwrap();

            let mut bob = SyncEngine::new(&bob_path).await.unwrap();
            bob.init_identity().unwrap();

            // Start networking
            alice.start_networking().await.unwrap();
            bob.start_networking().await.unwrap();

            // Exchange addresses
            if let (Some(alice_addr), Some(bob_addr)) = (alice.endpoint_addr(), bob.endpoint_addr())
            {
                alice.add_peer_addr(bob_addr);
                bob.add_peer_addr(alice_addr);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Subscribe to events
            let mut alice_events = alice.subscribe_events();

            // Create realm and invite
            let realm_id = alice.create_realm("Offline Sync Test").await.unwrap();
            let invite_str = alice.create_invite(&realm_id).await.unwrap();

            // Bob joins
            let _joined = bob.join_realm(&invite_str).await.unwrap();

            // Wait for connection
            let connected = tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    match alice_events.recv().await {
                        Ok(SyncEvent::PeerConnected { realm_id: r, .. }) if r == realm_id => {
                            return true;
                        }
                        Ok(_) => continue,
                        Err(_) => return false,
                    }
                }
            })
            .await
            .unwrap_or(false);

            assert!(connected, "Initial connection should succeed");

            // Alice adds initial task and waits for sync
            alice
                .add_task(&realm_id, "Initial shared task")
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(500)).await;
            bob.process_pending_sync();

            let bob_tasks = bob.list_tasks(&realm_id).unwrap();
            assert!(!bob_tasks.is_empty(), "Bob should have initial task");

            // Shutdown both
            alice.shutdown().await.unwrap();
            bob.shutdown().await.unwrap();

            realm_id
        };

        info!(?realm_id, "Phase 1 complete - both engines shutdown");

        // === Phase 2: Offline changes ===
        // Alice adds tasks while offline (no networking)
        {
            let mut alice = SyncEngine::new(&alice_path).await.unwrap();
            alice.init_identity().unwrap();
            alice.open_realm(&realm_id).await.unwrap();

            alice
                .add_task(&realm_id, "Alice offline task 1")
                .await
                .unwrap();
            alice
                .add_task(&realm_id, "Alice offline task 2")
                .await
                .unwrap();

            let alice_tasks = alice.list_tasks(&realm_id).unwrap();
            info!(count = alice_tasks.len(), "Alice offline tasks");

            // Shutdown without starting networking
            alice.shutdown().await.unwrap();
        }

        // Bob adds tasks while offline
        {
            let mut bob = SyncEngine::new(&bob_path).await.unwrap();
            bob.init_identity().unwrap();
            bob.open_realm(&realm_id).await.unwrap();

            bob.add_task(&realm_id, "Bob offline task 1").await.unwrap();
            bob.add_task(&realm_id, "Bob offline task 2").await.unwrap();

            let bob_tasks = bob.list_tasks(&realm_id).unwrap();
            info!(count = bob_tasks.len(), "Bob offline tasks");

            bob.shutdown().await.unwrap();
        }

        info!("Phase 2 complete - both added offline tasks");

        // === Phase 3: Restart and sync ===
        let mut alice = SyncEngine::new(&alice_path).await.unwrap();
        alice.init_identity().unwrap();

        let mut bob = SyncEngine::new(&bob_path).await.unwrap();
        bob.init_identity().unwrap();

        // Start networking
        alice.start_networking().await.unwrap();
        bob.start_networking().await.unwrap();

        // Exchange addresses AND update bootstrap_peers in storage
        // This simulates what happens when users re-exchange invites after restart
        // (The iroh endpoint has a new node ID after restart, so old saved addresses are stale)
        if let (Some(alice_addr), Some(bob_addr)) = (alice.endpoint_addr(), bob.endpoint_addr()) {
            alice.add_peer_addr(bob_addr.clone());
            bob.add_peer_addr(alice_addr.clone());

            // Update Alice's realm with Bob's fresh address as bootstrap peer
            if let Ok(Some(mut alice_realm_info)) = alice.storage.load_realm(&realm_id) {
                alice_realm_info.bootstrap_peers =
                    vec![NodeAddrBytes::from_endpoint_addr(&bob_addr)];
                alice.storage.save_realm(&alice_realm_info).unwrap();
            }

            // Update Bob's realm with Alice's fresh address as bootstrap peer
            if let Ok(Some(mut bob_realm_info)) = bob.storage.load_realm(&realm_id) {
                bob_realm_info.bootstrap_peers =
                    vec![NodeAddrBytes::from_endpoint_addr(&alice_addr)];
                bob.storage.save_realm(&bob_realm_info).unwrap();
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Open realms (should auto-start sync for shared realms)
        alice.open_realm(&realm_id).await.unwrap();
        bob.open_realm(&realm_id).await.unwrap();

        // Wait for sync to complete
        let mut synced = false;
        for i in 0..100 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            alice.process_pending_sync();
            bob.process_pending_sync();

            let alice_tasks = alice.list_tasks(&realm_id).unwrap();
            let bob_tasks = bob.list_tasks(&realm_id).unwrap();

            debug!(
                iteration = i,
                alice_count = alice_tasks.len(),
                bob_count = bob_tasks.len(),
                "Checking sync progress"
            );

            // Both should have 5 tasks:
            // 1. Initial shared task
            // 2. Alice offline task 1
            // 3. Alice offline task 2
            // 4. Bob offline task 1
            // 5. Bob offline task 2
            if alice_tasks.len() >= 5 && bob_tasks.len() >= 5 {
                synced = true;
                break;
            }
        }

        let alice_tasks = alice.list_tasks(&realm_id).unwrap();
        let bob_tasks = bob.list_tasks(&realm_id).unwrap();

        info!(
            alice_count = alice_tasks.len(),
            bob_count = bob_tasks.len(),
            "Final task counts"
        );

        // Log actual task titles for debugging
        for (i, task) in alice_tasks.iter().enumerate() {
            debug!(i, title = %task.title, "Alice task");
        }
        for (i, task) in bob_tasks.iter().enumerate() {
            debug!(i, title = %task.title, "Bob task");
        }

        assert!(
            synced,
            "Both should have all 5 tasks. Alice has {}, Bob has {}",
            alice_tasks.len(),
            bob_tasks.len()
        );

        // Verify specific tasks exist on both
        let alice_titles: std::collections::HashSet<_> =
            alice_tasks.iter().map(|t| t.title.as_str()).collect();
        let bob_titles: std::collections::HashSet<_> =
            bob_tasks.iter().map(|t| t.title.as_str()).collect();

        assert!(
            alice_titles.contains("Alice offline task 1"),
            "Alice should have her offline task 1"
        );
        assert!(
            alice_titles.contains("Bob offline task 1"),
            "Alice should have Bob's offline task 1"
        );
        assert!(
            bob_titles.contains("Alice offline task 1"),
            "Bob should have Alice's offline task 1"
        );
        assert!(
            bob_titles.contains("Bob offline task 1"),
            "Bob should have his offline task 1"
        );

        // Cleanup
        alice.shutdown().await.unwrap();
        bob.shutdown().await.unwrap();
    }
}
