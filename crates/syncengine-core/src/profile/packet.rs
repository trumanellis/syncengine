//! Packet types for the profile layer
//!
//! This module defines the core packet structures:
//! - [`PacketEnvelope`]: Signed, encrypted wrapper with cleartext metadata
//! - [`PacketPayload`]: The encrypted content (various message types)
//! - [`PacketAddress`]: Addressing modes (individual, list, group, global)
//!
//! ## Packet Structure
//!
//! ```text
//! PacketEnvelope (cleartext header):
//! ┌─────────────────────────────────────────────────────────┐
//! │  sender: Did           - Who created this packet        │
//! │  sequence: u64         - Monotonic counter              │
//! │  prev_hash: [u8; 32]   - Hash of previous packet        │
//! │  timestamp: i64        - Unix timestamp (ms)            │
//! │  signature: HybridSig  - Signs all the above + payload  │
//! ├─────────────────────────────────────────────────────────┤
//! │  sealed_keys: Vec<SealedKey>  - Per-recipient keys      │
//! │  nonce: [u8; 12]       - Encryption nonce               │
//! │  ciphertext: Vec<u8>   - Encrypted PacketPayload        │
//! └─────────────────────────────────────────────────────────┘
//! ```

use crate::crypto::{RealmCrypto, NONCE_SIZE};
use crate::error::SyncError;
use crate::identity::{Did, HybridSignature};
use crate::types::RealmId;

use super::keys::{ProfileKeys, ProfilePublicKeys};
use super::sealed::SealedKey;

use serde::{Deserialize, Serialize};

/// Packet envelope containing signed, encrypted content.
///
/// The envelope has cleartext metadata (sender, sequence, timestamp) that
/// allows relays to route and store packets without decrypting them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketEnvelope {
    /// Sender's DID
    pub sender: Did,
    /// Monotonically increasing sequence number within sender's log
    pub sequence: u64,
    /// BLAKE3 hash of the previous packet (for hash chain)
    pub prev_hash: [u8; 32],
    /// Unix timestamp in milliseconds when packet was created
    pub timestamp: i64,
    /// Hybrid signature (Ed25519 + ML-DSA-65) over envelope + payload
    pub signature: HybridSignature,
    /// Per-recipient sealed keys (for decrypting content)
    pub sealed_keys: Vec<SealedKey>,
    /// Nonce for content encryption
    pub nonce: [u8; NONCE_SIZE],
    /// Encrypted payload
    pub ciphertext: Vec<u8>,
}

impl PacketEnvelope {
    /// Create a new packet envelope.
    ///
    /// This:
    /// 1. Serializes the payload
    /// 2. Generates a random content key
    /// 3. Seals the content key for each recipient
    /// 4. Encrypts the payload
    /// 5. Signs the entire envelope
    pub fn create(
        sender_keys: &ProfileKeys,
        payload: &PacketPayload,
        recipients: &[ProfilePublicKeys],
        sequence: u64,
        prev_hash: [u8; 32],
    ) -> Result<Self, SyncError> {
        let sender = sender_keys.did();
        let timestamp = chrono::Utc::now().timestamp_millis();

        // Serialize the payload
        let payload_bytes = postcard::to_allocvec(payload)
            .map_err(|e| SyncError::Serialization(format!("Failed to serialize payload: {}", e)))?;

        // Generate content key and seal for recipients
        let content_key = RealmCrypto::generate_key();
        let sealed_keys: Result<Vec<_>, _> = recipients
            .iter()
            .map(|r| SealedKey::seal_for_recipient(&content_key, r))
            .collect();
        let sealed_keys = sealed_keys?;

        // Encrypt the payload
        let nonce = RealmCrypto::generate_nonce();
        let crypto = RealmCrypto::new(&content_key);
        let ciphertext = crypto.encrypt_with_nonce(&payload_bytes, &nonce)?;

        // Create the signature payload (everything except the signature itself)
        let sign_payload = Self::create_sign_payload(
            &sender,
            sequence,
            &prev_hash,
            timestamp,
            &sealed_keys,
            &nonce,
            &ciphertext,
        );

        // Sign with hybrid signature
        let signature = sender_keys.sign(&sign_payload);

        Ok(Self {
            sender,
            sequence,
            prev_hash,
            timestamp,
            signature,
            sealed_keys,
            nonce,
            ciphertext,
        })
    }

