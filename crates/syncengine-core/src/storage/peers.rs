//! Unified Peer Storage
//!
//! This module provides CRUD operations for the unified `Peer` type, which
//! consolidates the previous separate `ContactInfo` and `PeerInfo` storage.
//!
//! ## Storage Design
//!
//! - **Primary table**: `unified_peers` - keyed by hex-encoded endpoint_id
//! - **Secondary index**: `peer_did_index` - maps DID → endpoint_id for fast DID lookups
//!
//! ## Migration
//!
//! The `migrate_to_unified_peers` function handles migration from the old
//! `CONTACTS_TABLE` and `PEERS_TABLE` to the new unified format.

use crate::error::SyncError;
use crate::types::contact::ContactInfo;
use crate::types::peer::{ContactDetails, Peer, PeerSource, PeerStatus};
use iroh::PublicKey;
use redb::{ReadableTable, TableDefinition};

use super::Storage;

/// Table for unified peers (key: hex endpoint_id, value: serialized Peer)
pub(crate) const UNIFIED_PEERS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("unified_peers");

/// Index for DID → endpoint_id lookup (key: DID string, value: hex endpoint_id)
pub(crate) const PEER_DID_INDEX: TableDefinition<&str, &str> =
    TableDefinition::new("peer_did_index");

/// Flag table to track migration status
pub(crate) const MIGRATION_FLAGS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("migration_flags");

const MIGRATION_FLAG_KEY: &str = "peers_unified_v1";

impl Storage {
    // ═══════════════════════════════════════════════════════════════════════
    // Unified Peer Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Save a unified peer to the database
    ///
    /// If a peer with the same endpoint_id exists, it will be overwritten.
    /// This also updates the DID index if the peer has a DID.
    pub fn save_peer(&self, peer: &Peer) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut peers_table = write_txn.open_table(UNIFIED_PEERS_TABLE)?;
            let mut did_index = write_txn.open_table(PEER_DID_INDEX)?;

            let key = hex::encode(&peer.endpoint_id);
            let serialized = postcard::to_allocvec(peer)
                .map_err(|e| SyncError::Serialization(e.to_string()))?;

            peers_table.insert(key.as_str(), serialized.as_slice())?;

