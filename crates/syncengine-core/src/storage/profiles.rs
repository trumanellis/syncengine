//! Profile Storage - CRUD operations for user profiles
//!
//! Stores profile data in redb with peer_id as the key.

use crate::error::SyncError;
use crate::types::UserProfile;
use redb::{ReadableTable, TableDefinition};

use super::Storage;

/// Table for storing user profiles (key: peer_id string, value: serialized UserProfile)
pub(crate) const PROFILES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("profiles");

impl Storage {
    /// Save a profile to the database
    ///
    /// If a profile with the same peer_id exists, it will be overwritten.
    pub fn save_profile(&self, profile: &UserProfile) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(PROFILES_TABLE)?;
            let serialized = postcard::to_allocvec(profile)
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            table.insert(profile.peer_id.as_str(), serialized.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load a profile by peer ID
    ///
    /// Returns `None` if no profile exists for the given peer.
    pub fn load_profile(&self, peer_id: &str) -> Result<Option<UserProfile>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(PROFILES_TABLE)?;

        if let Some(data) = table.get(peer_id)? {
            let profile: UserProfile = postcard::from_bytes(data.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            Ok(Some(profile))
        } else {
            Ok(None)
        }
    }

    /// Delete a profile by peer ID
    ///
    /// Returns `Ok(())` even if the profile doesn't exist.
    pub fn delete_profile(&self, peer_id: &str) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(PROFILES_TABLE)?;
            table.remove(peer_id)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// List all profiles in the database
    ///
    /// Returns a vector of all stored profiles.
    pub fn list_profiles(&self) -> Result<Vec<UserProfile>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(PROFILES_TABLE)?;

        let mut profiles = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let profile: UserProfile = postcard::from_bytes(value.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            profiles.push(profile);
        }

        Ok(profiles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_profile() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let profile = UserProfile::new("peer123".to_string(), "Love".to_string());

        // Save
        storage.save_profile(&profile).unwrap();

        // Load
        let loaded = storage.load_profile("peer123").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().display_name, "Love");
    }

    #[test]
    fn test_load_nonexistent_profile() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let loaded = storage.load_profile("nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_delete_profile() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let profile = UserProfile::new("peer456".to_string(), "Joy".to_string());
        storage.save_profile(&profile).unwrap();

        // Verify it exists
        assert!(storage.load_profile("peer456").unwrap().is_some());

        // Delete
        storage.delete_profile("peer456").unwrap();

        // Verify it's gone
        assert!(storage.load_profile("peer456").unwrap().is_none());
    }

    #[test]
    fn test_list_profiles() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        // Create multiple profiles
        let profile1 = UserProfile::new("peer1".to_string(), "Love".to_string());
        let profile2 = UserProfile::new("peer2".to_string(), "Joy".to_string());
        let profile3 = UserProfile::new("peer3".to_string(), "Charlie".to_string());

        storage.save_profile(&profile1).unwrap();
        storage.save_profile(&profile2).unwrap();
        storage.save_profile(&profile3).unwrap();

        // List all
        let profiles = storage.list_profiles().unwrap();
        assert_eq!(profiles.len(), 3);

        let names: Vec<String> = profiles.iter().map(|p| p.display_name.clone()).collect();
        assert!(names.contains(&"Love".to_string()));
        assert!(names.contains(&"Joy".to_string()));
        assert!(names.contains(&"Charlie".to_string()));
    }

    #[test]
    fn test_overwrite_profile() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let mut profile = UserProfile::new("peer789".to_string(), "Original".to_string());
        storage.save_profile(&profile).unwrap();

        // Update and save again
        profile.display_name = "Updated".to_string();
        storage.save_profile(&profile).unwrap();

        // Load and verify
        let loaded = storage.load_profile("peer789").unwrap().unwrap();
        assert_eq!(loaded.display_name, "Updated");
    }
}
