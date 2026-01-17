//! Profile Pinners Storage - Track peers who are pinning OUR profile
//!
//! This is the reverse index of pinned_profiles. While pinned_profiles tracks
//! profiles WE pin, profile_pinners tracks peers who pin US.
//!
//! This enables the "Souls Carrying Your Light" feature in the Network page,
//! showing users who is caching their profile for P2P redundancy.

use crate::error::SyncError;
use redb::{ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};

use super::Storage;

/// Table for storing profile pinners (key: pinner DID, value: serialized PinnerInfo)
pub(crate) const PROFILE_PINNERS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("profile_pinners");

/// Information about a peer who is pinning our profile.
///
/// This is received via PinAcknowledgment gossip messages when peers
/// decide to pin our profile (e.g., after becoming contacts or joining
/// a shared realm).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PinnerInfo {
    /// DID of the peer who is pinning our profile
    pub pinner_did: String,
    /// Unix timestamp when they started pinning
    pub pinned_at: i64,
    /// Relationship type: "contact", "realm_member", "manual"
    pub relationship: String,
    /// Unix timestamp when we last received confirmation they're still pinning
    pub last_confirmed: i64,
    /// Optional display name (if we have it from their profile)
    pub display_name: Option<String>,
}

impl PinnerInfo {
    /// Create a new PinnerInfo from a received PinAcknowledgment.
    pub fn new(pinner_did: String, pinned_at: i64, relationship: String) -> Self {
        Self {
            pinner_did,
            pinned_at,
            relationship,
            last_confirmed: chrono::Utc::now().timestamp(),
            display_name: None,
        }
    }

    /// Update the last_confirmed timestamp.
    pub fn confirm(&mut self) {
        self.last_confirmed = chrono::Utc::now().timestamp();
    }

    /// Set the display name.
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Check if this is a contact relationship.
    pub fn is_contact(&self) -> bool {
        self.relationship == "contact"
    }

    /// Check if this is a realm member relationship.
    pub fn is_realm_member(&self) -> bool {
        self.relationship.starts_with("realm_member")
    }
}

impl Storage {
    /// Save a pinner record to the database.
    ///
    /// Called when we receive a PinAcknowledgment message from a peer.
    pub fn save_pinner(&self, pinner: &PinnerInfo) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(PROFILE_PINNERS_TABLE)?;
            let serialized = postcard::to_allocvec(pinner)
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            table.insert(pinner.pinner_did.as_str(), serialized.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load a pinner by their DID.
    pub fn load_pinner(&self, pinner_did: &str) -> Result<Option<PinnerInfo>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(PROFILE_PINNERS_TABLE)?;

        if let Some(data) = table.get(pinner_did)? {
            let pinner: PinnerInfo = postcard::from_bytes(data.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            Ok(Some(pinner))
        } else {
            Ok(None)
        }
    }

    /// Delete a pinner record (when they unpin our profile).
    pub fn delete_pinner(&self, pinner_did: &str) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(PROFILE_PINNERS_TABLE)?;
            table.remove(pinner_did)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// List all peers who are pinning our profile.
    ///
    /// Returns a vector sorted by pinned_at (newest first).
    pub fn list_pinners(&self) -> Result<Vec<PinnerInfo>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(PROFILE_PINNERS_TABLE)?;

        let mut pinners = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let pinner: PinnerInfo = postcard::from_bytes(value.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            pinners.push(pinner);
        }

        // Sort by pinned_at descending (newest first)
        pinners.sort_by(|a, b| b.pinned_at.cmp(&a.pinned_at));

        Ok(pinners)
    }

    /// List pinners filtered by relationship type.
    pub fn list_pinners_by_relationship(&self, relationship: &str) -> Result<Vec<PinnerInfo>, SyncError> {
        let all_pinners = self.list_pinners()?;
        Ok(all_pinners
            .into_iter()
            .filter(|p| p.relationship == relationship)
            .collect())
    }

    /// Count the number of peers pinning our profile.
    pub fn count_pinners(&self) -> Result<usize, SyncError> {
        Ok(self.list_pinners()?.len())
    }

