//! Signed and encrypted sync message envelopes
//!
//! The `SyncEnvelope` wraps [`SyncMessage`] with encryption and signatures
//! for secure transmission over gossip.
//!
//! ## Security Model
//!
//! Uses **Encrypt-then-Sign** pattern:
//! 1. Serialize the `SyncMessage` to bytes
//! 2. Encrypt with realm's symmetric key (ChaCha20-Poly1305)
//! 3. Sign the envelope fields (version, sender, ciphertext, nonce)
//!
//! This ensures:
//! - **Confidentiality**: Only realm members can read messages
//! - **Authenticity**: Messages can be verified as coming from the claimed sender
//! - **Integrity**: Any tampering is detected via AEAD tag and signature
//!
//! ## Wire Format
//!
//! ```text
//! +----------+--------+------------+-------+-----------+
//! | version  | sender | ciphertext | nonce | signature |
//! | (1 byte) | (DID)  | (variable) | (12)  | (variable)|
//! +----------+--------+------------+-------+-----------+
//! ```
//!
//! ## Example
//!
//! ```ignore
//! use syncengine_core::sync::{SyncMessage, SyncEnvelope};
//! use syncengine_core::RealmId;
//!
//! // Create a message
//! let msg = SyncMessage::Announce {
//!     realm_id: RealmId::new(),
//!     heads: vec![],
//! };
//!
//! // Seal (encrypt and sign)
//! let envelope = SyncEnvelope::seal(
//!     &msg,
//!     "did:example:love",
//!     &realm_key,
//!     |data| sign_with_private_key(data),
//! )?;
//!
//! // Send over gossip...
//! let bytes = envelope.to_bytes()?;
//!
//! // Receive and open (verify and decrypt)
//! let envelope = SyncEnvelope::from_bytes(&bytes)?;
//! let msg = envelope.open(
//!     &realm_key,
//!     |sender, data, sig| verify_signature(sender, data, sig),
//! )?;
//! ```

use serde::{Deserialize, Serialize};

use crate::crypto::{RealmCrypto, NONCE_SIZE};
use crate::error::SyncError;
use crate::sync::protocol::SyncMessage;

/// Current envelope protocol version
pub const ENVELOPE_VERSION: u8 = 1;

/// A signed and encrypted wrapper for sync messages.
///
/// The envelope provides:
/// - Encryption using the realm's symmetric key
/// - Signature from the sender for authentication
/// - Version field for protocol evolution
///
/// ## Security Properties
///
/// - **Encrypt-then-Sign**: The signature covers the ciphertext, not plaintext.
///   This prevents an attacker from stripping/replacing signatures on encrypted data.
/// - **AEAD encryption**: ChaCha20-Poly1305 provides authenticated encryption,
///   detecting any tampering with the ciphertext.
/// - **Random nonce**: Each envelope uses a fresh 12-byte random nonce.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEnvelope {
    /// Protocol version for forward compatibility
    pub version: u8,

    /// Sender's DID (will be Did type once identity module ready)
    pub sender: String,

    /// Encrypted payload (SyncMessage serialized then encrypted)
    pub ciphertext: Vec<u8>,

    /// Nonce used for encryption (12 bytes for ChaCha20-Poly1305)
    pub nonce: [u8; NONCE_SIZE],

    /// Signature over (version || sender || ciphertext || nonce)
    /// Will be HybridSignature once identity module ready
    pub signature: Vec<u8>,
}

impl SyncEnvelope {
    /// Create a new envelope by encrypting and signing a message.
    ///
    /// This implements the **Encrypt-then-Sign** pattern:
    /// 1. Serialize the message with postcard
    /// 2. Encrypt with the realm key using a random nonce
    /// 3. Sign the envelope fields (not the plaintext)
    ///
    /// # Arguments
    ///
    /// * `message` - The sync message to seal
    /// * `sender_did` - The sender's DID string
    /// * `realm_key` - The 32-byte realm encryption key
    /// * `sign_fn` - Function that signs data and returns signature bytes
    ///
    /// # Returns
    ///
    /// A sealed envelope ready for transmission.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Serialization` if message serialization fails,
    /// or `SyncError::Crypto` if encryption fails.
    pub fn seal(
        message: &SyncMessage,
        sender_did: &str,
        realm_key: &[u8; 32],
        sign_fn: impl Fn(&[u8]) -> Vec<u8>,
    ) -> Result<Self, SyncError> {
        // 1. Serialize the message
        let plaintext = message
            .encode()
            .map_err(|e| SyncError::Serialization(format!("Failed to encode message: {}", e)))?;

        // 2. Generate random nonce
        let nonce = RealmCrypto::generate_nonce();

        // 3. Encrypt with realm key
        let crypto = RealmCrypto::new(realm_key);
        let ciphertext = crypto.encrypt_with_nonce(&plaintext, &nonce)?;

        // 4. Build envelope (without signature yet)
        let mut envelope = Self {
            version: ENVELOPE_VERSION,
            sender: sender_did.to_string(),
            ciphertext,
            nonce,
            signature: Vec::new(),
        };

        // 5. Sign the envelope data
        let signed_data = envelope.signed_data();
        envelope.signature = sign_fn(&signed_data);

        Ok(envelope)
    }

