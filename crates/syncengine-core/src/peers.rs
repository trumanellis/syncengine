//! Peer registry for tracking discovered peers and enabling auto-reconnection
//!
//! This module maintains a local database of all peers ever encountered through
//! realm invites and gossip topics. The registry enables:
//! - Automatic peer discovery through realm membership
//! - Periodic reconnection to previously-seen peers
//! - Offline tracking of peer metadata (nicknames, last seen)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  Peer Discovery Flow                                            │
//! │  1. User joins realm via invite                                 │
//! │  2. GossipEvent::NeighborUp → record peer                       │
//! │  3. Peer stored in redb with metadata                           │
//! │  4. Background task periodically tries to reconnect             │
//! │  5. Connection status updated based on results                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use crate::error::SyncError;
use crate::types::RealmId;
use iroh::PublicKey;
use parking_lot::RwLock;
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

// Table definition for peer registry
const PEERS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("peers");

/// How a peer was discovered
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerSource {
    /// Discovered through a specific realm's gossip topic
    FromRealm(RealmId),
    /// Added through an invite (not yet seen on gossip)
    FromInvite,
}

/// Connection status of a peer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PeerStatus {
    /// Currently connected (seen recently)
    Online,
    /// Not currently connected
    Offline,
    /// Status unknown (never attempted connection)
    #[default]
    Unknown,
}

impl std::fmt::Display for PeerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerStatus::Online => write!(f, "online"),
            PeerStatus::Offline => write!(f, "offline"),
            PeerStatus::Unknown => write!(f, "unknown"),
        }
    }
}

/// Information about a discovered peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// The peer's public key (iroh endpoint ID)
    pub endpoint_id: [u8; 32],
    /// Optional nickname for this peer
    pub nickname: Option<String>,
    /// When we last saw this peer (Unix timestamp)
    pub last_seen: u64,
    /// How we discovered this peer
    pub source: PeerSource,
    /// Current connection status
    pub status: PeerStatus,
    /// List of realm IDs we share with this peer
    pub shared_realms: Vec<RealmId>,
}

impl PeerInfo {
    /// Create a new PeerInfo
    pub fn new(endpoint_id: PublicKey, source: PeerSource) -> Self {
        Self {
            endpoint_id: *endpoint_id.as_bytes(),
            nickname: None,
            last_seen: Self::current_timestamp(),
            source,
            status: PeerStatus::Unknown,
            shared_realms: Vec::new(),
        }
    }

    /// Get current Unix timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Update the last_seen timestamp to now
    pub fn touch(&mut self) {
        self.last_seen = Self::current_timestamp();
    }

    /// Set the peer's nickname
    pub fn with_nickname(mut self, nickname: impl Into<String>) -> Self {
        self.nickname = Some(nickname.into());
        self
    }

    /// Set the peer's status
    pub fn with_status(mut self, status: PeerStatus) -> Self {
        self.status = status;
        self
    }

    /// Add a realm to the shared_realms list if not already present
    pub fn add_realm(&mut self, realm_id: RealmId) {
        if !self.shared_realms.contains(&realm_id) {
            self.shared_realms.push(realm_id);
        }
    }

    /// Get the endpoint ID as a PublicKey
    pub fn public_key(&self) -> PublicKey {
        PublicKey::from_bytes(&self.endpoint_id).expect("stored endpoint_id should always be valid")
    }
}

/// Peer registry for managing discovered peers
#[derive(Clone)]
pub struct PeerRegistry {
    db: Arc<RwLock<Database>>,
}

impl PeerRegistry {
    /// Create a new peer registry using the same database as Storage
    ///
    /// This reuses the existing database connection to avoid having multiple
    /// database instances pointing at the same file.
    pub fn new(db: Arc<RwLock<Database>>) -> Result<Self, SyncError> {
        // Initialize the peers table if it doesn't exist
        {
            let database = db.read();
            let write_txn = database.begin_write()?;
            {
                let _ = write_txn.open_table(PEERS_TABLE)?;
            }
            write_txn.commit()?;
        }

        Ok(Self { db })
    }