    /// Update or create a pinner record.
    ///
    /// If the pinner already exists, updates last_confirmed.
    /// If not, creates a new record.
    pub fn upsert_pinner(
        &self,
        pinner_did: &str,
        pinned_at: i64,
        relationship: &str,
    ) -> Result<(), SyncError> {
        if let Some(mut existing) = self.load_pinner(pinner_did)? {
            // Update existing pinner
            existing.confirm();
            // Update relationship if it changed (e.g., realm_member -> contact)
            if existing.relationship != relationship {
                existing.relationship = relationship.to_string();
            }
            self.save_pinner(&existing)
        } else {
            // Create new pinner
            let pinner = PinnerInfo::new(
                pinner_did.to_string(),
                pinned_at,
                relationship.to_string(),
            );
            self.save_pinner(&pinner)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_storage() -> Storage {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        Storage::new(&db_path).unwrap()
    }

    #[test]
    fn test_save_and_load_pinner() {
        let storage = create_test_storage();
        let pinner = PinnerInfo::new(
            "did:sync:alice".to_string(),
            1234567890,
            "contact".to_string(),
        );

        storage.save_pinner(&pinner).unwrap();

        let loaded = storage.load_pinner("did:sync:alice").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.pinner_did, "did:sync:alice");
        assert_eq!(loaded.pinned_at, 1234567890);
        assert_eq!(loaded.relationship, "contact");
    }

    #[test]
    fn test_load_nonexistent_pinner() {
        let storage = create_test_storage();
        let loaded = storage.load_pinner("did:sync:nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_delete_pinner() {
        let storage = create_test_storage();
        let pinner = PinnerInfo::new(
            "did:sync:bob".to_string(),
            1234567890,
            "contact".to_string(),
        );

        storage.save_pinner(&pinner).unwrap();
        assert!(storage.load_pinner("did:sync:bob").unwrap().is_some());

        storage.delete_pinner("did:sync:bob").unwrap();
        assert!(storage.load_pinner("did:sync:bob").unwrap().is_none());
    }

    #[test]
    fn test_list_pinners() {
        let storage = create_test_storage();

        let pinner1 = PinnerInfo::new("did:sync:alice".to_string(), 100, "contact".to_string());
        let pinner2 = PinnerInfo::new("did:sync:bob".to_string(), 200, "contact".to_string());
        let pinner3 = PinnerInfo::new("did:sync:charlie".to_string(), 150, "realm_member".to_string());

        storage.save_pinner(&pinner1).unwrap();
        storage.save_pinner(&pinner2).unwrap();
        storage.save_pinner(&pinner3).unwrap();

        let pinners = storage.list_pinners().unwrap();
        assert_eq!(pinners.len(), 3);

        // Should be sorted by pinned_at descending
        assert_eq!(pinners[0].pinner_did, "did:sync:bob"); // 200
        assert_eq!(pinners[1].pinner_did, "did:sync:charlie"); // 150
        assert_eq!(pinners[2].pinner_did, "did:sync:alice"); // 100
    }

    #[test]
    fn test_list_pinners_by_relationship() {
        let storage = create_test_storage();

        let pinner1 = PinnerInfo::new("did:sync:alice".to_string(), 100, "contact".to_string());
        let pinner2 = PinnerInfo::new("did:sync:bob".to_string(), 200, "contact".to_string());
        let pinner3 = PinnerInfo::new("did:sync:charlie".to_string(), 150, "realm_member".to_string());

        storage.save_pinner(&pinner1).unwrap();
        storage.save_pinner(&pinner2).unwrap();
        storage.save_pinner(&pinner3).unwrap();

        let contacts = storage.list_pinners_by_relationship("contact").unwrap();
        assert_eq!(contacts.len(), 2);

        let realm_members = storage.list_pinners_by_relationship("realm_member").unwrap();
        assert_eq!(realm_members.len(), 1);
    }

    #[test]
    fn test_count_pinners() {
        let storage = create_test_storage();

        assert_eq!(storage.count_pinners().unwrap(), 0);

        let pinner1 = PinnerInfo::new("did:sync:alice".to_string(), 100, "contact".to_string());
        let pinner2 = PinnerInfo::new("did:sync:bob".to_string(), 200, "contact".to_string());

        storage.save_pinner(&pinner1).unwrap();
        assert_eq!(storage.count_pinners().unwrap(), 1);

        storage.save_pinner(&pinner2).unwrap();
        assert_eq!(storage.count_pinners().unwrap(), 2);
    }

    #[test]
    fn test_upsert_pinner_creates_new() {
        let storage = create_test_storage();

        storage
            .upsert_pinner("did:sync:alice", 1234567890, "contact")
            .unwrap();

        let loaded = storage.load_pinner("did:sync:alice").unwrap().unwrap();
        assert_eq!(loaded.pinner_did, "did:sync:alice");
        assert_eq!(loaded.pinned_at, 1234567890);
    }

    #[test]
    fn test_upsert_pinner_updates_existing() {
        let storage = create_test_storage();

        // Create initial pinner
        storage
            .upsert_pinner("did:sync:alice", 1234567890, "realm_member")
            .unwrap();

        let original = storage.load_pinner("did:sync:alice").unwrap().unwrap();
        let original_confirmed = original.last_confirmed;

        // Small delay to ensure timestamp changes
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Upsert with new relationship
        storage
            .upsert_pinner("did:sync:alice", 1234567890, "contact")
            .unwrap();

        let updated = storage.load_pinner("did:sync:alice").unwrap().unwrap();
        assert_eq!(updated.relationship, "contact"); // Relationship updated
        assert!(updated.last_confirmed >= original_confirmed); // Confirmed updated
    }

    #[test]
    fn test_pinner_info_helpers() {
        let pinner = PinnerInfo::new("did:sync:alice".to_string(), 100, "contact".to_string());
        assert!(pinner.is_contact());
        assert!(!pinner.is_realm_member());

        let realm_pinner =
            PinnerInfo::new("did:sync:bob".to_string(), 100, "realm_member:abc123".to_string());
        assert!(!realm_pinner.is_contact());
        assert!(realm_pinner.is_realm_member());
    }

    #[test]
    fn test_pinner_info_with_display_name() {
        let pinner = PinnerInfo::new("did:sync:alice".to_string(), 100, "contact".to_string())
            .with_display_name("Alice");

        assert_eq!(pinner.display_name, Some("Alice".to_string()));
    }
}
