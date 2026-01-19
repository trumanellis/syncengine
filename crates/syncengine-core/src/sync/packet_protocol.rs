//! Packet sync protocol for profile logs
//!
//! This module extends the sync protocol with messages for synchronizing
//! profile packet logs alongside realm documents.
//!
//! ## Protocol Overview
//!
//! ```text
//! Node A                          Node B
//!   |                               |
//!   |--- LogHead {did, seq, hash}-->|  (announce current state)
//!   |<-- LogHead {did, seq, hash}---|
//!   |                               |
//!   |    (if B is behind A)         |
//!   |                               |
//!   |<-- LogRequest {did, from} ----|  (request missing packets)
//!   |--- LogEntries {did, packets}->|  (send requested packets)
//!   |                               |
//!   |    (single packet broadcast)  |
//!   |                               |
//!   |--- Packet {envelope} -------->|  (new packet announcement)
//! ```
//!
//! ## Message Types
//!
//! | Message | Purpose |
//! |---------|---------|
//! | LogHead | Announce current log state (sequence + hash) |
//! | LogRequest | Request packets from a specific sequence |
//! | LogEntries | Response with multiple packets |
//! | Packet | Single packet broadcast |

use crate::identity::Did;
use crate::profile::PacketEnvelope;

use serde::{Deserialize, Serialize};

/// Messages for profile packet synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketSyncMessage {
    /// Announce current log head state.
    ///
    /// Sent periodically or after receiving new packets to inform peers
    /// of the current log state.
    LogHead {
        /// Profile DID this log belongs to
        did: Did,
        /// Current head sequence number
        sequence: u64,
        /// Hash of the head packet
        hash: [u8; 32],
    },

    /// Request packets from a specific sequence.
    ///
    /// Sent when a peer is behind and needs to catch up.
    LogRequest {
        /// Profile DID to request packets for
        did: Did,
        /// Starting sequence (exclusive - request packets AFTER this)
        from_sequence: u64,
    },

    /// Response with multiple packets.
    ///
    /// Response to LogRequest containing the requested packets.
    LogEntries {
        /// Profile DID these packets belong to
        did: Did,
        /// Serialized PacketEnvelopes
        entries: Vec<Vec<u8>>,
    },

    /// Single packet broadcast.
    ///
    /// Sent when a new packet is created or received.
    Packet {
        /// The packet envelope (serialized)
        envelope: Vec<u8>,
    },

    /// Receipt acknowledgment.
    ///
    /// Automatically sent when a packet addressed to us is received.
    /// This enables garbage collection on the sender's side.
    Receipt {
        /// DID of the original packet sender
        sender: Did,
        /// Sequence number of the received packet
        sequence: u64,
        /// Our DID (who is acknowledging)
        recipient: Did,
    },

    /// Depin broadcast.
    ///
    /// Sent after all expected receipts are received.
    /// Tells relays they can delete packets before the given sequence.
    Depin {
        /// Profile DID
        did: Did,
        /// Delete packets with sequence < this value
        before_sequence: u64,
    },
}

impl PacketSyncMessage {
    /// Encode message to bytes using postcard.
    pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    /// Decode message from bytes using postcard.
    pub fn decode(data: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(data)
    }

    /// Create a LogHead message from a packet envelope.
    pub fn log_head_from_envelope(envelope: &PacketEnvelope) -> Self {
        PacketSyncMessage::LogHead {
            did: envelope.sender.clone(),
            sequence: envelope.sequence,
            hash: envelope.hash(),
        }
    }

    /// Create a Packet message from an envelope.
    pub fn packet_from_envelope(envelope: &PacketEnvelope) -> Result<Self, postcard::Error> {
        Ok(PacketSyncMessage::Packet {
            envelope: postcard::to_allocvec(envelope)?,
        })
    }

    /// Extract envelope from a Packet message.
    pub fn extract_envelope(&self) -> Option<Result<PacketEnvelope, postcard::Error>> {
        match self {
            PacketSyncMessage::Packet { envelope } => {
                Some(postcard::from_bytes(envelope))
            }
            _ => None,
        }
    }

    /// Check if this is a LogHead message.
    pub fn is_log_head(&self) -> bool {
        matches!(self, PacketSyncMessage::LogHead { .. })
    }

    /// Check if this is a LogRequest message.
    pub fn is_log_request(&self) -> bool {
        matches!(self, PacketSyncMessage::LogRequest { .. })
    }

    /// Check if this is a LogEntries message.
    pub fn is_log_entries(&self) -> bool {
        matches!(self, PacketSyncMessage::LogEntries { .. })
    }

    /// Check if this is a Packet message.
    pub fn is_packet(&self) -> bool {
        matches!(self, PacketSyncMessage::Packet { .. })
    }

    /// Check if this is a Receipt message.
    pub fn is_receipt(&self) -> bool {
        matches!(self, PacketSyncMessage::Receipt { .. })
    }

    /// Check if this is a Depin message.
    pub fn is_depin(&self) -> bool {
        matches!(self, PacketSyncMessage::Depin { .. })
    }

