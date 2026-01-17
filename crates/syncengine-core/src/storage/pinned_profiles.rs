//! Pinned Profiles Storage - CRUD operations for profile pins
//!
//! Stores pinned profile data in redb with DID as the key.
//! Profile pins provide P2P redundancy by allowing nodes to serve
//! profiles for peers who may be offline.

use crate::error::SyncError;
use crate::types::{PinRelationship, ProfilePin};
use redb::{ReadableTable, TableDefinition};

use super::Storage;

/// Table for storing pinned profiles (key: DID string, value: serialized ProfilePin)
pub(crate) const PINNED_PROFILES_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("pinned_profiles");

/// Configuration for profile pinning storage limits
#[derive(Debug, Clone)]
pub struct PinningConfig {
    /// Maximum number of pinned profiles (excluding own)
    pub max_pins: usize,
    /// Maximum total avatar storage in bytes
    pub max_avatar_bytes: usize,
}

impl Default for PinningConfig {
    fn default() -> Self {
        Self {
            max_pins: 100,
            max_avatar_bytes: 5 * 1024 * 1024, // 5MB
        }
    }
}

impl Storage {
    /// Save a pinned profile to the database.
    ///
    /// If a pin with the same DID already exists, it will be overwritten.
    /// This does NOT enforce storage limits - use `save_pinned_profile_with_limits`
    /// for automatic eviction.
    pub fn save_pinned_profile(&self, pin: &ProfilePin) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(PINNED_PROFILES_TABLE)?;
            let serialized =
                postcard::to_allocvec(pin).map_err(|e| SyncError::Serialization(e.to_string()))?;
            table.insert(pin.did.as_str(), serialized.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load a pinned profile by DID.
    ///
    /// Returns `None` if no pin exists for the given DID.
    pub fn load_pinned_profile(&self, did: &str) -> Result<Option<ProfilePin>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(PINNED_PROFILES_TABLE)?;

        if let Some(data) = table.get(did)? {
            let pin: ProfilePin = postcard::from_bytes(data.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            Ok(Some(pin))
        } else {
            Ok(None)
        }
    }

    /// Delete a pinned profile by DID.
    ///
    /// Returns `Ok(())` even if the pin doesn't exist.
    pub fn delete_pinned_profile(&self, did: &str) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(PINNED_PROFILES_TABLE)?;
            table.remove(did)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// List all pinned profiles.
    ///
    /// Returns a vector of all stored profile pins.
    pub fn list_pinned_profiles(&self) -> Result<Vec<ProfilePin>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(PINNED_PROFILES_TABLE)?;

        let mut pins = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let pin: ProfilePin = postcard::from_bytes(value.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            pins.push(pin);
        }

        Ok(pins)
    }

    /// List pinned profiles filtered by relationship type.
    pub fn list_pinned_profiles_by_relationship(
        &self,
        relationship: &PinRelationship,
    ) -> Result<Vec<ProfilePin>, SyncError> {
        let all_pins = self.list_pinned_profiles()?;
        let filtered = all_pins
            .into_iter()
            .filter(|pin| {
                match (&pin.relationship, relationship) {
                    (PinRelationship::Own, PinRelationship::Own) => true,
                    (PinRelationship::Contact, PinRelationship::Contact) => true,
                    (PinRelationship::Manual, PinRelationship::Manual) => true,
                    (
                        PinRelationship::RealmMember { .. },
                        PinRelationship::RealmMember { .. },
                    ) => true, // Any realm member matches
                    _ => false,
                }
            })
            .collect();
        Ok(filtered)
    }

    /// Count the number of pinned profiles (excluding own).
    pub fn count_pinned_profiles(&self) -> Result<usize, SyncError> {
        let pins = self.list_pinned_profiles()?;
        Ok(pins.iter().filter(|p| !p.is_own()).count())
    }

    /// Get the pinned profile count by relationship type.
    pub fn count_pins_by_relationship(
        &self,
        relationship: &PinRelationship,
    ) -> Result<usize, SyncError> {
        let pins = self.list_pinned_profiles_by_relationship(relationship)?;
        Ok(pins.len())
    }

    /// Save a pinned profile with automatic eviction if limits are exceeded.
    ///
    /// When storage limits are reached, this will evict the lowest-priority
    /// pins (based on relationship and pinned_at timestamp) to make room.
    ///
    /// Returns the DIDs of any evicted profiles.
    pub fn save_pinned_profile_with_limits(
        &self,
        pin: &ProfilePin,
        config: &PinningConfig,
    ) -> Result<Vec<String>, SyncError> {
        let mut evicted = Vec::new();

        // Own profile is never evicted and doesn't count against limits
        if pin.is_own() {
            self.save_pinned_profile(pin)?;
            return Ok(evicted);
        }

        // Check if we need to evict
        let current_count = self.count_pinned_profiles()?;
        if current_count >= config.max_pins {
            // Need to evict lowest priority pins
            evicted = self.evict_lowest_priority_pins(1)?;
        }

        self.save_pinned_profile(pin)?;
        Ok(evicted)
    }