    /// Create a global (public) packet that is signed but not encrypted.
    ///
    /// Global packets have no recipients and the payload is in cleartext.
    pub fn create_global(
        sender_keys: &ProfileKeys,
        payload: &PacketPayload,
        sequence: u64,
        prev_hash: [u8; 32],
    ) -> Result<Self, SyncError> {
        let sender = sender_keys.did();
        let timestamp = chrono::Utc::now().timestamp_millis();

        // Serialize the payload (no encryption for global packets)
        let ciphertext = postcard::to_allocvec(payload)
            .map_err(|e| SyncError::Serialization(format!("Failed to serialize payload: {}", e)))?;

        // Empty sealed keys and zero nonce for global packets
        let sealed_keys = Vec::new();
        let nonce = [0u8; NONCE_SIZE];

        // Create the signature payload
        let sign_payload = Self::create_sign_payload(
            &sender,
            sequence,
            &prev_hash,
            timestamp,
            &sealed_keys,
            &nonce,
            &ciphertext,
        );

        let signature = sender_keys.sign(&sign_payload);

        Ok(Self {
            sender,
            sequence,
            prev_hash,
            timestamp,
            signature,
            sealed_keys,
            nonce,
            ciphertext,
        })
    }

    /// Verify the envelope's signature.
    ///
    /// This verifies that the envelope was signed by the sender and hasn't
    /// been tampered with. It does NOT decrypt the payload.
    pub fn verify(&self, sender_public: &ProfilePublicKeys) -> bool {
        // Verify sender matches
        if sender_public.did() != self.sender {
            return false;
        }

        let sign_payload = Self::create_sign_payload(
            &self.sender,
            self.sequence,
            &self.prev_hash,
            self.timestamp,
            &self.sealed_keys,
            &self.nonce,
            &self.ciphertext,
        );

        sender_public.signing.verify(&sign_payload, &self.signature)
    }

    /// Open the envelope and decrypt the payload.
    ///
    /// Returns the decrypted payload if the recipient is in the sealed keys list.
    pub fn open(&self, recipient_keys: &ProfileKeys) -> Result<PacketPayload, SyncError> {
        // Check if this is a global packet (no sealed keys)
        if self.sealed_keys.is_empty() {
            // Global packet - payload is not encrypted
            let payload = postcard::from_bytes(&self.ciphertext)
                .map_err(|e| SyncError::Serialization(format!("Failed to deserialize payload: {}", e)))?;
            return Ok(payload);
        }

        // Find our sealed key
        let my_did = recipient_keys.did();
        let sealed_key = self
            .sealed_keys
            .iter()
            .find(|sk| sk.recipient == my_did)
            .ok_or_else(|| SyncError::Crypto("Not a recipient of this packet".to_string()))?;

        // Unseal the content key
        let content_key = sealed_key.unseal(recipient_keys)?;

        // Decrypt the payload
        let crypto = RealmCrypto::new(&content_key);
        let payload_bytes = crypto.decrypt_with_nonce(&self.ciphertext, &self.nonce)?;

        // Deserialize
        let payload = postcard::from_bytes(&payload_bytes)
            .map_err(|e| SyncError::Serialization(format!("Failed to deserialize payload: {}", e)))?;

        Ok(payload)
    }

    /// Check if this envelope is addressed to a specific DID.
    pub fn is_addressed_to(&self, did: &Did) -> bool {
        // Global packets are addressed to everyone
        if self.sealed_keys.is_empty() {
            return true;
        }
        self.sealed_keys.iter().any(|sk| &sk.recipient == did)
    }

    /// Check if this is a global (public) packet.
    pub fn is_global(&self) -> bool {
        self.sealed_keys.is_empty()
    }

    /// Get all recipient DIDs.
    pub fn recipients(&self) -> Vec<&Did> {
        self.sealed_keys.iter().map(|sk| &sk.recipient).collect()
    }

    /// Compute the hash of this envelope (for hash chain).
    pub fn hash(&self) -> [u8; 32] {
        let bytes = postcard::to_allocvec(self).expect("Envelope should serialize");
        *blake3::hash(&bytes).as_bytes()
    }

