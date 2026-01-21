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
use rand::{Rng, RngCore};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::blobs::BlobManager;
use crate::error::SyncError;
use crate::identity::{Did, HybridKeypair, HybridPublicKey};
// Indra's Network: Profile packet layer
use crate::profile::{
    MirrorStore, PacketAddress, PacketEnvelope, PacketPayload, ProfileKeys, ProfileLog,
    ProfileTopicTracker,
};
use crate::invite::{InviteTicket, NodeAddrBytes};
use crate::peers::{PeerInfo, PeerRegistry, PeerSource, PeerStatus};
use crate::realm::RealmDoc;
use crate::storage::Storage;
use crate::sync::{
    ContactEvent, ContactManager, GossipSync, NetworkDebugInfo, SyncEnvelope, SyncEvent,
    SyncMessage, SyncStatus, TopicEvent, TopicReceiver, TopicSender,
};
use crate::types::contact::{ContactInfo, HybridContactInvite, PeerContactInvite, PendingContact, ProfileSnapshot};
use crate::types::{RealmId, RealmInfo, Task, TaskId};

/// Reserved name for the default Private realm
const PRIVATE_REALM_NAME: &str = "Private";

/// Sacred onboarding tasks for the Private realm
const ONBOARDING_TASKS: &[(&str, &str)] = &[
    (
        "🜃 Welcome to Synchronicity Engine",
        "You have entered a space where intentions become manifest. This Private realm is yours alone, a sanctuary for personal quests that need not be shared."
    ),
    (
        "⚛︎ Manifest Your First Intention",
        "Try creating a new intention (task) by using the 'add' command or UI. Watch as your thought crystallizes into form. This is the beginning of conscious co-creation with the field."
    ),
    (
        "◎ The Nature of Synchronicity",
        "Synchronicity is always listening. When you're ready to co-create with others, you can establish new realms and share them via invite links. Each realm is a quantum entanglement of intentions across peers."
    ),
    (
        "☍ Mark Synchronicities Complete",
        "As intentions manifest in reality, mark them complete. Notice how the act of acknowledgment closes one loop and opens space for new possibilities to emerge."
    ),
    (
        "∞ This Space Remains Private",
        "Unlike shared realms, this Private realm cannot be synchronized with others. It's your personal laboratory for experimenting with manifestation before sharing intentions with the collective."
    ),
];

/// Check if a realm name is the reserved "Private" name (case-insensitive)
fn is_private_realm_name(name: &str) -> bool {
    name.eq_ignore_ascii_case(PRIVATE_REALM_NAME)
}

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

