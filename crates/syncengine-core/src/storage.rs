//! Persistent storage using redb (replaces sled).
//!
//! This module provides ACID-compliant storage for:
//! - Realms (workspaces/projects)
//! - Documents (Automerge CRDT blobs)
//! - Identity and device credentials
//! - Realm encryption keys
//! - User profiles
//! - Image blobs (content-addressed)

use crate::error::SyncError;
use crate::types::{RealmId, RealmInfo};
use parking_lot::RwLock;
use redb::{Database, ReadableTable, TableDefinition};
use std::path::Path;
use std::sync::Arc;

// Submodules
mod blobs;
mod contacts;
mod peers;
mod pinned_profiles;
mod profile_pinners;
mod profiles;

// Re-export initialization helpers (used in Storage::new)
use blobs::BLOBS_TABLE;
use contacts::{CONTACTS_TABLE, PENDING_CONTACTS_TABLE, REVOKED_INVITES_TABLE};
use peers::{MIGRATION_FLAGS_TABLE, PEER_DID_INDEX, UNIFIED_PEERS_TABLE};
use pinned_profiles::PINNED_PROFILES_TABLE;
use profiles::PROFILES_TABLE;

// Re-export pinning configuration
pub use pinned_profiles::PinningConfig;

// Re-export pinner info for network page
pub use profile_pinners::PinnerInfo;

// Table definitions
const REALMS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("realms");
const DOCUMENTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("documents");
const IDENTITY_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("identity");
const REALM_KEYS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("realm_keys");
const ENDPOINT_SECRET_KEY_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("endpoint_secret_key");

/// Storage layer using redb for ACID-compliant persistence
#[derive(Clone)]
pub struct Storage {
    db: Arc<RwLock<Database>>,
}

impl Storage {
    /// Get a reference to the shared database handle
    ///
    /// This allows other components (like PeerRegistry) to share the same
    /// database connection instead of opening multiple instances of the same file.
    pub fn db_handle(&self) -> Arc<RwLock<Database>> {
        self.db.clone()
    }
}

