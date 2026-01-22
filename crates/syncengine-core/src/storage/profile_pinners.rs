//! Profile Pinners - Indra's Net Implicit Mirroring
//!
//! In Indra's Net, each jewel reflects all other jewels. Mirroring is implicit,
//! not announced. Being a contact IS the acknowledgment of mutual mirroring.
//!
//! This module provides the `PinnerInfo` type for UI compatibility, but the
//! actual pinner data is derived from contacts - no separate storage needed.
//!
//! # Philosophy
//!
//! Previous approach (broken):
//! - Send PinAcknowledgment messages via gossip
//! - Store pinners in a separate table
//! - Bug: Messages sent to wrong topic, never received
//!
//! Indra's Net approach (current):
//! - Contact acceptance = mutual mirroring agreement
//! - Pinner count = contact count
//! - No explicit acknowledgment messages needed
//!
//! # Benefits
//!
//! 1. **Simpler**: No complex acknowledgment protocol
//! 2. **Reliable**: No gossip messages that can be lost
//! 3. **Consistent**: Pinner count always matches contacts
//! 4. **Philosophical**: Aligns with mutual reflection principle

use serde::{Deserialize, Serialize};

use crate::types::contact::ContactInfo;
use super::Storage;
use crate::error::SyncError;

/// Information about a peer who is mirroring your profile.
///
/// In Indra's Net, contacts automatically mirror each other's profiles.
/// This struct is derived from contact information for UI display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PinnerInfo {
    /// DID of the peer who is mirroring your profile
    pub pinner_did: String,
    /// Unix timestamp when they became a contact (= started mirroring)
    pub pinned_at: i64,
    /// Relationship type: always "contact" in Indra's Net model
    pub relationship: String,
    /// Unix timestamp when we last saw them online
    pub last_confirmed: i64,
    /// Display name from their profile (if available)
    pub display_name: Option<String>,
}

impl PinnerInfo {
    /// Create a PinnerInfo from a ContactInfo.
    ///
    /// In Indra's Net, contacts ARE pinners - mutual mirroring is implicit.
    pub fn from_contact(contact: &ContactInfo) -> Self {
        Self {
            pinner_did: contact.peer_did.clone(),
            pinned_at: contact.accepted_at,
            relationship: "contact".to_string(),
            last_confirmed: contact.last_seen as i64,
            display_name: Some(contact.profile.display_name.clone()),
        }
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
    /// List all peers who are mirroring your profile (Indra's Net).
    ///
    /// In Indra's Net, contacts automatically mirror each other.
    /// This returns contacts transformed into PinnerInfo for UI compatibility.
    ///
    /// Returns a vector sorted by pinned_at (newest first).
    pub fn list_pinners(&self) -> Result<Vec<PinnerInfo>, SyncError> {
        let contacts = self.list_contacts()?;
        let mut pinners: Vec<PinnerInfo> = contacts
            .iter()
            .map(PinnerInfo::from_contact)
            .collect();

        // Sort by pinned_at descending (newest first)
        pinners.sort_by(|a, b| b.pinned_at.cmp(&a.pinned_at));

        Ok(pinners)
    }

    /// Count the number of peers mirroring your profile (Indra's Net).
    ///
    /// In Indra's Net, your contact count IS your pinner count.
    /// Contacts automatically mirror each other's profiles.
    pub fn count_pinners(&self) -> Result<usize, SyncError> {
        Ok(self.list_contacts()?.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invite::NodeAddrBytes;
    use crate::types::contact::{ContactStatus, ProfileSnapshot};
    use tempfile::tempdir;

    fn create_test_storage() -> Storage {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        Storage::new(&db_path).unwrap()
    }

    fn create_test_contact(did: &str, name: &str, accepted_at: i64) -> ContactInfo {
        ContactInfo {
            peer_did: did.to_string(),
            peer_endpoint_id: [0u8; 32],
            profile: ProfileSnapshot {
                display_name: name.to_string(),
                subtitle: None,
                avatar_blob_id: None,
                bio: String::new(),
            },
            node_addr: NodeAddrBytes {
                node_id: [0u8; 32],
                relay_url: None,
                direct_addresses: Vec::new(),
            },
            contact_topic: [0u8; 32],
            contact_key: [0u8; 32],
            accepted_at,
            last_seen: (accepted_at + 100) as u64,
            status: ContactStatus::Offline,
            is_favorite: false,
            encryption_keys: None,
            mutual_peers: vec![],
        }
    }

    #[test]
    fn test_pinner_info_from_contact() {
        let contact = create_test_contact("did:sync:love", "Love", 1000);
        let pinner = PinnerInfo::from_contact(&contact);

        assert_eq!(pinner.pinner_did, "did:sync:love");
        assert_eq!(pinner.pinned_at, 1000);
        assert_eq!(pinner.relationship, "contact");
        assert_eq!(pinner.display_name, Some("Love".to_string()));
        assert!(pinner.is_contact());
        assert!(!pinner.is_realm_member());
    }

    #[test]
    fn test_list_pinners_returns_contacts() {
        let storage = create_test_storage();

        // Add some contacts
        let contact1 = create_test_contact("did:sync:love", "Love", 100);
        let contact2 = create_test_contact("did:sync:joy", "Joy", 200);
        let contact3 = create_test_contact("did:sync:charlie", "Charlie", 150);

        storage.save_contact(&contact1).unwrap();
        storage.save_contact(&contact2).unwrap();
        storage.save_contact(&contact3).unwrap();

        // List pinners (should be contacts)
        let pinners = storage.list_pinners().unwrap();
        assert_eq!(pinners.len(), 3);

        // Should be sorted by pinned_at (accepted_at) descending
        assert_eq!(pinners[0].pinner_did, "did:sync:joy"); // 200
        assert_eq!(pinners[1].pinner_did, "did:sync:charlie"); // 150
        assert_eq!(pinners[2].pinner_did, "did:sync:love"); // 100
    }

    #[test]
    fn test_count_pinners_equals_contact_count() {
        let storage = create_test_storage();

        // Initially no pinners
        assert_eq!(storage.count_pinners().unwrap(), 0);

        // Add contacts
        let contact1 = create_test_contact("did:sync:love", "Love", 100);
        let contact2 = create_test_contact("did:sync:joy", "Joy", 200);

        storage.save_contact(&contact1).unwrap();
        assert_eq!(storage.count_pinners().unwrap(), 1);

        storage.save_contact(&contact2).unwrap();
        assert_eq!(storage.count_pinners().unwrap(), 2);

        // Verify contacts count matches - Indra's Net principle
        assert_eq!(storage.list_contacts().unwrap().len(), storage.count_pinners().unwrap());
    }
}