    /// Create the payload to be signed.
    fn create_sign_payload(
        sender: &Did,
        sequence: u64,
        prev_hash: &[u8; 32],
        timestamp: i64,
        sealed_keys: &[SealedKey],
        nonce: &[u8; NONCE_SIZE],
        ciphertext: &[u8],
    ) -> Vec<u8> {
        // Compute deterministic representation for signing
        let mut payload = Vec::new();

        // Sender DID
        let sender_bytes = sender.as_str().as_bytes();
        payload.extend_from_slice(&(sender_bytes.len() as u32).to_le_bytes());
        payload.extend_from_slice(sender_bytes);

        // Sequence
        payload.extend_from_slice(&sequence.to_le_bytes());

        // Prev hash
        payload.extend_from_slice(prev_hash);

        // Timestamp
        payload.extend_from_slice(&timestamp.to_le_bytes());

        // Sealed keys count and recipient DIDs
        payload.extend_from_slice(&(sealed_keys.len() as u32).to_le_bytes());
        for sk in sealed_keys {
            let recipient_bytes = sk.recipient.as_str().as_bytes();
            payload.extend_from_slice(&(recipient_bytes.len() as u32).to_le_bytes());
            payload.extend_from_slice(recipient_bytes);
        }

        // Nonce
        payload.extend_from_slice(nonce);

        // Ciphertext
        payload.extend_from_slice(&(ciphertext.len() as u32).to_le_bytes());
        payload.extend_from_slice(ciphertext);

        payload
    }

    /// Encode envelope to bytes.
    pub fn encode(&self) -> Result<Vec<u8>, SyncError> {
        postcard::to_allocvec(self)
            .map_err(|e| SyncError::Serialization(format!("Failed to encode envelope: {}", e)))
    }

    /// Decode envelope from bytes.
    pub fn decode(bytes: &[u8]) -> Result<Self, SyncError> {
        postcard::from_bytes(bytes)
            .map_err(|e| SyncError::Serialization(format!("Failed to decode envelope: {}", e)))
    }

    /// Decode the payload of a global (public) packet.
    ///
    /// Global packets are not encrypted, so the ciphertext field contains
    /// the serialized payload directly.
    ///
    /// # Errors
    ///
    /// Returns an error if this is not a global packet or if deserialization fails.
    pub fn decode_global_payload(&self) -> Result<PacketPayload, SyncError> {
        if !self.is_global() {
            return Err(SyncError::Crypto(
                "Cannot decode global payload from sealed packet".to_string()
            ));
        }

        postcard::from_bytes(&self.ciphertext)
            .map_err(|e| SyncError::Serialization(format!("Failed to decode payload: {}", e)))
    }

    /// Decrypt the payload for a specific recipient.
    ///
    /// This attempts to find our sealed key and decrypt the content.
    ///
    /// # Arguments
    ///
    /// * `recipient_keys` - Our profile keys (to find our sealed key and decrypt)
    ///
    /// # Returns
    ///
    /// The decrypted payload if we are a recipient and decryption succeeds.
    pub fn decrypt_for_recipient(&self, recipient_keys: &ProfileKeys) -> Result<PacketPayload, SyncError> {
        let our_did = recipient_keys.did();

        // Find our sealed key
        let sealed_key = self.sealed_keys.iter()
            .find(|sk| sk.recipient == our_did)
            .ok_or_else(|| SyncError::Crypto("Not a recipient of this packet".to_string()))?;

        // Unseal the content key
        let content_key = sealed_key.unseal(recipient_keys)?;

        // Decrypt the ciphertext
        let crypto = RealmCrypto::new(&content_key);
        let plaintext = crypto.decrypt_with_nonce(&self.ciphertext, &self.nonce)?;

        // Deserialize the payload
        postcard::from_bytes(&plaintext)
            .map_err(|e| SyncError::Serialization(format!("Failed to decode payload: {}", e)))
    }
}

/// Payload types for profile packets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PacketPayload {
    /// Profile metadata update.
    ProfileUpdate {
        /// Display name (optional update)
        display_name: Option<String>,
        /// Bio text (optional update)
        bio: Option<String>,
        /// Avatar blob ID (optional update)
        avatar_blob_id: Option<String>,
    },

    /// Realm invitation.
    RealmInvite {
        /// The realm being shared
        realm_id: RealmId,
        /// Symmetric key for the realm
        realm_key: [u8; 32],
        /// Human-readable realm name
        realm_name: String,
    },

    /// Task/intention reference within a realm.
    TaskReference {
        /// The realm containing the task
        realm_id: RealmId,
        /// Task identifier within the realm
        task_id: String,
        /// Brief description
        description: String,
    },

    /// Direct message.
    DirectMessage {
        /// Message content
        content: String,
        /// Recipient DID (for tracking sent messages when using topic-level privacy)
        recipient: Did,
    },

    /// Automatic receipt acknowledging packet reception.
    Receipt {
        /// DID of the original packet sender
        original_sender: Did,
        /// Sequence number of the received packet
        packet_seq: u64,
    },

    /// Garbage collection signal (sent after all receipts received).
    Depin {
        /// All packets with sequence < this can be deleted
        before_sequence: u64,
        /// Optional Merkle root of compacted history
        merkle_root: Option<[u8; 32]>,
    },

    /// Heartbeat / presence signal.
    Heartbeat {
        /// Current local time
        timestamp: i64,
    },

    /// Key rotation announcement.
    KeyRotation {
        /// New public key bundle (serialized ProfilePublicKeys)
        new_public_keys: Vec<u8>,
        /// Signature from old keys over new keys (proves continuity)
        old_key_signature: Vec<u8>,
    },
}