impl Storage {
    /// Create a new storage instance at the given path.
    ///
    /// This will:
    /// - Create the database directory if it doesn't exist
    /// - Initialize the database file
    /// - Create all required tables
    pub fn new(path: impl AsRef<Path>) -> Result<Self, SyncError> {
        let path = path.as_ref();

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open/create database
        let db = Database::create(path)?;

        // Initialize all tables
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(REALMS_TABLE)?;
            let _ = write_txn.open_table(DOCUMENTS_TABLE)?;
            let _ = write_txn.open_table(IDENTITY_TABLE)?;
            let _ = write_txn.open_table(REALM_KEYS_TABLE)?;
            let _ = write_txn.open_table(ENDPOINT_SECRET_KEY_TABLE)?;
            let _ = write_txn.open_table(PROFILES_TABLE)?;
            let _ = write_txn.open_table(BLOBS_TABLE)?;
            let _ = write_txn.open_table(CONTACTS_TABLE)?;
            let _ = write_txn.open_table(PENDING_CONTACTS_TABLE)?;
            let _ = write_txn.open_table(REVOKED_INVITES_TABLE)?;
            let _ = write_txn.open_table(PINNED_PROFILES_TABLE)?;
            // Note: PROFILE_PINNERS_TABLE removed - Indra's Net derives pinners from contacts
            // Unified peers tables
            let _ = write_txn.open_table(UNIFIED_PEERS_TABLE)?;
            let _ = write_txn.open_table(PEER_DID_INDEX)?;
            let _ = write_txn.open_table(MIGRATION_FLAGS_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self {
            db: Arc::new(RwLock::new(db)),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Realm Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Save a realm to the database.
    ///
    /// If a realm with the same ID already exists, it will be overwritten.
    pub fn save_realm(&self, info: &RealmInfo) -> Result<(), SyncError> {
        let db = self.db.read();
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(REALMS_TABLE)?;
            let data =
                serde_json::to_vec(info).map_err(|e| SyncError::Serialization(e.to_string()))?;
            let key = info.id.to_base58();
            table.insert(key.as_str(), data.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load a single realm by ID from the database.
    ///
    /// Returns `None` if no realm with the given ID exists.
    pub fn load_realm(&self, realm_id: &RealmId) -> Result<Option<RealmInfo>, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(REALMS_TABLE)?;
        let key = realm_id.to_base58();

        match table.get(key.as_str())? {
            Some(v) => {
                let info: RealmInfo = serde_json::from_slice(v.value())
                    .map_err(|e| SyncError::Serialization(e.to_string()))?;
                Ok(Some(info))
            }
            None => Ok(None),
        }
    }

    /// Load all realms from the database.
    pub fn list_realms(&self) -> Result<Vec<RealmInfo>, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(REALMS_TABLE)?;

        let mut realms = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let info: RealmInfo = serde_json::from_slice(value.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            realms.push(info);
        }
        Ok(realms)
    }

    /// Delete a realm and all associated data (documents, keys).
    pub fn delete_realm(&self, realm_id: &RealmId) -> Result<(), SyncError> {
        let db = self.db.read();
        let write_txn = db.begin_write()?;
        {
            let key = realm_id.to_base58();
            // Delete from all related tables
            let mut realms = write_txn.open_table(REALMS_TABLE)?;
            let mut documents = write_txn.open_table(DOCUMENTS_TABLE)?;
            let mut keys = write_txn.open_table(REALM_KEYS_TABLE)?;

            realms.remove(key.as_str())?;
            documents.remove(key.as_str())?;
            keys.remove(key.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Document Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Save a document (Automerge blob) for a realm.
    ///
    /// Documents are stored as raw bytes and can be any size.
    pub fn save_document(&self, realm_id: &RealmId, data: &[u8]) -> Result<(), SyncError> {
        let db = self.db.read();
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(DOCUMENTS_TABLE)?;
            let key = realm_id.to_base58();
            table.insert(key.as_str(), data)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load a document for a realm.
    ///
    /// Returns `None` if no document exists for the given realm.
    pub fn load_document(&self, realm_id: &RealmId) -> Result<Option<Vec<u8>>, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(DOCUMENTS_TABLE)?;
        let key = realm_id.to_base58();

        Ok(table.get(key.as_str())?.map(|v| v.value().to_vec()))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Realm Key Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Save a realm's encryption key (32 bytes).
    ///
    /// Each shared realm has its own symmetric encryption key.
    pub fn save_realm_key(&self, realm_id: &RealmId, key: &[u8; 32]) -> Result<(), SyncError> {
        let db = self.db.read();
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(REALM_KEYS_TABLE)?;
            let realm_key = realm_id.to_base58();
            table.insert(realm_key.as_str(), key.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load a realm's encryption key.
    ///
    /// Returns `None` if the realm is not shared or has no key.
    pub fn load_realm_key(&self, realm_id: &RealmId) -> Result<Option<[u8; 32]>, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(REALM_KEYS_TABLE)?;
        let key = realm_id.to_base58();

        Ok(table.get(key.as_str())?.map(|v| {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(v.value());
            arr
        }))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Identity Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Identity storage key (there's only one identity per node)
    const IDENTITY_KEY: &'static str = "node_identity";

    /// Save the node's identity keypair to storage.
    ///
    /// There is only one identity per node, stored with a fixed key.
    pub fn save_identity(&self, keypair: &crate::identity::HybridKeypair) -> Result<(), SyncError> {
        let db = self.db.read();
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(IDENTITY_TABLE)?;
            let data = keypair.to_bytes();
            table.insert(Self::IDENTITY_KEY, data.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load the node's identity keypair from storage.
    ///
    /// Returns `None` if no identity has been created yet.
    pub fn load_identity(&self) -> Result<Option<crate::identity::HybridKeypair>, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(IDENTITY_TABLE)?;

        match table.get(Self::IDENTITY_KEY)? {
            Some(v) => {
                let keypair = crate::identity::HybridKeypair::from_bytes(v.value())?;
                Ok(Some(keypair))
            }
            None => Ok(None),
        }
    }

    /// Check if an identity exists in storage.
    pub fn has_identity(&self) -> Result<bool, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(IDENTITY_TABLE)?;

        Ok(table.get(Self::IDENTITY_KEY)?.is_some())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Endpoint Secret Key Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Endpoint secret key storage key (there's only one endpoint per node)
    const ENDPOINT_SECRET_KEY: &'static str = "endpoint_secret_key";

    /// Save the endpoint's secret key to storage.
    ///
    /// There is only one endpoint per node, stored with a fixed key.
    /// This ensures stable node identity across restarts.
    pub fn save_endpoint_secret_key(&self, secret_key: &[u8; 32]) -> Result<(), SyncError> {
        let db = self.db.read();
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(ENDPOINT_SECRET_KEY_TABLE)?;
            table.insert(Self::ENDPOINT_SECRET_KEY, secret_key.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load the endpoint's secret key from storage.
    ///
    /// Returns `None` if no endpoint secret key has been created yet.
    pub fn load_endpoint_secret_key(&self) -> Result<Option<[u8; 32]>, SyncError> {
        let db = self.db.read();
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(ENDPOINT_SECRET_KEY_TABLE)?;

        match table.get(Self::ENDPOINT_SECRET_KEY)? {
            Some(v) => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(v.value());
                Ok(Some(arr))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_storage() -> (Storage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.redb");
        let storage = Storage::new(&db_path).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_storage_can_be_created() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.redb");
        let storage = Storage::new(&db_path);
        assert!(storage.is_ok());
    }

    #[test]
    fn test_storage_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("nested/path/to/test.redb");
        let storage = Storage::new(&db_path);
        assert!(storage.is_ok());
        assert!(db_path.exists());
    }

    #[test]
    fn test_save_and_load_realm() {
        let (storage, _temp) = create_test_storage();

        let realm = RealmInfo::new("Test Realm");
        let realm_id = realm.id.clone();

        // Save
        storage.save_realm(&realm).unwrap();

        // Load
        let loaded = storage.load_realm(&realm_id).unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.name, "Test Realm");
        assert_eq!(loaded.id, realm_id);
    }

    #[test]
    fn test_load_nonexistent_realm() {
        let (storage, _temp) = create_test_storage();

        let realm_id = RealmId::new();
        let loaded = storage.load_realm(&realm_id).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_list_realms() {
        let (storage, _temp) = create_test_storage();

        // Save multiple realms
        let realm1 = RealmInfo::new("Realm 1");
        let realm2 = RealmInfo::new("Realm 2");
        let realm3 = RealmInfo::new("Realm 3");

        storage.save_realm(&realm1).unwrap();
        storage.save_realm(&realm2).unwrap();
        storage.save_realm(&realm3).unwrap();

        // List
        let realms = storage.list_realms().unwrap();
        assert_eq!(realms.len(), 3);

        let names: Vec<_> = realms.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"Realm 1"));
        assert!(names.contains(&"Realm 2"));
        assert!(names.contains(&"Realm 3"));
    }

    #[test]
    fn test_delete_realm() {
        let (storage, _temp) = create_test_storage();

        let realm = RealmInfo::new("To Delete");
        let realm_id = realm.id.clone();

        // Save
        storage.save_realm(&realm).unwrap();
        assert!(storage.load_realm(&realm_id).unwrap().is_some());

        // Delete
        storage.delete_realm(&realm_id).unwrap();
        assert!(storage.load_realm(&realm_id).unwrap().is_none());
    }

    #[test]
    fn test_save_and_load_document() {
        let (storage, _temp) = create_test_storage();

        let realm_id = RealmId::new();
        let document_data = b"test document data".to_vec();

        // Save
        storage.save_document(&realm_id, &document_data).unwrap();

        // Load
        let loaded = storage.load_document(&realm_id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), document_data);
    }

    #[test]
    fn test_load_nonexistent_document() {
        let (storage, _temp) = create_test_storage();

        let realm_id = RealmId::new();
        let loaded = storage.load_document(&realm_id).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_save_and_load_realm_key() {
        let (storage, _temp) = create_test_storage();

        let realm_id = RealmId::new();
        let key = [42u8; 32];

        // Save
        storage.save_realm_key(&realm_id, &key).unwrap();

        // Load
        let loaded = storage.load_realm_key(&realm_id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), key);
    }

    #[test]
    fn test_delete_realm_removes_associated_data() {
        let (storage, _temp) = create_test_storage();

        let realm = RealmInfo::new("With Data");
        let realm_id = realm.id.clone();

        // Save realm, document, and key
        storage.save_realm(&realm).unwrap();
        storage.save_document(&realm_id, b"doc data").unwrap();
        storage.save_realm_key(&realm_id, &[1u8; 32]).unwrap();

        // Verify all exist
        assert!(storage.load_realm(&realm_id).unwrap().is_some());
        assert!(storage.load_document(&realm_id).unwrap().is_some());
        assert!(storage.load_realm_key(&realm_id).unwrap().is_some());

        // Delete realm
        storage.delete_realm(&realm_id).unwrap();

        // Verify all removed
        assert!(storage.load_realm(&realm_id).unwrap().is_none());
        assert!(storage.load_document(&realm_id).unwrap().is_none());
        assert!(storage.load_realm_key(&realm_id).unwrap().is_none());
    }

    #[test]
    fn test_save_and_load_identity() {
        use crate::identity::HybridKeypair;
        let (storage, _temp) = create_test_storage();

        // Initially no identity
        assert!(!storage.has_identity().unwrap());
        assert!(storage.load_identity().unwrap().is_none());

        // Generate and save identity
        let keypair = HybridKeypair::generate();
        let public_key = keypair.public_key();
        storage.save_identity(&keypair).unwrap();

        // Verify identity exists
        assert!(storage.has_identity().unwrap());

        // Load identity and verify public key matches
        let loaded = storage.load_identity().unwrap().unwrap();
        assert_eq!(loaded.public_key(), public_key);

        // Verify loaded keypair can sign and verify
        let message = b"test message";
        let signature = loaded.sign(message);
        assert!(public_key.verify(message, &signature));
    }

    #[test]
    fn test_identity_persists_across_instances() {
        use crate::identity::HybridKeypair;
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.redb");

        // Save identity in first storage instance
        let public_key = {
            let storage = Storage::new(&db_path).unwrap();
            let keypair = HybridKeypair::generate();
            let pk = keypair.public_key();
            storage.save_identity(&keypair).unwrap();
            pk
        };

        // Load identity in second storage instance
        {
            let storage = Storage::new(&db_path).unwrap();
            let loaded = storage.load_identity().unwrap().unwrap();
            assert_eq!(loaded.public_key(), public_key);
        }
    }

    #[test]
    fn test_save_and_load_endpoint_secret_key() {
        let (storage, _temp) = create_test_storage();

        // Initially no endpoint secret key
        assert!(storage.load_endpoint_secret_key().unwrap().is_none());

        // Generate and save endpoint secret key
        let secret_key = [42u8; 32];
        storage.save_endpoint_secret_key(&secret_key).unwrap();

        // Load endpoint secret key and verify it matches
        let loaded = storage.load_endpoint_secret_key().unwrap().unwrap();
        assert_eq!(loaded, secret_key);
    }

    #[test]
    fn test_endpoint_secret_key_persists_across_instances() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.redb");

        // Save endpoint secret key in first storage instance
        let secret_key = [137u8; 32];
        {
            let storage = Storage::new(&db_path).unwrap();
            storage.save_endpoint_secret_key(&secret_key).unwrap();
        }

        // Load endpoint secret key in second storage instance
        {
            let storage = Storage::new(&db_path).unwrap();
            let loaded = storage.load_endpoint_secret_key().unwrap().unwrap();
            assert_eq!(loaded, secret_key);
        }
    }

    #[test]
    fn test_endpoint_secret_key_can_be_overwritten() {
        let (storage, _temp) = create_test_storage();

        // Save first key
        let key1 = [1u8; 32];
        storage.save_endpoint_secret_key(&key1).unwrap();
        assert_eq!(storage.load_endpoint_secret_key().unwrap().unwrap(), key1);

        // Overwrite with second key
        let key2 = [2u8; 32];
        storage.save_endpoint_secret_key(&key2).unwrap();
        assert_eq!(storage.load_endpoint_secret_key().unwrap().unwrap(), key2);
    }
}
