//! Hybrid signature combining Ed25519 and ML-DSA-65 (Dilithium5)
//!
//! Both signatures must verify for the overall signature to be valid,
//! providing quantum-resistant security while maintaining classical security.

use ed25519_dalek::Signature as Ed25519Signature;
use pqcrypto_dilithium::dilithium5;
use pqcrypto_traits::sign::SignedMessage;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Hybrid signature containing both Ed25519 and ML-DSA-65 (Dilithium5) signatures.
///
/// Both signatures must verify for the overall signature to be valid.
/// This provides security against both classical and quantum attacks.
#[derive(Clone)]
pub struct HybridSignature {
    /// Classical Ed25519 signature (64 bytes)
    pub(crate) ed25519: Ed25519Signature,
    /// Post-quantum ML-DSA-65 (Dilithium5) signed message
    pub(crate) ml_dsa: dilithium5::SignedMessage,
}

impl HybridSignature {
    /// Create a new hybrid signature from components
    pub(crate) fn new(ed25519: Ed25519Signature, ml_dsa: dilithium5::SignedMessage) -> Self {
        Self { ed25519, ml_dsa }
    }

    /// Get the Ed25519 signature component
    pub fn ed25519(&self) -> &Ed25519Signature {
        &self.ed25519
    }

    /// Get the ML-DSA-65 signed message component
    pub fn ml_dsa(&self) -> &dilithium5::SignedMessage {
        &self.ml_dsa
    }

    /// Serialize the signature to bytes
    ///
    /// Format: [ed25519_sig: 64 bytes][ml_dsa_len: 4 bytes LE][ml_dsa_sig: variable]
    pub fn to_bytes(&self) -> Vec<u8> {
        let ed25519_bytes = self.ed25519.to_bytes();
        let ml_dsa_bytes = self.ml_dsa.as_bytes();
        let ml_dsa_len = ml_dsa_bytes.len() as u32;

        let mut bytes = Vec::with_capacity(64 + 4 + ml_dsa_bytes.len());
        bytes.extend_from_slice(&ed25519_bytes);
        bytes.extend_from_slice(&ml_dsa_len.to_le_bytes());
        bytes.extend_from_slice(ml_dsa_bytes);
        bytes
    }

    /// Deserialize a signature from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, crate::SyncError> {
        if bytes.len() < 68 {
            return Err(crate::SyncError::Identity(
                "Signature too short".to_string(),
            ));
        }

        // Ed25519 signature (64 bytes)
        let ed25519_bytes: [u8; 64] = bytes[..64]
            .try_into()
            .map_err(|_| crate::SyncError::Identity("Invalid Ed25519 signature length".to_string()))?;
        let ed25519 = Ed25519Signature::from_bytes(&ed25519_bytes);

        // ML-DSA length (4 bytes)
        let ml_dsa_len = u32::from_le_bytes(
            bytes[64..68]
                .try_into()
                .map_err(|_| crate::SyncError::Identity("Invalid ML-DSA length".to_string()))?,
        ) as usize;

        if bytes.len() < 68 + ml_dsa_len {
            return Err(crate::SyncError::Identity(
                "Signature data truncated".to_string(),
            ));
        }

        // ML-DSA signed message
        let ml_dsa = dilithium5::SignedMessage::from_bytes(&bytes[68..68 + ml_dsa_len])
            .map_err(|_| crate::SyncError::Identity("Invalid ML-DSA signature".to_string()))?;

        Ok(Self { ed25519, ml_dsa })
    }
}

impl std::fmt::Debug for HybridSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HybridSignature")
            .field("ed25519", &hex::encode(self.ed25519.to_bytes()))
            .field("ml_dsa_len", &self.ml_dsa.as_bytes().len())
            .finish()
    }
}

impl Serialize for HybridSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

impl<'de> Deserialize<'de> for HybridSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = <Vec<u8>>::deserialize(deserializer)?;
        Self::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_serialization_roundtrip() {
        // Create a test signature using actual crypto operations
        use crate::identity::HybridKeypair;

        let keypair = HybridKeypair::generate();
        let message = b"test message for signature";
        let signature = keypair.sign(message);

        // Serialize and deserialize
        let bytes = signature.to_bytes();
        let recovered = HybridSignature::from_bytes(&bytes).expect("Failed to deserialize");

        // Verify both signatures are equal
        assert_eq!(signature.ed25519.to_bytes(), recovered.ed25519.to_bytes());
        assert_eq!(signature.ml_dsa.as_bytes(), recovered.ml_dsa.as_bytes());
    }

    #[test]
    fn test_signature_serde_roundtrip() {
        use crate::identity::HybridKeypair;

        let keypair = HybridKeypair::generate();
        let message = b"test message for serde";
        let signature = keypair.sign(message);

        // Serialize with serde
        let json = serde_json::to_string(&signature).expect("Failed to serialize");
        let recovered: HybridSignature = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(signature.ed25519.to_bytes(), recovered.ed25519.to_bytes());
    }

    #[test]
    fn test_signature_debug_format() {
        use crate::identity::HybridKeypair;

        let keypair = HybridKeypair::generate();
        let signature = keypair.sign(b"test");

        let debug_str = format!("{:?}", signature);
        assert!(debug_str.contains("HybridSignature"));
        assert!(debug_str.contains("ed25519"));
        assert!(debug_str.contains("ml_dsa_len"));
    }
}
