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

// Re-export from types::peer for backwards compatibility
pub use crate::types::peer::{PeerSource, PeerStatus};

// Table definition for peer registry
const PEERS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("peers");

/// Information about a discovered peer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Total number of connection attempts
    #[serde(default)]
    pub connection_attempts: u32,
    /// Number of successful connections
    #[serde(default)]
    pub successful_connections: u32,
    /// Unix timestamp of last connection attempt
    #[serde(default)]
    pub last_attempt: u64,
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
            connection_attempts: 0,
            successful_connections: 0,
            last_attempt: 0,
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

    /// Record a connection attempt
    pub fn record_attempt(&mut self) {
        self.connection_attempts += 1;
        self.last_attempt = Self::current_timestamp();
    }

    /// Record a successful connection
    pub fn record_success(&mut self) {
        self.successful_connections += 1;
        self.status = PeerStatus::Online;
        self.touch();
    }

    /// Record a connection failure
    pub fn record_failure(&mut self) {
        self.status = PeerStatus::Offline;
    }

    /// Calculate success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.connection_attempts == 0 {
            0.0
        } else {
            self.successful_connections as f64 / self.connection_attempts as f64
        }
    }

    /// Calculate the number of consecutive failures
    fn consecutive_failures(&self) -> u32 {
        // If we have no attempts or no successes, count all attempts as failures
        if self.connection_attempts == 0 {
            return 0;
        }
        // Since we don't track consecutive failures explicitly,
        // we'll use a simple heuristic: total attempts - successful connections
        // This is an approximation that works well enough for backoff purposes
        self.connection_attempts
            .saturating_sub(self.successful_connections)
    }

    /// Calculate Fibonacci backoff delay in seconds
    /// Sequence: 1min, 1min, 2min, 3min, 5min, 8min, 13min, 21min, 34min, 55min, capped at 60min
    pub fn backoff_delay(&self) -> u64 {
        let failures = self.consecutive_failures();
        let base_unit = 60u64; // 1 minute in seconds
        let max_delay = 3600u64; // 60 minutes in seconds

        // Calculate Fibonacci number for the failure count
        let fib = Self::fibonacci(failures);

        // Convert to seconds (Fibonacci number * 1 minute)
        let delay = fib.saturating_mul(base_unit);

        // Cap at 60 minutes
        delay.min(max_delay)
    }

    /// Calculate the nth Fibonacci number efficiently (iterative)
    /// F(0) = 1, F(1) = 1, F(n) = F(n-1) + F(n-2)
    fn fibonacci(n: u32) -> u64 {
        match n {
            0 => 1,
            1 => 1,
            _ => {
                let mut a = 1u64;
                let mut b = 1u64;
                for _ in 2..=n {
                    let next = a.saturating_add(b);
                    a = b;
                    b = next;
                }
                b
            }
        }
    }

    /// Check if enough time has passed since last attempt to retry
    pub fn should_retry_now(&self) -> bool {
        if self.last_attempt == 0 {
            // Never attempted, can retry immediately
            return true;
        }

        let now = Self::current_timestamp();
        let elapsed = now.saturating_sub(self.last_attempt);
        let required_delay = self.backoff_delay();

        elapsed >= required_delay
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
            .with_nickname("Love")
            .with_status(PeerStatus::Online);

        assert_eq!(peer.nickname, Some("Love".to_string()));
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
        peer.nickname = Some("Joy".to_string());
        registry.add_or_update(&peer).unwrap();

        // Verify update
        let retrieved = registry.get(&endpoint_id).unwrap().unwrap();
        assert_eq!(retrieved.status, PeerStatus::Online);
        assert_eq!(retrieved.nickname, Some("Joy".to_string()));
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
                .with_nickname("Love")
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
            assert_eq!(retrieved.nickname, Some("Love".to_string()));
            assert_eq!(retrieved.status, PeerStatus::Online);
        }
    }

    #[test]
    fn test_connection_metrics_record_attempt() {
        let endpoint_id = create_test_public_key();
        let mut peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        assert_eq!(peer.connection_attempts, 0);
        assert_eq!(peer.last_attempt, 0);

        peer.record_attempt();
        assert_eq!(peer.connection_attempts, 1);
        assert!(peer.last_attempt > 0);

        let first_attempt = peer.last_attempt;
        peer.record_attempt();
        assert_eq!(peer.connection_attempts, 2);
        assert!(peer.last_attempt >= first_attempt);
    }

    #[test]
    fn test_connection_metrics_record_success() {
        let endpoint_id = create_test_public_key();
        let mut peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        assert_eq!(peer.successful_connections, 0);
        assert_eq!(peer.status, PeerStatus::Unknown);

        peer.record_success();
        assert_eq!(peer.successful_connections, 1);
        assert_eq!(peer.status, PeerStatus::Online);
        assert!(peer.last_seen > 0);
    }

    #[test]
    fn test_connection_metrics_record_failure() {
        let endpoint_id = create_test_public_key();
        let mut peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        peer.record_failure();
        assert_eq!(peer.status, PeerStatus::Offline);
    }

    #[test]
    fn test_connection_metrics_success_rate() {
        let endpoint_id = create_test_public_key();
        let mut peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        // No attempts yet
        assert_eq!(peer.success_rate(), 0.0);

        // 1 success out of 1 attempt
        peer.connection_attempts = 1;
        peer.successful_connections = 1;
        assert_eq!(peer.success_rate(), 1.0);

        // 1 success out of 2 attempts
        peer.connection_attempts = 2;
        assert_eq!(peer.success_rate(), 0.5);

        // 2 successes out of 5 attempts
        peer.connection_attempts = 5;
        peer.successful_connections = 2;
        assert_eq!(peer.success_rate(), 0.4);
    }

    #[test]
    fn test_fibonacci_backoff_no_failures() {
        let endpoint_id = create_test_public_key();
        let peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        // No attempts = no backoff, F(0) = 1 minute
        assert_eq!(peer.backoff_delay(), 60); // 1 minute
    }

    #[test]
    fn test_fibonacci_backoff_progression() {
        let endpoint_id = create_test_public_key();
        let mut peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        // 0 failures (0 attempts, 0 successes) - F(0) = 1
        assert_eq!(peer.backoff_delay(), 60); // 1 minute

        // 1 failure (1 attempt, 0 successes) - F(1) = 1
        peer.connection_attempts = 1;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 60); // 1 minute

        // 2 failures (2 attempts, 0 successes) - F(2) = 2
        peer.connection_attempts = 2;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 120); // 2 minutes

        // 3 failures (3 attempts, 0 successes) - F(3) = 3
        peer.connection_attempts = 3;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 180); // 3 minutes

        // 4 failures (4 attempts, 0 successes) - F(4) = 5
        peer.connection_attempts = 4;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 300); // 5 minutes

        // 5 failures (5 attempts, 0 successes) - F(5) = 8
        peer.connection_attempts = 5;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 480); // 8 minutes

        // 6 failures (6 attempts, 0 successes) - F(6) = 13
        peer.connection_attempts = 6;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 780); // 13 minutes

        // 7 failures (7 attempts, 0 successes) - F(7) = 21
        peer.connection_attempts = 7;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 1260); // 21 minutes

        // 8 failures (8 attempts, 0 successes) - F(8) = 34
        peer.connection_attempts = 8;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 2040); // 34 minutes

        // 9 failures (9 attempts, 0 successes) - F(9) = 55
        peer.connection_attempts = 9;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 3300); // 55 minutes

        // 10 failures (10 attempts, 0 successes) - F(10) = 89, but capped at 60 min
        peer.connection_attempts = 10;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 3600); // 60 minutes (capped)

        // 15 failures - still capped
        peer.connection_attempts = 15;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 3600); // Still capped at 60 minutes
    }

    #[test]
    fn test_fibonacci_backoff_success_resets() {
        let endpoint_id = create_test_public_key();
        let mut peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        // Build up failures - F(9) = 55 minutes
        peer.connection_attempts = 9;
        peer.successful_connections = 0;
        assert_eq!(peer.backoff_delay(), 3300); // 55 minutes

        // Record a success
        peer.successful_connections = 1;
        // Now we have 8 failures (9 - 1), F(8) = 34
        assert_eq!(peer.backoff_delay(), 2040); // 34 minutes

        // Another success
        peer.successful_connections = 2;
        // Now we have 7 failures (9 - 2), F(7) = 21
        assert_eq!(peer.backoff_delay(), 1260); // 21 minutes

        // All successes
        peer.successful_connections = 9;
        // Now we have 0 failures (9 - 9), F(0) = 1
        assert_eq!(peer.backoff_delay(), 60); // Back to 1 minute
    }

    #[test]
    fn test_should_retry_now_never_attempted() {
        let endpoint_id = create_test_public_key();
        let peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        // Never attempted, should be able to retry immediately
        assert!(peer.should_retry_now());
    }

    #[test]
    fn test_should_retry_now_with_backoff() {
        let endpoint_id = create_test_public_key();
        let mut peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        // Set up a failure scenario (1 failure = F(1) = 1 min)
        peer.connection_attempts = 1;
        peer.successful_connections = 0;
        peer.last_attempt = PeerInfo::current_timestamp();

        // Should not be able to retry immediately (needs 1 min backoff)
        assert!(!peer.should_retry_now());

        // Simulate time passing (set last_attempt to 1 minute ago)
        peer.last_attempt = PeerInfo::current_timestamp() - 60;
        assert!(peer.should_retry_now());
    }

    #[test]
    fn test_should_retry_now_respects_backoff_increase() {
        let endpoint_id = create_test_public_key();
        let mut peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        // Set up 5 failures (F(5) = 8 min backoff)
        peer.connection_attempts = 5;
        peer.successful_connections = 0;
        peer.last_attempt = PeerInfo::current_timestamp() - 300; // 5 minutes ago

        // 5 minutes passed, but needs 8 minutes
        assert!(!peer.should_retry_now());

        // Simulate 8 minutes passing
        peer.last_attempt = PeerInfo::current_timestamp() - 480;
        assert!(peer.should_retry_now());
    }

    #[test]
    fn test_connection_metrics_persist() {
        let (registry, _temp) = create_test_registry();

        let endpoint_id = create_test_public_key();
        let mut peer = PeerInfo::new(endpoint_id, PeerSource::FromInvite);

        // Set up metrics
        peer.connection_attempts = 5;
        peer.successful_connections = 2;
        peer.last_attempt = 123456;

        // Save to registry
        registry.add_or_update(&peer).unwrap();

        // Retrieve and verify
        let retrieved = registry.get(&endpoint_id).unwrap().unwrap();
        assert_eq!(retrieved.connection_attempts, 5);
        assert_eq!(retrieved.successful_connections, 2);
        assert_eq!(retrieved.last_attempt, 123456);
    }
}
