//! Blob Storage - Content-addressed image storage
//!
//! Stores image blobs in redb with content hashes as keys.
//! Uses BLAKE3 for content addressing (same as Iroh).

use crate::error::SyncError;
use redb::TableDefinition;

use super::Storage;

/// Table for storing image blobs (key: BLAKE3 hash hex string, value: raw bytes)
pub(crate) const BLOBS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("blobs");

impl Storage {
    /// Save image blob and return its content hash
    ///
    /// Uses BLAKE3 for content addressing. If the blob already exists,
    /// returns the existing hash without re-storing.
    pub fn save_image_blob(&self, data: Vec<u8>) -> Result<String, SyncError> {
        // Compute content hash (BLAKE3)
        let hash = blake3::hash(&data);
        let hash_hex = hash.to_hex().to_string();

        let db = self.db_handle();
        let db_guard = db.read();

        // Check if blob already exists (content-addressed deduplication)
        {
            let read_txn = db_guard.begin_read()?;
            let table = read_txn.open_table(BLOBS_TABLE)?;
            if table.get(hash_hex.as_str())?.is_some() {
                // Already exists, return hash
                return Ok(hash_hex);
            }
        }

        // Store new blob
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(BLOBS_TABLE)?;
            table.insert(hash_hex.as_str(), data.as_slice())?;
        }
        write_txn.commit()?;

        Ok(hash_hex)
    }

    /// Load image blob by content hash
    ///
    /// Returns `None` if the blob doesn't exist.
    pub fn load_image_blob(&self, hash_hex: &str) -> Result<Option<Vec<u8>>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(BLOBS_TABLE)?;

        if let Some(data) = table.get(hash_hex)? {
            Ok(Some(data.value().to_vec()))
        } else {
            Ok(None)
        }
    }

    /// Delete a blob by hash
    ///
    /// Returns `Ok(())` even if the blob doesn't exist.
    /// Note: Since blobs are content-addressed, deleting may affect
    /// multiple references if they share the same content.
    pub fn delete_image_blob(&self, hash_hex: &str) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(BLOBS_TABLE)?;
            table.remove(hash_hex)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Check if a blob exists by hash
    pub fn blob_exists(&self, hash_hex: &str) -> Result<bool, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(BLOBS_TABLE)?;

        Ok(table.get(hash_hex)?.is_some())
    }

    /// Get the size of a blob in bytes
    pub fn blob_size(&self, hash_hex: &str) -> Result<Option<usize>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(BLOBS_TABLE)?;

        if let Some(data) = table.get(hash_hex)? {
            Ok(Some(data.value().len()))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_blob() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let data = b"Hello, world!".to_vec();

        // Save
        let hash = storage.save_image_blob(data.clone()).unwrap();
        assert!(!hash.is_empty());

        // Load
        let loaded = storage.load_image_blob(&hash).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), data);
    }

    #[test]
    fn test_content_addressing() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let data = b"Same content".to_vec();

        // Save twice - should return same hash
        let hash1 = storage.save_image_blob(data.clone()).unwrap();
        let hash2 = storage.save_image_blob(data.clone()).unwrap();

        assert_eq!(hash1, hash2);

        // Should only be stored once (verify by loading)
        let loaded = storage.load_image_blob(&hash1).unwrap();
        assert!(loaded.is_some());
    }

    #[test]
    fn test_different_content_different_hash() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let data1 = b"Content A".to_vec();
        let data2 = b"Content B".to_vec();

        let hash1 = storage.save_image_blob(data1).unwrap();
        let hash2 = storage.save_image_blob(data2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_blob_exists() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let data = b"Test data".to_vec();
        let hash = storage.save_image_blob(data).unwrap();

        assert!(storage.blob_exists(&hash).unwrap());
        assert!(!storage.blob_exists("nonexistent_hash").unwrap());
    }

    #[test]
    fn test_blob_size() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let data = b"Test data with known size".to_vec();
        let hash = storage.save_image_blob(data.clone()).unwrap();

        let size = storage.blob_size(&hash).unwrap();
        assert_eq!(size, Some(data.len()));

        let nonexistent_size = storage.blob_size("nonexistent").unwrap();
        assert_eq!(nonexistent_size, None);
    }

    #[test]
    fn test_delete_blob() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let data = b"Delete me".to_vec();
        let hash = storage.save_image_blob(data).unwrap();

        // Verify it exists
        assert!(storage.blob_exists(&hash).unwrap());

        // Delete
        storage.delete_image_blob(&hash).unwrap();

        // Verify it's gone
        assert!(!storage.blob_exists(&hash).unwrap());
    }

    #[test]
    fn test_load_nonexistent_blob() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let loaded = storage.load_image_blob("nonexistent").unwrap();
        assert!(loaded.is_none());
    }
}
