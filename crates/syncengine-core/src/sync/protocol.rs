//! Gossip sync protocol for Automerge documents
//!
//! Messages are serialized with postcard and broadcast via gossip.
//!
//! ## Protocol Overview
//!
//! The sync protocol enables Automerge document synchronization over gossip:
//!
//! 1. **Announce**: Nodes periodically announce their document heads
//! 2. **SyncRequest**: When heads differ, request full document sync
//! 3. **SyncResponse**: Return full document state
//! 4. **Changes**: Broadcast incremental changes as they happen
//!
//! ## Message Flow
//!
//! ```text
//! Node A                          Node B
//!   |                               |
//!   |--- Announce {heads: [h1]} --->|
//!   |<-- Announce {heads: [h2]} ----|
//!   |                               |
//!   |    (heads differ, sync!)      |
//!   |                               |
//!   |--- SyncRequest -------------->|
//!   |<-- SyncResponse {doc} --------|
//!   |                               |
//!   |    (merge documents)          |
//!   |                               |
//!   |<-- Changes {data} ------------|
//!   |    (incremental updates)      |
//! ```

use serde::{Deserialize, Serialize};

use crate::RealmId;

/// Messages sent over gossip for document sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Announce presence and document heads
    ///
    /// Sent periodically to inform peers of the current document state.
    /// Peers compare heads to detect if they need to sync.
    Announce {
        /// The realm this announcement is for
        realm_id: RealmId,
        /// Automerge ChangeHash bytes for each head
        heads: Vec<Vec<u8>>,
    },

    /// Request full document sync
    ///
    /// Sent when a peer detects it is behind and needs the full document.
    SyncRequest {
        /// The realm to request sync for
        realm_id: RealmId,
    },

    /// Full document state
    ///
    /// Response to SyncRequest containing the complete Automerge document.
    SyncResponse {
        /// The realm this document belongs to
        realm_id: RealmId,
        /// Full Automerge document bytes (via `doc.save()`)
        document: Vec<u8>,
    },

    /// Incremental changes
    ///
    /// Broadcast when local changes are made, allowing peers to merge
    /// without needing a full document sync.
    Changes {
        /// The realm these changes belong to
        realm_id: RealmId,
        /// Automerge incremental save data (via `doc.save_after(&heads)`)
        data: Vec<u8>,
    },
}

impl SyncMessage {
    /// Encode message to bytes using postcard
    pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    /// Decode message from bytes using postcard
    pub fn decode(data: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(data)
    }

    /// Get the realm ID this message relates to
    pub fn realm_id(&self) -> &RealmId {
        match self {
            SyncMessage::Announce { realm_id, .. } => realm_id,
            SyncMessage::SyncRequest { realm_id } => realm_id,
            SyncMessage::SyncResponse { realm_id, .. } => realm_id,
            SyncMessage::Changes { realm_id, .. } => realm_id,
        }
    }

    /// Check if this is an announcement message
    pub fn is_announce(&self) -> bool {
        matches!(self, SyncMessage::Announce { .. })
    }

    /// Check if this is a sync request message
    pub fn is_sync_request(&self) -> bool {
        matches!(self, SyncMessage::SyncRequest { .. })
    }

    /// Check if this is a sync response message
    pub fn is_sync_response(&self) -> bool {
        matches!(self, SyncMessage::SyncResponse { .. })
    }

    /// Check if this is a changes message
    pub fn is_changes(&self) -> bool {
        matches!(self, SyncMessage::Changes { .. })
    }
}

/// Wrapper for versioned messages (future-proofing)
///
/// Allows protocol evolution while maintaining backward compatibility.
/// New versions can be added as variants without breaking existing nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WireMessage {
    /// Protocol version 1
    V1(SyncMessage),
}

impl WireMessage {
    /// Create a new wire message wrapping a sync message
    pub fn new(msg: SyncMessage) -> Self {
        WireMessage::V1(msg)
    }

    /// Encode wire message to bytes using postcard
    pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    /// Decode wire message from bytes using postcard
    pub fn decode(data: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(data)
    }

    /// Unwrap the inner SyncMessage
    pub fn into_inner(self) -> SyncMessage {
        match self {
            WireMessage::V1(msg) => msg,
        }
    }

    /// Get a reference to the inner SyncMessage
    pub fn as_inner(&self) -> &SyncMessage {
        match self {
            WireMessage::V1(msg) => msg,
        }
    }

