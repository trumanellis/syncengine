//! Contact Storage - CRUD operations for contact exchange system
//!
//! Manages persistent storage for:
//! - Accepted contacts (ContactInfo)
//! - Pending contact requests (PendingContact)
//! - Revoked invite IDs

use crate::error::SyncError;
use crate::types::contact::{ContactInfo, ContactState, ContactStatus, PendingContact};
use redb::{ReadableTable, TableDefinition};

use super::Storage;

/// Table for accepted contacts (key: peer_did string, value: serialized ContactInfo)
pub(crate) const CONTACTS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("contacts");

/// Table for pending contacts (key: hex invite_id, value: serialized PendingContact)
pub(crate) const PENDING_CONTACTS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("pending_contacts");

/// Table for revoked invites (key: hex invite_id, value: timestamp bytes)
pub(crate) const REVOKED_INVITES_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("revoked_invites");

/// Table for invites we generated (key: hex invite_id, value: timestamp bytes)
/// Used to auto-accept incoming requests that use our invites
pub(crate) const GENERATED_INVITES_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("generated_invites");

impl Storage {
    // ═══════════════════════════════════════════════════════════════════════
    // Contact Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Save a contact to the database
    ///
    /// If a contact with the same peer_did exists, it will be overwritten.
    pub fn save_contact(&self, contact: &ContactInfo) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(CONTACTS_TABLE)?;
            let serialized = postcard::to_allocvec(contact)
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            table.insert(contact.peer_did.as_str(), serialized.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load a contact by peer DID
    ///
    /// Returns `None` if no contact exists for the given DID.
    pub fn load_contact(&self, did: &str) -> Result<Option<ContactInfo>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(CONTACTS_TABLE)?;

        if let Some(data) = table.get(did)? {
            let contact: ContactInfo = postcard::from_bytes(data.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            Ok(Some(contact))
        } else {
            Ok(None)
        }
    }

    /// Delete a contact by peer DID
    ///
    /// Returns `Ok(())` even if the contact doesn't exist.
    pub fn delete_contact(&self, did: &str) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(CONTACTS_TABLE)?;
            table.remove(did)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// List all contacts in the database
    ///
    /// Returns a vector of all stored contacts.
    pub fn list_contacts(&self) -> Result<Vec<ContactInfo>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(CONTACTS_TABLE)?;

        let mut contacts = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let contact: ContactInfo = postcard::from_bytes(value.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            contacts.push(contact);
        }

        Ok(contacts)
    }

