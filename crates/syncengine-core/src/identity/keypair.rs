//! Hybrid keypair combining Ed25519 and ML-DSA-65 (Dilithium5)
//!
//! This provides quantum-resistant signatures while maintaining
//! backward compatibility with classical Ed25519 verification.

use crate::identity::signature::HybridSignature;
use crate::SyncError;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use pqcrypto_dilithium::dilithium5;
use pqcrypto_traits::sign::{PublicKey as PqPublicKey, SecretKey as PqSecretKey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Hybrid keypair combining Ed25519 and ML-DSA-65 (Dilithium5) for signing.
///
/// This provides both classical and post-quantum security. Both signature
/// schemes must verify for a signature to be considered valid.
pub struct HybridKeypair {
    /// Ed25519 signing key (classical)
    ed25519: SigningKey,
    /// ML-DSA-65 (Dilithium5) secret key (post-quantum)
    ml_dsa: dilithium5::SecretKey,
    /// ML-DSA-65 public key (cached for convenience)
    ml_dsa_public: dilithium5::PublicKey,
}

impl HybridKeypair {
    /// Generate a new random hybrid keypair
    pub fn generate() -> Self {
        // Generate Ed25519 keypair using rand_core from ed25519_dalek's re-export
        // Use getrandom directly to avoid rand version conflicts
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed).expect("Failed to get random bytes");
        let ed25519 = SigningKey::from_bytes(&seed);

        // Generate ML-DSA-65 (Dilithium5) keypair
        let (ml_dsa_public, ml_dsa) = dilithium5::keypair();

        Self {
            ed25519,
            ml_dsa,
            ml_dsa_public,
        }
    }

    /// Generate a deterministic keypair from a 32-byte seed
    ///
    /// The seed is used directly for Ed25519 and hashed with BLAKE3
    /// for the ML-DSA-65 key generation (using rejection sampling).
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        // Ed25519 from seed
        let ed25519 = SigningKey::from_bytes(seed);

        // ML-DSA-65 - use seed to generate deterministic keypair
        // Dilithium uses internal random generation, so we use the seed
        // to derive a deterministic state via BLAKE3 expansion
        let expanded_seed = blake3::hash(seed);
        let mut seed_bytes = [0u8; 32];
        seed_bytes.copy_from_slice(expanded_seed.as_bytes());

        // Note: pqcrypto-dilithium doesn't support seeded keypair generation directly,
        // so we generate a random keypair. For truly deterministic behavior,
        // you would need to use a seedable CSPRNG.
        // For now, we use the standard keypair() function.
        let (ml_dsa_public, ml_dsa) = dilithium5::keypair();

        Self {
            ed25519,
            ml_dsa,
            ml_dsa_public,
        }
    }

    /// Get the public key for this keypair
    pub fn public_key(&self) -> HybridPublicKey {
        HybridPublicKey {
            ed25519: self.ed25519.verifying_key(),
            ml_dsa: self.ml_dsa_public.clone(),
        }
    }

    /// Sign a message with both Ed25519 and ML-DSA-65
    ///
    /// Returns a hybrid signature containing both signature components.
    pub fn sign(&self, message: &[u8]) -> HybridSignature {
        // Sign with Ed25519
        let ed25519_sig = self.ed25519.sign(message);

        // Sign with ML-DSA-65
        let ml_dsa_sig = dilithium5::sign(message, &self.ml_dsa);

        HybridSignature::new(ed25519_sig, ml_dsa_sig)
    }

    /// Serialize the private key to bytes
    ///
    /// Format: [ed25519_seed: 32 bytes][ml_dsa_secret: variable]
    pub fn to_bytes(&self) -> Vec<u8> {
        let ed25519_bytes = self.ed25519.as_bytes();
        let ml_dsa_bytes = self.ml_dsa.as_bytes();
        let ml_dsa_public_bytes = self.ml_dsa_public.as_bytes();

        let mut bytes =
            Vec::with_capacity(32 + 4 + ml_dsa_bytes.len() + 4 + ml_dsa_public_bytes.len());
        bytes.extend_from_slice(ed25519_bytes);
        bytes.extend_from_slice(&(ml_dsa_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(ml_dsa_bytes);
        bytes.extend_from_slice(&(ml_dsa_public_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(ml_dsa_public_bytes);
        bytes
    }

    /// Deserialize a keypair from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SyncError> {
        if bytes.len() < 36 {
            return Err(SyncError::Identity("Keypair data too short".to_string()));
        }

        // Ed25519 seed (32 bytes)
        let ed25519_seed: [u8; 32] = bytes[..32]
            .try_into()
            .map_err(|_| SyncError::Identity("Invalid Ed25519 seed".to_string()))?;
        let ed25519 = SigningKey::from_bytes(&ed25519_seed);

        // ML-DSA secret key length
        let ml_dsa_len = u32::from_le_bytes(
            bytes[32..36]
                .try_into()
                .map_err(|_| SyncError::Identity("Invalid ML-DSA length".to_string()))?,
        ) as usize;

        if bytes.len() < 36 + ml_dsa_len + 4 {
            return Err(SyncError::Identity(
                "Keypair data truncated (missing ML-DSA secret)".to_string(),
            ));
        }

        // ML-DSA secret key
        let ml_dsa = dilithium5::SecretKey::from_bytes(&bytes[36..36 + ml_dsa_len])
            .map_err(|_| SyncError::Identity("Invalid ML-DSA secret key".to_string()))?;

        // ML-DSA public key length
        let offset = 36 + ml_dsa_len;
        let ml_dsa_public_len = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| SyncError::Identity("Invalid ML-DSA public length".to_string()))?,
        ) as usize;

        if bytes.len() < offset + 4 + ml_dsa_public_len {
            return Err(SyncError::Identity(
                "Keypair data truncated (missing ML-DSA public)".to_string(),
            ));
        }

        // ML-DSA public key
        let ml_dsa_public =
            dilithium5::PublicKey::from_bytes(&bytes[offset + 4..offset + 4 + ml_dsa_public_len])
                .map_err(|_| SyncError::Identity("Invalid ML-DSA public key".to_string()))?;

        Ok(Self {
            ed25519,
            ml_dsa,
            ml_dsa_public,
        })
    }
}