    /// Add or update a peer in the registry
    ///
    /// If the peer already exists, updates the last_seen timestamp and status.
    /// If adding a realm, it will be added to the shared_realms list.
    pub fn add_or_update(&self, peer_info: &PeerInfo) -> Result<(), SyncError> {
        let db = self.db.read();
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(PEERS_TABLE)?;

            // Serialize the peer info
            let data = postcard::to_allocvec(peer_info)
                .map_err(|e| SyncError::Serialization(e.to_string()))?;

            // Use endpoint_id as key
            table.insert(&peer_info.endpoint_id[..], data.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Get a peer's info by endpoint ID
    pub fn get(&self, endpoint_id: &PublicKey) -> Result<Option<PeerInfo>, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(PEERS_TABLE)?;

        let key = endpoint_id.as_bytes();
        match table.get(&key[..])? {
            Some(v) => {
                let peer_info: PeerInfo = postcard::from_bytes(v.value())
                    .map_err(|e| SyncError::Serialization(e.to_string()))?;
                Ok(Some(peer_info))
            }
            None => Ok(None),
        }
    }

    /// List all peers in the registry
    pub fn list_all(&self) -> Result<Vec<PeerInfo>, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(PEERS_TABLE)?;

        let mut peers = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let peer_info: PeerInfo = postcard::from_bytes(value.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            peers.push(peer_info);
        }
        Ok(peers)
    }

    /// List all peers with a specific status
    pub fn list_by_status(&self, status: PeerStatus) -> Result<Vec<PeerInfo>, SyncError> {
        let all_peers = self.list_all()?;
        Ok(all_peers
            .into_iter()
            .filter(|p| p.status == status)
            .collect())
    }

    /// List all peers that are currently offline (for reconnection attempts)
    pub fn list_inactive(&self) -> Result<Vec<PeerInfo>, SyncError> {
        let all_peers = self.list_all()?;
        Ok(all_peers
            .into_iter()
            .filter(|p| matches!(p.status, PeerStatus::Offline | PeerStatus::Unknown))
            .collect())
    }

    /// Update a peer's status
    pub fn update_status(
        &self,
        endpoint_id: &PublicKey,
        status: PeerStatus,
    ) -> Result<(), SyncError> {
        if let Some(mut peer_info) = self.get(endpoint_id)? {
            peer_info.status = status;
            peer_info.touch(); // Update last_seen timestamp
            self.add_or_update(&peer_info)?;
        }
        Ok(())
    }

    /// Update a peer's nickname
    pub fn update_nickname(
        &self,
        endpoint_id: &PublicKey,
        nickname: impl Into<String>,
    ) -> Result<(), SyncError> {
        if let Some(mut peer_info) = self.get(endpoint_id)? {
            peer_info.nickname = Some(nickname.into());
            self.add_or_update(&peer_info)?;
        }
        Ok(())
    }

    /// Add a realm to a peer's shared_realms list
    pub fn add_peer_realm(
        &self,
        endpoint_id: &PublicKey,
        realm_id: &RealmId,
    ) -> Result<(), SyncError> {
        if let Some(mut peer_info) = self.get(endpoint_id)? {
            peer_info.add_realm(realm_id.clone());
            peer_info.touch();
            self.add_or_update(&peer_info)?;
        }
        Ok(())
    }

    /// Count total peers in registry
    pub fn count(&self) -> Result<usize, SyncError> {
        Ok(self.list_all()?.len())
    }