    /// Open an envelope by verifying the signature and decrypting.
    ///
    /// This reverses the seal operation:
    /// 1. Verify the signature over envelope fields
    /// 2. Decrypt the ciphertext using the realm key
    /// 3. Deserialize the message
    ///
    /// # Arguments
    ///
    /// * `realm_key` - The 32-byte realm encryption key
    /// * `verify_fn` - Function that verifies (sender_did, data, signature) -> bool
    ///
    /// # Returns
    ///
    /// The decrypted and verified `SyncMessage`.
    ///
    /// # Errors
    ///
    /// - `SyncError::EnvelopeVersionUnsupported` if version is unknown
    /// - `SyncError::SignatureInvalid` if signature verification fails
    /// - `SyncError::DecryptionFailed` if decryption fails
    /// - `SyncError::Serialization` if message deserialization fails
    pub fn open(
        &self,
        realm_key: &[u8; 32],
        verify_fn: impl Fn(&str, &[u8], &[u8]) -> bool,
    ) -> Result<SyncMessage, SyncError> {
        // 1. Check version
        if self.version != ENVELOPE_VERSION {
            return Err(SyncError::EnvelopeVersionUnsupported(self.version));
        }

        // 2. Verify signature
        let signed_data = self.signed_data();
        if !verify_fn(&self.sender, &signed_data, &self.signature) {
            return Err(SyncError::SignatureInvalid(format!(
                "Signature verification failed for sender: {}",
                self.sender
            )));
        }

        // 3. Decrypt
        let crypto = RealmCrypto::new(realm_key);
        let plaintext = crypto.decrypt_with_nonce(&self.ciphertext, &self.nonce)?;

        // 4. Deserialize
        let message = SyncMessage::decode(&plaintext)
            .map_err(|e| SyncError::Serialization(format!("Failed to decode message: {}", e)))?;

        Ok(message)
    }