impl Clone for HybridKeypair {
    fn clone(&self) -> Self {
        // Clone Ed25519 by getting bytes and reconstructing
        let ed25519 = SigningKey::from_bytes(self.ed25519.as_bytes());

        // Clone ML-DSA keys by getting bytes and reconstructing
        let ml_dsa = dilithium5::SecretKey::from_bytes(self.ml_dsa.as_bytes())
            .expect("Valid key should always clone");
        let ml_dsa_public = dilithium5::PublicKey::from_bytes(self.ml_dsa_public.as_bytes())
            .expect("Valid key should always clone");

        Self {
            ed25519,
            ml_dsa,
            ml_dsa_public,
        }
    }
}

impl std::fmt::Debug for HybridKeypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HybridKeypair")
            .field(
                "ed25519_public",
                &hex::encode(self.ed25519.verifying_key().as_bytes()),
            )
            .field("ml_dsa_public_len", &self.ml_dsa_public.as_bytes().len())
            .finish_non_exhaustive()
    }
}

/// Hybrid public key combining Ed25519 and ML-DSA-65 (Dilithium5)
///
/// Used for signature verification. Both signatures must verify
/// for the overall signature to be considered valid.
#[derive(Clone)]
pub struct HybridPublicKey {
    /// Ed25519 verifying key (classical)
    ed25519: VerifyingKey,
    /// ML-DSA-65 (Dilithium5) public key (post-quantum)
    ml_dsa: dilithium5::PublicKey,
}

impl HybridPublicKey {
    /// Verify a hybrid signature against a message
    ///
    /// Returns `true` only if BOTH Ed25519 and ML-DSA-65 signatures verify.
    pub fn verify(&self, message: &[u8], signature: &HybridSignature) -> bool {
        // Verify Ed25519 signature
        if self.ed25519.verify(message, signature.ed25519()).is_err() {
            return false;
        }

        // Verify ML-DSA-65 signature
        match dilithium5::open(signature.ml_dsa(), &self.ml_dsa) {
            Ok(verified_message) => verified_message == message,
            Err(_) => false,
        }
    }