/// Result of startup sync operation
///
/// Contains statistics about the startup sync attempt, including:
/// - How many peers were attempted
/// - How many succeeded
/// - How many were skipped due to backoff
/// - The jitter delay that was applied
#[derive(Debug, Clone, Default)]
pub struct StartupSyncResult {
    /// Number of peers we attempted to connect to
    pub peers_attempted: usize,
    /// Number of successful connections
    pub peers_succeeded: usize,
    /// Number of peers skipped due to backoff timer
    pub peers_skipped_backoff: usize,
    /// Number of profile updates received
    pub profiles_updated: usize,
    /// Jitter delay applied in milliseconds (0-30000)
    pub jitter_delay_ms: u64,
}

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
    /// Contact manager for P2P contact exchange (lazy-initialized)
    contact_manager: Option<Arc<ContactManager>>,
    /// Blob manager for content-addressed image storage (P2P capable)
    blob_manager: BlobManager,
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
    /// Contact event broadcast channel for contact exchange events
    contact_event_tx: broadcast::Sender<crate::sync::ContactEvent>,
    /// Receiver for sync messages from background listener tasks
    sync_rx: tokio::sync::mpsc::UnboundedReceiver<SyncChannelMessage>,
    /// Sender for sync messages (cloned to background tasks)
    sync_tx: tokio::sync::mpsc::UnboundedSender<SyncChannelMessage>,
    /// Persistent sender for our own per-peer profile topic.
    /// Used to broadcast profile announcements to contacts.
    /// Initialized by start_profile_sync(), used by announce_profile() and related methods.
    profile_gossip_sender: Option<TopicSender>,
    /// Persistent sender for the global profile topic.
    /// Used to broadcast packets (messages) to all peers on the global topic.
    global_profile_gossip_sender: Option<TopicSender>,
    /// Persistent receiver for our own profile topic.
    /// MUST be kept alive to maintain the gossip subscription - dropping it closes the topic.
    /// When contacts subscribe to our profile topic, they join as peers to this subscription.
    #[allow(dead_code)] // Held to keep subscription alive, not read
    profile_gossip_receiver: Option<TopicReceiver>,
    /// Shared active contact topics map.
    /// This map is shared between ContactProtocolHandler and ContactManager so both
    /// can add and access contact topic senders. Initialized when gossip is created.
    active_contact_topics: Option<crate::sync::ActiveContactTopics>,

    // ═══════════════════════════════════════════════════════════════════════
    // Indra's Network: Profile Packet Layer
    // ═══════════════════════════════════════════════════════════════════════

    /// Profile keys for packet signing and sealed box key exchange.
    /// Contains hybrid signing keys (ML-DSA-65 + Ed25519) and key exchange keys
    /// (X25519 + ML-KEM-768).
    profile_keys: Option<ProfileKeys>,

    /// Our own append-only log of packets we've created.
    /// Each packet is signed and hash-chained for integrity.
    /// Initialized when profile_keys are initialized.
    profile_log: Option<ProfileLog>,

    /// Mirror store for other profiles' packet logs.
    /// Stores packets from contacts for offline sync and relay.
    mirror_store: Option<MirrorStore>,

    /// Topic tracker for profile packet subscriptions.
    /// Manages subscriptions to profile and realm packet topics.
    #[allow(dead_code)] // Will be used when packet sync is implemented
    profile_topic_tracker: ProfileTopicTracker,

    /// Flag indicating whether networking was explicitly started via `start_networking()`.
    /// Used to prevent auto-sync in `open_realm` when the user intends to work offline.
    networking_requested: bool,

    // ═══════════════════════════════════════════════════════════════════════
    // Packet Event Logging (for Indra's Network visualization)
    // ═══════════════════════════════════════════════════════════════════════

    /// In-memory buffer for packet events (per-peer, for UI visualization).
    /// Used to display packet flow in the network page.
    packet_event_buffer: Arc<crate::sync::PacketEventBuffer>,
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

        // Initialize blob manager with persistent FsStore
        let blob_path = data_dir.join("blobs");
        let blob_manager = BlobManager::new_persistent(&blob_path).await?;
        info!(?blob_path, "Blob manager initialized with persistent storage");

        let (event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        let (contact_event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);

        let (sync_tx, sync_rx) = tokio::sync::mpsc::unbounded_channel();

        // Initialize mirror store for profile packet storage
        let mirror_store = MirrorStore::new(storage.db_handle())?;

        // Initialize packet event buffer for UI visualization
        let packet_event_buffer = crate::sync::PacketEventBuffer::with_defaults();

        let mut engine = Self {
            storage,
            peer_registry,
            gossip: None,
            contact_manager: None,
            blob_manager,
            realms: HashMap::new(),
            data_dir,
            identity: None,
            sync_status: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            contact_event_tx,
            sync_rx,
            sync_tx,
            profile_gossip_sender: None,
            global_profile_gossip_sender: None,
            profile_gossip_receiver: None,
            active_contact_topics: None,
            // Indra's Network packet layer
            profile_keys: None,
            profile_log: None, // Initialized when profile_keys are initialized
            mirror_store: Some(mirror_store),
            profile_topic_tracker: ProfileTopicTracker::new(),
            networking_requested: false,
            packet_event_buffer,
        };

        // Initialize the Private realm if it doesn't exist
        engine.ensure_private_realm().await?;

        Ok(engine)
    }

    /// Ensure the Private realm exists, creating it if necessary
    ///
    /// This is called automatically during engine initialization.
    /// The Private realm is a special, non-shareable realm that contains
    /// sacred onboarding tasks to guide new users.
    async fn ensure_private_realm(&mut self) -> Result<(), SyncError> {
        // Check if Private realm already exists
        let existing_realms = self.storage.list_realms()?;
        if existing_realms
            .iter()
            .any(|r| r.name.eq_ignore_ascii_case(PRIVATE_REALM_NAME))
        {
            debug!("Private realm already exists");
            return Ok(());
        }

        info!("Creating default Private realm with sacred onboarding");

        // Create realm info
        let realm_info = RealmInfo::new(PRIVATE_REALM_NAME);
        let realm_id = realm_info.id.clone();

        // Generate encryption key
        let mut realm_key = [0u8; 32];
        rand::rng().fill_bytes(&mut realm_key);

        // Create document with onboarding tasks
        let mut doc = RealmDoc::new();
        for (title, description) in ONBOARDING_TASKS {
            doc.add_quest(title, None, description)?;
        }

        // Save to storage
        self.storage.save_realm(&realm_info)?;
        self.storage.save_realm_key(&realm_id, &realm_key)?;
        self.storage.save_document(&realm_id, &doc.save())?;

        debug!(%realm_id, "Private realm created with {} onboarding tasks", ONBOARDING_TASKS.len());
        Ok(())
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

    /// Get this node's endpoint ID.
    ///
    /// Returns the endpoint ID used for P2P networking.
    /// Returns `None` if networking has not been started.
    pub fn endpoint_id(&self) -> Option<iroh::PublicKey> {
        self.gossip.as_ref().map(|g| g.endpoint_id())
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
    // Profile Packet Operations (Indra's Network)
    // ═══════════════════════════════════════════════════════════════════════

    /// Initialize profile keys, loading from storage or generating new ones.
    ///
    /// Profile keys are used for:
    /// - Signing packets in the append-only log
    /// - Hybrid key exchange for sealed boxes (X25519 + ML-KEM-768)
    ///
    /// If profile keys already exist in storage, they are loaded.
    /// Otherwise, new keys are generated and persisted.
    ///
    /// # Note
    ///
    /// Profile keys are derived from the identity keypair to ensure DID consistency.
    /// This means `profile_did() == did()` - both return the same DID.
    /// The identity keypair provides signing, while profile keys add key exchange
    /// capabilities (X25519 + ML-KEM) for sealed box encryption.
    ///
    /// **Important**: `init_identity()` must be called before this method.
    pub fn init_profile_keys(&mut self) -> Result<(), SyncError> {
        if self.profile_keys.is_some() {
            return Ok(());
        }

        // Profile keys MUST be derived from identity to ensure DID consistency.
        // This fixes the DID mismatch bug where contacts were stored with identity DID
        // but packets were signed with a different profile DID.
        let identity = self.identity.as_ref().ok_or_else(|| {
            SyncError::Identity(
                "Identity must be initialized before profile keys. Call init_identity() first.".to_string()
            )
        })?;

        let keys = if let Some(keys) = self.storage.load_profile_keys()? {
            // Verify loaded keys match identity DID (migration safety check)
            let identity_did = Did::from_public_key(&identity.public_key());
            if keys.did() != identity_did {
                warn!(
                    stored_profile_did = %keys.did(),
                    identity_did = %identity_did,
                    "Stored profile keys have different DID than identity - regenerating from identity"
                );
                // Regenerate from identity to fix the mismatch
                let new_keys = ProfileKeys::from_signing_keypair(identity.clone());
                self.storage.save_profile_keys(&new_keys)?;
                info!("Regenerated profile keys from identity keypair");
                new_keys
            } else {
                info!("Loaded existing profile keys (DID matches identity)");
                keys
            }
        } else {
            info!("Deriving profile keys from identity keypair");
            let keys = ProfileKeys::from_signing_keypair(identity.clone());
            self.storage.save_profile_keys(&keys)?;
            keys
        };

        // Initialize the profile log with our DID
        // Try to load existing packets from MirrorStore for persistence across restarts
        let did = keys.did();
        let log = if let Some(ref mirror) = self.mirror_store {
            match mirror.load_log(&did) {
                Ok(loaded_log) => {
                    let packet_count = loaded_log.len();
                    if packet_count > 0 {
                        info!(
                            packet_count,
                            head_seq = ?loaded_log.head_sequence(),
                            "Loaded existing profile log from MirrorStore"
                        );
                    } else {
                        debug!("MirrorStore has no existing packets for our profile");
                    }
                    loaded_log
                }
                Err(e) => {
                    warn!(error = %e, "Failed to load profile log from MirrorStore, starting fresh");
                    ProfileLog::new(did.clone())
                }
            }
        } else {
            debug!("MirrorStore not available, starting with empty profile log");
            ProfileLog::new(did.clone())
        };
        self.profile_log = Some(log);

        self.profile_keys = Some(keys);
        Ok(())
    }

    /// Get the profile DID (decentralized identifier) for packets.
    ///
    /// Returns `None` if profile keys have not been initialized.
    /// Call `init_profile_keys()` first to ensure keys are available.
    pub fn profile_did(&self) -> Option<Did> {
        self.profile_keys.as_ref().map(|k| k.did())
    }

    /// Check if profile keys have been initialized.
    pub fn has_profile_keys(&self) -> bool {
        self.profile_keys.is_some()
    }

    /// Get a reference to our own profile log.
    ///
    /// The profile log contains our signed, hash-chained packets.
    /// Returns `None` if profile keys have not been initialized.
    pub fn my_log(&self) -> Option<&ProfileLog> {
        self.profile_log.as_ref()
    }

    /// Get the current head sequence number of our profile log.
    pub fn log_head_sequence(&self) -> u64 {
        self.profile_log
            .as_ref()
            .and_then(|log| log.head_sequence())
            .unwrap_or(0)
    }

    /// Get a mirror of another profile's packet log.
    ///
    /// Returns `None` if we don't have any packets from this profile.
    pub fn mirror_head(&self, did: &Did) -> Option<u64> {
        self.mirror_store.as_ref()?.get_head(did).ok()?
    }

    /// Get packets from a profile's mirror.
    ///
    /// Returns packets from the given profile, starting after `from_sequence`.
    pub fn mirror_packets_since(
        &self,
        did: &Did,
        from_sequence: u64,
    ) -> Result<Vec<PacketEnvelope>, SyncError> {
        let mirror = self.mirror_store.as_ref().ok_or_else(|| {
            SyncError::Storage("Mirror store not initialized".to_string())
        })?;
        mirror.get_since(did, from_sequence)
    }

    /// Get packets from a mirror for a specific sequence range (inclusive).
    ///
    /// Returns packets from `from_sequence` to `to_sequence` (both inclusive).
    pub fn mirror_packets_range(
        &self,
        did: &Did,
        from_sequence: u64,
        to_sequence: u64,
    ) -> Result<Vec<PacketEnvelope>, SyncError> {
        let mirror = self.mirror_store.as_ref().ok_or_else(|| {
            SyncError::Storage("Mirror store not initialized".to_string())
        })?;
        mirror.get_range(did, from_sequence, to_sequence)
    }

    /// Get ALL packets from a mirror (inclusive of sequence 0).
    ///
    /// Unlike `mirror_packets_since(did, 0)` which excludes sequence 0,
    /// this method returns all packets including the very first one.
    pub fn mirror_packets_all(
        &self,
        did: &Did,
    ) -> Result<Vec<PacketEnvelope>, SyncError> {
        let mirror = self.mirror_store.as_ref().ok_or_else(|| {
            SyncError::Storage("Mirror store not initialized".to_string())
        })?;
        mirror.get_all(did)
    }

    /// List all DIDs we have mirrors for.
    pub fn list_mirrored_dids(&self) -> Result<Vec<Did>, SyncError> {
        let mirror = self.mirror_store.as_ref().ok_or_else(|| {
            SyncError::Storage("Mirror store not initialized".to_string())
        })?;
        mirror.list_mirrored_dids()
    }

    /// Create and sign a new packet.
    ///
    /// Creates a packet with the given payload, signs it with our profile keys,
    /// and appends it to our log.
    ///
    /// # Arguments
    ///
    /// * `payload` - The packet content (message, profile update, etc.)
    /// * `address` - Who can decrypt this packet
    ///
    /// # Returns
    ///
    /// The sequence number of the newly created packet.
    ///
    /// # Errors
    ///
    /// Returns an error if profile keys have not been initialized.
    pub fn create_packet(
        &mut self,
        payload: PacketPayload,
        address: PacketAddress,
    ) -> Result<u64, SyncError> {
        let keys = self.profile_keys.as_ref().ok_or_else(|| {
            SyncError::Identity("Profile keys not initialized. Call init_profile_keys() first.".to_string())
        })?;

        let log = self.profile_log.as_ref().ok_or_else(|| {
            SyncError::Identity("Profile log not initialized. Call init_profile_keys() first.".to_string())
        })?;

        // Get the previous hash for chaining
        let prev_hash = log.head_hash();

        // Calculate next sequence
        let sequence = log.head_sequence().map(|s| s + 1).unwrap_or(0);

        // Create the envelope based on addressing
        let envelope = match &address {
            PacketAddress::Global => {
                // Global packets are signed but not encrypted
                PacketEnvelope::create_global(keys, &payload, sequence, prev_hash)?
            }
            PacketAddress::Individual(recipient) => {
                // Individual packet for direct messaging - E2E encrypted
                // Uses sealed box encryption with hybrid (X25519 + ML-KEM) keys
                // If recipient has no encryption keys, sending fails (no cleartext fallback)
                // Include ourselves as a recipient so we can decrypt our own sent messages
                let recipient_keys = self.get_recipient_public_keys(recipient)?;
                let sender_keys = keys.public_bundle();
                let recipients = vec![recipient_keys, sender_keys];
                PacketEnvelope::create(keys, &payload, &recipients, sequence, prev_hash)?
            }
            PacketAddress::List(recipients_dids) => {
                // Multi-recipient packet - E2E encrypted for all recipients
                // Each recipient can decrypt with their own keys
                // Include ourselves as a recipient so we can decrypt our own sent messages
                let mut recipient_keys: Vec<_> = recipients_dids
                    .iter()
                    .map(|r| self.get_recipient_public_keys(r))
                    .collect::<Result<Vec<_>, _>>()?;
                let sender_keys = keys.public_bundle();
                recipient_keys.push(sender_keys);
                PacketEnvelope::create(keys, &payload, &recipient_keys, sequence, prev_hash)?
            }
            PacketAddress::Group(_realm_id) => {
                // Group packet for realm members (encrypted with realm key)
                // For now, treat as global within the realm
                // TODO: Implement realm-key encryption for group messages
                PacketEnvelope::create_global(keys, &payload, sequence, prev_hash)?
            }
        };

        // Append to our log (in-memory)
        let seq = envelope.sequence;
        // Need mutable reference now
        self.profile_log.as_mut().unwrap().append(envelope.clone())?;

        // Persist to MirrorStore for durability across restarts
        // Store under our own DID so we can retrieve our sent messages on startup
        if let Some(ref mirror) = self.mirror_store {
            if let Err(e) = mirror.store_packet(&envelope) {
                warn!(error = %e, seq, "Failed to persist sent packet to MirrorStore");
            } else {
                debug!(seq, "Persisted sent packet to MirrorStore");
            }
        }

        debug!(seq, "Created packet");
        Ok(seq)
    }

    /// Broadcast a packet from our log to the gossip network.
    ///
    /// Routes packets based on the address:
    /// - `Individual(did)` → 1:1 contact topic (direct messaging)
    /// - `List(dids)` → each recipient's 1:1 contact topic
    /// - `Global` → global profile gossip topic
    /// - `Group(realm_id)` → realm topic (not yet implemented)
    ///
    /// # Arguments
    ///
    /// * `sequence` - The sequence number of the packet to broadcast
    /// * `address` - The addressing mode for routing
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Profile log is not initialized
    /// - The packet with the given sequence doesn't exist
    /// - Gossip network is not started
    /// - Contact manager is not initialized (for Individual/List addresses)
    pub async fn broadcast_packet(
        &self,
        sequence: u64,
        address: &PacketAddress,
    ) -> Result<(), SyncError> {
        // Get the packet from our log
        let log = self.profile_log.as_ref().ok_or_else(|| {
            SyncError::Identity("Profile log not initialized".to_string())
        })?;

        let entry = log.get(sequence).ok_or_else(|| {
            SyncError::Identity(format!("Packet {} not found in log", sequence))
        })?;
        let envelope = entry.envelope.clone();

        // Create the gossip message
        let msg = crate::sync::ProfileGossipMessage::packet(envelope);
        let bytes = msg.to_bytes()?;

        // Route based on address type
        match address {
            PacketAddress::Individual(did) => {
                // Send via 1:1 contact topic
                if let Some(ref contact_mgr) = self.contact_manager {
                    contact_mgr
                        .send_packet_to_contact(&did.to_string(), &bytes)
                        .await?;
                    debug!(sequence, to = %did, "Sent packet to contact via 1:1 topic");
                } else {
                    return Err(SyncError::Network(
                        "Contact manager not initialized. Cannot send to individual contact."
                            .to_string(),
                    ));
                }
            }
            PacketAddress::List(dids) => {
                // Send to each contact in the list
                if let Some(ref contact_mgr) = self.contact_manager {
                    let mut errors = Vec::new();
                    for did in dids {
                        if let Err(e) = contact_mgr
                            .send_packet_to_contact(&did.to_string(), &bytes)
                            .await
                        {
                            warn!(to = %did, error = %e, "Failed to send packet to contact");
                            errors.push(format!("{}: {}", did, e));
                        } else {
                            debug!(sequence, to = %did, "Sent packet to contact via 1:1 topic");
                        }
                    }
                    if errors.len() == dids.len() {
                        return Err(SyncError::Network(format!(
                            "Failed to send to all recipients: {:?}",
                            errors
                        )));
                    }
                    // Partial success is acceptable
                } else {
                    return Err(SyncError::Network(
                        "Contact manager not initialized. Cannot send to contacts.".to_string(),
                    ));
                }
            }
            PacketAddress::Global => {
                // Broadcast on the GLOBAL profile gossip topic
                if let Some(sender) = self.global_profile_gossip_sender.as_ref() {
                    sender.broadcast(bytes).await.map_err(|e| {
                        SyncError::Gossip(format!("Failed to broadcast packet: {}", e))
                    })?;
                    debug!(sequence, "Broadcast packet to global gossip topic");
                } else {
                    return Err(SyncError::Gossip(
                        "Global profile gossip not started. Call start_profile_sync() first."
                            .to_string(),
                    ));
                }
            }
            PacketAddress::Group(_realm_id) => {
                // TODO: Broadcast to realm topic
                // For now, fallback to global topic
                warn!(sequence, "Group addressing not yet implemented, using global topic");
                if let Some(sender) = self.global_profile_gossip_sender.as_ref() {
                    sender.broadcast(bytes).await.map_err(|e| {
                        SyncError::Gossip(format!("Failed to broadcast packet: {}", e))
                    })?;
                    debug!(sequence, "Broadcast packet to global gossip topic (group fallback)");
                } else {
                    return Err(SyncError::Gossip(
                        "Global profile gossip not started. Call start_profile_sync() first."
                            .to_string(),
                    ));
                }
            }
        }

        // Record outgoing packet event for UI visualization (Indra's Network)
        let my_did = self.profile_did()
            .map(|d| d.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Get destination info from address
        let (destination_did, destination_name) = match address {
            PacketAddress::Individual(did) => {
                let name = self.storage.load_peer_by_did(did.as_str())
                    .ok()
                    .flatten()
                    .map(|p| p.display_name())
                    .unwrap_or_else(|| format!("{}...", &did.as_str()[..16.min(did.as_str().len())]));
                (did.as_str().to_string(), name)
            }
            PacketAddress::List(dids) => {
                if let Some(first) = dids.first() {
                    let name = self.storage.load_peer_by_did(first.as_str())
                        .ok()
                        .flatten()
                        .map(|p| p.display_name())
                        .unwrap_or_else(|| format!("{}...", &first.as_str()[..16.min(first.as_str().len())]));
                    (first.as_str().to_string(), format!("{} +{}", name, dids.len().saturating_sub(1)))
                } else {
                    ("unknown".to_string(), "Unknown".to_string())
                }
            }
            PacketAddress::Global => ("global".to_string(), "Everyone".to_string()),
            PacketAddress::Group(realm_id) => (realm_id.to_string(), "Realm".to_string()),
        };

        // Content preview for outgoing packets
        // Note: The envelope is encrypted, so we'd need to cache plaintext at creation time.
        // For now, just show a sent indicator. The UI can show the actual content
        // from the conversation view if needed.
        let content_preview = "[sent]".to_string();

        let event = crate::sync::PacketEvent {
            id: crate::sync::PacketEvent::make_id(&my_did, sequence),
            timestamp: chrono::Utc::now().timestamp_millis(),
            direction: crate::sync::PacketDirection::Outgoing,
            sequence,
            author_did: my_did.clone(),
            author_name: "Me".to_string(),
            relay_did: None,
            relay_name: None,
            destination_did: destination_did.clone(),
            destination_name,
            decryption_status: crate::sync::DecryptionStatus::Decrypted,
            content_preview,
            is_delivered: false,
            peer_did: destination_did.clone(),
        };
        info!(
            peer_did = %destination_did,
            sequence,
            "Recording OUTGOING packet event"
        );
        self.packet_event_buffer.record(event);

        Ok(())
    }

    /// Create a packet and broadcast it to the network.
    ///
    /// This is the recommended way to send packets as it ensures they
    /// are both stored locally and broadcast to peers in one step.
    ///
    /// # Arguments
    ///
    /// * `payload` - The packet content
    /// * `address` - Who can decrypt this packet and where to route the packet
    ///
    /// # Returns
    ///
    /// The sequence number of the newly created packet.
    pub async fn create_and_broadcast_packet(
        &mut self,
        payload: PacketPayload,
        address: PacketAddress,
    ) -> Result<u64, SyncError> {
        // Create the packet (stores it in our log)
        let seq = self.create_packet(payload, address.clone())?;

        // Broadcast it to the network via the appropriate topic based on address
        self.broadcast_packet(seq, &address).await?;

        Ok(seq)
    }

    /// Helper to get recipient's public keys for sealed boxes.
    ///
    /// This looks up the recipient's ProfilePublicKeys from stored contacts.
    /// Returns an error if:
    /// - No contact found for the recipient DID
    /// - Contact exists but has no encryption keys (legacy contact)
    /// - Encryption keys are malformed
    fn get_recipient_public_keys(
        &self,
        recipient: &Did,
    ) -> Result<crate::profile::ProfilePublicKeys, SyncError> {
        // Look up contact by DID
        let contact = self.storage.load_contact(recipient.as_ref())?
            .ok_or_else(|| SyncError::Identity(format!(
                "No contact found for recipient: {}. \
                 Contact exchange must complete before sending E2E encrypted messages.",
                recipient
            )))?;

        // Extract encryption keys from contact
        let enc_keys_bytes = contact.encryption_keys
            .ok_or_else(|| SyncError::Identity(format!(
                "Contact {} does not have encryption keys. \
                 This is a legacy contact that predates E2E encryption support. \
                 Re-exchanging contact will enable encrypted messaging.",
                recipient
            )))?;

        // Deserialize ProfilePublicKeys
        crate::profile::ProfilePublicKeys::from_bytes(&enc_keys_bytes)
            .map_err(|e| SyncError::Identity(format!(
                "Failed to parse encryption keys for {}: {}",
                recipient, e
            )))
    }

    /// Handle an incoming packet from a peer.
    ///
    /// This validates the packet signature, checks the hash chain,
    /// and stores it in the appropriate mirror.
    ///
    /// # Arguments
    ///
    /// * `envelope` - The received packet envelope
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the packet was new and stored successfully.
    /// `Ok(false)` if we already had this packet.
    /// `Err` if the packet is invalid.
    pub fn handle_incoming_packet(&mut self, envelope: PacketEnvelope) -> Result<bool, SyncError> {
        // TODO: Verify signature using sender's stored public keys
        // For now, we store without full verification since contact key lookup isn't implemented.
        // In production, this should call: envelope.verify(&sender_public_keys)
        // The MirrorStore validates hash chain integrity internally.

        // Store in mirror
        let mirror = self.mirror_store.as_ref().ok_or_else(|| {
            SyncError::Storage("Mirror store not initialized".to_string())
        })?;

        // Check if we already have this packet
        if let Some(existing_seq) = mirror.get_head(&envelope.sender).ok().flatten() {
            if envelope.sequence <= existing_seq {
                // We already have this or newer
                return Ok(false);
            }
        }

        // Store the packet (validates hash chain)
        mirror.store_packet(&envelope)?;
        debug!(sender = %envelope.sender, sequence = envelope.sequence, "Stored incoming packet");

        Ok(true)
    }

    /// Try to decrypt a packet addressed to us.
    ///
    /// For global (public) packets, returns the plaintext payload directly.
    /// For sealed packets, attempts to unseal using our profile keys.
    ///
    /// # Arguments
    ///
    /// * `envelope` - The packet to decrypt
    ///
    /// # Returns
    ///
    /// The decrypted payload if this packet was addressed to us and decryption succeeded.
    /// Returns `None` if we're not a recipient or can't decrypt.
    pub fn decrypt_packet(&self, envelope: &PacketEnvelope) -> Option<PacketPayload> {
        // Global packets are not encrypted
        if envelope.is_global() {
            return envelope.decode_global_payload().ok();
        }

        // Sealed packets require our keys to decrypt
        let keys = self.profile_keys.as_ref()?;
        envelope.decrypt_for_recipient(keys).ok()
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
        // Prevent creating realms with reserved "Private" name
        if is_private_realm_name(name) {
            return Err(SyncError::PrivateRealmOperation(
                "Cannot create realm with reserved name 'Private'".to_string(),
            ));
        }

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

        // AUTO-START SYNC: If this is a shared realm AND networking was explicitly requested,
        // start P2P sync automatically. This ensures sync resumes after app restart for shared
        // realms, but only when the user intends to be online.
        // Note: We call start_sync_internal to avoid circular recursion with start_sync
        if info.is_shared && self.networking_requested {
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
        } else if info.is_shared && !self.networking_requested {
            debug!(
                %realm_id,
                "Shared realm opened in offline mode - sync will start when networking is enabled"
            );
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
        // Check if this is the Private realm
        if let Ok(Some(info)) = self.storage.load_realm(realm_id) {
            if is_private_realm_name(&info.name) {
                return Err(SyncError::PrivateRealmOperation(
                    "Cannot delete Private realm".to_string(),
                ));
            }
        }

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

    /// Add a rich "quest" (task with metadata) to a realm
    ///
    /// Auto-saves the realm after adding the quest.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::RealmNotFound` if the realm is not open.
    pub async fn add_quest(
        &mut self,
        realm_id: &RealmId,
        title: &str,
        subtitle: Option<String>,
        description: &str,
        category: Option<String>,
        image_blob_id: Option<String>,
    ) -> Result<TaskId, SyncError> {
        // First, ensure realm is open (load from storage if needed)
        if !self.realms.contains_key(realm_id) {
            self.open_realm(realm_id).await?;
        }

        let (task_id, sync_data) = {
            let state = self
                .realms
                .get_mut(realm_id)
                .ok_or_else(|| SyncError::RealmNotFound(realm_id.to_string()))?;

            let task_id =
                state
                    .doc
                    .add_quest_full(title, subtitle, description, category, image_blob_id)?;

            // Capture incremental changes BEFORE save (save resets the checkpoint)
            let sync_data = state.doc.generate_sync_message();

            (task_id, sync_data)
        };

        // Auto-save
        self.save_realm(realm_id).await?;

        // Broadcast changes to peers if syncing
        if !sync_data.is_empty() {
            if let Err(e) = self.broadcast_changes_with_data(realm_id, sync_data).await {
                debug!(%realm_id, error = %e, "Failed to broadcast quest addition (may not be syncing)");
            }
        }

        debug!(%realm_id, %task_id, title, "Quest added");
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
        self.networking_requested = true;
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
    fn get_effective_bootstrap_peers(
        &self,
        realm_id: &RealmId,
    ) -> Result<Vec<iroh::PublicKey>, SyncError> {
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

        // Pass storage, event_tx, and local_did so Router can register contact protocol handler
        // The local_did is required for the simplified protocol's local key derivation
        let contact_deps = self.identity.as_ref().map(|keypair| {
            let did = Did::from_public_key(&keypair.public_key());
            (Arc::new(self.storage.clone()), self.contact_event_tx.clone(), did.to_string())
        });

        // Pass profile handler deps if identity is available
        let profile_deps = self.identity.as_ref().map(|keypair| {
            let did = Did::from_public_key(&keypair.public_key());
            (Arc::new(self.storage.clone()), Arc::new(keypair.clone()), did)
        });

        // Pass blob manager for P2P image transfer capability
        // GossipSync::with_secret_key returns (GossipSync, Option<ActiveContactTopics>)
        // We store the active_topics for later use by ContactManager
        let (gossip_sync, active_topics) = GossipSync::with_secret_key(
            Some(secret_key),
            contact_deps,
            profile_deps,
            Some(&self.blob_manager),
        ).await?;
        let gossip = Arc::new(gossip_sync);
        self.gossip = Some(gossip.clone());
        self.active_contact_topics = active_topics;
        Ok(gossip)
    }

    /// Get a reference to the gossip instance if initialized
    ///
    /// Returns an error if gossip has not been initialized yet.
    /// Use `ensure_gossip()` if you want to initialize it automatically.
    fn ensure_gossip_ref(&self) -> Result<&GossipSync, SyncError> {
        self.gossip
            .as_ref()
            .map(|g| g.as_ref())
            .ok_or_else(|| SyncError::NotReady("Gossip networking not initialized".to_string()))
    }

    /// Ensure contact manager is initialized
    ///
    /// Initializes the contact manager if not already initialized.
    /// Requires both gossip and identity to be initialized first.
    async fn ensure_contact_manager(&mut self) -> Result<Arc<ContactManager>, SyncError> {
        if let Some(ref manager) = self.contact_manager {
            return Ok(manager.clone());
        }

        info!("Initializing contact manager");

        // Ensure gossip is initialized
        let gossip = self.ensure_gossip().await?;

        // Ensure identity is initialized
        self.init_identity()?;

        // Ensure our own profile is signed and pinned (required for announcements to work)
        // This must be done BEFORE we borrow keypair to avoid borrow checker issues
        // This is idempotent - if already signed, it just updates the pin
        if let Err(e) = self.sign_and_pin_own_profile() {
            warn!("Failed to sign and pin own profile: {}", e);
        } else {
            debug!("Own profile signed and pinned for contact exchange announcements");
        }

        let keypair = self
            .identity
            .as_ref()
            .ok_or_else(|| SyncError::Identity("Identity not initialized".to_string()))?;
        let did = Did::from_public_key(&keypair.public_key());

        // Clone gossip for the profile announcer before moving into ContactManager
        let gossip_for_announcer = gossip.clone();

        // Get the shared active_topics - this was created when gossip was initialized
        // and is shared with ContactProtocolHandler for coordinating contact topic senders
        let active_topics = self.active_contact_topics.clone().ok_or_else(|| {
            SyncError::NotReady(
                "active_contact_topics not initialized - gossip may have been created without contact deps".to_string()
            )
        })?;

        // Create contact manager (shares event_tx and active_topics with ContactProtocolHandler)
        let manager = Arc::new(ContactManager::new(
            gossip,
            Arc::new(keypair.clone()),
            did,
            Arc::new(self.storage.clone()),
            self.contact_event_tx.clone(),
            active_topics,
            Some(self.packet_event_buffer.clone()),
        ));

        // Start the auto-accept task for our own invites
        manager.clone().start_auto_accept_task();

        // Start the contact subscription task (ensures we subscribe to contact topics for RECEIVING)
        // The handler adds sender to active_topics for sending, but this task handles receiving.
        // This fixes the bug where handler's listener consumed Packet messages before we could process them.
        manager.clone().start_contact_subscription_task();

        // Start the contact accepted profile announcer
        // This broadcasts our profile when a contact is accepted, triggering auto-pinning
        Self::start_contact_accepted_profile_announcer(
            self.contact_event_tx.subscribe(),
            gossip_for_announcer,
            self.storage.clone(),
        );

        // Start profile sync listener to process incoming announcements (enables auto-pinning)
        // This uses empty bootstrap peers since we discover peers through contact exchange
        if let Err(e) = self.start_profile_sync(vec![]).await {
            warn!("Failed to start profile sync listener: {}", e);
        }

        self.contact_manager = Some(manager.clone());
        Ok(manager)
    }

    /// Start a background task that announces our profile when a contact is accepted.
    ///
    /// When the ContactAccepted event fires (either direction - we accept or they accept us),
    /// this broadcasts our signed profile to the global profile topic. The peer's profile
    /// sync listener will receive it and auto-pin because we're now contacts.
    ///
    /// This enables mutual profile pinning after contact exchange completes.
    fn start_contact_accepted_profile_announcer(
        mut event_rx: broadcast::Receiver<ContactEvent>,
        gossip: Arc<GossipSync>,
        storage: Storage,
    ) {
        tokio::spawn(async move {
            loop {
                match event_rx.recv().await {
                    Ok(ContactEvent::ContactAccepted { contact }) => {
                        info!(peer_did = %contact.peer_did, "Contact accepted, announcing profile for auto-pinning");

                        // Load our signed profile from storage
                        match storage.get_own_pinned_profile() {
                            Ok(Some(pin)) => {
                                let signed_profile = pin.signed_profile;

                                // Subscribe to the global profile topic and broadcast
                                let topic_id = crate::sync::global_profile_topic();
                                match gossip.subscribe_split(topic_id, vec![]).await {
                                    Ok((sender, _receiver)) => {
                                        // Create announcement (no avatar ticket needed for basic pinning)
                                        let announcement = crate::sync::ProfileGossipMessage::announce(
                                            signed_profile,
                                            None,
                                        );
                                        match announcement.to_bytes() {
                                            Ok(bytes) => {
                                                if let Err(e) = sender.broadcast(bytes).await {
                                                    warn!(
                                                        peer_did = %contact.peer_did,
                                                        error = %e,
                                                        "Failed to broadcast profile after contact accepted"
                                                    );
                                                } else {
                                                    info!(
                                                        peer_did = %contact.peer_did,
                                                        "Profile announced after contact accepted - peer should auto-pin"
                                                    );
                                                }
                                            }
                                            Err(e) => {
                                                warn!(
                                                    error = %e,
                                                    "Failed to serialize profile announcement"
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!(
                                            error = %e,
                                            "Failed to subscribe to profile topic for announcement"
                                        );
                                    }
                                }
                            }
                            Ok(None) => {
                                warn!("No own profile found, cannot announce after contact accepted");
                            }
                            Err(e) => {
                                warn!(error = %e, "Failed to load own profile for announcement");
                            }
                        }
                    }
                    Ok(_) => {} // Ignore other contact events
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        debug!("Contact event receiver lagged by {} messages", n);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        debug!("Contact event channel closed, stopping profile announcer");
                        break;
                    }
                }
            }
        });

        info!("Contact accepted profile announcer started");
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
            debug!(
                ?peer_id,
                attempt_number = peer_info.connection_attempts,
                "Attempting to reconnect"
            );

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

    /// Perform immediate startup sync with all known peers
    ///
    /// This should be called after engine initialization to establish connections
    /// with known peers as quickly as possible while avoiding the "simultaneous
    /// wake-up problem" where two peers starting at the same time both fail to
    /// connect.
    ///
    /// ## Behavior
    ///
    /// 1. **Initialize gossip** - Ensures we're listening for incoming connections
    /// 2. **Apply jitter** - Random delay (0-30 seconds) to avoid thundering herd
    /// 3. **Announce presence** - Broadcasts our profile to the global topic
    /// 4. **Connect to peers** - Attempts to connect to all known peers, respecting
    ///    Fibonacci backoff for peers that have failed recently
    ///
    /// ## The Simultaneous Wake-up Problem
    ///
    /// When peers A and B both start at the same time:
    /// - A tries to connect to B → B not ready → fails → backs off
    /// - B tries to connect to A → A not ready → fails → backs off
    ///
    /// The jitter + listen-first strategy solves this:
    /// - Both start listening immediately
    /// - Random jitter means one will attempt outbound first
    /// - The other is already listening and accepts the connection
    ///
    /// ## Prioritization
    ///
    /// Peers are sorted by priority:
    /// 1. Contacts (mutually accepted) first
    /// 2. Then by last_seen (most recently active first)
    ///
    /// # Returns
    ///
    /// A `StartupSyncResult` containing statistics about the sync attempt.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::NotReady` if gossip cannot be initialized.
    pub async fn startup_sync(&mut self) -> Result<StartupSyncResult, SyncError> {
        info!("Starting startup sync...");

        // 1. Ensure gossip is initialized (starts listening for incoming connections)
        self.ensure_gossip().await?;

        // 2. Initialize profile sync (sets profile_gossip_sender so announce_profile works)
        // This must be called before announce_profile() or profile broadcasts will fail
        if self.identity.is_some() {
            if let Err(e) = self.start_profile_sync(vec![]).await {
                warn!(error = %e, "Failed to start profile sync (non-fatal)");
            }
        }

        // 3. Generate random jitter (0-2 seconds) to avoid thundering herd
        // Kept short to not delay app responsiveness noticeably
        let jitter_ms: u64 = rand::rng().random_range(0..2_000);
        debug!(jitter_ms, "Applying startup jitter");
        tokio::time::sleep(std::time::Duration::from_millis(jitter_ms)).await;

        // 4. Initialize contact manager and reconnect to contact profile topics FIRST
        // This must happen BEFORE announce_profile() so that:
        // a) We're subscribed to receive profile updates from contacts
        // b) contact_manager exists so announce_profile() can broadcast to contact topics
        if self.identity.is_some() {
            match self.ensure_contact_manager().await {
                Ok(manager) => {
                    if let Err(e) = manager.reconnect_contacts().await {
                        warn!(error = %e, "Failed to reconnect contacts (non-fatal)");
                    }
                }
                Err(e) => {
                    warn!(error = %e, "Failed to initialize contact manager at startup (non-fatal)");
                }
            }
        }

        // 5. Announce presence on global profile topic AND contact topics (if we have a profile)
        // This now works correctly because contact_manager was initialized above
        if self.identity.is_some() {
            match self.announce_profile(None).await {
                Ok(()) => info!("Presence announced on profile and contact topics"),
                Err(e) => {
                    // Non-fatal - we can still connect to peers without announcing
                    warn!(error = %e, "Failed to announce presence (non-fatal)");
                }
            }
        } else {
            debug!("No identity initialized, skipping presence announcement");
        }

        // 6. Get all peers and sort by priority (contacts first, then by last_seen)
        let mut all_peers = self.storage.list_peers()?;
        all_peers.sort_by(|a, b| {
            // Contacts first
            match (a.is_contact(), b.is_contact()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    // Then by last_seen (most recent first)
                    b.last_seen.cmp(&a.last_seen)
                }
            }
        });

        // Skip if no gossip or no peers
        let Some(ref gossip) = self.gossip else {
            warn!("Gossip not initialized after ensure_gossip (unexpected)");
            return Ok(StartupSyncResult {
                jitter_delay_ms: jitter_ms,
                ..Default::default()
            });
        };

        if all_peers.is_empty() {
            debug!("No known peers for startup sync");
            return Ok(StartupSyncResult {
                jitter_delay_ms: jitter_ms,
                ..Default::default()
            });
        }

        info!(
            peer_count = all_peers.len(),
            "Attempting startup sync with known peers"
        );

        // 5. Attempt connections with Fibonacci backoff
        let mut result = StartupSyncResult {
            jitter_delay_ms: jitter_ms,
            ..Default::default()
        };

        for mut peer in all_peers {
            let peer_id = peer.public_key();

            // Skip contacts - they're already connected via gossip topic subscription
            // from reconnect_contacts() above. Creating a second direct connection here
            // can interfere with the gossip mesh and cause messages to not be delivered.
            if peer.is_contact() {
                // Still record as success since gossip connection is established
                result.peers_succeeded += 1;
                debug!(
                    ?peer_id,
                    "Skipping contact - already connected via gossip topic"
                );
                continue;
            }

            // Check Fibonacci backoff
            if !peer.should_retry_now() {
                result.peers_skipped_backoff += 1;
                debug!(
                    ?peer_id,
                    backoff_secs = peer.backoff_delay(),
                    "Skipping peer - backoff not elapsed"
                );
                continue;
            }

            result.peers_attempted += 1;
            peer.record_attempt();
            debug!(
                ?peer_id,
                is_contact = peer.is_contact(),
                attempt_number = peer.connection_attempts,
                "Attempting startup connection"
            );

            // Try to connect with 10 second timeout
            match tokio::time::timeout(
                std::time::Duration::from_secs(10),
                gossip.endpoint().connect(peer_id, iroh_gossip::net::GOSSIP_ALPN),
            )
            .await
            {
                Ok(Ok(_conn)) => {
                    peer.record_success();
                    result.peers_succeeded += 1;
                    info!(
                        ?peer_id,
                        is_contact = peer.is_contact(),
                        success_rate = format!("{:.1}%", peer.success_rate() * 100.0),
                        "Connected on startup"
                    );
                }
                Ok(Err(e)) => {
                    peer.record_failure();
                    debug!(
                        ?peer_id,
                        error = ?e,
                        next_retry_in_secs = peer.backoff_delay(),
                        "Failed to connect on startup"
                    );
                }
                Err(_) => {
                    peer.record_failure();
                    debug!(
                        ?peer_id,
                        next_retry_in_secs = peer.backoff_delay(),
                        "Connection timed out on startup"
                    );
                }
            }

            // Save updated peer metrics (with connection attempt results)
            self.storage.save_peer(&peer)?;
        }

        info!(
            attempted = result.peers_attempted,
            succeeded = result.peers_succeeded,
            skipped = result.peers_skipped_backoff,
            jitter_ms = result.jitter_delay_ms,
            "Startup sync complete"
        );

        Ok(result)
    }

    /// Manually trigger a sync to refresh peer information and broadcast profile changes.
    ///
    /// This is a user-initiated sync that:
    /// 1. Broadcasts our current profile to all contacts (announces changes)
    /// 2. Reconnects to contact profile topics (to receive their updates)
    ///
    /// Unlike `startup_sync()`, this does not include jitter delays or extensive
    /// reconnection attempts - it's designed for immediate user feedback.
    ///
    /// # Returns
    ///
    /// Returns the number of contacts reconnected to.
    ///
    /// # Errors
    ///
    /// Returns error if profile announcement fails (non-fatal errors are logged).
    pub async fn manual_sync(&mut self) -> Result<usize, SyncError> {
        info!("Manual sync triggered by user");

        // Count contacts from storage (regardless of reconnection success)
        let contacts_count = self.storage.list_contacts().map(|c| c.len()).unwrap_or(0);

        // 1. Broadcast our current profile to announce any changes
        if self.identity.is_some() {
            match self.announce_profile(None).await {
                Ok(()) => {
                    info!("Profile broadcast successful");
                }
                Err(e) => {
                    // Log but don't fail - we can still reconnect to contacts
                    warn!(error = %e, "Failed to broadcast profile during manual sync");
                }
            }
        }

        // 2. Reconnect to contact profile topics to receive their updates
        // Use ensure_contact_manager to initialize if needed (it's lazy-loaded)
        if self.identity.is_some() {
            match self.ensure_contact_manager().await {
                Ok(manager) => {
                    if let Err(e) = manager.reconnect_contacts().await {
                        warn!(error = %e, "Failed to reconnect contacts during manual sync");
                    } else {
                        info!(contacts = contacts_count, "Reconnected to contact topics");
                    }
                }
                Err(e) => {
                    warn!(error = %e, "Failed to initialize contact manager for sync");
                }
            }
        }

        info!(contacts_count, "Manual sync complete");
        Ok(contacts_count)
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

        // Create verification function that looks up sender's public key from pinned profiles
        let verify_fn = Self::make_verify_fn(&self.storage);

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
    /// Verifies hybrid Ed25519 + ML-DSA-65 signatures against the sender's
    /// public key from their pinned profile. Unknown senders (those without
    /// a pinned profile) are rejected.
    fn make_verify_fn(storage: &Storage) -> impl Fn(&str, &[u8], &[u8]) -> bool + '_ {
        move |sender_did: &str, data: &[u8], sig_bytes: &[u8]| -> bool {
            use crate::identity::HybridSignature;
            use tracing::{debug, warn};

            // 1. Deserialize the signature from bytes
            let signature = match HybridSignature::from_bytes(sig_bytes) {
                Ok(sig) => sig,
                Err(e) => {
                    warn!(sender = %sender_did, error = ?e, "Failed to parse signature");
                    return false;
                }
            };

            // 2. Look up sender's public key from pinned profiles
            let public_key = match storage.load_pinned_profile(sender_did) {
                Ok(Some(pin)) => pin.signed_profile.public_key,
                Ok(None) => {
                    debug!(sender = %sender_did, "Unknown sender - no pinned profile");
                    return false;
                }
                Err(e) => {
                    warn!(sender = %sender_did, error = ?e, "Failed to load pinned profile");
                    return false;
                }
            };

            // 3. Verify BOTH Ed25519 and ML-DSA-65 signatures
            let valid = public_key.verify(data, &signature);
            if !valid {
                warn!(sender = %sender_did, "Signature verification failed");
            }
            valid
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
        // Check if this is the Private realm
        if let Ok(Some(info)) = self.storage.load_realm(realm_id) {
            if is_private_realm_name(&info.name) {
                return Err(SyncError::PrivateRealmOperation(
                    "Cannot generate invite for Private realm".to_string(),
                ));
            }
        }

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
        // by forcing Joy to send a message first, which establishes the QUIC connection
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

    // ═══════════════════════════════════════════════════════════════════════
    // Profile Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Save a user profile
    pub fn save_profile(&self, profile: &crate::types::UserProfile) -> Result<(), SyncError> {
        self.storage.save_profile(profile)
    }

    /// Load a profile by peer ID
    pub fn load_profile(
        &self,
        peer_id: &str,
    ) -> Result<Option<crate::types::UserProfile>, SyncError> {
        self.storage.load_profile(peer_id)
    }

    /// Delete a profile by peer ID
    pub fn delete_profile(&self, peer_id: &str) -> Result<(), SyncError> {
        self.storage.delete_profile(peer_id)
    }

    /// List all profiles
    pub fn list_profiles(&self) -> Result<Vec<crate::types::UserProfile>, SyncError> {
        self.storage.list_profiles()
    }

    /// Get or create profile for this node
    pub fn get_own_profile(&self) -> Result<crate::types::UserProfile, SyncError> {
        // Use DID as peer_id (stable identifier, available immediately)
        let peer_id = self
            .did()
            .ok_or_else(|| SyncError::Identity("Identity not initialized".to_string()))?
            .to_string();

        // Try to load existing profile
        if let Some(profile) = self.storage.load_profile(&peer_id)? {
            Ok(profile)
        } else {
            // Create default profile with placeholder text to trigger edit mode
            let mut profile = crate::types::UserProfile::new(peer_id.clone(), "Anonymous User".to_string());
            profile.bio = "**Add your bio here**\n\nDescribe yourself, your interests, or your role in the network.\n\n- Use *markdown* for formatting\n- Add links, lists, and more\n- Express your unique identity".to_string();
            self.storage.save_profile(&profile)?;
            Ok(profile)
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Profile Pinning Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Create and store a signed version of our own profile.
    ///
    /// This should be called whenever the profile is updated. The signed profile
    /// is stored as a pin with relationship `Own` and can be announced to peers.
    ///
    /// # Returns
    ///
    /// The signed profile that was created and stored.
    pub fn sign_and_pin_own_profile(&mut self) -> Result<crate::types::SignedProfile, SyncError> {
        let profile = self.get_own_profile()?;

        // Ensure identity is initialized
        self.init_identity()?;
        let keypair = self.identity.as_ref().ok_or_else(|| {
            SyncError::Identity("Identity not initialized".to_string())
        })?;

        // Create signed profile
        let signed = crate::types::SignedProfile::sign(&profile, keypair);
        let did = signed.did().to_string();

        // Create pin with Own relationship
        let pin = crate::types::ProfilePin::new(
            did.clone(),
            signed.clone(),
            crate::types::PinRelationship::Own,
        );

        // Save to storage (Own pins are never evicted)
        self.storage.save_pinned_profile(&pin)?;

        debug!(did = %did, "Signed and pinned own profile");
        Ok(signed)
    }

    /// Get our own pinned profile (if exists).
    pub fn get_own_pinned_profile(&self) -> Result<Option<crate::types::ProfilePin>, SyncError> {
        self.storage.get_own_pinned_profile()
    }

    /// Pin a signed profile from another peer.
    ///
    /// This stores the profile for redundancy, allowing us to serve it
    /// to other peers who request it.
    ///
    /// # Arguments
    ///
    /// * `signed_profile` - The cryptographically signed profile to pin
    /// * `relationship` - Why we're pinning this profile
    ///
    /// # Returns
    ///
    /// A list of DIDs that were evicted (if storage limits were reached).
    pub fn pin_profile(
        &self,
        signed_profile: crate::types::SignedProfile,
        relationship: crate::types::PinRelationship,
    ) -> Result<Vec<String>, SyncError> {
        // Verify signature first
        if !signed_profile.verify() {
            return Err(SyncError::SignatureInvalid(
                "Profile signature verification failed".to_string(),
            ));
        }

        let did = signed_profile.did().to_string();
        let pin = crate::types::ProfilePin::new(did.clone(), signed_profile, relationship);

        // Use default pinning config
        let config = crate::storage::PinningConfig::default();
        let evicted = self.storage.save_pinned_profile_with_limits(&pin, &config)?;

        debug!(did = %did, evicted = evicted.len(), "Pinned profile");
        Ok(evicted)
    }

    /// Get a pinned profile by DID.
    pub fn get_pinned_profile(&self, did: &str) -> Result<Option<crate::types::ProfilePin>, SyncError> {
        self.storage.load_pinned_profile(did)
    }

    /// Unpin a profile by DID.
    ///
    /// Note: Own profile cannot be unpinned.
    pub fn unpin_profile(&self, did: &str) -> Result<(), SyncError> {
        // Check if this is our own profile
        if let Some(pin) = self.storage.load_pinned_profile(did)? {
            if pin.is_own() {
                return Err(SyncError::InvalidOperation(
                    "Cannot unpin own profile".to_string(),
                ));
            }
        }

        self.storage.delete_pinned_profile(did)
    }

    /// List all pinned profiles.
    pub fn list_pinned_profiles(&self) -> Result<Vec<crate::types::ProfilePin>, SyncError> {
        self.storage.list_pinned_profiles()
    }

    /// List pinned profiles by relationship type.
    pub fn list_pinned_profiles_by_relationship(
        &self,
        relationship: &crate::types::PinRelationship,
    ) -> Result<Vec<crate::types::ProfilePin>, SyncError> {
        self.storage.list_pinned_profiles_by_relationship(relationship)
    }

    /// Update an existing pin with a new signed profile.
    ///
    /// This is used when receiving profile announcements for peers we already have pinned.
    pub fn update_pinned_profile(
        &self,
        did: &str,
        signed_profile: crate::types::SignedProfile,
    ) -> Result<bool, SyncError> {
        // Verify signature first
        if !signed_profile.verify() {
            return Err(SyncError::SignatureInvalid(
                "Profile signature verification failed".to_string(),
            ));
        }

        // Check if we have an existing pin
        if let Some(mut pin) = self.storage.load_pinned_profile(did)? {
            // Update the profile
            if pin.update_profile(signed_profile) {
                self.storage.save_pinned_profile(&pin)?;
                debug!(did = %did, "Updated pinned profile");
                Ok(true)
            } else {
                // update_profile returns false if signature is invalid
                // (but we already verified above, so this shouldn't happen)
                Ok(false)
            }
        } else {
            // No existing pin for this DID
            Ok(false)
        }
    }

    /// Get the count of pinned profiles (excluding own).
    pub fn pinned_profile_count(&self) -> Result<usize, SyncError> {
        self.storage.count_pinned_profiles()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Network Page: Pinners (Who Pins Our Profile)
    // ═══════════════════════════════════════════════════════════════════════

    /// List all peers who are pinning our profile ("Souls Carrying Your Light").
    ///
    /// Returns pinners sorted by pinned_at (newest first).
    pub fn list_profile_pinners(&self) -> Result<Vec<crate::storage::PinnerInfo>, SyncError> {
        self.storage.list_pinners()
    }

    /// Count the number of peers pinning our profile.
    pub fn count_profile_pinners(&self) -> Result<usize, SyncError> {
        self.storage.count_pinners()
    }

    // Note: record_pinner() and remove_pinner() removed - Indra's Net derives pinners from contacts

    /// Get network statistics for the Network page.
    ///
    /// Returns counts for peers, pinners, and pinned profiles.
    pub fn network_stats(&self) -> NetworkStats {
        let total_peers = self.peer_registry.count().unwrap_or(0);
        let online_peers = self.peer_registry.count_by_status(PeerStatus::Online).unwrap_or(0);
        let pinners_count = self.storage.count_pinners().unwrap_or(0);
        let pinning_count = self.storage.count_pinned_profiles().unwrap_or(0);

        NetworkStats {
            total_peers,
            online_peers,
            pinners_count,
            pinning_count,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Auto-Pinning Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if a DID should be automatically pinned.
    ///
    /// Returns `true` if the DID belongs to a contact or realm member.
    pub fn should_auto_pin(&self, did: &str) -> bool {
        // Check if this DID is an accepted contact
        if let Ok(contacts) = self.storage.list_contacts() {
            for contact in contacts {
                if contact.peer_did == did {
                    return true;
                }
            }
        }

        // Check if this DID is a member of any realm we're in
        // For now, we auto-pin any contact; realm member pinning can be
        // added when realm membership tracking is implemented
        false
    }

    /// Determine the relationship for auto-pinning a DID.
    ///
    /// Returns the appropriate `PinRelationship` based on our relationship with the DID.
    pub fn determine_pin_relationship(&self, did: &str) -> Option<crate::types::PinRelationship> {
        // Check if this DID is a contact
        if let Ok(contacts) = self.storage.list_contacts() {
            for contact in contacts {
                if contact.peer_did == did {
                    return Some(crate::types::PinRelationship::Contact);
                }
            }
        }

        // TODO: Check if this DID is a realm member
        // For now, return None if not a contact
        None
    }

    /// Process a profile gossip announcement for potential auto-pinning.
    ///
    /// If the announcement is from a contact or realm member, their profile
    /// will be automatically pinned or updated.
    ///
    /// # Arguments
    ///
    /// * `message` - The profile gossip message to process
    ///
    /// # Returns
    ///
    /// The action taken (UpdatePin, Ignore, etc.) for logging/testing purposes.
    pub fn process_profile_announcement(
        &self,
        message: &crate::sync::ProfileGossipMessage,
    ) -> Result<crate::sync::ProfileAction, SyncError> {
        use crate::sync::{ProfileAction, ProfileGossipMessage};

        match message {
            ProfileGossipMessage::Announce {
                signed_profile,
                avatar_ticket,
            } => {
                // Verify signature first
                if !signed_profile.verify() {
                    warn!("Received profile announcement with invalid signature");
                    return Ok(ProfileAction::Ignore);
                }

                let signer_did = signed_profile.did().to_string();

                // Check if we should auto-pin this DID
                if let Some(relationship) = self.determine_pin_relationship(&signer_did) {
                    // Check if we already have this profile pinned
                    if let Some(existing) = self.storage.load_pinned_profile(&signer_did)? {
                        // Update existing pin
                        let mut updated = existing.clone();
                        if updated.update_profile(signed_profile.clone()) {
                            if let Some(ticket) = avatar_ticket {
                                // Parse the ticket and extract the hash for avatar tracking
                                if let Ok(blob_ticket) =
                                    ticket.parse::<iroh_blobs::ticket::BlobTicket>()
                                {
                                    updated.avatar_hash = Some(*blob_ticket.hash().as_bytes());
                                }
                            }
                            self.storage.save_pinned_profile(&updated)?;
                            debug!(did = %signer_did, "Updated pinned profile from announcement");
                        }
                    } else {
                        // Create new pin
                        let mut pin = crate::types::ProfilePin::new(
                            signer_did.clone(),
                            signed_profile.clone(),
                            relationship,
                        );
                        if let Some(ticket) = avatar_ticket {
                            if let Ok(blob_ticket) =
                                ticket.parse::<iroh_blobs::ticket::BlobTicket>()
                            {
                                pin.avatar_hash = Some(*blob_ticket.hash().as_bytes());
                            }
                        }
                        let config = crate::storage::PinningConfig::default();
                        self.storage.save_pinned_profile_with_limits(&pin, &config)?;
                        debug!(did = %signer_did, "Auto-pinned profile from announcement");
                    }

                    Ok(ProfileAction::UpdatePin {
                        signed_profile: signed_profile.clone(),
                        avatar_ticket: avatar_ticket.clone(),
                    })
                } else {
                    // Not a contact or realm member, ignore
                    Ok(ProfileAction::Ignore)
                }
            }
            _ => {
                // Request and Response messages are not auto-pin triggers
                Ok(ProfileAction::Ignore)
            }
        }
    }

    /// Auto-pin a contact's profile when they are accepted.
    ///
    /// This creates a placeholder pin for the contact that will be updated
    /// when we receive their profile announcement.
    ///
    /// # Arguments
    ///
    /// * `contact` - The contact info from the acceptance event
    ///
    /// # Returns
    ///
    /// True if a new pin was created, false if already pinned.
    pub fn auto_pin_contact(&self, contact: &crate::types::contact::ContactInfo) -> Result<bool, SyncError> {
        let did = &contact.peer_did;

        // Check if already pinned
        if self.storage.load_pinned_profile(did)?.is_some() {
            debug!(did = %did, "Contact already pinned");
            return Ok(false);
        }

        // Note: We can't create a proper SignedProfile without the contact's keypair.
        // The actual pinning will happen when we receive their profile announcement
        // via process_profile_announcement(). For now, just log that we're interested.
        debug!(
            did = %did,
            name = %contact.profile.display_name,
            "Contact accepted, will auto-pin on profile announcement"
        );

        // Return true to indicate we registered interest
        Ok(true)
    }

    /// Get all contact DIDs that we should auto-pin profiles for.
    ///
    /// This returns a list of DIDs from our accepted contacts.
    pub fn get_auto_pin_interests(&self) -> Result<Vec<String>, SyncError> {
        let contacts = self.storage.list_contacts()?;
        Ok(contacts.into_iter().map(|c| c.peer_did).collect())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Profile Sync Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Announce our profile to the global profile topic.
    ///
    /// This broadcasts a signed profile announcement to all peers subscribed
    /// to the global profile topic. Peers who are interested in our profile
    /// (contacts, realm members) will auto-pin it.
    ///
    /// # Arguments
    ///
    /// * `avatar_ticket` - Optional blob ticket for avatar download
    ///
    /// # Returns
    ///
    /// Ok(()) if announcement was broadcast successfully.
    ///
    /// # Errors
    ///
    /// Returns error if gossip is not initialized or broadcast fails.
    pub async fn announce_profile(&mut self, avatar_ticket: Option<String>) -> Result<(), SyncError> {
        // Ensure we have a signed profile
        let signed = self.sign_and_pin_own_profile()?;

        // Create the announcement message
        let announcement = crate::sync::ProfileGossipMessage::announce(signed.clone(), avatar_ticket.clone());
        let bytes = announcement.to_bytes()?;

        // 1. Broadcast on our own per-peer profile topic (for backwards compatibility)
        if let Some(sender) = self.profile_gossip_sender.as_ref() {
            if let Err(e) = sender.broadcast(bytes.clone()).await {
                warn!(error = %e, "Failed to broadcast on per-peer profile topic (non-fatal)");
            } else {
                debug!("Profile announcement broadcast on per-peer topic");
            }
        }

        // 2. Broadcast on all 1:1 contact topics (the WORKING channel!)
        // This is the primary mechanism for profile propagation since contact topics
        // have proper bi-directional mesh formation from the contact exchange.
        if let Some(ref manager) = self.contact_manager {
            let contacts_updated = manager.broadcast_profile_to_contacts(&bytes).await;
            info!(contacts_updated, "Profile announcement broadcast on contact topics");
        }

        info!("Profile announcement broadcast complete");
        Ok(())
    }

    // Note: send_pin_acknowledgment() and send_pin_removal() removed - Indra's Net derives pinners from contacts
    // In Indra's Net, contact acceptance = implicit mutual mirroring. No explicit acknowledgment needed.

    /// Update our own profile and broadcast the change to peers.
    ///
    /// This is the main method to use when the user edits their profile.
    /// It saves the profile locally and broadcasts an announcement so peers
    /// who have pinned our profile can update their copy.
    ///
    /// # Arguments
    ///
    /// * `profile` - The updated profile data
    /// * `avatar_ticket` - Optional blob ticket for avatar (if avatar changed)
    ///
    /// # Returns
    ///
    /// Ok(()) if profile was saved and broadcast successfully.
    pub async fn update_own_profile(
        &mut self,
        profile: &crate::types::UserProfile,
        avatar_ticket: Option<String>,
    ) -> Result<(), SyncError> {
        // Save the profile locally
        self.storage.save_profile(profile)?;

        // Sign, pin, and broadcast the update
        self.announce_profile(avatar_ticket).await?;

        info!(peer_id = %profile.peer_id, "Profile updated and broadcast");
        Ok(())
    }

    /// Start listening for profile announcements.
    ///
    /// This method sets up profile sync by subscribing to:
    /// 1. **Our own profile topic** (derived from our DID) - for broadcasting our profile updates
    /// 2. **Global profile topic** (legacy) - for backwards compatibility during migration
    ///
    /// When a profile announcement is received from a contact, their profile is auto-pinned.
    /// Contacts also subscribe to our per-peer topic via `finalize_contact()`.
    ///
    /// # Per-Peer Topic Architecture
    ///
    /// Each peer has their own profile topic: `BLAKE3("sync-profile" || peer_did)`
    /// - Only contacts receive your updates (targeted, not global)
    /// - Scalability: O(contacts) messages instead of O(all users)
    /// - Privacy: Non-contacts don't receive your profile updates
    ///
    /// # Arguments
    ///
    /// * `bootstrap_peers` - Initial peers to connect to (can be empty)
    ///
    /// # Returns
    ///
    /// Ok(()) if subscription started successfully.
    pub async fn start_profile_sync(
        &mut self,
        bootstrap_peers: Vec<iroh::PublicKey>,
    ) -> Result<(), SyncError> {
        // Check if already initialized - don't replace existing subscription!
        // Replacing the subscription would drop the old receiver, closing the topic
        // and preventing contacts from joining.
        if self.profile_gossip_sender.is_some() {
            debug!("Profile sync already initialized, skipping");
            return Ok(());
        }

        // Get our DID for processing messages directed at us
        let our_did = self
            .did()
            .ok_or_else(|| SyncError::Identity("Identity not initialized".to_string()))?
            .to_string();

        // Ensure gossip is initialized
        let gossip = self.ensure_gossip().await?;

        // Subscribe to our OWN profile topic (for broadcasting)
        // Contacts subscribe to this topic via finalize_contact() to receive our updates
        let own_topic_id = crate::sync::derive_profile_topic(&our_did);
        let (sender, own_receiver) = gossip.subscribe_split(own_topic_id, vec![]).await?;

        // Store sender for reuse by announce_profile() and other profile broadcast methods.
        // This allows all profile announcements to use the same persistent subscription.
        self.profile_gossip_sender = Some(sender.clone());

        // CRITICAL: Store receiver to keep the subscription alive!
        // Dropping the receiver would close the gossip topic, preventing contacts from joining.
        self.profile_gossip_receiver = Some(own_receiver);

        info!(
            did = %our_did,
            ?own_topic_id,
            "Subscribed to own profile topic for broadcasting"
        );

        // Also subscribe to the global topic for backwards compatibility and packet broadcast
        // Packets (messages) are broadcast on the global topic so all contacts receive them
        let global_topic_id = crate::sync::global_profile_topic();
        let (global_sender, mut receiver) = gossip.subscribe_split(global_topic_id, bootstrap_peers).await?;

        // Store global sender for packet broadcasts (messages to contacts)
        self.global_profile_gossip_sender = Some(global_sender);

        // Clone dependencies for the background task
        let storage = self.storage.clone();
        let blob_manager = self.blob_manager.clone();
        let endpoint = gossip.endpoint().clone();
        let contact_event_tx = self.contact_event_tx.clone();

        // Spawn background task to process incoming profile messages
        tokio::spawn(async move {
            use crate::sync::TopicEvent;

            info!("Profile sync listener started with P2P blob download support");

            while let Some(event) = receiver.recv_event().await {
                match event {
                    TopicEvent::Message(msg) => {
                        // Try to parse as profile gossip message
                        match crate::sync::ProfileGossipMessage::from_bytes(&msg.content) {
                            Ok(profile_msg) => {
                                match profile_msg {
                                    crate::sync::ProfileGossipMessage::Announce {
                                        signed_profile,
                                        avatar_ticket,
                                    } => {
                                        // Verify signature
                                        if !signed_profile.verify() {
                                            warn!("Received profile announcement with invalid signature");
                                            continue;
                                        }

                                        let signer_did = signed_profile.did().to_string();

                                        // Check if we should auto-pin (is a contact)
                                        if let Ok(contacts) = storage.list_contacts() {
                                            let is_contact = contacts.iter().any(|c| c.peer_did == signer_did);

                                            if is_contact {
                                                // Parse avatar ticket and check if we need to download
                                                let avatar_hash = if let Some(ticket_str) = &avatar_ticket {
                                                    if let Ok(blob_ticket) = ticket_str.parse::<iroh_blobs::ticket::BlobTicket>() {
                                                        let hash = blob_ticket.hash();

                                                        // Check if we already have this avatar
                                                        let has_blob = blob_manager.has_blob(&hash).await.unwrap_or(false);

                                                        if !has_blob {
                                                            // Download avatar via P2P
                                                            match blob_manager.download_blob(&blob_ticket, &endpoint).await {
                                                                Ok(downloaded_hash) => {
                                                                    info!(did = %signer_did, ?downloaded_hash, "Downloaded avatar via P2P");
                                                                }
                                                                Err(e) => {
                                                                    warn!(did = %signer_did, error = %e, "Failed to download avatar via P2P");
                                                                }
                                                            }
                                                        }

                                                        Some(*hash.as_bytes())
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                };

                                                // Check if already pinned
                                                let is_new_pin = storage.load_pinned_profile(&signer_did)
                                                    .ok()
                                                    .flatten()
                                                    .is_none();

                                                if let Ok(Some(mut existing)) = storage.load_pinned_profile(&signer_did) {
                                                    // Update existing pin
                                                    if existing.update_profile(signed_profile.clone()) {
                                                        if let Some(hash) = avatar_hash {
                                                            existing.avatar_hash = Some(hash);
                                                        }
                                                        let _ = storage.save_pinned_profile(&existing);
                                                        debug!(did = %signer_did, "Updated pinned profile from gossip");
                                                    }
                                                } else {
                                                    // Create new pin
                                                    let mut pin = crate::types::ProfilePin::new(
                                                        signer_did.clone(),
                                                        signed_profile.clone(),
                                                        crate::types::PinRelationship::Contact,
                                                    );
                                                    if let Some(hash) = avatar_hash {
                                                        pin.avatar_hash = Some(hash);
                                                    }
                                                    let config = crate::storage::PinningConfig::default();
                                                    let _ = storage.save_pinned_profile_with_limits(&pin, &config);
                                                    debug!(did = %signer_did, "Auto-pinned contact profile from gossip");
                                                }

                                                // Also update Peer.profile for UI display
                                                // This keeps the unified Peer record in sync with profile changes
                                                if let Ok(Some(mut peer)) = storage.load_peer_by_did(&signer_did) {
                                                    peer.profile = Some(crate::types::ProfileSnapshot {
                                                        display_name: signed_profile.profile.display_name.clone(),
                                                        subtitle: signed_profile.profile.subtitle.clone(),
                                                        avatar_blob_id: signed_profile.profile.avatar_blob_id.clone(),
                                                        bio: crate::types::ProfileSnapshot::truncate_bio(&signed_profile.profile.bio),
                                                    });
                                                    if let Err(e) = storage.save_peer(&peer) {
                                                        warn!(did = %signer_did, error = %e, "Failed to update Peer.profile");
                                                    } else {
                                                        info!(did = %signer_did, "Updated peer profile from topic");
                                                        // Notify UI of profile update
                                                        let _ = contact_event_tx.send(crate::sync::ContactEvent::ProfileUpdated {
                                                            did: signer_did.clone(),
                                                        });
                                                    }
                                                }

                                                // Note: PinAcknowledgment removed - Indra's Net derives pinners from contacts
                                                // In Indra's Net, contacts automatically mirror each other
                                                if is_new_pin {
                                                    debug!(target_did = %signer_did, "New profile pinned (implicit mirroring)");
                                                }
                                            }
                                        }
                                    }

                                    // Note: PinAcknowledgment and PinRemoval handlers removed
                                    // Indra's Net derives pinners from contacts instead

                                    // Request and Response are for direct profile lookups
                                    // They're handled by the profile processor in a different flow
                                    crate::sync::ProfileGossipMessage::Request { .. } => {
                                        debug!("Received profile request (handled by processor)");
                                    }
                                    crate::sync::ProfileGossipMessage::Response { .. } => {
                                        debug!("Received profile response (handled by processor)");
                                    }

                                    // Packet from a profile's append-only log
                                    crate::sync::ProfileGossipMessage::Packet { envelope } => {
                                        let sender_did = envelope.sender.to_string();
                                        debug!(
                                            sender = %sender_did,
                                            sequence = envelope.sequence,
                                            "Received packet from gossip"
                                        );

                                        // Check if sender is a contact (we mirror contacts)
                                        let should_mirror = storage.list_contacts()
                                            .map(|contacts| contacts.iter().any(|c| c.peer_did == sender_did))
                                            .unwrap_or(false);

                                        if should_mirror {
                                            // Create MirrorStore and store the packet
                                            match MirrorStore::new(storage.db_handle()) {
                                                Ok(mirror) => {
                                                    match mirror.store_packet(&envelope) {
                                                        Ok(_) => {
                                                            info!(
                                                                sender = %sender_did,
                                                                sequence = envelope.sequence,
                                                                "Stored packet in mirror"
                                                            );
                                                        }
                                                        Err(e) => {
                                                            warn!(
                                                                sender = %sender_did,
                                                                sequence = envelope.sequence,
                                                                error = %e,
                                                                "Failed to store packet in mirror"
                                                            );
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    warn!(error = %e, "Failed to create MirrorStore");
                                                }
                                            }
                                        } else {
                                            debug!(
                                                sender = %sender_did,
                                                "Ignoring packet from non-contact"
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                debug!("Failed to parse profile message: {}", e);
                            }
                        }
                    }
                    TopicEvent::NeighborUp(peer) => {
                        debug!(?peer, "Profile topic neighbor joined");
                    }
                    TopicEvent::NeighborDown(peer) => {
                        debug!(?peer, "Profile topic neighbor left");
                    }
                }
            }

            info!("Profile sync listener stopped");
        });

        info!("Profile sync started on global topic");
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Contact Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Generate a contact invite for sharing with peers
    ///
    /// Creates a signed invite containing profile information that can be shared
    /// via QR code or text. The invite expires after the specified number of hours.
    ///
    /// # Arguments
    ///
    /// * `expiry_hours` - Hours until invite expires (max 168 = 7 days)
    ///
    /// # Returns
    ///
    /// A base58-encoded invite string prefixed with "sync-contact:"
    ///
    /// # Errors
    ///
    /// Returns error if contact manager initialization fails or profile cannot be loaded.
    pub async fn generate_contact_invite(
        &mut self,
        expiry_hours: u8,
    ) -> Result<String, SyncError> {
        // Ensure contact manager is initialized
        let manager = self.ensure_contact_manager().await?;

        // Get own profile to include in invite
        let profile = self.get_own_profile()?;

        // Create profile snapshot
        let snapshot = ProfileSnapshot {
            display_name: profile.display_name.clone(),
            subtitle: profile.subtitle.clone(),
            avatar_blob_id: profile.avatar_blob_id.clone(),
            bio: ProfileSnapshot::truncate_bio(&profile.bio),
        };

        // Generate invite
        manager.generate_invite(snapshot, expiry_hours)
    }

    /// Decode a contact invite string
    ///
    /// Validates the invite signature, checks expiry, and verifies it hasn't been revoked.
    ///
    /// # Arguments
    ///
    /// * `invite_str` - The invite code (e.g., "sync-contact:...")
    ///
    /// # Returns
    ///
    /// The decoded and validated invite with profile information
    ///
    /// # Errors
    ///
    /// Returns error if invite is invalid, expired, or revoked.
    pub async fn decode_contact_invite(
        &mut self,
        invite_str: &str,
    ) -> Result<HybridContactInvite, SyncError> {
        // Ensure contact manager is initialized
        let manager = self.ensure_contact_manager().await?;

        // Decode invite
        manager.decode_invite(invite_str)
    }

    /// Send a contact request to a peer
    ///
    /// Initiates the contact request protocol by sending a request to the inviter.
    /// The request will be saved as OutgoingPending until the peer responds.
    ///
    /// # Arguments
    ///
    /// * `invite` - The decoded contact invite
    ///
    /// # Errors
    ///
    /// Returns error if contact manager initialization fails or request cannot be sent.
    pub async fn send_contact_request(
        &mut self,
        invite: HybridContactInvite,
    ) -> Result<(), SyncError> {
        // Ensure contact manager is initialized
        let manager = self.ensure_contact_manager().await?;

        // Get own profile to include in request
        let profile = self.get_own_profile()?;

        // Create profile snapshot
        let snapshot = ProfileSnapshot {
            display_name: profile.display_name.clone(),
            subtitle: profile.subtitle.clone(),
            avatar_blob_id: profile.avatar_blob_id.clone(),
            bio: ProfileSnapshot::truncate_bio(&profile.bio),
        };

        // Send contact request
        manager.send_contact_request(invite, snapshot).await
    }

    /// Accept an incoming contact request
    ///
    /// Accepts a pending contact request and finalizes the connection if both
    /// parties have accepted. Once mutually accepted, the contact is saved and
    /// both nodes subscribe to a shared 1:1 gossip topic.
    ///
    /// # Arguments
    ///
    /// * `invite_id` - The unique invite ID from the pending request
    ///
    /// # Errors
    ///
    /// Returns error if invite_id not found or contact finalization fails.
    pub async fn accept_contact(&mut self, invite_id: &[u8; 16]) -> Result<(), SyncError> {
        // Ensure contact manager is initialized
        let manager = self.ensure_contact_manager().await?;

        // Accept contact request
        manager.accept_contact_request(invite_id).await
    }

    /// Decline an incoming contact request
    ///
    /// Rejects a pending contact request and removes it from storage.
    ///
    /// # Arguments
    ///
    /// * `invite_id` - The unique invite ID from the pending request
    ///
    /// # Errors
    ///
    /// Returns error if invite_id not found.
    pub async fn decline_contact(&mut self, invite_id: &[u8; 16]) -> Result<(), SyncError> {
        // Ensure contact manager is initialized
        let manager = self.ensure_contact_manager().await?;

        // Decline contact request
        manager.decline_contact_request(invite_id).await
    }

    /// Cancel an outgoing contact request
    ///
    /// Removes the pending request and revokes the invite so it can no longer be used.
    ///
    /// # Arguments
    ///
    /// * `invite_id` - The unique invite ID of the outgoing request
    ///
    /// # Errors
    ///
    /// Returns error if invite_id not found or not in OutgoingPending state.
    pub async fn cancel_outgoing_request(&mut self, invite_id: &[u8; 16]) -> Result<(), SyncError> {
        // Ensure contact manager is initialized
        let manager = self.ensure_contact_manager().await?;

        // Cancel outgoing request
        manager.cancel_outgoing_request(invite_id)
    }

    /// List all accepted contacts
    ///
    /// Returns all contacts that have been mutually accepted.
    ///
    /// # Returns
    ///
    /// Vector of all stored contacts with their online/offline status.
    pub fn list_contacts(&self) -> Result<Vec<ContactInfo>, SyncError> {
        self.storage.list_contacts()
    }

    /// List pending contact requests
    ///
    /// Returns both incoming (awaiting our response) and outgoing (awaiting their response)
    /// pending contact requests.
    ///
    /// # Returns
    ///
    /// Tuple of (incoming_requests, outgoing_requests)
    pub fn list_pending_contacts(&self) -> Result<(Vec<PendingContact>, Vec<PendingContact>), SyncError> {
        let incoming = self.storage.list_incoming_pending()?;
        let outgoing = self.storage.list_outgoing_pending()?;
        Ok((incoming, outgoing))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Unified Peer Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// List all unified peers
    ///
    /// Returns all known network participants, including both contacts
    /// (mutually accepted peers with full identity) and discovered peers
    /// (seen via realm gossip but not yet contacts).
    pub fn list_peers(&self) -> Result<Vec<crate::types::peer::Peer>, SyncError> {
        self.storage.list_peers()
    }

    /// Get a peer by endpoint ID
    pub fn get_peer(&self, endpoint_id: &iroh::PublicKey) -> Result<Option<crate::types::peer::Peer>, SyncError> {
        self.storage.load_peer(endpoint_id)
    }

    /// Get a peer by DID
    ///
    /// Uses the DID index for fast lookup.
    pub fn get_peer_by_did(&self, did: &str) -> Result<Option<crate::types::peer::Peer>, SyncError> {
        self.storage.load_peer_by_did(did)
    }

    /// List only contacts (peers with mutual acceptance)
    ///
    /// These are peers where `contact_info` is Some, meaning both parties
    /// have accepted the contact exchange.
    pub fn list_peer_contacts(&self) -> Result<Vec<crate::types::peer::Peer>, SyncError> {
        self.storage.list_peer_contacts()
    }

    /// List only discovered peers (non-contacts)
    ///
    /// These are peers seen via realm gossip but not yet mutually accepted
    /// as contacts.
    pub fn list_discovered_peers(&self) -> Result<Vec<crate::types::peer::Peer>, SyncError> {
        self.storage.list_discovered_peers()
    }

    /// Save a unified peer
    ///
    /// Updates or creates a peer record. Also updates the DID index
    /// if the peer has a DID.
    pub fn save_peer(&self, peer: &crate::types::peer::Peer) -> Result<(), SyncError> {
        self.storage.save_peer(peer)
    }

    /// Run migration from old contact/peer tables to unified peers
    ///
    /// This is idempotent - safe to call multiple times.
    /// Returns the number of peers migrated.
    pub fn migrate_to_unified_peers(&self) -> Result<usize, SyncError> {
        self.storage.migrate_to_unified_peers()
    }

    /// Subscribe to contact events
    ///
    /// Returns a broadcast receiver for real-time contact event notifications.
    /// Events include new requests, acceptances, online/offline status changes, etc.
    ///
    /// # Returns
    ///
    /// Broadcast receiver for ContactEvent messages
    pub async fn subscribe_contact_events(
        &mut self,
    ) -> Result<broadcast::Receiver<ContactEvent>, SyncError> {
        // Ensure contact manager is initialized
        let manager = self.ensure_contact_manager().await?;

        // Subscribe to events
        Ok(manager.subscribe_events())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Packet Event API (for Indra's Network visualization)
    // ═══════════════════════════════════════════════════════════════════════

    /// Get packet events for a specific peer.
    ///
    /// Returns the recent packet events associated with this peer,
    /// useful for displaying packet flow in the network visualization.
    ///
    /// # Arguments
    ///
    /// * `peer_did` - The peer's decentralized identifier
    ///
    /// # Returns
    ///
    /// A vector of [`PacketEvent`] in chronological order (oldest first).
    pub fn get_packet_events_for_peer(&self, peer_did: &str) -> Vec<crate::sync::PacketEvent> {
        let events = self.packet_event_buffer.get_events_for_peer(peer_did);
        info!(
            peer_did = %peer_did,
            event_count = events.len(),
            total_in_buffer = self.packet_event_buffer.total_event_count(),
            "Getting packet events for peer"
        );
        events
    }

    /// Get all packet events for all peers.
    ///
    /// Returns a map of peer DID → events for the network visualization.
    pub fn get_all_packet_events(
        &self,
    ) -> std::collections::HashMap<String, Vec<crate::sync::PacketEvent>> {
        self.packet_event_buffer.get_all_events()
    }

    /// Subscribe to real-time packet events.
    ///
    /// Returns a receiver that will receive all new packet events as they occur.
    /// Useful for updating the UI in real-time as packets flow through the network.
    pub fn subscribe_packet_events(
        &self,
    ) -> tokio::sync::broadcast::Receiver<crate::sync::PacketEvent> {
        self.packet_event_buffer.subscribe()
    }

    /// Record a packet event (internal use).
    ///
    /// This is called when packets are sent or received to log them
    /// for the network visualization.
    pub(crate) fn record_packet_event(&self, event: crate::sync::PacketEvent) {
        self.packet_event_buffer.record(event);
    }

    /// Get the packet event buffer (for sharing with other components).
    pub(crate) fn packet_event_buffer(&self) -> Arc<crate::sync::PacketEventBuffer> {
        self.packet_event_buffer.clone()
    }

    /// Load historical packet events from storage into the in-memory buffer.
    ///
    /// This method populates the packet event buffer with packets that were
    /// stored in MirrorStore (received) and ProfileLog (sent) from previous
    /// sessions. Call this on startup to show historical packets in the
    /// network visualization.
    ///
    /// # Returns
    ///
    /// The number of events loaded into the buffer.
    pub fn load_historical_packet_events(&self) -> Result<usize, SyncError> {
        let mut loaded_count = 0;

        // Get our DID for identifying sent messages
        let my_did = self.profile_did()
            .ok_or_else(|| SyncError::Identity("Profile keys not initialized".to_string()))?;
        let my_did_str = my_did.as_str().to_string();

        // Get profile keys for decryption
        let profile_keys = self.storage.load_profile_keys()?;

        // Get all contacts
        let contacts = self.list_peer_contacts()?;

        for contact in &contacts {
            let contact_did = match &contact.did {
                Some(did) => did.clone(),
                None => continue,
            };
            let contact_name = contact.display_name();

            // Parse contact DID
            let did = match Did::parse(&contact_did) {
                Ok(d) => d,
                Err(_) => continue,
            };

            // Load received packets from MirrorStore for this contact
            if let Ok(received_packets) = self.mirror_packets_all(&did) {
                for envelope in received_packets {
                    let sender_did = envelope.sender.as_str().to_string();

                    // Determine decryption status and content preview
                    let (decryption_status, content_preview) = if envelope.is_global() {
                        let preview = match envelope.decode_global_payload() {
                            Ok(PacketPayload::DirectMessage { content, .. }) => {
                                crate::sync::PacketEvent::preview_content(&content)
                            }
                            Ok(_) => "[profile update]".to_string(),
                            Err(_) => "[global]".to_string(),
                        };
                        (crate::sync::DecryptionStatus::Global, preview)
                    } else if let Some(ref keys) = profile_keys {
                        match envelope.decrypt_for_recipient(keys) {
                            Ok(payload) => {
                                let preview = match payload {
                                    PacketPayload::DirectMessage { content, .. } => {
                                        crate::sync::PacketEvent::preview_content(&content)
                                    }
                                    _ => "[packet]".to_string(),
                                };
                                (crate::sync::DecryptionStatus::Decrypted, preview)
                            }
                            Err(_) => {
                                (crate::sync::DecryptionStatus::CannotDecrypt {
                                    reason: "not recipient".to_string()
                                }, "[encrypted]".to_string())
                            }
                        }
                    } else {
                        (crate::sync::DecryptionStatus::NotAttempted, "[encrypted]".to_string())
                    };

                    let event = crate::sync::PacketEvent {
                        id: crate::sync::PacketEvent::make_id(&sender_did, envelope.sequence),
                        timestamp: chrono::Utc::now().timestamp_millis(), // Historical, use now
                        direction: crate::sync::PacketDirection::Incoming,
                        sequence: envelope.sequence,
                        author_did: sender_did,
                        author_name: contact_name.clone(),
                        relay_did: None,
                        relay_name: None,
                        destination_did: my_did_str.clone(),
                        destination_name: "Me".to_string(),
                        decryption_status,
                        content_preview,
                        is_delivered: false,
                        peer_did: contact_did.clone(),
                    };
                    self.packet_event_buffer.record(event);
                    loaded_count += 1;
                }
            }

            // Load sent packets from ProfileLog for this contact
            if let Some(ref log) = self.profile_log {
                for entry in log.entries_ordered() {
                    // Check if this packet is addressed to this contact
                    if let Some(payload) = self.decrypt_packet(&entry.envelope) {
                        if let PacketPayload::DirectMessage { ref recipient, content, .. } = payload {
                            if recipient == &did {
                                let event = crate::sync::PacketEvent {
                                    id: crate::sync::PacketEvent::make_id(&my_did_str, entry.envelope.sequence),
                                    timestamp: chrono::Utc::now().timestamp_millis(),
                                    direction: crate::sync::PacketDirection::Outgoing,
                                    sequence: entry.envelope.sequence,
                                    author_did: my_did_str.clone(),
                                    author_name: "Me".to_string(),
                                    relay_did: None,
                                    relay_name: None,
                                    destination_did: contact_did.clone(),
                                    destination_name: contact_name.clone(),
                                    decryption_status: crate::sync::DecryptionStatus::Decrypted,
                                    content_preview: crate::sync::PacketEvent::preview_content(&content),
                                    is_delivered: false,
                                    peer_did: contact_did.clone(),
                                };
                                self.packet_event_buffer.record(event);
                                loaded_count += 1;
                            }
                        }
                    }
                }
            }
        }

        info!(
            loaded_count,
            contact_count = contacts.len(),
            "Loaded historical packet events from storage"
        );

        Ok(loaded_count)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Chat API
    // ═══════════════════════════════════════════════════════════════════════

    /// Send a direct message to a contact.
    ///
    /// Creates a DirectMessage packet, encrypts it for the recipient,
    /// and broadcasts it via the 1:1 contact gossip topic.
    ///
    /// # Arguments
    ///
    /// * `contact_did` - The recipient's decentralized identifier
    /// * `content` - The message content to send
    ///
    /// # Returns
    ///
    /// The sequence number of the sent packet.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Profile keys not initialized
    /// - Contact not found or keys not exchanged
    /// - Network error during broadcast
    ///
    /// # Example
    ///
    /// ```ignore
    /// let seq = engine.send_message("did:sync:friend", "Hello!").await?;
    /// println!("Sent message with sequence {}", seq);
    /// ```
    pub async fn send_message(&mut self, contact_did: &str, content: &str) -> Result<u64, SyncError> {
        let did = Did::parse(contact_did)?;

        let payload = PacketPayload::DirectMessage {
            content: content.to_string(),
            recipient: did.clone(),
        };

        let address = PacketAddress::Individual(did);

        self.create_and_broadcast_packet(payload, address).await
    }

    /// Get conversation with a specific contact.
    ///
    /// Loads all messages exchanged with the contact (both sent and received)
    /// and returns them as a [`Conversation`].
    ///
    /// # Arguments
    ///
    /// * `contact_did` - The contact's decentralized identifier
    ///
    /// # Returns
    ///
    /// A [`Conversation`] containing all messages with this contact.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let convo = engine.get_conversation("did:sync:friend")?;
    /// println!("Messages with {}: {}", convo.display_name(), convo.len());
    /// for msg in convo.messages() {
    ///     println!("{}: {}", msg.display_sender(), msg.content);
    /// }
    /// ```
    pub fn get_conversation(&self, contact_did: &str) -> Result<crate::chat::Conversation, SyncError> {
        let did = Did::parse(contact_did)?;

        // Get contact info for display name
        let peer = self.get_peer_by_did(contact_did)?;
        let contact_name = crate::chat::get_contact_display_name(peer.as_ref());

        // Get our DID for identifying sent messages
        let my_did = self.profile_did()
            .ok_or_else(|| SyncError::Identity("Profile keys not initialized".to_string()))?;
        let my_did_str = my_did.as_str().to_string();

        // Load received packets from this contact
        // DIAGNOSTIC: Use info level to trace message retrieval
        info!(
            contact_did = %contact_did,
            did_parsed = %did.as_str(),
            did_bytes = ?did.as_str().as_bytes(),
            "get_conversation: loading packets from MirrorStore"
        );

        // Use mirror_packets_all() instead of mirror_packets_since(&did, 0)
        // because get_since(0) excludes sequence 0 (off-by-one bug fix)
        let received_packets = match self.mirror_packets_all(&did) {
            Ok(packets) => {
                info!(
                    contact_did = %contact_did,
                    did_parsed = %did.as_str(),
                    packet_count = packets.len(),
                    "get_conversation: loaded {} received packets from MirrorStore",
                    packets.len()
                );
                packets
            }
            Err(e) => {
                warn!(
                    contact_did = %contact_did,
                    did_parsed = %did.as_str(),
                    error = %e,
                    "get_conversation: FAILED to load received packets from MirrorStore"
                );
                vec![]
            }
        };

        // Load ALL sent packets first (can't borrow self inside closure due to borrow checker)
        let all_sent_packets: Vec<PacketEnvelope> = self
            .profile_log
            .as_ref()
            .map(|log| {
                log.entries_ordered()
                    .into_iter()
                    .map(|entry| entry.envelope.clone())
                    .collect()
            })
            .unwrap_or_default();

        // Filter to only DirectMessages addressed to this contact by checking payload recipient
        // NOTE: We can't use is_addressed_to() because global packets (sealed_keys empty)
        // return true for ALL DIDs. Instead, we decrypt and check the actual recipient field.
        let sent_packets: Vec<PacketEnvelope> = all_sent_packets
            .into_iter()
            .filter(|envelope| {
                if let Some(payload) = self.decrypt_packet(envelope) {
                    if let PacketPayload::DirectMessage { ref recipient, .. } = payload {
                        return recipient == &did;
                    }
                }
                false
            })
            .collect();

        // DIAGNOSTIC: Summary at info level for easy debugging
        info!(
            contact_did = %contact_did,
            total_in_profile_log = self.profile_log.as_ref().map(|l| l.len()).unwrap_or(0),
            sent_to_this_contact = sent_packets.len(),
            received_from_this_contact = received_packets.len(),
            "get_conversation: SUMMARY - sent={}, received={}",
            sent_packets.len(),
            received_packets.len()
        );

        // Build conversation using the helper
        let conversation = crate::chat::build_conversation(
            contact_did,
            contact_name,
            received_packets,
            sent_packets,
            &my_did_str,
            |envelope| self.decrypt_packet(envelope),
        );

        Ok(conversation)
    }

    /// List all conversations sorted by last activity.
    ///
    /// Returns conversations with all contacts who have exchanged messages,
    /// sorted with most recent activity first.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let conversations = engine.list_conversations()?;
    /// for convo in conversations {
    ///     if let Some(preview) = convo.preview(50) {
    ///         println!("{}: {} - \"{}\"",
    ///             convo.display_name(),
    ///             convo.last_message().map(|m| m.relative_time()).unwrap_or_default(),
    ///             preview
    ///         );
    ///     }
    /// }
    /// ```
    pub fn list_conversations(&self) -> Result<Vec<crate::chat::Conversation>, SyncError> {
        // Get all contacts
        let contacts = self.list_peer_contacts()?;

        let mut conversations = Vec::new();

        for contact in contacts {
            // Get contact DID from the peer's did field
            if let Some(ref did) = contact.did {
                if let Ok(convo) = self.get_conversation(did) {
                    // Only include conversations with messages
                    if !convo.is_empty() {
                        conversations.push(convo);
                    }
                }
            }
        }

        // Sort by last activity (most recent first)
        conversations.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));

        Ok(conversations)
    }

    /// Get new messages from a contact since a specific sequence.
    ///
    /// Useful for polling for updates in UI.
    ///
    /// # Arguments
    ///
    /// * `contact_did` - The contact's DID
    /// * `since_seq` - Return messages with sequence > this value
    ///
    /// # Returns
    ///
    /// Vector of new messages since the given sequence.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut last_seq = 0;
    /// loop {
    ///     let new_msgs = engine.get_new_messages("did:sync:friend", last_seq)?;
    ///     for msg in &new_msgs {
    ///         println!("New: {}", msg.content);
    ///         if msg.sequence > last_seq {
    ///             last_seq = msg.sequence;
    ///         }
    ///     }
    ///     tokio::time::sleep(Duration::from_secs(1)).await;
    /// }
    /// ```
    pub fn get_new_messages(
        &self,
        contact_did: &str,
        since_seq: u64,
    ) -> Result<Vec<crate::chat::ChatMessage>, SyncError> {
        let did = Did::parse(contact_did)?;

        // Get contact info for display name
        let peer = self.get_peer_by_did(contact_did)?;
        let contact_name = crate::chat::get_contact_display_name(peer.as_ref());

        // Get our DID
        let my_did = self.profile_did()
            .ok_or_else(|| SyncError::Identity("Profile keys not initialized".to_string()))?;
        let my_did_str = my_did.as_str().to_string();

        // Get packets since the sequence
        let packets = self.mirror_packets_since(&did, since_seq)?;

        // Convert to ChatMessages
        let messages: Vec<crate::chat::ChatMessage> = packets
            .iter()
            .filter_map(|envelope| {
                self.decrypt_packet(envelope).and_then(|payload| {
                    crate::chat::extract_chat_message(
                        envelope,
                        &payload,
                        &my_did_str,
                        contact_name.clone(),
                    )
                })
            })
            .collect();

        Ok(messages)
    }

    /// Get unread message count across all conversations.
    ///
    /// Returns the total number of unread messages (messages received
    /// after our last reply in each conversation).
    pub fn total_unread_count(&self) -> Result<usize, SyncError> {
        let conversations = self.list_conversations()?;
        Ok(conversations.iter().map(|c| c.unread_count()).sum())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Image Blob Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Upload an image and return its content hash
    ///
    /// The image data is stored content-addressed using BLAKE3.
    /// Returns the hash hex string that can be used as image_blob_id.
    pub async fn upload_image(&self, data: Vec<u8>) -> Result<String, SyncError> {
        let hash = self.blob_manager.import_image(data).await?;
        Ok(BlobManager::hash_to_blob_id(&hash))
    }

    /// Upload an avatar image with size validation (256 KB limit)
    ///
    /// Returns the content hash as a hex string blob ID.
    pub async fn upload_avatar(&self, data: Vec<u8>) -> Result<String, SyncError> {
        let hash = self.blob_manager.import_avatar(data).await?;
        Ok(BlobManager::hash_to_blob_id(&hash))
    }

    /// Load an image by its content hash
    pub async fn load_image(&self, hash_hex: &str) -> Result<Option<Vec<u8>>, SyncError> {
        let hash = BlobManager::blob_id_to_hash(hash_hex)?;
        let bytes = self.blob_manager.get_bytes(&hash).await?;
        Ok(bytes.map(|b| b.to_vec()))
    }

    /// Check if an image blob exists
    pub async fn image_exists(&self, hash_hex: &str) -> Result<bool, SyncError> {
        let hash = BlobManager::blob_id_to_hash(hash_hex)?;
        self.blob_manager.has_blob(&hash).await
    }

    /// Get the size of an image blob in bytes
    pub async fn image_size(&self, hash_hex: &str) -> Result<Option<u64>, SyncError> {
        let hash = BlobManager::blob_id_to_hash(hash_hex)?;
        self.blob_manager.blob_size(&hash).await
    }

    /// Delete an image blob
    ///
    /// Note: iroh-blobs uses garbage collection based on tags.
    /// Blobs without tags are eventually cleaned up.
    pub async fn delete_image(&self, hash_hex: &str) -> Result<(), SyncError> {
        let hash = BlobManager::blob_id_to_hash(hash_hex)?;
        self.blob_manager.delete_blob(&hash).await
    }

    /// Get a reference to the blob manager
    ///
    /// This can be used for advanced blob operations like P2P downloads.
    pub fn blob_manager(&self) -> &BlobManager {
        &self.blob_manager
    }

    /// Create a blob ticket for sharing an image via P2P
    ///
    /// Returns the base58-encoded ticket string.
    /// Requires gossip networking to be initialized.
    pub async fn create_image_ticket(&self, hash_hex: &str) -> Result<String, SyncError> {
        let hash = BlobManager::blob_id_to_hash(hash_hex)?;
        let gossip = self.ensure_gossip_ref()?;
        let ticket = self.blob_manager.create_ticket(hash, gossip.endpoint());
        Ok(ticket.to_string())
    }

    /// Download an image from a peer using a ticket
    ///
    /// Returns the content hash as a hex string blob ID.
    pub async fn download_image_from_ticket(&mut self, ticket_str: &str) -> Result<String, SyncError> {
        let gossip = self.ensure_gossip().await?;
        let hash = self.blob_manager.download_from_ticket_str(ticket_str, gossip.endpoint()).await?;
        Ok(BlobManager::hash_to_blob_id(&hash))
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

/// Network statistics for the Network page.
///
/// Provides summary counts for peers, pinners, and pins.
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    /// Total number of known peers
    pub total_peers: usize,
    /// Number of currently online peers
    pub online_peers: usize,
    /// Number of peers pinning our profile ("Souls Carrying Your Light")
    pub pinners_count: usize,
    /// Number of profiles we are pinning ("Souls You Carry")
    pub pinning_count: usize,
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
        // Private realm is auto-created
        assert_eq!(info.realm_count, 1);
    }

    #[tokio::test]
    async fn test_engine_create_realm_persists() {
        let (mut engine, _temp) = create_test_engine().await;

        // Create a realm
        let realm_id = engine.create_realm("Test Realm").await.unwrap();

        // Verify it's in storage (should have Private + Test Realm)
        let realms = engine.list_realms().await.unwrap();
        assert_eq!(realms.len(), 2);
        let test_realm = realms.iter().find(|r| r.name == "Test Realm").unwrap();
        assert_eq!(test_realm.id, realm_id);

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
            // Should have Private + Persistent
            assert_eq!(realms.len(), 2);
            let persistent_realm = realms
                .iter()
                .find(|r| r.name == "Persistent")
                .expect("Persistent realm should exist");

            // Need to open the realm to access tasks
            engine.open_realm(&persistent_realm.id).await.unwrap();
            let tasks = engine.list_tasks(&persistent_realm.id).unwrap();
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

        // Verify list_realms (Private + 3 created = 4)
        let realms = engine.list_realms().await.unwrap();
        assert_eq!(realms.len(), 4);

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
    // Profile Packet Tests (Indra's Network)
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_init_profile_keys_creates_new() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initially no profile keys
        assert!(!engine.has_profile_keys());
        assert!(engine.profile_did().is_none());

        // Identity must be initialized before profile keys
        engine.init_identity().unwrap();
        let identity_did = engine.did().unwrap();

        // Initialize profile keys (derived from identity)
        engine.init_profile_keys().unwrap();

        // Profile keys should now exist
        assert!(engine.has_profile_keys());
        let profile_did = engine.profile_did().unwrap();
        assert!(profile_did.as_str().starts_with("did:sync:z"));

        // Profile DID should match Identity DID (unified system)
        assert_eq!(profile_did, identity_did, "Profile DID should match Identity DID");
    }

    #[tokio::test]
    async fn test_profile_keys_persist_across_instances() {
        let temp_dir = TempDir::new().unwrap();

        // Create profile keys in first engine instance
        let original_did = {
            let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
            engine.init_identity().unwrap();
            engine.init_profile_keys().unwrap();
            engine.profile_did().unwrap().to_string()
        };

        // Load profile keys in new engine instance
        let mut engine2 = SyncEngine::new(temp_dir.path()).await.unwrap();

        // Before init, no keys in memory
        assert!(!engine2.has_profile_keys());

        // Init identity and profile keys (should load existing)
        engine2.init_identity().unwrap();
        engine2.init_profile_keys().unwrap();

        // Verify it's the same identity
        let loaded_did = engine2.profile_did().unwrap().to_string();
        assert_eq!(original_did, loaded_did);

        // Also verify profile DID matches identity DID
        assert_eq!(engine2.did().unwrap(), engine2.profile_did().unwrap());
    }

    #[tokio::test]
    async fn test_profile_log_initialized_with_keys() {
        let (mut engine, _temp) = create_test_engine().await;

        // Before profile keys init, log should be None
        assert!(engine.my_log().is_none());
        assert_eq!(engine.log_head_sequence(), 0);

        // Identity must be initialized before profile keys
        engine.init_identity().unwrap();

        // Initialize profile keys (derived from identity)
        engine.init_profile_keys().unwrap();

        // Log should now be available
        assert!(engine.my_log().is_some());
        assert_eq!(engine.log_head_sequence(), 0);
    }

    #[tokio::test]
    async fn test_create_global_packet() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity and profile keys
        engine.init_identity().unwrap();
        engine.init_profile_keys().unwrap();

        // Create a global (public) packet
        let payload = crate::profile::PacketPayload::Heartbeat {
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        let sequence = engine
            .create_packet(payload, crate::profile::PacketAddress::Global)
            .unwrap();

        // Should be sequence 0 (first packet)
        assert_eq!(sequence, 0);

        // Log should now have head sequence 0
        assert_eq!(engine.log_head_sequence(), 0);

        // Create another packet
        let payload2 = crate::profile::PacketPayload::Heartbeat {
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        let sequence2 = engine
            .create_packet(payload2, crate::profile::PacketAddress::Global)
            .unwrap();

        assert_eq!(sequence2, 1);
        assert_eq!(engine.log_head_sequence(), 1);
    }

    #[tokio::test]
    async fn test_create_packet_requires_profile_keys() {
        let (mut engine, _temp) = create_test_engine().await;

        // Don't initialize profile keys

        // Create packet should fail
        let payload = crate::profile::PacketPayload::Heartbeat {
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        let result = engine.create_packet(payload, crate::profile::PacketAddress::Global);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_incoming_packet() {
        let (mut engine, _temp) = create_test_engine().await;

        // Create a packet from another profile
        let other_keys = crate::profile::ProfileKeys::generate();
        let payload = crate::profile::PacketPayload::Heartbeat {
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        let envelope = crate::profile::PacketEnvelope::create_global(
            &other_keys,
            &payload,
            0,
            [0u8; 32],
        )
        .unwrap();

        // Handle the incoming packet
        let is_new = engine.handle_incoming_packet(envelope.clone()).unwrap();
        assert!(is_new);

        // Check mirror has the packet
        let head = engine.mirror_head(&other_keys.did());
        assert_eq!(head, Some(0));

        // Handle the same packet again (should be duplicate)
        let is_new2 = engine.handle_incoming_packet(envelope).unwrap();
        assert!(!is_new2);
    }

    #[tokio::test]
    async fn test_decrypt_global_packet() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity and profile keys
        engine.init_identity().unwrap();
        engine.init_profile_keys().unwrap();

        // Create a global packet from another profile
        let other_keys = crate::profile::ProfileKeys::generate();
        let my_did = engine.profile_did().unwrap();
        let payload = crate::profile::PacketPayload::DirectMessage {
            content: "Hello, world!".to_string(),
            recipient: my_did,
        };
        let envelope = crate::profile::PacketEnvelope::create_global(
            &other_keys,
            &payload,
            0,
            [0u8; 32],
        )
        .unwrap();

        // Global packets can be decrypted (really just decoded) by anyone
        let decrypted = engine.decrypt_packet(&envelope);
        assert!(decrypted.is_some());

        match decrypted.unwrap() {
            crate::profile::PacketPayload::DirectMessage { content, recipient: _ } => {
                assert_eq!(content, "Hello, world!");
            }
            _ => panic!("Wrong payload type"),
        }
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
        use crate::identity::HybridKeypair;
        use crate::types::{PinRelationship, SignedProfile, UserProfile};

        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity
        engine.init_identity().unwrap();

        // Create realm
        let realm_id = engine.create_realm("Incoming Test").await.unwrap();

        // Get the realm key to create a valid envelope
        let realm_key = engine.storage.load_realm_key(&realm_id).unwrap().unwrap();

        // Create a peer keypair and pin their profile (simulating a known contact)
        let peer_keypair = HybridKeypair::generate();
        let peer_profile = UserProfile::new("peer_sender".to_string(), "Peer Sender".to_string());
        let signed_profile = SignedProfile::sign(&peer_profile, &peer_keypair);
        let sender_did = signed_profile.did().to_string();

        // Pin the peer's profile so they're recognized
        engine
            .pin_profile(signed_profile, PinRelationship::Contact)
            .unwrap();

        // Create a valid envelope from the peer
        let message = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads: vec![vec![1, 2, 3]],
            sender_addr: None,
        };

        let sign_fn = |data: &[u8]| peer_keypair.sign(data).to_bytes();

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
    /// 1. Love creates a realm and generates an invite
    /// 2. Joy joins via the invite
    /// 3. Love adds a task
    /// 4. Joy should see Love's task (after sync propagates)
    #[tokio::test]
    async fn test_two_engines_sync_tasks_via_invite() {
        use crate::types::{PinRelationship, SignedProfile, UserProfile};
        use std::time::Duration;

        let _ = tracing_subscriber::fmt::try_init();

        // Create Love's engine
        let temp_dir_love = TempDir::new().unwrap();
        let mut love = SyncEngine::new(temp_dir_love.path()).await.unwrap();
        love.init_identity().unwrap(); // Required for signing sync messages

        // Create Joy's engine
        let temp_dir_joy = TempDir::new().unwrap();
        let mut joy = SyncEngine::new(temp_dir_joy.path()).await.unwrap();
        joy.init_identity().unwrap(); // Required for signing sync messages

        // CRITICAL: Exchange and pin profiles so signature verification works.
        // In production, this happens via contact exchange. For tests, we do it manually.
        {
            // Create signed profiles for both engines
            let love_keypair = love.identity.as_ref().unwrap();
            let love_profile = UserProfile::new("love_peer".to_string(), "Love".to_string());
            let love_signed = SignedProfile::sign(&love_profile, love_keypair);

            let joy_keypair = joy.identity.as_ref().unwrap();
            let joy_profile = UserProfile::new("joy_peer".to_string(), "Joy".to_string());
            let joy_signed = SignedProfile::sign(&joy_profile, joy_keypair);

            // Pin each other's profiles (Love knows Joy, Joy knows Love)
            love
                .pin_profile(joy_signed.clone(), PinRelationship::Contact)
                .unwrap();
            joy.pin_profile(love_signed.clone(), PinRelationship::Contact)
                .unwrap();
            debug!("Exchanged and pinned profiles between Love and Joy");
        }

        // CRITICAL: Start networking on BOTH engines and exchange addresses BEFORE
        // subscribing to any gossip topics. This matches the pattern used in the
        // working p2p_integration tests. The iroh-gossip layer seems to require
        // peer addresses to be in the static discovery BEFORE topic subscription
        // for message delivery to work properly.
        love.start_networking().await.unwrap();
        joy.start_networking().await.unwrap();

        // Exchange peer addresses bidirectionally
        if let (Some(love_addr), Some(joy_addr)) = (love.endpoint_addr(), joy.endpoint_addr()) {
            debug!("Adding bidirectional peer addresses before gossip subscription");
            love.add_peer_addr(joy_addr);
            joy.add_peer_addr(love_addr);
        }

        // Small delay to let discovery propagate
        tokio::time::sleep(Duration::from_millis(50)).await;

        // CRITICAL: Subscribe to events BEFORE any sync operations start.
        // This avoids the race condition where PeerConnected events fire
        // before we start listening for them.
        let mut love_events = love.subscribe_events();
        let mut joy_events = joy.subscribe_events();

        // Love creates a realm
        let realm_id = love.create_realm("Shared Tasks").await.unwrap();

        // Love generates an invite (this should auto-start sync!)
        let invite_str = love.create_invite(&realm_id).await.unwrap();
        debug!(
            "Love generated invite: {}...",
            &invite_str[..50.min(invite_str.len())]
        );

        // Verify Love is now syncing
        assert!(
            love.is_realm_syncing(&realm_id),
            "Love should be syncing after generating invite"
        );

        // Joy joins via invite
        let joined_realm_id = joy.join_realm(&invite_str).await.unwrap();
        assert_eq!(joined_realm_id, realm_id, "Joy should join the same realm");

        // Verify Joy is syncing
        assert!(
            joy.is_realm_syncing(&realm_id),
            "Joy should be syncing after joining"
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
        let love_connected =
            wait_for_peer_connected(&mut love_events, &realm_id, Duration::from_secs(10)).await;
        let joy_connected =
            wait_for_peer_connected(&mut joy_events, &realm_id, Duration::from_secs(10)).await;

        debug!(love_connected, joy_connected, "Peer connection status");

        // At least one side should see a connection (typically Joy sees Love first since he used Love as bootstrap)
        assert!(
            love_connected || joy_connected,
            "At least one peer should have connected within 10 seconds"
        );

        // Give time for gossip mesh to stabilize after peer connection.
        // The iroh-gossip layer needs time to establish reliable message routing.
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Process any pending BroadcastRequests from NeighborUp events
        love.process_pending_sync();
        joy.process_pending_sync();

        // Additional stabilization - let any triggered broadcasts complete
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Love adds a task (this should broadcast via sync)
        debug!("Love adding task...");
        let _task_id = love.add_task(&realm_id, "Love's task").await.unwrap();
        debug!("Love added task, waiting for sync to Joy...");

        // Wait for sync to propagate to Joy (up to 5 seconds)
        let mut synced = false;
        for i in 0..50 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            // Process any pending sync messages
            let processed = joy.process_pending_sync();
            if processed > 0 {
                debug!(
                    "Joy processed {} sync messages at iteration {}",
                    processed, i
                );
            }
            let joy_tasks = joy.list_tasks(&realm_id).unwrap();
            debug!(
                iteration = i,
                processed,
                task_count = joy_tasks.len(),
                "Checking Joy's tasks"
            );
            if !joy_tasks.is_empty() {
                debug!("Joy received task after {}ms", (i + 1) * 100);
                assert_eq!(joy_tasks[0].title, "Love's task");
                synced = true;
                break;
            }
        }

        assert!(
            synced,
            "Joy should have received Love's task within 5 seconds"
        );

        // Cleanup
        love.shutdown().await.unwrap();
        joy.shutdown().await.unwrap();
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
        use crate::identity::HybridKeypair;
        use crate::invite::NodeAddrBytes;
        use crate::sync::{SyncEnvelope, SyncMessage};
        use crate::types::{PinRelationship, SignedProfile, UserProfile};

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

        // Create a joiner's keypair and pin their profile (required for signature verification)
        let joiner_keypair = HybridKeypair::generate();
        let joiner_profile = UserProfile::new("joiner_peer".to_string(), "Joiner".to_string());
        let joiner_signed = SignedProfile::sign(&joiner_profile, &joiner_keypair);
        let joiner_did = joiner_signed.did().to_string();
        engine
            .pin_profile(joiner_signed.clone(), PinRelationship::Contact)
            .unwrap();

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

        // Create signed envelope from the joiner
        let envelope_bytes = {
            let sign_fn = |data: &[u8]| joiner_keypair.sign(data).to_bytes().to_vec();
            let envelope = SyncEnvelope::seal(&announce, &joiner_did, &realm_key, sign_fn)
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
            let sign_fn = |data: &[u8]| joiner_keypair.sign(data).to_bytes().to_vec();
            let envelope = SyncEnvelope::seal(&announce, &joiner_did, &realm_key, sign_fn)
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

        // Create Love's engine
        let temp_dir_love = TempDir::new().unwrap();
        let mut love = SyncEngine::new(temp_dir_love.path()).await.unwrap();
        love.init_identity().unwrap();

        // Create Joy's engine
        let temp_dir_joy = TempDir::new().unwrap();
        let mut joy = SyncEngine::new(temp_dir_joy.path()).await.unwrap();
        joy.init_identity().unwrap();

        // Start networking on both
        love.start_networking().await.unwrap();
        joy.start_networking().await.unwrap();

        // Exchange peer addresses bidirectionally
        if let (Some(love_addr), Some(joy_addr)) = (love.endpoint_addr(), joy.endpoint_addr()) {
            love.add_peer_addr(joy_addr);
            joy.add_peer_addr(love_addr);
        }

        // Small delay to let discovery propagate
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Subscribe to events before sync starts
        let mut love_events = love.subscribe_events();
        let mut joy_events = joy.subscribe_events();

        // Love creates a realm and generates an invite
        let realm_id = love.create_realm("Peer Count Test").await.unwrap();
        let invite_str = love.create_invite(&realm_id).await.unwrap();

        // Verify Love is syncing (but peer_count should be 0 initially)
        let love_status_initial = love.sync_status(&realm_id);
        info!(?love_status_initial, "Love initial sync status");
        assert!(
            matches!(love_status_initial, SyncStatus::Syncing { .. }),
            "Love should be in Syncing state"
        );

        // Joy joins via invite
        let _joined_realm_id = joy.join_realm(&invite_str).await.unwrap();

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
        let love_connected = wait_for_peer(&mut love_events, &realm_id).await;
        let joy_connected = wait_for_peer(&mut joy_events, &realm_id).await;

        debug!(
            love_connected,
            joy_connected, "Peer connection events received"
        );

        assert!(
            love_connected || joy_connected,
            "At least one peer should have connected"
        );

        // Give a moment for status to settle
        tokio::time::sleep(Duration::from_millis(200)).await;

        // THE CRITICAL ASSERTION: After peers connect, sync_status should report peer_count >= 1
        let love_status = love.sync_status(&realm_id);
        let joy_status = joy.sync_status(&realm_id);

        info!(
            ?love_status,
            ?joy_status,
            "Final sync status after peer connection"
        );

        // Check Love's peer count
        match love_status {
            SyncStatus::Syncing { peer_count } => {
                assert!(
                    peer_count >= 1,
                    "Love should report at least 1 peer, but got peer_count={}",
                    peer_count
                );
            }
            other => panic!("Love should be Syncing, but got {:?}", other),
        }

        // Check Joy's peer count
        match joy_status {
            SyncStatus::Syncing { peer_count } => {
                assert!(
                    peer_count >= 1,
                    "Joy should report at least 1 peer, but got peer_count={}",
                    peer_count
                );
            }
            other => panic!("Joy should be Syncing, but got {:?}", other),
        }

        // Cleanup
        love.shutdown().await.unwrap();
        joy.shutdown().await.unwrap();
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
        use crate::types::{PinRelationship, SignedProfile, UserProfile};
        use std::time::Duration;

        let _ = tracing_subscriber::fmt::try_init();

        // Use persistent directories
        let temp_dir_love = TempDir::new().unwrap();
        let temp_dir_joy = TempDir::new().unwrap();
        let love_path = temp_dir_love.path().to_path_buf();
        let joy_path = temp_dir_joy.path().to_path_buf();

        // === Phase 1: Initial sync setup ===
        let realm_id = {
            let mut love = SyncEngine::new(&love_path).await.unwrap();
            love.init_identity().unwrap();

            let mut joy = SyncEngine::new(&joy_path).await.unwrap();
            joy.init_identity().unwrap();

            // CRITICAL: Exchange and pin profiles so signature verification works across restarts
            {
                let love_keypair = love.identity.as_ref().unwrap();
                let love_profile = UserProfile::new("love_offline".to_string(), "Love".to_string());
                let love_signed = SignedProfile::sign(&love_profile, love_keypair);

                let joy_keypair = joy.identity.as_ref().unwrap();
                let joy_profile = UserProfile::new("joy_offline".to_string(), "Joy".to_string());
                let joy_signed = SignedProfile::sign(&joy_profile, joy_keypair);

                love
                    .pin_profile(joy_signed.clone(), PinRelationship::Contact)
                    .unwrap();
                joy.pin_profile(love_signed.clone(), PinRelationship::Contact)
                    .unwrap();
                debug!("Exchanged and pinned profiles between Love and Joy");
            }

            // Start networking
            love.start_networking().await.unwrap();
            joy.start_networking().await.unwrap();

            // Exchange addresses
            if let (Some(love_addr), Some(joy_addr)) = (love.endpoint_addr(), joy.endpoint_addr())
            {
                love.add_peer_addr(joy_addr);
                joy.add_peer_addr(love_addr);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Subscribe to events
            let mut love_events = love.subscribe_events();

            // Create realm and invite
            let realm_id = love.create_realm("Offline Sync Test").await.unwrap();
            let invite_str = love.create_invite(&realm_id).await.unwrap();

            // Joy joins
            let _joined = joy.join_realm(&invite_str).await.unwrap();

            // Wait for connection
            let connected = tokio::time::timeout(Duration::from_secs(10), async {
                loop {
                    match love_events.recv().await {
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

            // Love adds initial task and waits for sync
            love
                .add_task(&realm_id, "Initial shared task")
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(500)).await;
            joy.process_pending_sync();

            let joy_tasks = joy.list_tasks(&realm_id).unwrap();
            assert!(!joy_tasks.is_empty(), "Joy should have initial task");

            // Shutdown both
            love.shutdown().await.unwrap();
            joy.shutdown().await.unwrap();

            realm_id
        };

        info!(?realm_id, "Phase 1 complete - both engines shutdown");

        // === Phase 2: Offline changes ===
        // Love adds tasks while offline (no networking)
        {
            let mut love = SyncEngine::new(&love_path).await.unwrap();
            love.init_identity().unwrap();
            love.open_realm(&realm_id).await.unwrap();

            love
                .add_task(&realm_id, "Love offline task 1")
                .await
                .unwrap();
            love
                .add_task(&realm_id, "Love offline task 2")
                .await
                .unwrap();

            let love_tasks = love.list_tasks(&realm_id).unwrap();
            info!(count = love_tasks.len(), "Love offline tasks");

            // Shutdown without starting networking
            love.shutdown().await.unwrap();
        }

        // Joy adds tasks while offline
        {
            let mut joy = SyncEngine::new(&joy_path).await.unwrap();
            joy.init_identity().unwrap();
            joy.open_realm(&realm_id).await.unwrap();

            joy.add_task(&realm_id, "Joy offline task 1").await.unwrap();
            joy.add_task(&realm_id, "Joy offline task 2").await.unwrap();

            let joy_tasks = joy.list_tasks(&realm_id).unwrap();
            info!(count = joy_tasks.len(), "Joy offline tasks");

            joy.shutdown().await.unwrap();
        }

        info!("Phase 2 complete - both added offline tasks");

        // Allow database handles to fully release
        tokio::time::sleep(Duration::from_millis(100)).await;

        // === Phase 3: Restart and sync ===
        let mut love = SyncEngine::new(&love_path).await.unwrap();
        love.init_identity().unwrap();

        let mut joy = SyncEngine::new(&joy_path).await.unwrap();
        joy.init_identity().unwrap();

        // Start networking
        love.start_networking().await.unwrap();
        joy.start_networking().await.unwrap();

        // Exchange addresses AND update bootstrap_peers in storage
        // This simulates what happens when users re-exchange invites after restart
        // (The iroh endpoint has a new node ID after restart, so old saved addresses are stale)
        if let (Some(love_addr), Some(joy_addr)) = (love.endpoint_addr(), joy.endpoint_addr()) {
            love.add_peer_addr(joy_addr.clone());
            joy.add_peer_addr(love_addr.clone());

            // Update Love's realm with Joy's fresh address as bootstrap peer
            if let Ok(Some(mut love_realm_info)) = love.storage.load_realm(&realm_id) {
                love_realm_info.bootstrap_peers =
                    vec![NodeAddrBytes::from_endpoint_addr(&joy_addr)];
                love.storage.save_realm(&love_realm_info).unwrap();
            }

            // Update Joy's realm with Love's fresh address as bootstrap peer
            if let Ok(Some(mut joy_realm_info)) = joy.storage.load_realm(&realm_id) {
                joy_realm_info.bootstrap_peers =
                    vec![NodeAddrBytes::from_endpoint_addr(&love_addr)];
                joy.storage.save_realm(&joy_realm_info).unwrap();
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Open realms (should auto-start sync for shared realms)
        love.open_realm(&realm_id).await.unwrap();
        joy.open_realm(&realm_id).await.unwrap();

        // Wait for sync to complete
        let mut synced = false;
        for i in 0..100 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            love.process_pending_sync();
            joy.process_pending_sync();

            let love_tasks = love.list_tasks(&realm_id).unwrap();
            let joy_tasks = joy.list_tasks(&realm_id).unwrap();

            debug!(
                iteration = i,
                love_count = love_tasks.len(),
                joy_count = joy_tasks.len(),
                "Checking sync progress"
            );

            // Both should have 5 tasks:
            // 1. Initial shared task
            // 2. Love offline task 1
            // 3. Love offline task 2
            // 4. Joy offline task 1
            // 5. Joy offline task 2
            if love_tasks.len() >= 5 && joy_tasks.len() >= 5 {
                synced = true;
                break;
            }
        }

        let love_tasks = love.list_tasks(&realm_id).unwrap();
        let joy_tasks = joy.list_tasks(&realm_id).unwrap();

        info!(
            love_count = love_tasks.len(),
            joy_count = joy_tasks.len(),
            "Final task counts"
        );

        // Log actual task titles for debugging
        for (i, task) in love_tasks.iter().enumerate() {
            debug!(i, title = %task.title, "Love task");
        }
        for (i, task) in joy_tasks.iter().enumerate() {
            debug!(i, title = %task.title, "Joy task");
        }

        assert!(
            synced,
            "Both should have all 5 tasks. Love has {}, Joy has {}",
            love_tasks.len(),
            joy_tasks.len()
        );

        // Verify specific tasks exist on both
        let love_titles: std::collections::HashSet<_> =
            love_tasks.iter().map(|t| t.title.as_str()).collect();
        let joy_titles: std::collections::HashSet<_> =
            joy_tasks.iter().map(|t| t.title.as_str()).collect();

        assert!(
            love_titles.contains("Love offline task 1"),
            "Love should have her offline task 1"
        );
        assert!(
            love_titles.contains("Joy offline task 1"),
            "Love should have Joy's offline task 1"
        );
        assert!(
            joy_titles.contains("Love offline task 1"),
            "Joy should have Love's offline task 1"
        );
        assert!(
            joy_titles.contains("Joy offline task 1"),
            "Joy should have his offline task 1"
        );

        // Cleanup
        love.shutdown().await.unwrap();
        joy.shutdown().await.unwrap();
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Private Realm Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_private_realm_auto_created() {
        let (mut engine, _temp) = create_test_engine().await;
        engine.init_identity().unwrap();

        // Private realm should exist after engine creation
        let realms = engine.list_realms().await.unwrap();
        assert_eq!(realms.len(), 1, "Private realm should auto-create");
        assert_eq!(realms[0].name, "Private");
        assert!(!realms[0].is_shared, "Private realm should not be shared");
    }

    #[tokio::test]
    async fn test_private_realm_has_onboarding_tasks() {
        let (mut engine, _temp) = create_test_engine().await;
        engine.init_identity().unwrap();

        // Find Private realm
        let realms = engine.list_realms().await.unwrap();
        let private_realm = realms.iter().find(|r| r.name == "Private").unwrap();

        // Open and check tasks
        engine.open_realm(&private_realm.id).await.unwrap();
        let tasks = engine.list_tasks(&private_realm.id).unwrap();

        // Should have sacred onboarding tasks
        assert!(
            tasks.len() >= 3,
            "Private realm should have onboarding tasks"
        );

        // Verify sacred language in task titles
        let titles: Vec<&str> = tasks.iter().map(|t| t.title.as_str()).collect();
        let has_sacred_language = titles.iter().any(|t| {
            let lower = t.to_lowercase();
            lower.contains("field")
                || lower.contains("intention")
                || lower.contains("manifest")
                || lower.contains("synchronicities")
        });
        assert!(
            has_sacred_language,
            "Onboarding tasks should use sacred language"
        );
    }

    #[tokio::test]
    async fn test_cannot_share_private_realm() {
        let (mut engine, _temp) = create_test_engine().await;
        engine.init_identity().unwrap();

        // Find Private realm
        let realms = engine.list_realms().await.unwrap();
        let private_realm = realms.iter().find(|r| r.name == "Private").unwrap();

        // Attempt to generate invite should fail
        let result = engine.generate_invite(&private_realm.id).await;
        assert!(
            result.is_err(),
            "Should not be able to generate invite for Private realm"
        );

        // Shutdown to avoid gossip background tasks
        engine.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_cannot_create_realm_named_private() {
        let (mut engine, _temp) = create_test_engine().await;
        engine.init_identity().unwrap();

        // Attempt to create another "Private" realm should fail
        let result = engine.create_realm("Private").await;
        assert!(
            result.is_err(),
            "Should not be able to create realm named 'Private'"
        );

        // Case variations should also fail
        let result = engine.create_realm("private").await;
        assert!(
            result.is_err(),
            "Should not be able to create realm named 'private'"
        );

        let result = engine.create_realm("PRIVATE").await;
        assert!(
            result.is_err(),
            "Should not be able to create realm named 'PRIVATE'"
        );
    }

    #[tokio::test]
    async fn test_cannot_delete_private_realm() {
        let (mut engine, _temp) = create_test_engine().await;
        engine.init_identity().unwrap();

        // Find Private realm
        let realms = engine.list_realms().await.unwrap();
        let private_realm = realms.iter().find(|r| r.name == "Private").unwrap();

        // Attempt to delete should fail
        let result = engine.delete_realm(&private_realm.id).await;
        assert!(
            result.is_err(),
            "Should not be able to delete Private realm"
        );

        // Verify it still exists
        let realms = engine.list_realms().await.unwrap();
        assert!(
            realms.iter().any(|r| r.name == "Private"),
            "Private realm should still exist after delete attempt"
        );
    }

    #[tokio::test]
    async fn test_private_realm_persists_across_restarts() {
        let temp_dir = TempDir::new().unwrap();

        // First engine instance
        {
            let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
            engine.init_identity().unwrap();

            let realms = engine.list_realms().await.unwrap();
            assert_eq!(realms.len(), 1);
            assert_eq!(realms[0].name, "Private");
        }

        // Second engine instance (simulating restart)
        {
            let engine = SyncEngine::new(temp_dir.path()).await.unwrap();
            let realms = engine.list_realms().await.unwrap();
            assert_eq!(
                realms.len(),
                1,
                "Private realm should not be duplicated on restart"
            );
            assert_eq!(realms[0].name, "Private");
        }
    }

    #[tokio::test]
    async fn test_private_realm_independent_from_identity() {
        let (mut engine, _temp) = create_test_engine().await;

        // Private realm should exist even before identity init
        let realms = engine.list_realms().await.unwrap();
        assert_eq!(realms.len(), 1);
        assert_eq!(realms[0].name, "Private");

        // Identity init should not affect Private realm
        engine.init_identity().unwrap();
        let realms = engine.list_realms().await.unwrap();
        assert_eq!(realms.len(), 1);
        assert_eq!(realms[0].name, "Private");
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Profile Pinning Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_sign_and_pin_own_profile() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity first (required for get_own_profile and signing)
        engine.init_identity().unwrap();
        let did = engine.did().unwrap().to_string();

        // Set up a profile - use the DID as peer_id
        let mut profile = crate::types::UserProfile::new(
            did.clone(),
            "Test User".to_string(),
        );
        profile.subtitle = Some("Test tagline".to_string());
        engine.save_profile(&profile).unwrap();

        // Sign and pin the profile
        let signed = engine.sign_and_pin_own_profile().unwrap();

        // Verify the signed profile
        assert!(signed.verify());
        assert_eq!(signed.profile.display_name, "Test User");

        // Should be able to retrieve our own pinned profile
        let retrieved = engine.get_own_pinned_profile().unwrap();
        assert!(retrieved.is_some());
        let pin = retrieved.unwrap();
        assert!(pin.is_own());
        assert_eq!(pin.signed_profile.profile.display_name, "Test User");
    }

    #[tokio::test]
    async fn test_pin_peer_profile() {
        let (engine, _temp) = create_test_engine().await;

        // Create a signed profile from a "remote peer"
        let keypair = crate::identity::HybridKeypair::generate();
        let profile = crate::types::UserProfile::new(
            "peer_remote".to_string(),
            "Remote User".to_string(),
        );
        let signed = crate::types::SignedProfile::sign(&profile, &keypair);
        let did = signed.did().to_string();

        // Pin as a contact
        let evicted = engine
            .pin_profile(signed.clone(), crate::types::PinRelationship::Contact)
            .unwrap();
        assert!(evicted.is_empty());

        // Should be retrievable
        let retrieved = engine.get_pinned_profile(&did).unwrap();
        assert!(retrieved.is_some());
        let pin = retrieved.unwrap();
        assert_eq!(pin.relationship, crate::types::PinRelationship::Contact);
        assert_eq!(pin.signed_profile.profile.display_name, "Remote User");
    }

    #[tokio::test]
    async fn test_pin_profile_rejects_invalid_signature() {
        let (engine, _temp) = create_test_engine().await;

        // Create a signed profile and tamper with it
        let keypair = crate::identity::HybridKeypair::generate();
        let profile = crate::types::UserProfile::new(
            "peer_tampered".to_string(),
            "Tampered User".to_string(),
        );
        let mut signed = crate::types::SignedProfile::sign(&profile, &keypair);

        // Tamper with the profile
        signed.profile.display_name = "HACKED".to_string();

        // Should reject the invalid signature
        let result = engine.pin_profile(signed, crate::types::PinRelationship::Manual);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SyncError::SignatureInvalid(_)));
    }

    #[tokio::test]
    async fn test_unpin_profile() {
        let (engine, _temp) = create_test_engine().await;

        // Pin a profile
        let keypair = crate::identity::HybridKeypair::generate();
        let profile = crate::types::UserProfile::new(
            "peer_unpin".to_string(),
            "Unpin User".to_string(),
        );
        let signed = crate::types::SignedProfile::sign(&profile, &keypair);
        let did = signed.did().to_string();

        engine
            .pin_profile(signed, crate::types::PinRelationship::Manual)
            .unwrap();

        // Verify it's pinned
        assert!(engine.get_pinned_profile(&did).unwrap().is_some());

        // Unpin
        engine.unpin_profile(&did).unwrap();

        // Should no longer be pinned
        assert!(engine.get_pinned_profile(&did).unwrap().is_none());
    }

    #[tokio::test]
    async fn test_cannot_unpin_own_profile() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity
        engine.init_identity().unwrap();
        let did = engine.did().unwrap().to_string();

        // Set up and pin our own profile
        let profile = crate::types::UserProfile::new(
            did.clone(),
            "Own User".to_string(),
        );
        engine.save_profile(&profile).unwrap();
        let signed = engine.sign_and_pin_own_profile().unwrap();
        let signed_did = signed.did().to_string();

        // Trying to unpin our own profile should fail
        let result = engine.unpin_profile(&signed_did);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SyncError::InvalidOperation(_)));

        // Should still be pinned
        assert!(engine.get_own_pinned_profile().unwrap().is_some());
    }

    #[tokio::test]
    async fn test_list_pinned_profiles_by_relationship() {
        let (engine, _temp) = create_test_engine().await;

        // Pin some profiles with different relationships
        for (suffix, relationship) in [
            ("contact1", crate::types::PinRelationship::Contact),
            ("contact2", crate::types::PinRelationship::Contact),
            ("manual1", crate::types::PinRelationship::Manual),
        ] {
            let keypair = crate::identity::HybridKeypair::generate();
            let profile = crate::types::UserProfile::new(
                format!("peer_{}", suffix),
                format!("User {}", suffix),
            );
            let signed = crate::types::SignedProfile::sign(&profile, &keypair);
            engine.pin_profile(signed, relationship).unwrap();
        }

        // List by contact relationship
        let contacts = engine
            .list_pinned_profiles_by_relationship(&crate::types::PinRelationship::Contact)
            .unwrap();
        assert_eq!(contacts.len(), 2);

        // List by manual relationship
        let manual = engine
            .list_pinned_profiles_by_relationship(&crate::types::PinRelationship::Manual)
            .unwrap();
        assert_eq!(manual.len(), 1);
    }

    #[tokio::test]
    async fn test_update_pinned_profile() {
        let (engine, _temp) = create_test_engine().await;

        // Pin a profile
        let keypair = crate::identity::HybridKeypair::generate();
        let profile = crate::types::UserProfile::new(
            "peer_update".to_string(),
            "Original Name".to_string(),
        );
        let signed = crate::types::SignedProfile::sign(&profile, &keypair);
        let did = signed.did().to_string();

        engine
            .pin_profile(signed, crate::types::PinRelationship::Contact)
            .unwrap();

        // Create an updated profile (same keypair, different data)
        let updated_profile = crate::types::UserProfile::new(
            "peer_update".to_string(),
            "Updated Name".to_string(),
        );
        let updated_signed = crate::types::SignedProfile::sign(&updated_profile, &keypair);

        // Update the pin
        let updated = engine.update_pinned_profile(&did, updated_signed).unwrap();
        assert!(updated);

        // Verify the update
        let retrieved = engine.get_pinned_profile(&did).unwrap().unwrap();
        assert_eq!(retrieved.signed_profile.profile.display_name, "Updated Name");
        // Relationship should be preserved
        assert_eq!(retrieved.relationship, crate::types::PinRelationship::Contact);
    }

    #[tokio::test]
    async fn test_pinned_profile_count() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initially should be 0 (own profile doesn't count)
        assert_eq!(engine.pinned_profile_count().unwrap(), 0);

        // Initialize identity and add our own profile (shouldn't count)
        engine.init_identity().unwrap();
        let did = engine.did().unwrap().to_string();
        let profile = crate::types::UserProfile::new(did.clone(), "Self".to_string());
        engine.save_profile(&profile).unwrap();
        engine.sign_and_pin_own_profile().unwrap();

        // Still 0 (own profile excluded)
        assert_eq!(engine.pinned_profile_count().unwrap(), 0);

        // Add peer profiles
        for i in 0..3 {
            let keypair = crate::identity::HybridKeypair::generate();
            let profile = crate::types::UserProfile::new(
                format!("peer_{}", i),
                format!("User {}", i),
            );
            let signed = crate::types::SignedProfile::sign(&profile, &keypair);
            engine
                .pin_profile(signed, crate::types::PinRelationship::Manual)
                .unwrap();
        }

        // Should now be 3
        assert_eq!(engine.pinned_profile_count().unwrap(), 3);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Auto-Pinning Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_process_profile_announcement_for_contact() {
        let (engine, _temp) = create_test_engine().await;

        // Create a "contact's" keypair and signed profile
        let contact_keypair = crate::identity::HybridKeypair::generate();
        let contact_profile = crate::types::UserProfile::new(
            "peer_contact".to_string(),
            "Contact User".to_string(),
        );
        let signed = crate::types::SignedProfile::sign(&contact_profile, &contact_keypair);
        let contact_did = signed.did().to_string();

        // Create a fake ContactInfo and save it to storage to simulate an accepted contact
        let contact_info = crate::types::contact::ContactInfo {
            peer_did: contact_did.clone(),
            peer_endpoint_id: [0u8; 32],
            profile: crate::types::contact::ProfileSnapshot {
                display_name: "Contact User".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: crate::invite::NodeAddrBytes {
                node_id: [0u8; 32],
                relay_url: None,
                direct_addresses: vec![],
            },
            contact_topic: [0u8; 32],
            contact_key: [0u8; 32],
            accepted_at: chrono::Utc::now().timestamp(),
            last_seen: 0,
            status: crate::types::contact::ContactStatus::Offline,
            is_favorite: false,
            encryption_keys: None,
        };
        engine.storage.save_contact(&contact_info).unwrap();

        // Verify we should auto-pin this contact
        assert!(engine.should_auto_pin(&contact_did));
        assert_eq!(
            engine.determine_pin_relationship(&contact_did),
            Some(crate::types::PinRelationship::Contact)
        );

        // Process a profile announcement
        let announcement = crate::sync::ProfileGossipMessage::announce(signed.clone(), None);
        let action = engine.process_profile_announcement(&announcement).unwrap();

        // Should have auto-pinned
        match action {
            crate::sync::ProfileAction::UpdatePin { signed_profile, .. } => {
                assert_eq!(signed_profile.profile.display_name, "Contact User");
            }
            _ => panic!("Expected UpdatePin action"),
        }

        // Verify the pin was created
        let pin = engine.get_pinned_profile(&contact_did).unwrap();
        assert!(pin.is_some());
        let pin = pin.unwrap();
        assert_eq!(pin.relationship, crate::types::PinRelationship::Contact);
    }

    #[tokio::test]
    async fn test_process_profile_announcement_ignores_unknown_peer() {
        let (engine, _temp) = create_test_engine().await;

        // Create an unknown peer's profile
        let unknown_keypair = crate::identity::HybridKeypair::generate();
        let unknown_profile = crate::types::UserProfile::new(
            "peer_unknown".to_string(),
            "Unknown User".to_string(),
        );
        let signed = crate::types::SignedProfile::sign(&unknown_profile, &unknown_keypair);
        let unknown_did = signed.did().to_string();

        // Verify we should NOT auto-pin this peer
        assert!(!engine.should_auto_pin(&unknown_did));
        assert_eq!(engine.determine_pin_relationship(&unknown_did), None);

        // Process a profile announcement
        let announcement = crate::sync::ProfileGossipMessage::announce(signed, None);
        let action = engine.process_profile_announcement(&announcement).unwrap();

        // Should have been ignored
        assert!(matches!(action, crate::sync::ProfileAction::Ignore));

        // Verify no pin was created
        assert!(engine.get_pinned_profile(&unknown_did).unwrap().is_none());
    }

    #[tokio::test]
    async fn test_process_profile_announcement_updates_existing_pin() {
        let (engine, _temp) = create_test_engine().await;

        // Create a contact's keypair
        let contact_keypair = crate::identity::HybridKeypair::generate();
        let contact_did = crate::identity::Did::from_public_key(
            &contact_keypair.public_key()
        ).to_string();

        // Save as a contact
        let contact_info = crate::types::contact::ContactInfo {
            peer_did: contact_did.clone(),
            peer_endpoint_id: [0u8; 32],
            profile: crate::types::contact::ProfileSnapshot {
                display_name: "Original Name".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: crate::invite::NodeAddrBytes {
                node_id: [0u8; 32],
                relay_url: None,
                direct_addresses: vec![],
            },
            contact_topic: [0u8; 32],
            contact_key: [0u8; 32],
            accepted_at: chrono::Utc::now().timestamp(),
            last_seen: 0,
            status: crate::types::contact::ContactStatus::Offline,
            is_favorite: false,
            encryption_keys: None,
        };
        engine.storage.save_contact(&contact_info).unwrap();

        // Create and process initial announcement
        let initial_profile = crate::types::UserProfile::new(
            contact_did.clone(),
            "Original Name".to_string(),
        );
        let initial_signed = crate::types::SignedProfile::sign(&initial_profile, &contact_keypair);
        let announcement = crate::sync::ProfileGossipMessage::announce(initial_signed, None);
        engine.process_profile_announcement(&announcement).unwrap();

        // Verify initial pin
        let pin = engine.get_pinned_profile(&contact_did).unwrap().unwrap();
        assert_eq!(pin.signed_profile.profile.display_name, "Original Name");

        // Create and process updated announcement
        let updated_profile = crate::types::UserProfile::new(
            contact_did.clone(),
            "Updated Name".to_string(),
        );
        let updated_signed = crate::types::SignedProfile::sign(&updated_profile, &contact_keypair);
        let announcement = crate::sync::ProfileGossipMessage::announce(updated_signed, None);
        engine.process_profile_announcement(&announcement).unwrap();

        // Verify pin was updated
        let pin = engine.get_pinned_profile(&contact_did).unwrap().unwrap();
        assert_eq!(pin.signed_profile.profile.display_name, "Updated Name");
        // Relationship should be preserved
        assert_eq!(pin.relationship, crate::types::PinRelationship::Contact);
    }

    #[tokio::test]
    async fn test_get_auto_pin_interests() {
        let (engine, _temp) = create_test_engine().await;

        // Initially no interests
        let interests = engine.get_auto_pin_interests().unwrap();
        assert!(interests.is_empty());

        // Add some contacts
        for i in 0..3 {
            let contact_info = crate::types::contact::ContactInfo {
                peer_did: format!("did:sync:contact{}", i),
                peer_endpoint_id: [i as u8; 32],
                profile: crate::types::contact::ProfileSnapshot {
                    display_name: format!("Contact {}", i),
                    subtitle: None,
                    avatar_blob_id: None,
                    bio: String::new(),
                },
                node_addr: crate::invite::NodeAddrBytes {
                    node_id: [i as u8; 32],
                    relay_url: None,
                    direct_addresses: vec![],
                },
                contact_topic: [i as u8; 32],
                contact_key: [i as u8; 32],
                accepted_at: chrono::Utc::now().timestamp(),
                last_seen: 0,
                status: crate::types::contact::ContactStatus::Offline,
                is_favorite: false,
                encryption_keys: None,
            };
            engine.storage.save_contact(&contact_info).unwrap();
        }

        // Should now have 3 interests
        let interests = engine.get_auto_pin_interests().unwrap();
        assert_eq!(interests.len(), 3);
        assert!(interests.contains(&"did:sync:contact0".to_string()));
        assert!(interests.contains(&"did:sync:contact1".to_string()));
        assert!(interests.contains(&"did:sync:contact2".to_string()));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Startup Sync Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_startup_sync_with_no_peers() {
        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity for the engine
        engine.init_identity().unwrap();

        // Startup sync with no peers should complete immediately after jitter
        let result = engine.startup_sync().await.unwrap();

        // No peers to connect to
        assert_eq!(result.peers_attempted, 0);
        assert_eq!(result.peers_succeeded, 0);
        assert_eq!(result.peers_skipped_backoff, 0);
        // Jitter should be in range 0-30000 ms
        assert!(result.jitter_delay_ms < 30_000);
    }

    #[tokio::test]
    async fn test_startup_sync_result_default() {
        let result = StartupSyncResult::default();
        assert_eq!(result.peers_attempted, 0);
        assert_eq!(result.peers_succeeded, 0);
        assert_eq!(result.peers_skipped_backoff, 0);
        assert_eq!(result.profiles_updated, 0);
        assert_eq!(result.jitter_delay_ms, 0);
    }

    #[tokio::test]
    async fn test_startup_sync_respects_backoff() {
        let (mut engine, _temp) = create_test_engine().await;
        engine.init_identity().unwrap();

        // Add a peer with many failed attempts (should be in backoff)
        let mut peer = crate::types::peer::Peer::new(
            iroh::SecretKey::generate(&mut rand::rng()).public(),
            crate::types::peer::PeerSource::FromInvite,
        );
        // Simulate many failures
        peer.connection_attempts = 5;
        peer.successful_connections = 0;
        // Set last_attempt to now (still in backoff period)
        peer.last_attempt = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        engine.storage.save_peer(&peer).unwrap();

        // Run startup sync
        let result = engine.startup_sync().await.unwrap();

        // Peer should be skipped due to backoff
        assert_eq!(result.peers_attempted, 0);
        assert_eq!(result.peers_skipped_backoff, 1);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Chat/Messaging Integration Tests
    // ═══════════════════════════════════════════════════════════════════════

    /// Test that packets created with create_packet appear in get_conversation().
    ///
    /// This is the core regression test for the "sent messages don't appear in sender's chat" bug.
    /// It tests the fundamental flow WITHOUT network dependencies:
    ///
    /// 1. create_packet() → stores DirectMessage packet in profile_log
    /// 2. get_conversation() → loads sent packets from profile_log, filters by recipient
    /// 3. The sent message should appear with is_mine=true
    ///
    /// If this test FAILS: the bug is in the core library (packet storage or conversation loading)
    /// If this test PASSES: the bug is in either:
    ///   - The UI layer (Dioxus state management), OR
    ///   - Network-related code paths (contact manager, broadcast)
    ///
    /// Note: This test sets up a contact with encryption keys to enable E2E encryption.
    #[tokio::test]
    async fn test_created_packet_appears_in_conversation() {
        use crate::profile::{PacketAddress, PacketPayload, ProfileKeys};
        use crate::types::contact::{ContactInfo, ContactStatus, ProfileSnapshot};
        use crate::invite::NodeAddrBytes;

        let (mut engine, _temp) = create_test_engine().await;

        // Initialize identity and profile keys (required for create_packet)
        engine.init_identity().unwrap();
        engine.init_profile_keys().unwrap();

        // Generate ProfileKeys for the contact to enable E2E encryption
        let contact_profile_keys = ProfileKeys::generate();
        let contact_pubkeys = contact_profile_keys.public_bundle();
        let contact_did_str = contact_pubkeys.did().to_string();

        // Set up the contact with encryption keys
        let contact = ContactInfo {
            peer_did: contact_did_str.clone(),
            peer_endpoint_id: [0u8; 32],
            profile: ProfileSnapshot {
                display_name: "Test Contact".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: NodeAddrBytes::new([0u8; 32]),
            contact_topic: [1u8; 32],
            contact_key: [2u8; 32],
            accepted_at: 0,
            last_seen: 0,
            status: ContactStatus::Offline,
            is_favorite: false,
            encryption_keys: Some(contact_pubkeys.to_bytes()),
        };
        engine.storage.save_contact(&contact).unwrap();

        let contact_did = Did::parse(&contact_did_str).unwrap();

        // Create an E2E encrypted DirectMessage payload
        let content = "Hello from integration test!";
        let payload = PacketPayload::DirectMessage {
            content: content.to_string(),
            recipient: contact_did.clone(),
        };
        let address = PacketAddress::Individual(contact_did);

        // Create the packet (stores it in our log with E2E encryption)
        let seq_result = engine.create_packet(payload, address);
        assert!(
            seq_result.is_ok(),
            "create_packet should succeed: {:?}",
            seq_result.err()
        );
        let seq = seq_result.unwrap();
        println!("Created E2E encrypted packet with sequence: {}", seq);

        // Verify profile_log has the packet
        let log_count = engine
            .profile_log
            .as_ref()
            .map(|log| log.entries_ordered().len())
            .unwrap_or(0);
        println!("profile_log has {} entries", log_count);
        assert!(
            log_count > 0,
            "profile_log should have at least 1 entry after create_packet"
        );

        // Get the conversation - THIS IS THE KEY TEST
        let conversation_result = engine.get_conversation(&contact_did_str);
        assert!(
            conversation_result.is_ok(),
            "get_conversation should succeed: {:?}",
            conversation_result.err()
        );

        let conversation = conversation_result.unwrap();
        println!("Conversation has {} messages", conversation.len());

        // THE KEY ASSERTION: sent message should appear
        assert_eq!(
            conversation.len(),
            1,
            "Conversation should have exactly 1 message (the created one)"
        );

        // Verify message properties
        let msg = conversation
            .messages()
            .first()
            .expect("Should have a message");
        assert_eq!(msg.content, content, "Message content should match");
        assert!(msg.is_mine, "Message should be marked as is_mine=true");
        println!(
            "SUCCESS: Message content='{}', is_mine={}",
            msg.content, msg.is_mine
        );
    }

    /// Test that multiple messages to the same contact all appear in conversation.
    ///
    /// This catches potential issues with sequence handling or duplicate filtering.
    /// Note: This test sets up a contact with encryption keys to enable E2E encryption.
    #[tokio::test]
    async fn test_multiple_messages_appear_in_conversation() {
        use crate::profile::{PacketAddress, PacketPayload, ProfileKeys};
        use crate::types::contact::{ContactInfo, ContactStatus, ProfileSnapshot};
        use crate::invite::NodeAddrBytes;

        let (mut engine, _temp) = create_test_engine().await;
        engine.init_identity().unwrap();
        engine.init_profile_keys().unwrap();

        // Generate ProfileKeys for the contact to enable E2E encryption
        let contact_profile_keys = ProfileKeys::generate();
        let contact_pubkeys = contact_profile_keys.public_bundle();
        let contact_did_str = contact_pubkeys.did().to_string();

        // Set up the contact with encryption keys
        let contact = ContactInfo {
            peer_did: contact_did_str.clone(),
            peer_endpoint_id: [0u8; 32],
            profile: ProfileSnapshot {
                display_name: "Test Contact".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: NodeAddrBytes::new([0u8; 32]),
            contact_topic: [1u8; 32],
            contact_key: [2u8; 32],
            accepted_at: 0,
            last_seen: 0,
            status: ContactStatus::Offline,
            is_favorite: false,
            encryption_keys: Some(contact_pubkeys.to_bytes()),
        };
        engine.storage.save_contact(&contact).unwrap();

        let contact_did = Did::parse(&contact_did_str).unwrap();

        // Send multiple E2E encrypted messages
        let messages = ["First message", "Second message", "Third message"];
        for msg in &messages {
            let payload = PacketPayload::DirectMessage {
                content: msg.to_string(),
                recipient: contact_did.clone(),
            };
            let address = PacketAddress::Individual(contact_did.clone());
            engine.create_packet(payload, address).unwrap();
        }

        // Verify all messages appear
        let conversation = engine.get_conversation(&contact_did_str).unwrap();
        assert_eq!(
            conversation.len(),
            3,
            "Conversation should have all 3 messages"
        );

        // Verify each message content (they should be sorted by timestamp/sequence)
        let conv_messages = conversation.messages();
        for (i, msg) in conv_messages.iter().enumerate() {
            assert_eq!(msg.content, messages[i], "Message {} content should match", i);
            assert!(msg.is_mine, "Message {} should be marked as is_mine=true", i);
        }
    }

    /// Test that messages to different contacts are properly separated.
    ///
    /// This ensures the recipient filtering in get_conversation works correctly.
    /// Note: This test sets up contacts with encryption keys to enable E2E encryption.
    #[tokio::test]
    async fn test_messages_separated_by_contact() {
        use crate::profile::{PacketAddress, PacketPayload, ProfileKeys};
        use crate::types::contact::{ContactInfo, ContactStatus, ProfileSnapshot};
        use crate::invite::NodeAddrBytes;

        let (mut engine, _temp) = create_test_engine().await;
        engine.init_identity().unwrap();
        engine.init_profile_keys().unwrap();

        // Generate profile keys for contacts to enable E2E encryption
        let contact1_keys = ProfileKeys::generate();
        let contact1_pubkeys = contact1_keys.public_bundle();
        let contact1_did = contact1_pubkeys.did().to_string();

        let contact2_keys = ProfileKeys::generate();
        let contact2_pubkeys = contact2_keys.public_bundle();
        let contact2_did = contact2_pubkeys.did().to_string();

        // Create contacts with encryption keys
        let contact1 = ContactInfo {
            peer_did: contact1_did.clone(),
            peer_endpoint_id: [1u8; 32],
            profile: ProfileSnapshot {
                display_name: "Contact 1".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: NodeAddrBytes::new([1u8; 32]),
            contact_topic: [10u8; 32],
            contact_key: [11u8; 32],
            accepted_at: 0,
            last_seen: 0,
            status: ContactStatus::Offline,
            is_favorite: false,
            encryption_keys: Some(contact1_pubkeys.to_bytes()),
        };
        engine.storage.save_contact(&contact1).unwrap();

        let contact2 = ContactInfo {
            peer_did: contact2_did.clone(),
            peer_endpoint_id: [2u8; 32],
            profile: ProfileSnapshot {
                display_name: "Contact 2".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: NodeAddrBytes::new([2u8; 32]),
            contact_topic: [20u8; 32],
            contact_key: [21u8; 32],
            accepted_at: 0,
            last_seen: 0,
            status: ContactStatus::Offline,
            is_favorite: false,
            encryption_keys: Some(contact2_pubkeys.to_bytes()),
        };
        engine.storage.save_contact(&contact2).unwrap();

        let did1 = Did::parse(&contact1_did).unwrap();
        let did2 = Did::parse(&contact2_did).unwrap();

        // Send E2E encrypted message to contact 1
        let payload1 = PacketPayload::DirectMessage {
            content: "Hello Contact 1".to_string(),
            recipient: did1.clone(),
        };
        engine.create_packet(payload1, PacketAddress::Individual(did1)).unwrap();

        // Send E2E encrypted message to contact 2
        let payload2 = PacketPayload::DirectMessage {
            content: "Hello Contact 2".to_string(),
            recipient: did2.clone(),
        };
        engine.create_packet(payload2, PacketAddress::Individual(did2)).unwrap();

        // Verify contact 1's conversation only has their message
        let convo1 = engine.get_conversation(&contact1_did).unwrap();
        assert_eq!(convo1.len(), 1, "Contact 1 should have exactly 1 message");
        assert_eq!(convo1.messages()[0].content, "Hello Contact 1");

        // Verify contact 2's conversation only has their message
        let convo2 = engine.get_conversation(&contact2_did).unwrap();
        assert_eq!(convo2.len(), 1, "Contact 2 should have exactly 1 message");
        assert_eq!(convo2.messages()[0].content, "Hello Contact 2");
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Signature Verification Tests (make_verify_fn)
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_verify_fn_known_sender_valid_signature() {
        use crate::identity::HybridKeypair;
        use crate::types::{PinRelationship, SignedProfile, UserProfile};

        let (engine, _temp) = create_test_engine().await;

        // Create a contact's keypair and profile
        let contact_keypair = HybridKeypair::generate();
        let contact_profile = UserProfile::new("peer_love".to_string(), "Love".to_string());
        let signed_profile = SignedProfile::sign(&contact_profile, &contact_keypair);
        let contact_did = signed_profile.did().to_string();

        // Pin the profile so they're a known contact
        engine
            .pin_profile(signed_profile.clone(), PinRelationship::Contact)
            .unwrap();

        // Sign some test data
        let test_data = b"Hello, world!";
        let signature = contact_keypair.sign(test_data);
        let sig_bytes = signature.to_bytes();

        // Create verify function and test
        let verify_fn = SyncEngine::make_verify_fn(&engine.storage);
        assert!(
            verify_fn(&contact_did, test_data, &sig_bytes),
            "Valid signature from known sender should verify"
        );
    }

    #[tokio::test]
    async fn test_verify_fn_unknown_sender_returns_false() {
        use crate::identity::HybridKeypair;
        use crate::types::{SignedProfile, UserProfile};

        let (engine, _temp) = create_test_engine().await;

        // Create a stranger's keypair (NOT pinned)
        let stranger_keypair = HybridKeypair::generate();
        let stranger_profile = UserProfile::new("peer_stranger".to_string(), "Stranger".to_string());
        let signed_profile = SignedProfile::sign(&stranger_profile, &stranger_keypair);
        let stranger_did = signed_profile.did().to_string();

        // Sign some test data
        let test_data = b"Hello from stranger";
        let signature = stranger_keypair.sign(test_data);
        let sig_bytes = signature.to_bytes();

        // Create verify function and test - should fail because sender is unknown
        let verify_fn = SyncEngine::make_verify_fn(&engine.storage);
        assert!(
            !verify_fn(&stranger_did, test_data, &sig_bytes),
            "Signature from unknown sender should be rejected"
        );
    }

    #[tokio::test]
    async fn test_verify_fn_wrong_signature_returns_false() {
        use crate::identity::HybridKeypair;
        use crate::types::{PinRelationship, SignedProfile, UserProfile};

        let (engine, _temp) = create_test_engine().await;

        // Create a contact's keypair and profile
        let contact_keypair = HybridKeypair::generate();
        let contact_profile = UserProfile::new("peer_joy".to_string(), "Joy".to_string());
        let signed_profile = SignedProfile::sign(&contact_profile, &contact_keypair);
        let contact_did = signed_profile.did().to_string();

        // Pin the profile
        engine
            .pin_profile(signed_profile.clone(), PinRelationship::Contact)
            .unwrap();

        // Create a different keypair and sign with it (attacker trying to impersonate)
        let attacker_keypair = HybridKeypair::generate();
        let test_data = b"Forged message from attacker";
        let forged_signature = attacker_keypair.sign(test_data);
        let sig_bytes = forged_signature.to_bytes();

        // Create verify function and test - should fail because signature doesn't match
        let verify_fn = SyncEngine::make_verify_fn(&engine.storage);
        assert!(
            !verify_fn(&contact_did, test_data, &sig_bytes),
            "Forged signature should be rejected"
        );
    }

    #[tokio::test]
    async fn test_verify_fn_tampered_message_returns_false() {
        use crate::identity::HybridKeypair;
        use crate::types::{PinRelationship, SignedProfile, UserProfile};

        let (engine, _temp) = create_test_engine().await;

        // Create a contact's keypair and profile
        let contact_keypair = HybridKeypair::generate();
        let contact_profile = UserProfile::new("peer_peace".to_string(), "Peace".to_string());
        let signed_profile = SignedProfile::sign(&contact_profile, &contact_keypair);
        let contact_did = signed_profile.did().to_string();

        // Pin the profile
        engine
            .pin_profile(signed_profile.clone(), PinRelationship::Contact)
            .unwrap();

        // Sign original data
        let original_data = b"Original message";
        let signature = contact_keypair.sign(original_data);
        let sig_bytes = signature.to_bytes();

        // Try to verify with tampered data
        let tampered_data = b"Tampered message";

        // Create verify function and test - should fail because data was tampered
        let verify_fn = SyncEngine::make_verify_fn(&engine.storage);
        assert!(
            !verify_fn(&contact_did, tampered_data, &sig_bytes),
            "Signature should fail for tampered message"
        );
    }

    #[tokio::test]
    async fn test_verify_fn_malformed_signature_returns_false() {
        use crate::identity::HybridKeypair;
        use crate::types::{PinRelationship, SignedProfile, UserProfile};

        let (engine, _temp) = create_test_engine().await;

        // Create a contact's keypair and profile
        let contact_keypair = HybridKeypair::generate();
        let contact_profile = UserProfile::new("peer_dave".to_string(), "Dave".to_string());
        let signed_profile = SignedProfile::sign(&contact_profile, &contact_keypair);
        let contact_did = signed_profile.did().to_string();

        // Pin the profile
        engine
            .pin_profile(signed_profile.clone(), PinRelationship::Contact)
            .unwrap();

        let test_data = b"Test message";

        // Test with various malformed signatures
        let verify_fn = SyncEngine::make_verify_fn(&engine.storage);

        // Empty signature
        assert!(
            !verify_fn(&contact_did, test_data, &[]),
            "Empty signature should be rejected"
        );

        // Signature too short (less than 68 bytes needed for Ed25519 + length)
        assert!(
            !verify_fn(&contact_did, test_data, &[0u8; 50]),
            "Too-short signature should be rejected"
        );

        // Random garbage bytes (invalid signature format)
        let garbage: Vec<u8> = (0..400).map(|i| (i % 256) as u8).collect();
        assert!(
            !verify_fn(&contact_did, test_data, &garbage),
            "Garbage signature should be rejected"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════
    // E2E Encryption Key Lookup Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_get_recipient_public_keys_no_contact() {
        let (engine, _temp) = create_test_engine().await;

        let unknown_did = Did::parse("did:sync:zUnknownContact123").unwrap();
        let result = engine.get_recipient_public_keys(&unknown_did);

        assert!(result.is_err(), "Should fail for unknown contact");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("No contact found"),
            "Error should indicate no contact: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_get_recipient_public_keys_legacy_contact_no_keys() {
        use crate::types::contact::{ContactInfo, ContactStatus, ProfileSnapshot};
        use crate::invite::NodeAddrBytes;

        let (engine, _temp) = create_test_engine().await;

        // Create a legacy contact without encryption keys
        let peer_did = "did:sync:zLegacyContact456";
        let legacy_contact = ContactInfo {
            peer_did: peer_did.to_string(),
            peer_endpoint_id: [0u8; 32],
            profile: ProfileSnapshot {
                display_name: "Legacy Contact".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: NodeAddrBytes::new([0u8; 32]),
            contact_topic: [1u8; 32],
            contact_key: [2u8; 32],
            accepted_at: 0,
            last_seen: 0,
            status: ContactStatus::Offline,
            is_favorite: false,
            encryption_keys: None, // Legacy contact - no encryption keys
        };
        engine.storage.save_contact(&legacy_contact).unwrap();

        let did = Did::parse(peer_did).unwrap();
        let result = engine.get_recipient_public_keys(&did);

        assert!(result.is_err(), "Should fail for legacy contact without keys");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("does not have encryption keys"),
            "Error should indicate missing encryption keys: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_get_recipient_public_keys_with_valid_keys() {
        use crate::types::contact::{ContactInfo, ContactStatus, ProfileSnapshot};
        use crate::invite::NodeAddrBytes;
        use crate::profile::ProfileKeys;

        let (engine, _temp) = create_test_engine().await;

        // Generate real ProfileKeys to get valid encryption keys
        let contact_profile_keys = ProfileKeys::generate();
        let public_keys = contact_profile_keys.public_bundle();
        let enc_keys_bytes = public_keys.to_bytes();

        // Create a contact with encryption keys
        let peer_did = public_keys.did().to_string();
        let contact = ContactInfo {
            peer_did: peer_did.clone(),
            peer_endpoint_id: [0u8; 32],
            profile: ProfileSnapshot {
                display_name: "Encrypted Contact".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: NodeAddrBytes::new([0u8; 32]),
            contact_topic: [1u8; 32],
            contact_key: [2u8; 32],
            accepted_at: 0,
            last_seen: 0,
            status: ContactStatus::Offline,
            is_favorite: false,
            encryption_keys: Some(enc_keys_bytes),
        };
        engine.storage.save_contact(&contact).unwrap();

        let did = Did::parse(&peer_did).unwrap();
        let result = engine.get_recipient_public_keys(&did);

        assert!(result.is_ok(), "Should succeed for contact with valid keys");
        let retrieved_keys = result.unwrap();

        // Verify the retrieved keys match the original
        assert_eq!(
            retrieved_keys.did().to_string(),
            public_keys.did().to_string(),
            "Retrieved keys should have matching DID"
        );
    }

    #[tokio::test]
    async fn test_get_recipient_public_keys_malformed_keys() {
        use crate::types::contact::{ContactInfo, ContactStatus, ProfileSnapshot};
        use crate::invite::NodeAddrBytes;

        let (engine, _temp) = create_test_engine().await;

        // Create a contact with malformed encryption keys
        // Note: DID identifier must be valid base58 (excludes 0, O, I, lowercase l)
        let peer_did = "did:sync:zBadKeyDataContact789";
        let contact = ContactInfo {
            peer_did: peer_did.to_string(),
            peer_endpoint_id: [0u8; 32],
            profile: ProfileSnapshot {
                display_name: "Malformed Contact".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: NodeAddrBytes::new([0u8; 32]),
            contact_topic: [1u8; 32],
            contact_key: [2u8; 32],
            accepted_at: 0,
            last_seen: 0,
            status: ContactStatus::Offline,
            is_favorite: false,
            encryption_keys: Some(vec![0xDE, 0xAD, 0xBE, 0xEF]), // Invalid key data
        };
        engine.storage.save_contact(&contact).unwrap();

        let did = Did::parse(peer_did).unwrap();
        let result = engine.get_recipient_public_keys(&did);

        assert!(result.is_err(), "Should fail for malformed keys");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to parse encryption keys"),
            "Error should indicate parsing failure: {}",
            err_msg
        );
    }
}
