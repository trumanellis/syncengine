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
use std::sync::Arc;

use iroh_gossip::proto::TopicId;
use rand::RngCore;
use tracing::{debug, info, warn};

use crate::error::SyncError;
use crate::identity::{Did, HybridKeypair, HybridPublicKey};
use crate::invite::InviteTicket;
use crate::realm::RealmDoc;
use crate::storage::Storage;
use crate::sync::{GossipSync, SyncEnvelope, SyncEvent, SyncMessage, SyncStatus, TopicHandle};
use crate::types::{RealmId, RealmInfo, Task, TaskId};

use tokio::sync::broadcast;

/// Internal state for an open realm
struct RealmState {
    /// The Automerge document containing tasks
    doc: RealmDoc,
    /// Handle to the gossip topic (if syncing)
    topic_handle: Option<TopicHandle>,
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

pub struct SyncEngine {
    /// Persistent storage for realms, documents, and keys
    storage: Storage,
    /// Gossip-based P2P networking (lazy-initialized)
    gossip: Option<Arc<GossipSync>>,
    /// Currently open realms with their in-memory state
    realms: HashMap<RealmId, RealmState>,
    /// Data directory path
    data_dir: PathBuf,
    /// Identity keypair (lazy-initialized)
    identity: Option<HybridKeypair>,
    /// Per-realm sync status tracking
    sync_status: HashMap<RealmId, SyncStatus>,
    /// Event broadcast channel for notifying listeners of realm changes
    event_tx: broadcast::Sender<SyncEvent>,
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

        let (event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);

        Ok(Self {
            storage,
            gossip: None,
            realms: HashMap::new(),
            data_dir,
            identity: None,
            sync_status: HashMap::new(),
            event_tx,
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
                topic_handle: None,
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

        // Verify realm exists
        let _info = self
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
                topic_handle: None,
                realm_key,
            },
        );

        debug!(%realm_id, "Realm opened");
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
            .map(|s| s.topic_handle.is_some())
            .unwrap_or(false)
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

        let task_id = {
            let state = self
                .realms
                .get_mut(realm_id)
                .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

            state.doc.add_task(title)?
        };

        // Auto-save
        self.save_realm(realm_id).await?;

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
    pub fn get_task(&self, realm_id: &RealmId, task_id: &TaskId) -> Result<Option<Task>, SyncError> {
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
    pub async fn toggle_task(&mut self, realm_id: &RealmId, task_id: &TaskId) -> Result<(), SyncError> {
        // Ensure realm is open
        if !self.realms.contains_key(realm_id) {
            self.open_realm(realm_id).await?;
        }

        {
            let state = self
                .realms
                .get_mut(realm_id)
                .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

            state.doc.toggle_task(task_id)?;
        }

        // Auto-save
        self.save_realm(realm_id).await?;

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
    pub async fn delete_task(&mut self, realm_id: &RealmId, task_id: &TaskId) -> Result<(), SyncError> {
        // Ensure realm is open
        if !self.realms.contains_key(realm_id) {
            self.open_realm(realm_id).await?;
        }

        {
            let state = self
                .realms
                .get_mut(realm_id)
                .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

            state.doc.delete_task(task_id)?;
        }

        // Auto-save
        self.save_realm(realm_id).await?;

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

    /// Ensure gossip networking is initialized
    async fn ensure_gossip(&mut self) -> Result<Arc<GossipSync>, SyncError> {
        if let Some(ref gossip) = self.gossip {
            return Ok(gossip.clone());
        }

        info!("Initializing gossip networking");
        let gossip = Arc::new(GossipSync::new().await?);
        self.gossip = Some(gossip.clone());
        Ok(gossip)
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

        // Check if already syncing
        if self.is_realm_syncing(realm_id) {
            debug!(%realm_id, "Already syncing");
            return Ok(());
        }

        info!(%realm_id, "Starting sync");

        // Update status to Connecting
        self.sync_status
            .insert(realm_id.clone(), SyncStatus::Connecting);
        let _ = self.event_tx.send(SyncEvent::StatusChanged {
            realm_id: realm_id.clone(),
            status: SyncStatus::Connecting,
        });

        // Initialize gossip
        let gossip = self.ensure_gossip().await?;

        // Subscribe to topic
        let topic_id = TopicId::from_bytes(*realm_id.as_bytes());
        let handle = gossip.subscribe(topic_id, vec![]).await?;

        // Store handle
        if let Some(state) = self.realms.get_mut(realm_id) {
            state.topic_handle = Some(handle);
        }

        // Update status to Syncing
        self.sync_status
            .insert(realm_id.clone(), SyncStatus::Syncing { peer_count: 0 });
        let _ = self.event_tx.send(SyncEvent::StatusChanged {
            realm_id: realm_id.clone(),
            status: SyncStatus::Syncing { peer_count: 0 },
        });

        debug!(%realm_id, "Sync started");
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

        if state.topic_handle.is_some() {
            state.topic_handle = None;

            // Update status to Idle
            self.sync_status.insert(realm_id.clone(), SyncStatus::Idle);
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
            .get(realm_id)
            .cloned()
            .unwrap_or(SyncStatus::Idle)
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

        let topic_handle = state
            .topic_handle
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

        // Broadcast via topic handle
        topic_handle.broadcast(envelope_bytes).await?;

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

        // Create announce message
        let message = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads,
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

        // Ensure gossip is initialized
        let gossip = self.ensure_gossip().await?;

        // Generate invite
        let invite = gossip.generate_invite(realm_id, realm_key)?;

        // Mark realm as shared
        if let Ok(Some(mut info)) = self.storage.load_realm(realm_id) {
            if !info.is_shared {
                info.is_shared = true;
                self.storage.save_realm(&info)?;
            }
        }

        info!(%realm_id, "Invite generated");
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

        // Join via invite (validates expiry)
        let handle = gossip.join_via_invite(invite).await?;

        // Create realm info
        let info = RealmInfo {
            id: realm_id.clone(),
            name: invite.realm_name.clone().unwrap_or_else(|| "Shared Realm".to_string()),
            is_shared: true,
            created_at: chrono::Utc::now().timestamp(),
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
                topic_handle: Some(handle),
                realm_key: invite.realm_key,
            },
        );

        debug!(%realm_id, "Joined realm");
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
        };

        let mut envelope =
            SyncEnvelope::seal(&message, "did:example:test", &realm_key, |_| vec![0x51, 0x9E, 1])
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
        assert!(matches!(engine.sync_status(&realm_id), SyncStatus::Syncing { .. }));

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
        assert!(matches!(engine.sync_status(&realm1), SyncStatus::Syncing { .. }));
        assert!(matches!(engine.sync_status(&realm2), SyncStatus::Syncing { .. }));
        assert!(matches!(engine.sync_status(&realm3), SyncStatus::Syncing { .. }));

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
        assert!(elapsed.as_millis() < 5000, "start_sync took too long: {:?}", elapsed);

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

        assert!(matches!(engine.sync_status(&realm1), SyncStatus::Syncing { .. }));
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
            match tokio::time::timeout(
                std::time::Duration::from_millis(100),
                events.recv()
            ).await {
                Ok(Ok(SyncEvent::StatusChanged { status: SyncStatus::Connecting, .. })) => {
                    found_connecting = true;
                }
                Ok(Ok(SyncEvent::StatusChanged { status: SyncStatus::Syncing { .. }, .. })) => {
                    found_syncing = true;
                }
                _ => break,
            }
        }

        // Should have received at least the connecting or syncing event
        assert!(found_connecting || found_syncing, "Should receive status change events");

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
}