    /// Count peers by status
    pub fn count_by_status(&self, status: PeerStatus) -> Result<usize, SyncError> {
        Ok(self.list_by_status(status)?.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iroh::SecretKey;
    use tempfile::TempDir;

    fn create_test_registry() -> (PeerRegistry, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.redb");
        let db = Database::create(&db_path).unwrap();
        let registry = PeerRegistry::new(Arc::new(RwLock::new(db))).unwrap();
        (registry, temp_dir)
    }

    fn create_test_public_key() -> PublicKey {
        SecretKey::generate(&mut rand::rng()).public()
    }

    #[test]
    fn test_peer_info_creation() {
        let endpoint_id = create_test_public_key();
        let realm_id = RealmId::new();

        let peer = PeerInfo::new(endpoint_id, PeerSource::FromRealm(realm_id));

        assert_eq!(peer.endpoint_id, *endpoint_id.as_bytes());
        assert_eq!(peer.status, PeerStatus::Unknown);
        assert!(peer.nickname.is_none());
        assert!(peer.last_seen > 0);
    }

    #[test]
    fn test_peer_info_with_nickname() {
        let endpoint_id = create_test_public_key();
        let peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite)
            .with_nickname("Alice")
            .with_status(PeerStatus::Online);

        assert_eq!(peer.nickname, Some("Alice".to_string()));
        assert_eq!(peer.status, PeerStatus::Online);
    }

    #[test]
    fn test_peer_info_add_realm() {
        let endpoint_id = create_test_public_key();
        let mut peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        let realm1 = RealmId::new();
        let realm2 = RealmId::new();

        peer.add_realm(realm1.clone());
        assert_eq!(peer.shared_realms.len(), 1);

        // Adding same realm again should not duplicate
        peer.add_realm(realm1.clone());
        assert_eq!(peer.shared_realms.len(), 1);

        peer.add_realm(realm2);
        assert_eq!(peer.shared_realms.len(), 2);
    }

    #[test]
    fn test_add_and_get_peer() {
        let (registry, _temp) = create_test_registry();

        let endpoint_id = create_test_public_key();
        let realm_id = RealmId::new();
        let peer = PeerInfo::new(endpoint_id, PeerSource::FromRealm(realm_id));

        // Add peer
        registry.add_or_update(&peer).unwrap();

        // Retrieve peer
        let retrieved = registry.get(&endpoint_id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.endpoint_id, *endpoint_id.as_bytes());
        assert_eq!(retrieved.status, PeerStatus::Unknown);
    }

    #[test]
    fn test_get_nonexistent_peer() {
        let (registry, _temp) = create_test_registry();

        let endpoint_id = create_test_public_key();
        let result = registry.get(&endpoint_id).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_existing_peer() {
        let (registry, _temp) = create_test_registry();

        let endpoint_id = create_test_public_key();
        let mut peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        // Add peer
        registry.add_or_update(&peer).unwrap();

        // Update peer with new status
        peer.status = PeerStatus::Online;
        peer.nickname = Some("Bob".to_string());
        registry.add_or_update(&peer).unwrap();

        // Verify update
        let retrieved = registry.get(&endpoint_id).unwrap().unwrap();
        assert_eq!(retrieved.status, PeerStatus::Online);
        assert_eq!(retrieved.nickname, Some("Bob".to_string()));
    }

    #[test]
    fn test_list_all_peers() {
        let (registry, _temp) = create_test_registry();

        // Add multiple peers
        for _ in 0..3 {
            let endpoint_id = create_test_public_key();
            let peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);
            registry.add_or_update(&peer).unwrap();
        }

        let all_peers = registry.list_all().unwrap();
        assert_eq!(all_peers.len(), 3);
    }

    #[test]
    fn test_list_by_status() {
        let (registry, _temp) = create_test_registry();

        // Add peers with different statuses
        let peer1 = PeerInfo::new(create_test_public_key(), PeerSource::FromInvite)
            .with_status(PeerStatus::Online);

        let peer2 = PeerInfo::new(create_test_public_key(), PeerSource::FromInvite)
            .with_status(PeerStatus::Offline);

        let peer3 = PeerInfo::new(create_test_public_key(), PeerSource::FromInvite)
            .with_status(PeerStatus::Online);

        registry.add_or_update(&peer1).unwrap();
        registry.add_or_update(&peer2).unwrap();
        registry.add_or_update(&peer3).unwrap();

        let online = registry.list_by_status(PeerStatus::Online).unwrap();
        assert_eq!(online.len(), 2);

        let offline = registry.list_by_status(PeerStatus::Offline).unwrap();
        assert_eq!(offline.len(), 1);
    }

    #[test]
    fn test_list_inactive_peers() {
        let (registry, _temp) = create_test_registry();

        // Add peers with different statuses
        let peer1 = PeerInfo::new(create_test_public_key(), PeerSource::FromInvite)
            .with_status(PeerStatus::Online);

        let peer2 = PeerInfo::new(create_test_public_key(), PeerSource::FromInvite)
            .with_status(PeerStatus::Offline);

        let peer3 = PeerInfo::new(create_test_public_key(), PeerSource::FromInvite); // Status: Unknown

        registry.add_or_update(&peer1).unwrap();
        registry.add_or_update(&peer2).unwrap();
        registry.add_or_update(&peer3).unwrap();

        let inactive = registry.list_inactive().unwrap();
        assert_eq!(inactive.len(), 2); // Offline + Unknown
    }

    #[test]
    fn test_update_status() {
        let (registry, _temp) = create_test_registry();

        let endpoint_id = create_test_public_key();
        let peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);
        registry.add_or_update(&peer).unwrap();

        // Update status
        registry
            .update_status(&endpoint_id, PeerStatus::Online)
            .unwrap();

        let retrieved = registry.get(&endpoint_id).unwrap().unwrap();
        assert_eq!(retrieved.status, PeerStatus::Online);
    }