/// Addressing modes for packets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PacketAddress {
    /// Single recipient
    Individual(Did),
    /// Multiple specific recipients
    List(Vec<Did>),
    /// All members of a realm
    Group(RealmId),
    /// Public broadcast (signed but not encrypted)
    Global,
}

impl PacketAddress {
    /// Check if this address includes a specific DID.
    pub fn includes(&self, did: &Did) -> bool {
        match self {
            PacketAddress::Individual(d) => d == did,
            PacketAddress::List(dids) => dids.iter().any(|d| d == did),
            PacketAddress::Group(_) => true, // Realm membership checked separately
            PacketAddress::Global => true,
        }
    }

    /// Get explicit recipient DIDs (if available).
    ///
    /// Returns None for Group and Global addresses.
    pub fn explicit_recipients(&self) -> Option<Vec<&Did>> {
        match self {
            PacketAddress::Individual(d) => Some(vec![d]),
            PacketAddress::List(dids) => Some(dids.iter().collect()),
            PacketAddress::Group(_) | PacketAddress::Global => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_envelope_roundtrip() {
        let sender_keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();
        let recipient_public = recipient_keys.public_bundle();

        let payload = PacketPayload::DirectMessage {
            content: "Hello, world!".to_string(),
            recipient: recipient_keys.did(),
        };

        let envelope = PacketEnvelope::create(
            &sender_keys,
            &payload,
            &[recipient_public],
            1,
            [0u8; 32],
        )
        .expect("Should create envelope");

        // Verify signature
        assert!(envelope.verify(&sender_keys.public_bundle()));

        // Open and verify payload
        let opened = envelope.open(&recipient_keys).expect("Should open");
        assert_eq!(opened, payload);
    }

    #[test]
    fn test_packet_envelope_multiple_recipients() {
        let sender_keys = ProfileKeys::generate();
        let recipient1_keys = ProfileKeys::generate();
        let recipient2_keys = ProfileKeys::generate();

        let recipients = vec![
            recipient1_keys.public_bundle(),
            recipient2_keys.public_bundle(),
        ];

        // Use recipient1 as the primary recipient in payload
        let payload = PacketPayload::DirectMessage {
            content: "To multiple recipients".to_string(),
            recipient: recipient1_keys.did(),
        };

        let envelope = PacketEnvelope::create(
            &sender_keys,
            &payload,
            &recipients,
            1,
            [0u8; 32],
        )
        .expect("Should create envelope");

        // Both recipients should be able to open
        assert_eq!(
            envelope.open(&recipient1_keys).expect("Recipient 1"),
            payload
        );
        assert_eq!(
            envelope.open(&recipient2_keys).expect("Recipient 2"),
            payload
        );
    }

    #[test]
    fn test_packet_envelope_global() {
        let sender_keys = ProfileKeys::generate();

        let payload = PacketPayload::Heartbeat {
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        let envelope = PacketEnvelope::create_global(
            &sender_keys,
            &payload,
            1,
            [0u8; 32],
        )
        .expect("Should create global envelope");

        assert!(envelope.is_global());
        assert!(envelope.verify(&sender_keys.public_bundle()));

        // Anyone can open a global packet
        let random_keys = ProfileKeys::generate();
        let opened = envelope.open(&random_keys).expect("Should open global");
        assert_eq!(opened, payload);
    }

    #[test]
    fn test_packet_envelope_non_recipient_cannot_open() {
        let sender_keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();
        let attacker_keys = ProfileKeys::generate();
        let recipient_public = recipient_keys.public_bundle();

        let payload = PacketPayload::DirectMessage {
            content: "Secret message".to_string(),
            recipient: recipient_keys.did(),
        };

        let envelope = PacketEnvelope::create(
            &sender_keys,
            &payload,
            &[recipient_public],
            1,
            [0u8; 32],
        )
        .expect("Should create envelope");

        let result = envelope.open(&attacker_keys);
        assert!(result.is_err());
    }

    #[test]
    fn test_packet_envelope_tamper_detection() {
        let sender_keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();
        let recipient_public = recipient_keys.public_bundle();

        let payload = PacketPayload::DirectMessage {
            content: "Original message".to_string(),
            recipient: recipient_keys.did(),
        };

        let mut envelope = PacketEnvelope::create(
            &sender_keys,
            &payload,
            &[recipient_public],
            1,
            [0u8; 32],
        )
        .expect("Should create envelope");

        // Tamper with the timestamp
        envelope.timestamp += 1;

        // Signature should no longer verify
        assert!(!envelope.verify(&sender_keys.public_bundle()));
    }

    #[test]
    fn test_packet_envelope_hash() {
        let sender_keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();
        let recipient_public = recipient_keys.public_bundle();

        let payload = PacketPayload::DirectMessage {
            content: "Test hash".to_string(),
            recipient: recipient_keys.did(),
        };

        let envelope = PacketEnvelope::create(
            &sender_keys,
            &payload,
            &[recipient_public],
            1,
            [0u8; 32],
        )
        .expect("Should create envelope");

        let hash1 = envelope.hash();
        let hash2 = envelope.hash();

        // Hash should be deterministic
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_packet_envelope_serialization() {
        let sender_keys = ProfileKeys::generate();
        let recipient_keys = ProfileKeys::generate();
        let recipient_public = recipient_keys.public_bundle();

        let payload = PacketPayload::DirectMessage {
            content: "Test serialization".to_string(),
            recipient: recipient_keys.did(),
        };

        let envelope = PacketEnvelope::create(
            &sender_keys,
            &payload,
            &[recipient_public],
            1,
            [0u8; 32],
        )
        .expect("Should create envelope");

        let encoded = envelope.encode().expect("Should encode");
        let decoded = PacketEnvelope::decode(&encoded).expect("Should decode");

        // Should still verify and open
        assert!(decoded.verify(&sender_keys.public_bundle()));
        let opened = decoded.open(&recipient_keys).expect("Should open");
        assert_eq!(opened, payload);
    }

    #[test]
    fn test_packet_payload_variants() {
        // Test all payload variants can be serialized
        let payloads = vec![
            PacketPayload::ProfileUpdate {
                display_name: Some("Love".to_string()),
                bio: Some("Hello".to_string()),
                avatar_blob_id: None,
            },
            PacketPayload::RealmInvite {
                realm_id: RealmId::new(),
                realm_key: [42u8; 32],
                realm_name: "Test Realm".to_string(),
            },
            PacketPayload::TaskReference {
                realm_id: RealmId::new(),
                task_id: "task-123".to_string(),
                description: "Do something".to_string(),
            },
            PacketPayload::DirectMessage {
                content: "Hello".to_string(),
                recipient: ProfileKeys::generate().did(),
            },
            PacketPayload::Receipt {
                original_sender: ProfileKeys::generate().did(),
                packet_seq: 42,
            },
            PacketPayload::Depin {
                before_sequence: 100,
                merkle_root: Some([1u8; 32]),
            },
            PacketPayload::Heartbeat {
                timestamp: 12345678,
            },
            PacketPayload::KeyRotation {
                new_public_keys: vec![1, 2, 3],
                old_key_signature: vec![4, 5, 6],
            },
        ];

        for payload in payloads {
            let bytes = postcard::to_allocvec(&payload).expect("Should serialize");
            let recovered: PacketPayload = postcard::from_bytes(&bytes).expect("Should deserialize");
            assert_eq!(payload, recovered);
        }
    }

    #[test]
    fn test_packet_address_includes() {
        let did1 = ProfileKeys::generate().did();
        let did2 = ProfileKeys::generate().did();
        let did3 = ProfileKeys::generate().did();

        // Individual
        let addr = PacketAddress::Individual(did1.clone());
        assert!(addr.includes(&did1));
        assert!(!addr.includes(&did2));

        // List
        let addr = PacketAddress::List(vec![did1.clone(), did2.clone()]);
        assert!(addr.includes(&did1));
        assert!(addr.includes(&did2));
        assert!(!addr.includes(&did3));

        // Group and Global include everyone
        let addr = PacketAddress::Group(RealmId::new());
        assert!(addr.includes(&did1));

        let addr = PacketAddress::Global;
        assert!(addr.includes(&did1));
    }

    #[test]
    fn test_packet_address_explicit_recipients() {
        let did1 = ProfileKeys::generate().did();
        let did2 = ProfileKeys::generate().did();

        assert!(PacketAddress::Individual(did1.clone()).explicit_recipients().is_some());
        assert!(PacketAddress::List(vec![did1, did2]).explicit_recipients().is_some());
        assert!(PacketAddress::Group(RealmId::new()).explicit_recipients().is_none());
        assert!(PacketAddress::Global.explicit_recipients().is_none());
    }
}