    /// Get the DID this message relates to (if applicable).
    pub fn related_did(&self) -> Option<&Did> {
        match self {
            PacketSyncMessage::LogHead { did, .. } => Some(did),
            PacketSyncMessage::LogRequest { did, .. } => Some(did),
            PacketSyncMessage::LogEntries { did, .. } => Some(did),
            PacketSyncMessage::Receipt { sender, .. } => Some(sender),
            PacketSyncMessage::Depin { did, .. } => Some(did),
            PacketSyncMessage::Packet { envelope } => {
                // Try to extract DID from envelope
                if let Ok(env) = postcard::from_bytes::<PacketEnvelope>(envelope) {
                    // Note: This creates an owned Did, can't return reference
                    None // Would need different approach to return reference
                } else {
                    None
                }
            }
        }
    }
}

/// Versioned wrapper for packet sync messages.
///
/// Allows protocol evolution while maintaining backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketWireMessage {
    /// Protocol version 1
    V1(PacketSyncMessage),
}

impl PacketWireMessage {
    /// Create a new wire message wrapping a packet sync message.
    pub fn new(msg: PacketSyncMessage) -> Self {
        PacketWireMessage::V1(msg)
    }

    /// Encode wire message to bytes.
    pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    /// Decode wire message from bytes.
    pub fn decode(data: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(data)
    }

    /// Unwrap the inner PacketSyncMessage.
    pub fn into_inner(self) -> PacketSyncMessage {
        match self {
            PacketWireMessage::V1(msg) => msg,
        }
    }

    /// Get a reference to the inner PacketSyncMessage.
    pub fn as_inner(&self) -> &PacketSyncMessage {
        match self {
            PacketWireMessage::V1(msg) => msg,
        }
    }

    /// Get the protocol version.
    pub fn version(&self) -> u8 {
        match self {
            PacketWireMessage::V1(_) => 1,
        }
    }
}

/// Discriminator for combined protocol messages.
///
/// This allows distinguishing between realm sync messages and packet sync messages
/// on the same gossip topic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Realm document sync (existing Automerge sync)
    RealmSync = 0,
    /// Profile packet sync (new)
    PacketSync = 1,
}

/// Combined message that can carry either realm or packet sync data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedMessage {
    /// Message type discriminator
    pub message_type: MessageType,
    /// Message payload
    pub payload: Vec<u8>,
}

impl CombinedMessage {
    /// Create a realm sync message.
    pub fn realm_sync(payload: Vec<u8>) -> Self {
        Self {
            message_type: MessageType::RealmSync,
            payload,
        }
    }

    /// Create a packet sync message.
    pub fn packet_sync(payload: Vec<u8>) -> Self {
        Self {
            message_type: MessageType::PacketSync,
            payload,
        }
    }

    /// Encode to bytes.
    pub fn encode(&self) -> Result<Vec<u8>, postcard::Error> {
        postcard::to_allocvec(self)
    }

    /// Decode from bytes.
    pub fn decode(data: &[u8]) -> Result<Self, postcard::Error> {
        postcard::from_bytes(data)
    }

    /// Check if this is a realm sync message.
    pub fn is_realm_sync(&self) -> bool {
        self.message_type == MessageType::RealmSync
    }