    #[test]
    fn test_update_nickname() {
        let (registry, _temp) = create_test_registry();

        let endpoint_id = create_test_public_key();
        let peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);
        registry.add_or_update(&peer).unwrap();

        // Update nickname
        registry.update_nickname(&endpoint_id, "Charlie").unwrap();

        let retrieved = registry.get(&endpoint_id).unwrap().unwrap();
        assert_eq!(retrieved.nickname, Some("Charlie".to_string()));
    }

    #[test]
    fn test_add_peer_realm() {
        let (registry, _temp) = create_test_registry();

        let endpoint_id = create_test_public_key();
        let peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);
        registry.add_or_update(&peer).unwrap();

        // Add realm
        let realm_id = RealmId::new();
        registry.add_peer_realm(&endpoint_id, &realm_id).unwrap();

        let retrieved = registry.get(&endpoint_id).unwrap().unwrap();
        assert_eq!(retrieved.shared_realms.len(), 1);
        assert_eq!(retrieved.shared_realms[0], realm_id);
    }

    #[test]
    fn test_count_peers() {
        let (registry, _temp) = create_test_registry();

        // Initially empty
        assert_eq!(registry.count().unwrap(), 0);

        // Add peers
        for i in 1u8..=5 {
            let endpoint_id = create_test_public_key();
            let status = if i % 2 == 0 {
                PeerStatus::Online
            } else {
                PeerStatus::Offline
            };
            let peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite).with_status(status);
            registry.add_or_update(&peer).unwrap();
        }

        assert_eq!(registry.count().unwrap(), 5);
        assert_eq!(registry.count_by_status(PeerStatus::Online).unwrap(), 2);
        assert_eq!(registry.count_by_status(PeerStatus::Offline).unwrap(), 3);
    }

    #[test]
    fn test_peer_registry_persists_across_instances() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.redb");

        let endpoint_id = create_test_public_key();

        // Create first registry instance and add peer
        {
            let db = Database::create(&db_path).unwrap();
            let registry = PeerRegistry::new(Arc::new(RwLock::new(db))).unwrap();
            let peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite)
                .with_nickname("Alice")
                .with_status(PeerStatus::Online);
            registry.add_or_update(&peer).unwrap();
        }

        // Create second registry instance and verify peer exists
        {
            let db = Database::create(&db_path).unwrap();
            let registry = PeerRegistry::new(Arc::new(RwLock::new(db))).unwrap();
            let retrieved = registry.get(&endpoint_id).unwrap();
            assert!(retrieved.is_some());
            let retrieved = retrieved.unwrap();
            assert_eq!(retrieved.nickname, Some("Alice".to_string()));
            assert_eq!(retrieved.status, PeerStatus::Online);
        }
    }
}