    /// Get the data that is signed.
    ///
    /// The signed data is: version || sender || ciphertext || nonce
    ///
    /// This is deterministic for the same envelope fields.
    fn signed_data(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Version (1 byte)
        data.push(self.version);

        // Sender length (4 bytes, little-endian) + sender bytes
        let sender_bytes = self.sender.as_bytes();
        data.extend_from_slice(&(sender_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(sender_bytes);

        // Ciphertext length (4 bytes, little-endian) + ciphertext
        data.extend_from_slice(&(self.ciphertext.len() as u32).to_le_bytes());
        data.extend_from_slice(&self.ciphertext);

        // Nonce (12 bytes, fixed size)
        data.extend_from_slice(&self.nonce);

        data
    }

    /// Encode the envelope to bytes for transmission.
    ///
    /// Uses postcard for compact binary serialization.
    pub fn to_bytes(&self) -> Result<Vec<u8>, SyncError> {
        postcard::to_allocvec(self)
            .map_err(|e| SyncError::Serialization(format!("Failed to encode envelope: {}", e)))
    }

    /// Decode an envelope from bytes.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Serialization` if deserialization fails.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SyncError> {
        postcard::from_bytes(bytes)
            .map_err(|e| SyncError::Serialization(format!("Failed to decode envelope: {}", e)))
    }

    /// Get the sender's DID.
    pub fn sender(&self) -> &str {
        &self.sender
    }

    /// Get the protocol version.
    pub fn version(&self) -> u8 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RealmId;

    /// Simple mock signer that just copies the data as "signature"
    fn mock_sign(data: &[u8]) -> Vec<u8> {
        // In real code, this would be a cryptographic signature
        // For testing, we just hash-like the data
        let mut sig = vec![0x51, 0x9E]; // "SIGN" marker bytes
        sig.extend_from_slice(&data[..data.len().min(32)]);
        sig
    }

    /// Mock verifier that checks our mock signature format
    fn mock_verify(_sender: &str, data: &[u8], signature: &[u8]) -> bool {
        if signature.len() < 2 {
            return false;
        }
        // Check marker and data prefix
        signature[0] == 0x51 && signature[1] == 0x9E && signature[2..] == data[..data.len().min(32)]
    }

    /// Mock verifier that always fails
    fn mock_verify_fail(_sender: &str, _data: &[u8], _signature: &[u8]) -> bool {
        false
    }

    #[test]
    fn test_envelope_seal_open_roundtrip() {
        let realm_id = RealmId::new();
        let realm_key = RealmCrypto::generate_key();
        let sender = "did:example:love";

        // Create a test message
        let message = SyncMessage::Changes {
            realm_id: realm_id.clone(),
            data: vec![1, 2, 3, 4, 5],
        };

        // Seal the envelope
        let envelope = SyncEnvelope::seal(&message, sender, &realm_key, mock_sign).unwrap();

        // Verify envelope fields
        assert_eq!(envelope.version, ENVELOPE_VERSION);
        assert_eq!(envelope.sender, sender);
        assert!(!envelope.ciphertext.is_empty());
        assert!(!envelope.signature.is_empty());

        // Open the envelope
        let opened = envelope.open(&realm_key, mock_verify).unwrap();

        // Verify the message matches
        match opened {
            SyncMessage::Changes {
                realm_id: rid,
                data,
            } => {
                assert_eq!(rid.0, realm_id.0);
                assert_eq!(data, vec![1, 2, 3, 4, 5]);
            }
            _ => panic!("Wrong message type after opening envelope"),
        }
    }

    #[test]
    fn test_envelope_wrong_key_fails() {
        let realm_id = RealmId::new();
        let realm_key = RealmCrypto::generate_key();
        let wrong_key = RealmCrypto::generate_key();
        let sender = "did:example:love";

        let message = SyncMessage::Announce {
            realm_id,
            heads: vec![vec![0u8; 32]],
            sender_addr: None,
        };

        // Seal with correct key
        let envelope = SyncEnvelope::seal(&message, sender, &realm_key, mock_sign).unwrap();

        // Try to open with wrong key
        let result = envelope.open(&wrong_key, mock_verify);

        assert!(result.is_err());
        match result {
            Err(SyncError::DecryptionFailed(_)) => {}
            Err(e) => panic!("Expected DecryptionFailed, got: {:?}", e),
            Ok(_) => panic!("Expected error, but got Ok"),
        }
    }

    #[test]
    fn test_envelope_tampered_ciphertext_fails() {
        let realm_id = RealmId::new();
        let realm_key = RealmCrypto::generate_key();
        let sender = "did:example:love";

        let message = SyncMessage::SyncRequest { realm_id };

        // Seal the envelope
        let mut envelope = SyncEnvelope::seal(&message, sender, &realm_key, mock_sign).unwrap();

        // Tamper with the ciphertext
        if !envelope.ciphertext.is_empty() {
            envelope.ciphertext[0] ^= 0xFF;
        }

        // Try to open - should fail signature verification (data changed)
        let result = envelope.open(&realm_key, mock_verify);

        assert!(result.is_err());
        // Could fail at signature or decryption depending on implementation
        match result {
            Err(SyncError::SignatureInvalid(_)) | Err(SyncError::DecryptionFailed(_)) => {}
            Err(e) => panic!(
                "Expected SignatureInvalid or DecryptionFailed, got: {:?}",
                e
            ),
            Ok(_) => panic!("Expected error, but got Ok"),
        }
    }

    #[test]
    fn test_envelope_invalid_signature_fails() {
        let realm_id = RealmId::new();
        let realm_key = RealmCrypto::generate_key();
        let sender = "did:example:love";

        let message = SyncMessage::Announce {
            realm_id,
            heads: vec![],
            sender_addr: None,
        };

        // Seal the envelope
        let envelope = SyncEnvelope::seal(&message, sender, &realm_key, mock_sign).unwrap();

        // Try to open with a verifier that always fails
        let result = envelope.open(&realm_key, mock_verify_fail);

        assert!(result.is_err());
        match result {
            Err(SyncError::SignatureInvalid(_)) => {}
            Err(e) => panic!("Expected SignatureInvalid, got: {:?}", e),
            Ok(_) => panic!("Expected error, but got Ok"),
        }
    }

    #[test]
    fn test_envelope_serialization() {
        let realm_id = RealmId::new();
        let realm_key = RealmCrypto::generate_key();
        let sender = "did:example:joy";

        let message = SyncMessage::SyncResponse {
            realm_id,
            document: vec![0x85, 0x6f, 0x4a, 0x83, 0x01, 0x02, 0x03],
        };

        // Seal and serialize
        let envelope = SyncEnvelope::seal(&message, sender, &realm_key, mock_sign).unwrap();
        let bytes = envelope.to_bytes().unwrap();

        // Deserialize
        let restored = SyncEnvelope::from_bytes(&bytes).unwrap();

        // Verify fields match
        assert_eq!(restored.version, envelope.version);
        assert_eq!(restored.sender, envelope.sender);
        assert_eq!(restored.ciphertext, envelope.ciphertext);
        assert_eq!(restored.nonce, envelope.nonce);
        assert_eq!(restored.signature, envelope.signature);

        // Should be able to open the restored envelope
        let opened = restored.open(&realm_key, mock_verify).unwrap();
        assert!(matches!(opened, SyncMessage::SyncResponse { .. }));
    }

    #[test]
    fn test_envelope_unsupported_version() {
        let realm_key = RealmCrypto::generate_key();

        // Manually create an envelope with unsupported version
        let envelope = SyncEnvelope {
            version: 99, // Unsupported version
            sender: "did:example:love".to_string(),
            ciphertext: vec![1, 2, 3],
            nonce: [0u8; NONCE_SIZE],
            signature: vec![0x51, 0x9E, 1, 2, 3],
        };

        let result = envelope.open(&realm_key, mock_verify);

        assert!(result.is_err());
        match result {
            Err(SyncError::EnvelopeVersionUnsupported(v)) => {
                assert_eq!(v, 99);
            }
            Err(e) => panic!("Expected EnvelopeVersionUnsupported, got: {:?}", e),
            Ok(_) => panic!("Expected error, but got Ok"),
        }
    }

    #[test]
    fn test_envelope_different_nonces() {
        let realm_id = RealmId::new();
        let realm_key = RealmCrypto::generate_key();
        let sender = "did:example:love";

        let message = SyncMessage::Changes {
            realm_id,
            data: vec![1, 2, 3],
        };

        // Create two envelopes with the same message
        let envelope1 = SyncEnvelope::seal(&message, sender, &realm_key, mock_sign).unwrap();
        let envelope2 = SyncEnvelope::seal(&message, sender, &realm_key, mock_sign).unwrap();

        // Nonces should be different (random)
        assert_ne!(envelope1.nonce, envelope2.nonce);

        // Ciphertexts should also be different due to different nonces
        assert_ne!(envelope1.ciphertext, envelope2.ciphertext);

        // But both should decrypt to the same message
        let opened1 = envelope1.open(&realm_key, mock_verify).unwrap();
        let opened2 = envelope2.open(&realm_key, mock_verify).unwrap();

        match (opened1, opened2) {
            (
                SyncMessage::Changes {
                    data: d1,
                    realm_id: _,
                },
                SyncMessage::Changes {
                    data: d2,
                    realm_id: _,
                },
            ) => {
                assert_eq!(d1, d2);
            }
            _ => panic!("Messages should match"),
        }
    }

    #[test]
    fn test_signed_data_deterministic() {
        let envelope = SyncEnvelope {
            version: 1,
            sender: "did:example:test".to_string(),
            ciphertext: vec![1, 2, 3, 4],
            nonce: [0u8; NONCE_SIZE],
            signature: vec![],
        };

        // signed_data should be deterministic
        let data1 = envelope.signed_data();
        let data2 = envelope.signed_data();

        assert_eq!(data1, data2);
    }

    #[test]
    fn test_envelope_accessors() {
        let envelope = SyncEnvelope {
            version: 1,
            sender: "did:example:peace".to_string(),
            ciphertext: vec![],
            nonce: [0u8; NONCE_SIZE],
            signature: vec![],
        };

        assert_eq!(envelope.sender(), "did:example:peace");
        assert_eq!(envelope.version(), 1);
    }

    #[test]
    fn test_envelope_empty_message() {
        let realm_id = RealmId::new();
        let realm_key = RealmCrypto::generate_key();
        let sender = "did:example:love";

        // Announce with empty heads
        let message = SyncMessage::Announce {
            realm_id: realm_id.clone(),
            heads: vec![],
            sender_addr: None,
        };

        let envelope = SyncEnvelope::seal(&message, sender, &realm_key, mock_sign).unwrap();
        let opened = envelope.open(&realm_key, mock_verify).unwrap();

        match opened {
            SyncMessage::Announce { heads, .. } => {
                assert!(heads.is_empty());
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_envelope_large_message() {
        let realm_id = RealmId::new();
        let realm_key = RealmCrypto::generate_key();
        let sender = "did:example:love";

        // Large document (100KB)
        let large_doc: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();

        let message = SyncMessage::SyncResponse {
            realm_id: realm_id.clone(),
            document: large_doc.clone(),
        };

        let envelope = SyncEnvelope::seal(&message, sender, &realm_key, mock_sign).unwrap();

        // Serialize and deserialize
        let bytes = envelope.to_bytes().unwrap();
        let restored = SyncEnvelope::from_bytes(&bytes).unwrap();

        let opened = restored.open(&realm_key, mock_verify).unwrap();

        match opened {
            SyncMessage::SyncResponse { document, .. } => {
                assert_eq!(document.len(), 100_000);
                assert_eq!(document, large_doc);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