            // Update DID index if peer has a DID
            if let Some(ref did) = peer.did {
                did_index.insert(did.as_str(), key.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load a peer by endpoint ID
    ///
    /// Returns `None` if no peer exists with the given endpoint ID.
    pub fn load_peer(&self, endpoint_id: &PublicKey) -> Result<Option<Peer>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(UNIFIED_PEERS_TABLE)?;

        let key = hex::encode(endpoint_id.as_bytes());
        if let Some(data) = table.get(key.as_str())? {
            let peer: Peer = postcard::from_bytes(data.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            Ok(Some(peer))
        } else {
            Ok(None)
        }
    }

    /// Load a peer by endpoint ID bytes
    ///
    /// Convenience method when you have raw bytes instead of a PublicKey.
    pub fn load_peer_by_bytes(&self, endpoint_id: &[u8; 32]) -> Result<Option<Peer>, SyncError> {
        let key = hex::encode(endpoint_id);
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(UNIFIED_PEERS_TABLE)?;

        if let Some(data) = table.get(key.as_str())? {
            let peer: Peer = postcard::from_bytes(data.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            Ok(Some(peer))
        } else {
            Ok(None)
        }
    }

    /// Load a peer by DID
    ///
    /// Uses the DID index for fast lookup. Returns `None` if no peer
    /// with the given DID exists.
    pub fn load_peer_by_did(&self, did: &str) -> Result<Option<Peer>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;

        // First, look up the endpoint_id in the DID index
        let did_index = read_txn.open_table(PEER_DID_INDEX)?;
        let endpoint_key = match did_index.get(did)? {
            Some(v) => v.value().to_string(),
            None => return Ok(None),
        };

        // Then look up the peer by endpoint_id
        let peers_table = read_txn.open_table(UNIFIED_PEERS_TABLE)?;
        if let Some(data) = peers_table.get(endpoint_key.as_str())? {
            let peer: Peer = postcard::from_bytes(data.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            Ok(Some(peer))
        } else {
            Ok(None)
        }
    }

    /// Delete a peer by endpoint ID
    ///
    /// Also removes the DID index entry if the peer had a DID.
    pub fn delete_peer(&self, endpoint_id: &PublicKey) -> Result<(), SyncError> {
        let key = hex::encode(endpoint_id.as_bytes());

        // First load the peer to get the DID (if any) for index cleanup
        let peer_did = self.load_peer(endpoint_id)?.and_then(|p| p.did);

        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut peers_table = write_txn.open_table(UNIFIED_PEERS_TABLE)?;
            let mut did_index = write_txn.open_table(PEER_DID_INDEX)?;

            peers_table.remove(key.as_str())?;

            // Remove from DID index if the peer had a DID
            if let Some(did) = peer_did {
                did_index.remove(did.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    /// List all unified peers
    pub fn list_peers(&self) -> Result<Vec<Peer>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(UNIFIED_PEERS_TABLE)?;

        let mut peers = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let peer: Peer = postcard::from_bytes(value.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            peers.push(peer);
        }

        Ok(peers)
    }

    /// List only contacts (peers with contact_info)
    pub fn list_peer_contacts(&self) -> Result<Vec<Peer>, SyncError> {
        Ok(self
            .list_peers()?
            .into_iter()
            .filter(|p| p.is_contact())
            .collect())
    }

    /// List only discovered peers (non-contacts)
    pub fn list_discovered_peers(&self) -> Result<Vec<Peer>, SyncError> {
        Ok(self
            .list_peers()?
            .into_iter()
            .filter(|p| p.is_discovered())
            .collect())
    }

    /// List peers by status
    pub fn list_peers_by_status(&self, status: PeerStatus) -> Result<Vec<Peer>, SyncError> {
        Ok(self
            .list_peers()?
            .into_iter()
            .filter(|p| p.status == status)
            .collect())
    }

    /// List peers that are inactive (offline or unknown) for reconnection
    pub fn list_inactive_peers(&self) -> Result<Vec<Peer>, SyncError> {
        Ok(self
            .list_peers()?
            .into_iter()
            .filter(|p| matches!(p.status, PeerStatus::Offline | PeerStatus::Unknown))
            .collect())
    }

    /// Count all peers
    pub fn count_peers(&self) -> Result<usize, SyncError> {
        Ok(self.list_peers()?.len())
    }

    /// Count contacts only
    pub fn count_contacts(&self) -> Result<usize, SyncError> {
        Ok(self.list_peer_contacts()?.len())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Migration
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if migration to unified peers has been completed
    pub fn is_peers_migrated(&self) -> Result<bool, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;

        // Try to open the migration flags table
        match read_txn.open_table(MIGRATION_FLAGS_TABLE) {
            Ok(table) => Ok(table.get(MIGRATION_FLAG_KEY)?.is_some()),
            Err(_) => Ok(false), // Table doesn't exist, not migrated
        }
    }

    /// Mark migration as complete
    fn mark_peers_migrated(&self) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(MIGRATION_FLAGS_TABLE)?;
            let timestamp = chrono::Utc::now().timestamp().to_le_bytes();
            table.insert(MIGRATION_FLAG_KEY, timestamp.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Migrate from old ContactInfo and PeerInfo tables to unified Peer table
    ///
    /// This is idempotent - it will skip if migration has already been performed.
    /// Returns the number of peers migrated.
    pub fn migrate_to_unified_peers(&self) -> Result<usize, SyncError> {
        // Skip if already migrated
        if self.is_peers_migrated()? {
            return Ok(0);
        }

        let mut migrated_count = 0;
        let mut migrated_endpoints: std::collections::HashSet<[u8; 32]> =
            std::collections::HashSet::new();

        // 1. Migrate contacts (they have full identity info)
        let contacts = self.list_contacts()?;
        for contact in contacts {
            let peer = Self::contact_info_to_peer(&contact);
            self.save_peer(&peer)?;
            migrated_endpoints.insert(peer.endpoint_id);
            migrated_count += 1;
        }

        // 2. Migrate discovered peers from PeerRegistry
        // Note: This reads from the old PEERS_TABLE via PeerRegistry
        // We'll migrate any that weren't already migrated as contacts
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;

        // Try to open the old peers table
        use redb::TableDefinition;
        const OLD_PEERS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("peers");

        if let Ok(old_peers_table) = read_txn.open_table(OLD_PEERS_TABLE) {
            for entry in old_peers_table.iter()? {
                let (_, value) = entry?;

                // Try to deserialize as old PeerInfo
                if let Ok(old_peer) =
                    postcard::from_bytes::<crate::peers::PeerInfo>(value.value())
                {
                    // Skip if already migrated via contact
                    if migrated_endpoints.contains(&old_peer.endpoint_id) {
                        continue;
                    }

                    let peer = Self::peer_info_to_peer(&old_peer);
                    self.save_peer(&peer)?;
                    migrated_endpoints.insert(peer.endpoint_id);
                    migrated_count += 1;
                }
            }
        }

        drop(read_txn);

        // Mark migration as complete
        self.mark_peers_migrated()?;

        Ok(migrated_count)
    }

    /// Convert a ContactInfo to the unified Peer type
    fn contact_info_to_peer(contact: &ContactInfo) -> Peer {
        Peer {
            endpoint_id: contact.peer_endpoint_id,
            did: Some(contact.peer_did.clone()),
            profile: Some(contact.profile.clone()),
            nickname: None,
            contact_info: Some(ContactDetails {
                contact_topic: contact.contact_topic,
                contact_key: contact.contact_key,
                accepted_at: contact.accepted_at,
                is_favorite: contact.is_favorite,
            }),
            source: PeerSource::FromContact,
            shared_realms: Vec::new(),
            node_addr: Some(contact.node_addr.clone()),
            status: match contact.status {
                crate::types::contact::ContactStatus::Online => PeerStatus::Online,
                crate::types::contact::ContactStatus::Offline => PeerStatus::Offline,
            },
            last_seen: contact.last_seen,
            connection_attempts: 0,
            successful_connections: 0,
            last_attempt: 0,
        }
    }

    /// Convert a PeerInfo to the unified Peer type
    fn peer_info_to_peer(old: &crate::peers::PeerInfo) -> Peer {
        Peer {
            endpoint_id: old.endpoint_id,
            did: None,
            profile: None,
            nickname: old.nickname.clone(),
            contact_info: None,
            source: old.source.clone(),
            shared_realms: old.shared_realms.clone(),
            node_addr: None,
            status: old.status,
            last_seen: old.last_seen,
            connection_attempts: old.connection_attempts,
            successful_connections: old.successful_connections,
            last_attempt: old.last_attempt,
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invite::NodeAddrBytes;
    use crate::types::contact::ProfileSnapshot;
    use iroh::SecretKey;
    use tempfile::tempdir;

    fn create_test_public_key() -> PublicKey {
        SecretKey::generate(&mut rand::rng()).public()
    }

    fn create_test_peer(endpoint_id: PublicKey) -> Peer {
        Peer::new(endpoint_id, PeerSource::FromInvite)
    }

    #[test]
    fn test_save_and_load_peer() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();
        
        let endpoint_id = create_test_public_key();
        let peer = create_test_peer(endpoint_id)
            .with_nickname("Love")
            .with_status(PeerStatus::Online);

        // Save
        storage.save_peer(&peer).unwrap();

        // Load
        let loaded = storage.load_peer(&endpoint_id).unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.nickname, Some("Love".to_string()));
        assert_eq!(loaded.status, PeerStatus::Online);
    }

    #[test]
    fn test_load_peer_by_did() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();
        
        let endpoint_id = create_test_public_key();
        let peer = create_test_peer(endpoint_id).with_did("did:sync:love123");

        // Save
        storage.save_peer(&peer).unwrap();

        // Load by DID
        let loaded = storage.load_peer_by_did("did:sync:love123").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().endpoint_id, *endpoint_id.as_bytes());

        // Non-existent DID
        let not_found = storage.load_peer_by_did("did:sync:unknown").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_delete_peer() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();
        
        let endpoint_id = create_test_public_key();
        let peer = create_test_peer(endpoint_id).with_did("did:sync:to_delete");

        storage.save_peer(&peer).unwrap();
        assert!(storage.load_peer(&endpoint_id).unwrap().is_some());
        assert!(storage.load_peer_by_did("did:sync:to_delete").unwrap().is_some());

        // Delete
        storage.delete_peer(&endpoint_id).unwrap();

        // Both lookups should fail
        assert!(storage.load_peer(&endpoint_id).unwrap().is_none());
        assert!(storage.load_peer_by_did("did:sync:to_delete").unwrap().is_none());
    }

    #[test]
    fn test_list_peers() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();
        
        // Create multiple peers
        for i in 0..3 {
            let endpoint_id = create_test_public_key();
            let peer = create_test_peer(endpoint_id).with_nickname(format!("Peer{}", i));
            storage.save_peer(&peer).unwrap();
        }

        let peers = storage.list_peers().unwrap();
        assert_eq!(peers.len(), 3);
    }

    #[test]
    fn test_list_contacts_vs_discovered() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();
        
        // Create 2 contacts
        for i in 0..2 {
            let endpoint_id = create_test_public_key();
            let contact_info = ContactDetails::new([i; 32], [i + 1; 32]);
            let peer = create_test_peer(endpoint_id)
                .with_contact_info(contact_info)
                .with_did(format!("did:sync:contact{}", i));
            storage.save_peer(&peer).unwrap();
        }

        // Create 3 discovered peers
        for i in 0..3 {
            let endpoint_id = create_test_public_key();
            let peer = create_test_peer(endpoint_id).with_nickname(format!("Discovered{}", i));
            storage.save_peer(&peer).unwrap();
        }

        // Verify counts
        assert_eq!(storage.count_peers().unwrap(), 5);
        assert_eq!(storage.count_contacts().unwrap(), 2);
        assert_eq!(storage.list_peer_contacts().unwrap().len(), 2);
        assert_eq!(storage.list_discovered_peers().unwrap().len(), 3);
    }

    #[test]
    fn test_list_peers_by_status() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();
        
        // Create peers with different statuses
        let peer1 = create_test_peer(create_test_public_key()).with_status(PeerStatus::Online);
        let peer2 = create_test_peer(create_test_public_key()).with_status(PeerStatus::Online);
        let peer3 = create_test_peer(create_test_public_key()).with_status(PeerStatus::Offline);
        let peer4 = create_test_peer(create_test_public_key()); // Unknown (default)

        storage.save_peer(&peer1).unwrap();
        storage.save_peer(&peer2).unwrap();
        storage.save_peer(&peer3).unwrap();
        storage.save_peer(&peer4).unwrap();

        assert_eq!(
            storage.list_peers_by_status(PeerStatus::Online).unwrap().len(),
            2
        );
        assert_eq!(
            storage.list_peers_by_status(PeerStatus::Offline).unwrap().len(),
            1
        );
        assert_eq!(
            storage.list_peers_by_status(PeerStatus::Unknown).unwrap().len(),
            1
        );
        assert_eq!(storage.list_inactive_peers().unwrap().len(), 2);
    }

    #[test]
    fn test_peer_update_preserves_did_index() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();
        
        let endpoint_id = create_test_public_key();
        let peer = create_test_peer(endpoint_id)
            .with_did("did:sync:test")
            .with_nickname("Original");

        storage.save_peer(&peer).unwrap();

        // Update the peer
        let mut updated = storage.load_peer(&endpoint_id).unwrap().unwrap();
        updated.nickname = Some("Updated".to_string());
        updated.status = PeerStatus::Online;
        storage.save_peer(&updated).unwrap();

        // DID lookup should still work
        let by_did = storage.load_peer_by_did("did:sync:test").unwrap();
        assert!(by_did.is_some());
        assert_eq!(by_did.unwrap().nickname, Some("Updated".to_string()));
    }