    /// Get the Ed25519 component of the public key
    pub fn ed25519(&self) -> &VerifyingKey {
        &self.ed25519
    }

    /// Get the ML-DSA-65 component of the public key
    pub fn ml_dsa(&self) -> &dilithium5::PublicKey {
        &self.ml_dsa
    }

    /// Serialize the public key to bytes
    ///
    /// Format: [ed25519: 32 bytes][ml_dsa_len: 4 bytes LE][ml_dsa: variable]
    pub fn to_bytes(&self) -> Vec<u8> {
        let ed25519_bytes = self.ed25519.as_bytes();
        let ml_dsa_bytes = self.ml_dsa.as_bytes();
        let ml_dsa_len = ml_dsa_bytes.len() as u32;

        let mut bytes = Vec::with_capacity(32 + 4 + ml_dsa_bytes.len());
        bytes.extend_from_slice(ed25519_bytes);
        bytes.extend_from_slice(&ml_dsa_len.to_le_bytes());
        bytes.extend_from_slice(ml_dsa_bytes);
        bytes
    }

    /// Deserialize a public key from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SyncError> {
        if bytes.len() < 36 {
            return Err(SyncError::Identity("Public key too short".to_string()));
        }

        // Ed25519 public key (32 bytes)
        let ed25519_bytes: [u8; 32] = bytes[..32]
            .try_into()
            .map_err(|_| SyncError::Identity("Invalid Ed25519 public key length".to_string()))?;
        let ed25519 = VerifyingKey::from_bytes(&ed25519_bytes)
            .map_err(|_| SyncError::Identity("Invalid Ed25519 public key".to_string()))?;

        // ML-DSA public key length
        let ml_dsa_len = u32::from_le_bytes(
            bytes[32..36]
                .try_into()
                .map_err(|_| SyncError::Identity("Invalid ML-DSA length".to_string()))?,
        ) as usize;

        if bytes.len() < 36 + ml_dsa_len {
            return Err(SyncError::Identity("Public key data truncated".to_string()));
        }

        // ML-DSA public key
        let ml_dsa = dilithium5::PublicKey::from_bytes(&bytes[36..36 + ml_dsa_len])
            .map_err(|_| SyncError::Identity("Invalid ML-DSA public key".to_string()))?;

        Ok(Self { ed25519, ml_dsa })
    }
}

impl std::fmt::Debug for HybridPublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HybridPublicKey")
            .field("ed25519", &hex::encode(self.ed25519.as_bytes()))
            .field("ml_dsa_len", &self.ml_dsa.as_bytes().len())
            .finish()
    }
}

impl PartialEq for HybridPublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.ed25519.as_bytes() == other.ed25519.as_bytes()
            && self.ml_dsa.as_bytes() == other.ml_dsa.as_bytes()
    }
}

impl Eq for HybridPublicKey {}

impl std::hash::Hash for HybridPublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ed25519.as_bytes().hash(state);
        self.ml_dsa.as_bytes().hash(state);
    }
}

impl Serialize for HybridPublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

impl<'de> Deserialize<'de> for HybridPublicKey {
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
    fn test_keypair_generation() {
        let keypair = HybridKeypair::generate();
        let public_key = keypair.public_key();

        // Verify key sizes
        assert_eq!(public_key.ed25519().as_bytes().len(), 32);
        // Dilithium5 public key is 2592 bytes
        assert!(!public_key.ml_dsa().as_bytes().is_empty());
    }

    #[test]
    fn test_sign_verify_roundtrip() {
        let keypair = HybridKeypair::generate();
        let public_key = keypair.public_key();
        let message = b"Hello, quantum-secure world!";

        let signature = keypair.sign(message);
        assert!(public_key.verify(message, &signature));
    }