    /// Evict the N lowest priority pinned profiles.
    ///
    /// Returns the DIDs of evicted profiles.
    /// Never evicts Own profiles.
    fn evict_lowest_priority_pins(&self, count: usize) -> Result<Vec<String>, SyncError> {
        let mut pins = self.list_pinned_profiles()?;

        // Filter out own profiles and sort by priority (lowest first), then by pinned_at (oldest first)
        pins.retain(|p| !p.is_own());
        pins.sort_by(|a, b| {
            a.priority()
                .cmp(&b.priority())
                .then_with(|| a.pinned_at.cmp(&b.pinned_at))
        });

        let mut evicted = Vec::new();
        for pin in pins.into_iter().take(count) {
            self.delete_pinned_profile(&pin.did)?;
            evicted.push(pin.did);
        }

        Ok(evicted)
    }

    /// Get own pinned profile (if exists).
    pub fn get_own_pinned_profile(&self) -> Result<Option<ProfilePin>, SyncError> {
        let pins = self.list_pinned_profiles_by_relationship(&PinRelationship::Own)?;
        Ok(pins.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::HybridKeypair;
    use crate::types::{RealmId, SignedProfile, UserProfile};
    use tempfile::tempdir;

    fn create_test_storage() -> Storage {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        Storage::new(&db_path).unwrap()
    }

    fn create_test_pin(did: &str, name: &str, relationship: PinRelationship) -> ProfilePin {
        let keypair = HybridKeypair::generate();
        let profile = UserProfile::new(did.to_string(), name.to_string());
        let signed = SignedProfile::sign(&profile, &keypair);
        ProfilePin::new(did.to_string(), signed, relationship)
    }

    #[test]
    fn test_save_and_load_pinned_profile() {
        let storage = create_test_storage();
        let pin = create_test_pin("did:sync:alice", "Alice", PinRelationship::Contact);

        storage.save_pinned_profile(&pin).unwrap();

        let loaded = storage.load_pinned_profile("did:sync:alice").unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.did, "did:sync:alice");
        assert_eq!(loaded.signed_profile.profile.display_name, "Alice");
    }

    #[test]
    fn test_load_nonexistent_pinned_profile() {
        let storage = create_test_storage();
        let loaded = storage.load_pinned_profile("did:sync:nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_delete_pinned_profile() {
        let storage = create_test_storage();
        let pin = create_test_pin("did:sync:bob", "Bob", PinRelationship::Contact);

        storage.save_pinned_profile(&pin).unwrap();
        assert!(storage.load_pinned_profile("did:sync:bob").unwrap().is_some());

        storage.delete_pinned_profile("did:sync:bob").unwrap();
        assert!(storage.load_pinned_profile("did:sync:bob").unwrap().is_none());
    }

    #[test]
    fn test_list_pinned_profiles() {
        let storage = create_test_storage();

        let pin1 = create_test_pin("did:sync:alice", "Alice", PinRelationship::Contact);
        let pin2 = create_test_pin("did:sync:bob", "Bob", PinRelationship::Contact);
        let pin3 = create_test_pin("did:sync:charlie", "Charlie", PinRelationship::Manual);

        storage.save_pinned_profile(&pin1).unwrap();
        storage.save_pinned_profile(&pin2).unwrap();
        storage.save_pinned_profile(&pin3).unwrap();

        let pins = storage.list_pinned_profiles().unwrap();
        assert_eq!(pins.len(), 3);
    }

    #[test]
    fn test_list_by_relationship() {
        let storage = create_test_storage();
        let realm_id = RealmId::new();

        let pin1 = create_test_pin("did:sync:alice", "Alice", PinRelationship::Contact);
        let pin2 = create_test_pin("did:sync:bob", "Bob", PinRelationship::Contact);
        let pin3 = create_test_pin(
            "did:sync:charlie",
            "Charlie",
            PinRelationship::RealmMember {
                realm_id: realm_id.clone(),
            },
        );
        let pin4 = create_test_pin("did:sync:dave", "Dave", PinRelationship::Manual);

        storage.save_pinned_profile(&pin1).unwrap();
        storage.save_pinned_profile(&pin2).unwrap();
        storage.save_pinned_profile(&pin3).unwrap();
        storage.save_pinned_profile(&pin4).unwrap();

        // List contacts only
        let contacts = storage
            .list_pinned_profiles_by_relationship(&PinRelationship::Contact)
            .unwrap();
        assert_eq!(contacts.len(), 2);

        // List realm members
        let realm_members = storage
            .list_pinned_profiles_by_relationship(&PinRelationship::RealmMember { realm_id })
            .unwrap();
        assert_eq!(realm_members.len(), 1);

        // List manual pins
        let manual = storage
            .list_pinned_profiles_by_relationship(&PinRelationship::Manual)
            .unwrap();
        assert_eq!(manual.len(), 1);
    }

    #[test]
    fn test_count_pinned_profiles_excludes_own() {
        let storage = create_test_storage();

        let own_pin = create_test_pin("did:sync:me", "Me", PinRelationship::Own);
        let contact_pin = create_test_pin("did:sync:alice", "Alice", PinRelationship::Contact);

        storage.save_pinned_profile(&own_pin).unwrap();
        storage.save_pinned_profile(&contact_pin).unwrap();

        // Count should exclude own
        let count = storage.count_pinned_profiles().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_save_with_limits_evicts_lowest_priority() {
        let storage = create_test_storage();
        let config = PinningConfig {
            max_pins: 2,
            max_avatar_bytes: 5 * 1024 * 1024,
        };

        // Add 2 pins to fill the limit
        let pin1 = create_test_pin("did:sync:alice", "Alice", PinRelationship::Contact);
        let pin2 = create_test_pin("did:sync:bob", "Bob", PinRelationship::Manual);

        storage
            .save_pinned_profile_with_limits(&pin1, &config)
            .unwrap();
        storage
            .save_pinned_profile_with_limits(&pin2, &config)
            .unwrap();

        assert_eq!(storage.count_pinned_profiles().unwrap(), 2);

        // Add a third pin - should evict the lowest priority (Manual)
        let pin3 = create_test_pin("did:sync:charlie", "Charlie", PinRelationship::Contact);
        let evicted = storage
            .save_pinned_profile_with_limits(&pin3, &config)
            .unwrap();

        assert_eq!(evicted.len(), 1);
        assert_eq!(evicted[0], "did:sync:bob"); // Manual was evicted

        // Bob should be gone, Alice and Charlie should remain
        assert!(storage
            .load_pinned_profile("did:sync:bob")
            .unwrap()
            .is_none());
        assert!(storage
            .load_pinned_profile("did:sync:alice")
            .unwrap()
            .is_some());
        assert!(storage
            .load_pinned_profile("did:sync:charlie")
            .unwrap()
            .is_some());
    }

    #[test]
    fn test_own_profile_not_counted_against_limits() {
        let storage = create_test_storage();
        let config = PinningConfig {
            max_pins: 1,
            max_avatar_bytes: 5 * 1024 * 1024,
        };

        // Add own profile (doesn't count against limit)
        let own_pin = create_test_pin("did:sync:me", "Me", PinRelationship::Own);
        storage
            .save_pinned_profile_with_limits(&own_pin, &config)
            .unwrap();

        // Add a contact (counts against limit)
        let contact_pin = create_test_pin("did:sync:alice", "Alice", PinRelationship::Contact);
        let evicted = storage
            .save_pinned_profile_with_limits(&contact_pin, &config)
            .unwrap();

        assert!(evicted.is_empty()); // Nothing evicted
        assert_eq!(storage.list_pinned_profiles().unwrap().len(), 2);
    }

    #[test]
    fn test_get_own_pinned_profile() {
        let storage = create_test_storage();

        let own_pin = create_test_pin("did:sync:me", "Me", PinRelationship::Own);
        let contact_pin = create_test_pin("did:sync:alice", "Alice", PinRelationship::Contact);

        storage.save_pinned_profile(&own_pin).unwrap();
        storage.save_pinned_profile(&contact_pin).unwrap();

        let own = storage.get_own_pinned_profile().unwrap();
        assert!(own.is_some());
        assert_eq!(own.unwrap().did, "did:sync:me");
    }

    #[test]
    fn test_overwrite_pinned_profile() {
        let storage = create_test_storage();

        let pin1 = create_test_pin("did:sync:alice", "Alice v1", PinRelationship::Contact);
        storage.save_pinned_profile(&pin1).unwrap();

        let pin2 = create_test_pin("did:sync:alice", "Alice v2", PinRelationship::Contact);
        storage.save_pinned_profile(&pin2).unwrap();

        let loaded = storage.load_pinned_profile("did:sync:alice").unwrap().unwrap();
        assert_eq!(loaded.signed_profile.profile.display_name, "Alice v2");
    }
}