    /// List only online contacts
    ///
    /// Returns contacts where `status == ContactStatus::Online`.
    pub fn list_online_contacts(&self) -> Result<Vec<ContactInfo>, SyncError> {
        Ok(self
            .list_contacts()?
            .into_iter()
            .filter(|c| c.status == ContactStatus::Online)
            .collect())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Pending Contact Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Save a pending contact request
    ///
    /// If a pending contact with the same invite_id exists, it will be overwritten.
    pub fn save_pending(&self, pending: &PendingContact) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(PENDING_CONTACTS_TABLE)?;
            let serialized = postcard::to_allocvec(pending)
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            let key = hex::encode(&pending.invite_id);
            table.insert(key.as_str(), serialized.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Load a pending contact by invite ID
    ///
    /// Returns `None` if no pending contact exists for the given invite ID.
    pub fn load_pending(&self, invite_id: &[u8; 16]) -> Result<Option<PendingContact>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(PENDING_CONTACTS_TABLE)?;
        let key = hex::encode(invite_id);

        if let Some(data) = table.get(key.as_str())? {
            let pending: PendingContact = postcard::from_bytes(data.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            Ok(Some(pending))
        } else {
            Ok(None)
        }
    }

    /// Delete a pending contact by invite ID
    ///
    /// Returns `Ok(())` even if the pending contact doesn't exist.
    pub fn delete_pending(&self, invite_id: &[u8; 16]) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(PENDING_CONTACTS_TABLE)?;
            let key = hex::encode(invite_id);
            table.remove(key.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// List all incoming pending contacts
    ///
    /// Returns pending contacts where `state == ContactState::IncomingPending`.
    pub fn list_incoming_pending(&self) -> Result<Vec<PendingContact>, SyncError> {
        Ok(self
            .list_all_pending()?
            .into_iter()
            .filter(|p| p.state == ContactState::IncomingPending)
            .collect())
    }

    /// List all outgoing pending contacts
    ///
    /// Returns pending contacts where `state == ContactState::OutgoingPending`.
    pub fn list_outgoing_pending(&self) -> Result<Vec<PendingContact>, SyncError> {
        Ok(self
            .list_all_pending()?
            .into_iter()
            .filter(|p| p.state == ContactState::OutgoingPending)
            .collect())
    }

    /// List all pending contacts (internal helper)
    fn list_all_pending(&self) -> Result<Vec<PendingContact>, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(PENDING_CONTACTS_TABLE)?;

        let mut pending = Vec::new();
        for entry in table.iter()? {
            let (_, value) = entry?;
            let contact: PendingContact = postcard::from_bytes(value.value())
                .map_err(|e| SyncError::Serialization(e.to_string()))?;
            pending.push(contact);
        }

        Ok(pending)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Invite Revocation Operations
    // ═══════════════════════════════════════════════════════════════════════

    /// Mark an invite as revoked
    ///
    /// Stores the current timestamp as the revocation time.
    pub fn revoke_invite(&self, invite_id: &[u8; 16]) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(REVOKED_INVITES_TABLE)?;
            let key = hex::encode(invite_id);
            let timestamp = chrono::Utc::now().timestamp().to_le_bytes();
            table.insert(key.as_str(), timestamp.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Check if an invite has been revoked
    ///
    /// Returns `true` if the invite is in the revocation list.
    pub fn is_invite_revoked(&self, invite_id: &[u8; 16]) -> Result<bool, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(REVOKED_INVITES_TABLE)?;
        let key = hex::encode(invite_id);

        Ok(table.get(key.as_str())?.is_some())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Generated Invite Tracking (for auto-accept)
    // ═══════════════════════════════════════════════════════════════════════

    /// Record that we generated an invite
    ///
    /// Used to auto-accept incoming requests that use our invites.
    pub fn save_generated_invite(&self, invite_id: &[u8; 16]) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(GENERATED_INVITES_TABLE)?;
            let key = hex::encode(invite_id);
            let timestamp = chrono::Utc::now().timestamp().to_le_bytes();
            table.insert(key.as_str(), timestamp.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Check if we generated this invite
    ///
    /// Returns `true` if this invite_id was created by us.
    pub fn is_our_generated_invite(&self, invite_id: &[u8; 16]) -> Result<bool, SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let read_txn = db_guard.begin_read()?;
        let table = read_txn.open_table(GENERATED_INVITES_TABLE)?;
        let key = hex::encode(invite_id);

        Ok(table.get(key.as_str())?.is_some())
    }

    /// Remove a generated invite record (after it's been used or cancelled)
    pub fn delete_generated_invite(&self, invite_id: &[u8; 16]) -> Result<(), SyncError> {
        let db = self.db_handle();
        let db_guard = db.read();
        let write_txn = db_guard.begin_write()?;
        {
            let mut table = write_txn.open_table(GENERATED_INVITES_TABLE)?;
            let key = hex::encode(invite_id);
            table.remove(key.as_str())?;
        }
        write_txn.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invite::NodeAddrBytes;
    use crate::types::contact::ProfileSnapshot;
    use tempfile::tempdir;

    fn create_test_contact(did: &str, name: &str) -> ContactInfo {
        ContactInfo {
            peer_did: did.to_string(),
            peer_endpoint_id: [0u8; 32],
            profile: ProfileSnapshot {
                display_name: name.to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: NodeAddrBytes::new([0u8; 32]),
            contact_topic: [0u8; 32],
            contact_key: [0u8; 32],
            accepted_at: chrono::Utc::now().timestamp(),
            last_seen: chrono::Utc::now().timestamp() as u64,
            status: ContactStatus::Offline,
            is_favorite: false,
        }
    }

    fn create_test_pending(invite_id: [u8; 16], did: &str, state: ContactState) -> PendingContact {
        PendingContact {
            invite_id,
            peer_did: did.to_string(),
            profile: ProfileSnapshot {
                display_name: "Test User".to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: NodeAddrBytes::new([0u8; 32]),
            state,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    #[test]
    fn test_save_and_load_contact() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let contact = create_test_contact("did:sync:alice", "Alice");

        // Save
        storage.save_contact(&contact).unwrap();

        // Load
        let loaded = storage.load_contact("did:sync:alice").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().profile.display_name, "Alice");
    }

    #[test]
    fn test_load_nonexistent_contact() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let loaded = storage.load_contact("did:sync:nonexistent").unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn test_delete_contact() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let contact = create_test_contact("did:sync:bob", "Bob");
        storage.save_contact(&contact).unwrap();

        // Verify it exists
        assert!(storage.load_contact("did:sync:bob").unwrap().is_some());

        // Delete
        storage.delete_contact("did:sync:bob").unwrap();

        // Verify it's gone
        assert!(storage.load_contact("did:sync:bob").unwrap().is_none());
    }

    #[test]
    fn test_list_contacts() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        // Create multiple contacts
        let contact1 = create_test_contact("did:sync:alice", "Alice");
        let contact2 = create_test_contact("did:sync:bob", "Bob");
        let contact3 = create_test_contact("did:sync:charlie", "Charlie");

        storage.save_contact(&contact1).unwrap();
        storage.save_contact(&contact2).unwrap();
        storage.save_contact(&contact3).unwrap();

        // List all
        let contacts = storage.list_contacts().unwrap();
        assert_eq!(contacts.len(), 3);

        let names: Vec<String> = contacts
            .iter()
            .map(|c| c.profile.display_name.clone())
            .collect();
        assert!(names.contains(&"Alice".to_string()));
        assert!(names.contains(&"Bob".to_string()));
        assert!(names.contains(&"Charlie".to_string()));
    }

    #[test]
    fn test_list_online_contacts() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        // Create contacts with different statuses
        let mut contact1 = create_test_contact("did:sync:online1", "Online1");
        contact1.status = ContactStatus::Online;

        let mut contact2 = create_test_contact("did:sync:online2", "Online2");
        contact2.status = ContactStatus::Online;

        let contact3 = create_test_contact("did:sync:offline", "Offline");
        // contact3 is offline (default)

        storage.save_contact(&contact1).unwrap();
        storage.save_contact(&contact2).unwrap();
        storage.save_contact(&contact3).unwrap();

        // List only online
        let online = storage.list_online_contacts().unwrap();
        assert_eq!(online.len(), 2);

        let names: Vec<String> = online
            .iter()
            .map(|c| c.profile.display_name.clone())
            .collect();
        assert!(names.contains(&"Online1".to_string()));
        assert!(names.contains(&"Online2".to_string()));
        assert!(!names.contains(&"Offline".to_string()));
    }

    #[test]
    fn test_save_and_load_pending() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let invite_id = [1u8; 16];
        let pending =
            create_test_pending(invite_id, "did:sync:test", ContactState::IncomingPending);

        // Save
        storage.save_pending(&pending).unwrap();

        // Load
        let loaded = storage.load_pending(&invite_id).unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().peer_did, "did:sync:test");
    }

    #[test]
    fn test_delete_pending() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let invite_id = [2u8; 16];
        let pending =
            create_test_pending(invite_id, "did:sync:test", ContactState::OutgoingPending);
        storage.save_pending(&pending).unwrap();

        // Verify it exists
        assert!(storage.load_pending(&invite_id).unwrap().is_some());

        // Delete
        storage.delete_pending(&invite_id).unwrap();

        // Verify it's gone
        assert!(storage.load_pending(&invite_id).unwrap().is_none());
    }

    #[test]
    fn test_list_incoming_pending() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        // Create pending contacts with different states
        let pending1 =
            create_test_pending([1u8; 16], "did:sync:test1", ContactState::IncomingPending);
        let pending2 =
            create_test_pending([2u8; 16], "did:sync:test2", ContactState::IncomingPending);
        let pending3 =
            create_test_pending([3u8; 16], "did:sync:test3", ContactState::OutgoingPending);

        storage.save_pending(&pending1).unwrap();
        storage.save_pending(&pending2).unwrap();
        storage.save_pending(&pending3).unwrap();

        // List only incoming
        let incoming = storage.list_incoming_pending().unwrap();
        assert_eq!(incoming.len(), 2);
    }