    #[test]
    fn test_migration_flag() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();
        
        // Initially not migrated
        assert!(!storage.is_peers_migrated().unwrap());

        // Run migration (no data, but should still mark complete)
        let count = storage.migrate_to_unified_peers().unwrap();
        assert_eq!(count, 0);

        // Now should be marked as migrated
        assert!(storage.is_peers_migrated().unwrap());

        // Running again should be a no-op
        let count2 = storage.migrate_to_unified_peers().unwrap();
        assert_eq!(count2, 0);
    }

    #[test]
    fn test_contact_info_to_peer_conversion() {
        let contact = ContactInfo {
            peer_did: "did:sync:love".to_string(),
            peer_endpoint_id: [1u8; 32],
            profile: ProfileSnapshot {
                display_name: "Love".to_string(),
                subtitle: Some("Explorer".to_string()),
                avatar_blob_id: None,
                bio: "Test bio".to_string(),
            },
            node_addr: NodeAddrBytes::new([2u8; 32]),
            contact_topic: [3u8; 32],
            contact_key: [4u8; 32],
            accepted_at: 1234567890,
            last_seen: 1234567900,
            status: crate::types::contact::ContactStatus::Online,
            is_favorite: true,
            encryption_keys: None,
        };

        let peer = Storage::contact_info_to_peer(&contact);

        assert_eq!(peer.endpoint_id, [1u8; 32]);
        assert_eq!(peer.did, Some("did:sync:love".to_string()));
        assert!(peer.profile.is_some());
        assert_eq!(peer.profile.as_ref().unwrap().display_name, "Love");
        assert!(peer.is_contact());
        assert!(peer.is_favorite());
        assert_eq!(peer.status, PeerStatus::Online);
        assert_eq!(peer.source, PeerSource::FromContact);
    }
}