    /// Check if this is a packet sync message.
    pub fn is_packet_sync(&self) -> bool {
        self.message_type == MessageType::PacketSync
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{ProfileKeys, PacketPayload};

    fn create_test_envelope(keys: &ProfileKeys, sequence: u64) -> PacketEnvelope {
        let payload = PacketPayload::Heartbeat {
            timestamp: chrono::Utc::now().timestamp_millis(),
        };
        PacketEnvelope::create_global(keys, &payload, sequence, [0u8; 32])
            .expect("Should create envelope")
    }

    #[test]
    fn test_log_head_encode_decode() {
        let keys = ProfileKeys::generate();
        let msg = PacketSyncMessage::LogHead {
            did: keys.did(),
            sequence: 42,
            hash: [1u8; 32],
        };

        let encoded = msg.encode().expect("Should encode");
        let decoded = PacketSyncMessage::decode(&encoded).expect("Should decode");

        match decoded {
            PacketSyncMessage::LogHead { did, sequence, hash } => {
                assert_eq!(did, keys.did());
                assert_eq!(sequence, 42);
                assert_eq!(hash, [1u8; 32]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_log_request_encode_decode() {
        let keys = ProfileKeys::generate();
        let msg = PacketSyncMessage::LogRequest {
            did: keys.did(),
            from_sequence: 10,
        };

        let encoded = msg.encode().expect("Should encode");
        let decoded = PacketSyncMessage::decode(&encoded).expect("Should decode");

        assert!(decoded.is_log_request());
    }

    #[test]
    fn test_log_entries_encode_decode() {
        let keys = ProfileKeys::generate();
        let envelope1 = create_test_envelope(&keys, 0);
        let envelope2 = create_test_envelope(&keys, 1);

        let msg = PacketSyncMessage::LogEntries {
            did: keys.did(),
            entries: vec![
                postcard::to_allocvec(&envelope1).unwrap(),
                postcard::to_allocvec(&envelope2).unwrap(),
            ],
        };

        let encoded = msg.encode().expect("Should encode");
        let decoded = PacketSyncMessage::decode(&encoded).expect("Should decode");

        match decoded {
            PacketSyncMessage::LogEntries { did, entries } => {
                assert_eq!(did, keys.did());
                assert_eq!(entries.len(), 2);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_packet_from_envelope() {
        let keys = ProfileKeys::generate();
        let envelope = create_test_envelope(&keys, 0);

        let msg = PacketSyncMessage::packet_from_envelope(&envelope)
            .expect("Should create message");

        assert!(msg.is_packet());

        let extracted = msg.extract_envelope()
            .expect("Should have envelope")
            .expect("Should decode");

        assert_eq!(extracted.sender, keys.did());
        assert_eq!(extracted.sequence, 0);
    }

    #[test]
    fn test_receipt_encode_decode() {
        let sender_keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();

        let msg = PacketSyncMessage::Receipt {
            sender: sender_keys.did(),
            sequence: 5,
            recipient: recipient_keys.did(),
        };

        let encoded = msg.encode().expect("Should encode");
        let decoded = PacketSyncMessage::decode(&encoded).expect("Should decode");

        assert!(decoded.is_receipt());
    }

    #[test]
    fn test_depin_encode_decode() {
        let keys = ProfileKeys::generate();

        let msg = PacketSyncMessage::Depin {
            did: keys.did(),
            before_sequence: 100,
        };

        let encoded = msg.encode().expect("Should encode");
        let decoded = PacketSyncMessage::decode(&encoded).expect("Should decode");

        assert!(decoded.is_depin());
    }

    #[test]
    fn test_wire_message_versioning() {
        let keys = ProfileKeys::generate();
        let msg = PacketSyncMessage::LogHead {
            did: keys.did(),
            sequence: 1,
            hash: [0u8; 32],
        };

        let wire = PacketWireMessage::new(msg);
        assert_eq!(wire.version(), 1);

        let encoded = wire.encode().expect("Should encode");
        let decoded = PacketWireMessage::decode(&encoded).expect("Should decode");

        assert_eq!(decoded.version(), 1);
        assert!(decoded.as_inner().is_log_head());
    }

    #[test]
    fn test_combined_message() {
        let realm_payload = vec![1, 2, 3, 4];
        let packet_payload = vec![5, 6, 7, 8];

        let realm_msg = CombinedMessage::realm_sync(realm_payload.clone());
        let packet_msg = CombinedMessage::packet_sync(packet_payload.clone());

        assert!(realm_msg.is_realm_sync());
        assert!(!realm_msg.is_packet_sync());

        assert!(packet_msg.is_packet_sync());
        assert!(!packet_msg.is_realm_sync());

        // Roundtrip
        let encoded = realm_msg.encode().expect("Should encode");
        let decoded = CombinedMessage::decode(&encoded).expect("Should decode");
        assert!(decoded.is_realm_sync());
        assert_eq!(decoded.payload, realm_payload);
    }

    #[test]
    fn test_log_head_from_envelope() {
        let keys = ProfileKeys::generate();
        let envelope = create_test_envelope(&keys, 42);
        let hash = envelope.hash();

        let msg = PacketSyncMessage::log_head_from_envelope(&envelope);

        match msg {
            PacketSyncMessage::LogHead { did, sequence, hash: h } => {
                assert_eq!(did, keys.did());
                assert_eq!(sequence, 42);
                assert_eq!(h, hash);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_message_type_checks() {
        let keys = ProfileKeys::generate();

        let log_head = PacketSyncMessage::LogHead {
            did: keys.did(),
            sequence: 0,
            hash: [0u8; 32],
        };
        assert!(log_head.is_log_head());
        assert!(!log_head.is_log_request());

        let log_request = PacketSyncMessage::LogRequest {
            did: keys.did(),
            from_sequence: 0,
        };
        assert!(log_request.is_log_request());
        assert!(!log_request.is_log_head());

        let log_entries = PacketSyncMessage::LogEntries {
            did: keys.did(),
            entries: vec![],
        };
        assert!(log_entries.is_log_entries());

        let packet = PacketSyncMessage::Packet {
            envelope: vec![],
        };
        assert!(packet.is_packet());

        let receipt = PacketSyncMessage::Receipt {
            sender: keys.did(),
            sequence: 0,
            recipient: keys.did(),
        };
        assert!(receipt.is_receipt());

        let depin = PacketSyncMessage::Depin {
            did: keys.did(),
            before_sequence: 0,
        };
        assert!(depin.is_depin());
    }

    #[test]
    fn test_related_did() {
        let keys = ProfileKeys::generate();

        let msg = PacketSyncMessage::LogHead {
            did: keys.did(),
            sequence: 0,
            hash: [0u8; 32],
        };
        assert_eq!(msg.related_did(), Some(&keys.did()));

        let msg = PacketSyncMessage::Packet {
            envelope: vec![],
        };
        assert!(msg.related_did().is_none());
    }
}