    #[test]
    fn test_list_outgoing_pending() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        // Create pending contacts with different states
        let pending1 =
            create_test_pending([4u8; 16], "did:sync:test4", ContactState::OutgoingPending);
        let pending2 =
            create_test_pending([5u8; 16], "did:sync:test5", ContactState::IncomingPending);
        let pending3 =
            create_test_pending([6u8; 16], "did:sync:test6", ContactState::OutgoingPending);

        storage.save_pending(&pending1).unwrap();
        storage.save_pending(&pending2).unwrap();
        storage.save_pending(&pending3).unwrap();

        // List only outgoing
        let outgoing = storage.list_outgoing_pending().unwrap();
        assert_eq!(outgoing.len(), 2);
    }

    #[test]
    fn test_revoke_and_check_invite() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let invite_id = [7u8; 16];

        // Not revoked initially
        assert!(!storage.is_invite_revoked(&invite_id).unwrap());

        // Revoke it
        storage.revoke_invite(&invite_id).unwrap();

        // Should be revoked now
        assert!(storage.is_invite_revoked(&invite_id).unwrap());
    }

    #[test]
    fn test_overwrite_contact() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let storage = Storage::new(&db_path).unwrap();

        let mut contact = create_test_contact("did:sync:test", "Original");
        storage.save_contact(&contact).unwrap();

        // Update and save again
        contact.profile.display_name = "Updated".to_string();
        storage.save_contact(&contact).unwrap();

        // Load and verify
        let loaded = storage.load_contact("did:sync:test").unwrap().unwrap();
        assert_eq!(loaded.profile.display_name, "Updated");
    }
}