    /// Get the protocol version
    pub fn version(&self) -> u8 {
        match self {
            WireMessage::V1(_) => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_message_encode_decode() {
        let realm_id = RealmId::new();
        let msg = SyncMessage::Changes {
            realm_id: realm_id.clone(),
            data: vec![1, 2, 3, 4],
        };

        let encoded = msg.encode().unwrap();
        let decoded = SyncMessage::decode(&encoded).unwrap();

        match decoded {
            SyncMessage::Changes {
                realm_id: rid,
                data,
            } => {
                assert_eq!(rid.0, realm_id.0);
                assert_eq!(data, vec![1, 2, 3, 4]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_wire_message_versioning() {
        let realm_id = RealmId::new();
        let msg = SyncMessage::SyncRequest { realm_id };
        let wire = WireMessage::new(msg);

        assert_eq!(wire.version(), 1);

        let encoded = wire.encode().unwrap();
        let decoded = WireMessage::decode(&encoded).unwrap();

        assert_eq!(decoded.version(), 1);
        match decoded.into_inner() {
            SyncMessage::SyncRequest { .. } => {}
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_announce_with_heads() {
        let realm_id = RealmId::new();
        let heads = vec![vec![0u8; 32], vec![1u8; 32]];

        let msg = SyncMessage::Announce {
            realm_id,
            heads: heads.clone(),
        };

        let encoded = msg.encode().unwrap();
        let decoded = SyncMessage::decode(&encoded).unwrap();

        match decoded {
            SyncMessage::Announce { heads: h, .. } => {
                assert_eq!(h.len(), 2);
                assert_eq!(h[0], vec![0u8; 32]);
                assert_eq!(h[1], vec![1u8; 32]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_realm_id_accessor() {
        let realm_id = RealmId::new();
        let msg = SyncMessage::Changes {
            realm_id: realm_id.clone(),
            data: vec![],
        };

        assert_eq!(msg.realm_id().0, realm_id.0);
    }

    #[test]
    fn test_sync_response_with_document() {
        let realm_id = RealmId::new();
        // Simulate a saved Automerge document (arbitrary bytes)
        let doc_bytes = vec![0x85, 0x6f, 0x4a, 0x83, 0x01, 0x02, 0x03];

        let msg = SyncMessage::SyncResponse {
            realm_id: realm_id.clone(),
            document: doc_bytes.clone(),
        };

        let encoded = msg.encode().unwrap();
        let decoded = SyncMessage::decode(&encoded).unwrap();

        match decoded {
            SyncMessage::SyncResponse {
                realm_id: rid,
                document,
            } => {
                assert_eq!(rid.0, realm_id.0);
                assert_eq!(document, doc_bytes);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_message_type_checks() {
        let realm_id = RealmId::new();

        let announce = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads: vec![],
        };
        assert!(announce.is_announce());
        assert!(!announce.is_sync_request());
        assert!(!announce.is_sync_response());
        assert!(!announce.is_changes());

        let request = SyncMessage::SyncRequest {
            realm_id: realm_id.clone(),
        };
        assert!(!request.is_announce());
        assert!(request.is_sync_request());

        let response = SyncMessage::SyncResponse {
            realm_id: realm_id.clone(),
            document: vec![],
        };
        assert!(response.is_sync_response());

        let changes = SyncMessage::Changes {
            realm_id,
            data: vec![],
        };
        assert!(changes.is_changes());
    }

    #[test]
    fn test_wire_message_as_inner() {
        let realm_id = RealmId::new();
        let msg = SyncMessage::Announce {
            realm_id,
            heads: vec![],
        };
        let wire = WireMessage::new(msg);

        assert!(wire.as_inner().is_announce());
    }

    #[test]
    fn test_empty_announce() {
        let realm_id = RealmId::new();
        let msg = SyncMessage::Announce {
            realm_id,
            heads: vec![],
        };

        let encoded = msg.encode().unwrap();
        let decoded = SyncMessage::decode(&encoded).unwrap();

        match decoded {
            SyncMessage::Announce { heads, .. } => {
                assert!(heads.is_empty());
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_large_document_sync() {
        let realm_id = RealmId::new();
        // Simulate a larger document (10KB)
        let large_doc: Vec<u8> = (0..10_000).map(|i| (i % 256) as u8).collect();

        let msg = SyncMessage::SyncResponse {
            realm_id,
            document: large_doc.clone(),
        };

        let encoded = msg.encode().unwrap();
        let decoded = SyncMessage::decode(&encoded).unwrap();

        match decoded {
            SyncMessage::SyncResponse { document, .. } => {
                assert_eq!(document.len(), 10_000);
                assert_eq!(document, large_doc);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