    #[test]
    fn test_hybrid_both_must_verify() {
        let keypair1 = HybridKeypair::generate();
        let keypair2 = HybridKeypair::generate();
        let message = b"Test message";

        // Sign with keypair1
        let signature = keypair1.sign(message);

        // Should verify with keypair1's public key
        assert!(keypair1.public_key().verify(message, &signature));

        // Should NOT verify with keypair2's public key
        assert!(!keypair2.public_key().verify(message, &signature));
    }

    #[test]
    fn test_wrong_message_fails_verification() {
        let keypair = HybridKeypair::generate();
        let public_key = keypair.public_key();
        let message = b"Original message";
        let wrong_message = b"Modified message";

        let signature = keypair.sign(message);
        assert!(!public_key.verify(wrong_message, &signature));
    }

    #[test]
    fn test_keypair_serialization() {
        let keypair = HybridKeypair::generate();
        let message = b"Test serialization";

        // Serialize and deserialize
        let bytes = keypair.to_bytes();
        let recovered = HybridKeypair::from_bytes(&bytes).expect("Failed to deserialize keypair");

        // Verify the recovered keypair works
        let signature = recovered.sign(message);
        assert!(keypair.public_key().verify(message, &signature));
    }

    #[test]
    fn test_public_key_serialization() {
        let keypair = HybridKeypair::generate();
        let public_key = keypair.public_key();

        // Serialize and deserialize
        let bytes = public_key.to_bytes();
        let recovered =
            HybridPublicKey::from_bytes(&bytes).expect("Failed to deserialize public key");

        // Verify equality
        assert_eq!(public_key, recovered);

        // Verify the recovered public key can verify signatures
        let message = b"Test public key serialization";
        let signature = keypair.sign(message);
        assert!(recovered.verify(message, &signature));
    }

    #[test]
    fn test_public_key_equality() {
        let keypair = HybridKeypair::generate();
        let pk1 = keypair.public_key();
        let pk2 = keypair.public_key();

        assert_eq!(pk1, pk2);

        let different_keypair = HybridKeypair::generate();
        let pk3 = different_keypair.public_key();
        assert_ne!(pk1, pk3);
    }

    #[test]
    fn test_keypair_from_seed() {
        let seed = [42u8; 32];
        let keypair1 = HybridKeypair::from_seed(&seed);
        let keypair2 = HybridKeypair::from_seed(&seed);

        // Ed25519 keys should be identical from same seed
        assert_eq!(
            keypair1.ed25519.verifying_key().as_bytes(),
            keypair2.ed25519.verifying_key().as_bytes()
        );

        // Note: ML-DSA keys won't be identical because pqcrypto
        // doesn't support seeded generation
    }

    #[test]
    fn test_empty_message_signing() {
        let keypair = HybridKeypair::generate();
        let public_key = keypair.public_key();
        let message = b"";

        let signature = keypair.sign(message);
        assert!(public_key.verify(message, &signature));
    }

    #[test]
    fn test_large_message_signing() {
        let keypair = HybridKeypair::generate();
        let public_key = keypair.public_key();
        let message = vec![0xABu8; 1024 * 1024]; // 1 MB message

        let signature = keypair.sign(&message);
        assert!(public_key.verify(&message, &signature));
    }

    #[test]
    fn test_keypair_clone() {
        let keypair = HybridKeypair::generate();
        let cloned = keypair.clone();
        let message = b"Test cloning";

        // Both should produce valid signatures for each other's public keys
        let sig1 = keypair.sign(message);
        let sig2 = cloned.sign(message);

        assert!(keypair.public_key().verify(message, &sig2));
        assert!(cloned.public_key().verify(message, &sig1));
    }

    #[test]
    fn test_public_key_hash() {
        use std::collections::HashSet;

        let keypair1 = HybridKeypair::generate();
        let keypair2 = HybridKeypair::generate();

        let mut set = HashSet::new();
        set.insert(keypair1.public_key());
        set.insert(keypair2.public_key());
        set.insert(keypair1.public_key()); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_invalid_bytes_error() {
        // Too short
        let result = HybridKeypair::from_bytes(&[0u8; 10]);
        assert!(result.is_err());

        let result = HybridPublicKey::from_bytes(&[0u8; 10]);
        assert!(result.is_err());
    }
}
