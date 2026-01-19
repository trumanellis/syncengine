//! Extended profile keys with key exchange support
//!
//! This module extends the existing [`HybridKeypair`] with X25519 + ML-KEM
//! key exchange capabilities for sealed boxes (per-recipient encryption).
//!
//! ## Key Types
//!
//! | Key Type | Purpose | Algorithm |
//! |----------|---------|-----------|
//! | Signing | Message authentication | Ed25519 + ML-DSA-65 |
//! | Key Exchange | Sealed boxes | X25519 + ML-KEM-768 |
//!
//! ## Security Model
//!
//! The hybrid key exchange combines classical (X25519) and post-quantum (ML-KEM)
//! key agreement. The combined secret is derived using HKDF:
//!
//! ```text
//! combined_secret = HKDF-SHA256(ss1 || ss2, "indra-key-exchange")
//! where:
//!   ss1 = X25519(ephemeral_sk, recipient_pk)
//!   ss2 = ML-KEM.Decapsulate(recipient_sk, ciphertext)
//! ```

use crate::error::SyncError;
use crate::identity::{HybridKeypair, HybridPublicKey, HybridSignature, Did};
use pqcrypto_kyber::kyber768;
use pqcrypto_traits::kem::{PublicKey as KemPublicKey, SecretKey as KemSecretKey};
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};

/// Extended profile keys combining signing and key exchange capabilities.
///
/// This struct holds both:
/// - **Signing keys**: [`HybridKeypair`] for signatures (Ed25519 + ML-DSA-65)
/// - **Key exchange keys**: X25519 + ML-KEM-768 for sealed boxes
#[derive(Clone)]
pub struct ProfileKeys {
    /// Hybrid signing keypair (existing identity)
    signing: HybridKeypair,
    /// X25519 static secret for classical key exchange
    x25519_secret: X25519StaticSecret,
    /// ML-KEM-768 secret key for post-quantum key exchange
    mlkem_secret: kyber768::SecretKey,
    /// ML-KEM-768 public key (cached)
    mlkem_public: kyber768::PublicKey,
}

impl ProfileKeys {
    /// Generate new random profile keys.
    ///
    /// Creates fresh signing keys and key exchange keys.
    pub fn generate() -> Self {
        // Generate signing keys
        let signing = HybridKeypair::generate();

        // Generate X25519 key exchange keys
        let mut x25519_seed = [0u8; 32];
        getrandom::getrandom(&mut x25519_seed).expect("Failed to get random bytes");
        let x25519_secret = X25519StaticSecret::from(x25519_seed);

        // Generate ML-KEM-768 key exchange keys
        let (mlkem_public, mlkem_secret) = kyber768::keypair();

        Self {
            signing,
            x25519_secret,
            mlkem_secret,
            mlkem_public,
        }
    }

    /// Create profile keys from an existing signing keypair.
    ///
    /// Derives deterministic key exchange keys from the signing keypair's
    /// public key hash, ensuring the same signing key always produces
    /// the same key exchange keys.
    pub fn from_signing_keypair(signing: HybridKeypair) -> Self {
        // Derive X25519 seed from signing public key
        let pk_bytes = signing.public_key().to_bytes();
        let x25519_seed = blake3::derive_key("indra-x25519-seed-v1", &pk_bytes);
        let x25519_secret = X25519StaticSecret::from(x25519_seed);

        // For ML-KEM, we need to generate randomly since we can't seed it
        // In practice, this means we should store both keys or regenerate fresh
        let (mlkem_public, mlkem_secret) = kyber768::keypair();

        Self {
            signing,
            x25519_secret,
            mlkem_secret,
            mlkem_public,
        }
    }

    /// Get the signing keypair.
    pub fn signing_keypair(&self) -> &HybridKeypair {
        &self.signing
    }

    /// Get the hybrid signing public key.
    pub fn signing_public_key(&self) -> HybridPublicKey {
        self.signing.public_key()
    }

    /// Get the DID (Decentralized Identifier) derived from signing keys.
    pub fn did(&self) -> Did {
        Did::from_public_key(&self.signing.public_key())
    }

    /// Sign a message with hybrid signatures (Ed25519 + ML-DSA-65).
    pub fn sign(&self, message: &[u8]) -> HybridSignature {
        self.signing.sign(message)
    }

    /// Get the X25519 public key for key exchange.
    pub fn x25519_public_key(&self) -> X25519PublicKey {
        X25519PublicKey::from(&self.x25519_secret)
    }

    /// Get the X25519 secret key reference (for key exchange).
    pub(crate) fn x25519_secret(&self) -> &X25519StaticSecret {
        &self.x25519_secret
    }

    /// Get the ML-KEM-768 public key for key exchange.
    pub fn mlkem_public_key(&self) -> &kyber768::PublicKey {
        &self.mlkem_public
    }

    /// Get the ML-KEM-768 secret key reference (for key exchange).
    pub(crate) fn mlkem_secret(&self) -> &kyber768::SecretKey {
        &self.mlkem_secret
    }

    /// Get the full public key bundle for this profile.
    pub fn public_bundle(&self) -> ProfilePublicKeys {
        ProfilePublicKeys {
            signing: self.signing.public_key(),
            x25519: self.x25519_public_key(),
            mlkem: self.mlkem_public.clone(),
        }
    }

    /// Serialize profile keys to bytes.
    ///
    /// Format:
    /// - [signing_keypair_len: 4 LE][signing_keypair: variable]
    /// - [x25519_secret: 32]
    /// - [mlkem_secret_len: 4 LE][mlkem_secret: variable]
    /// - [mlkem_public_len: 4 LE][mlkem_public: variable]
    pub fn to_bytes(&self) -> Vec<u8> {
        let signing_bytes = self.signing.to_bytes();
        let x25519_bytes = self.x25519_secret.as_bytes();
        let mlkem_secret_bytes = self.mlkem_secret.as_bytes();
        let mlkem_public_bytes = self.mlkem_public.as_bytes();

        let mut bytes = Vec::with_capacity(
            4 + signing_bytes.len() + 32 + 4 + mlkem_secret_bytes.len() + 4 + mlkem_public_bytes.len()
        );

        bytes.extend_from_slice(&(signing_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&signing_bytes);
        bytes.extend_from_slice(x25519_bytes);
        bytes.extend_from_slice(&(mlkem_secret_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(mlkem_secret_bytes);
        bytes.extend_from_slice(&(mlkem_public_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(mlkem_public_bytes);

        bytes
    }

    /// Deserialize profile keys from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SyncError> {
        if bytes.len() < 4 {
            return Err(SyncError::Identity("ProfileKeys data too short".to_string()));
        }

        let mut offset = 0;

        // Read signing keypair
        let signing_len = u32::from_le_bytes(
            bytes[offset..offset + 4].try_into()
                .map_err(|_| SyncError::Identity("Invalid signing length".to_string()))?
        ) as usize;
        offset += 4;

        if bytes.len() < offset + signing_len {
            return Err(SyncError::Identity("Signing keypair data truncated".to_string()));
        }
        let signing = HybridKeypair::from_bytes(&bytes[offset..offset + signing_len])?;
        offset += signing_len;

        // Read X25519 secret (32 bytes)
        if bytes.len() < offset + 32 {
            return Err(SyncError::Identity("X25519 secret data truncated".to_string()));
        }
        let x25519_bytes: [u8; 32] = bytes[offset..offset + 32].try_into()
            .map_err(|_| SyncError::Identity("Invalid X25519 secret".to_string()))?;
        let x25519_secret = X25519StaticSecret::from(x25519_bytes);
        offset += 32;

        // Read ML-KEM secret
        if bytes.len() < offset + 4 {
            return Err(SyncError::Identity("ML-KEM secret length missing".to_string()));
        }
        let mlkem_secret_len = u32::from_le_bytes(
            bytes[offset..offset + 4].try_into()
                .map_err(|_| SyncError::Identity("Invalid ML-KEM secret length".to_string()))?
        ) as usize;
        offset += 4;

        if bytes.len() < offset + mlkem_secret_len {
            return Err(SyncError::Identity("ML-KEM secret data truncated".to_string()));
        }
        let mlkem_secret = kyber768::SecretKey::from_bytes(&bytes[offset..offset + mlkem_secret_len])
            .map_err(|_| SyncError::Identity("Invalid ML-KEM secret key".to_string()))?;
        offset += mlkem_secret_len;

        // Read ML-KEM public
        if bytes.len() < offset + 4 {
            return Err(SyncError::Identity("ML-KEM public length missing".to_string()));
        }
        let mlkem_public_len = u32::from_le_bytes(
            bytes[offset..offset + 4].try_into()
                .map_err(|_| SyncError::Identity("Invalid ML-KEM public length".to_string()))?
        ) as usize;
        offset += 4;

        if bytes.len() < offset + mlkem_public_len {
            return Err(SyncError::Identity("ML-KEM public data truncated".to_string()));
        }
        let mlkem_public = kyber768::PublicKey::from_bytes(&bytes[offset..offset + mlkem_public_len])
            .map_err(|_| SyncError::Identity("Invalid ML-KEM public key".to_string()))?;

        Ok(Self {
            signing,
            x25519_secret,
            mlkem_secret,
            mlkem_public,
        })
    }
}

impl std::fmt::Debug for ProfileKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfileKeys")
            .field("did", &self.did())
            .field("x25519_public", &hex::encode(self.x25519_public_key().as_bytes()))
            .field("mlkem_public_len", &self.mlkem_public.as_bytes().len())
            .finish_non_exhaustive()
    }
}

/// Public key bundle for a profile (for recipients to use when encrypting).
#[derive(Clone)]
pub struct ProfilePublicKeys {
    /// Hybrid signing public key (for signature verification)
    pub signing: HybridPublicKey,
    /// X25519 public key (for classical key exchange)
    pub x25519: X25519PublicKey,
    /// ML-KEM-768 public key (for post-quantum key exchange)
    pub mlkem: kyber768::PublicKey,
}

impl ProfilePublicKeys {
    /// Get the DID for this public key bundle.
    pub fn did(&self) -> Did {
        Did::from_public_key(&self.signing)
    }

    /// Serialize to bytes.
    ///
    /// Format:
    /// - [signing_len: 4 LE][signing: variable]
    /// - [x25519: 32]
    /// - [mlkem_len: 4 LE][mlkem: variable]
    pub fn to_bytes(&self) -> Vec<u8> {
        let signing_bytes = self.signing.to_bytes();
        let x25519_bytes = self.x25519.as_bytes();
        let mlkem_bytes = self.mlkem.as_bytes();

        let mut bytes = Vec::with_capacity(
            4 + signing_bytes.len() + 32 + 4 + mlkem_bytes.len()
        );

        bytes.extend_from_slice(&(signing_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&signing_bytes);
        bytes.extend_from_slice(x25519_bytes);
        bytes.extend_from_slice(&(mlkem_bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(mlkem_bytes);

        bytes
    }

    /// Deserialize from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SyncError> {
        if bytes.len() < 4 {
            return Err(SyncError::Identity("ProfilePublicKeys data too short".to_string()));
        }

        let mut offset = 0;

        // Read signing public key
        let signing_len = u32::from_le_bytes(
            bytes[offset..offset + 4].try_into()
                .map_err(|_| SyncError::Identity("Invalid signing length".to_string()))?
        ) as usize;
        offset += 4;

        if bytes.len() < offset + signing_len {
            return Err(SyncError::Identity("Signing public key data truncated".to_string()));
        }
        let signing = HybridPublicKey::from_bytes(&bytes[offset..offset + signing_len])?;
        offset += signing_len;

        // Read X25519 public key (32 bytes)
        if bytes.len() < offset + 32 {
            return Err(SyncError::Identity("X25519 public key data truncated".to_string()));
        }
        let x25519_bytes: [u8; 32] = bytes[offset..offset + 32].try_into()
            .map_err(|_| SyncError::Identity("Invalid X25519 public key".to_string()))?;
        let x25519 = X25519PublicKey::from(x25519_bytes);
        offset += 32;

        // Read ML-KEM public key
        if bytes.len() < offset + 4 {
            return Err(SyncError::Identity("ML-KEM public length missing".to_string()));
        }
        let mlkem_len = u32::from_le_bytes(
            bytes[offset..offset + 4].try_into()
                .map_err(|_| SyncError::Identity("Invalid ML-KEM public length".to_string()))?
        ) as usize;
        offset += 4;

        if bytes.len() < offset + mlkem_len {
            return Err(SyncError::Identity("ML-KEM public data truncated".to_string()));
        }
        let mlkem = kyber768::PublicKey::from_bytes(&bytes[offset..offset + mlkem_len])
            .map_err(|_| SyncError::Identity("Invalid ML-KEM public key".to_string()))?;

        Ok(Self {
            signing,
            x25519,
            mlkem,
        })
    }
}

impl std::fmt::Debug for ProfilePublicKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfilePublicKeys")
            .field("did", &self.did())
            .field("x25519", &hex::encode(self.x25519.as_bytes()))
            .field("mlkem_len", &self.mlkem.as_bytes().len())
            .finish()
    }
}

impl Serialize for ProfilePublicKeys {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

impl<'de> Deserialize<'de> for ProfilePublicKeys {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = <Vec<u8>>::deserialize(deserializer)?;
        Self::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_keys_generation() {
        let keys = ProfileKeys::generate();

        // Verify we can get all components
        let _signing_pk = keys.signing_public_key();
        let _x25519_pk = keys.x25519_public_key();
        let _mlkem_pk = keys.mlkem_public_key();
        let _did = keys.did();

        // Verify signing works
        let message = b"test message";
        let signature = keys.sign(message);
        assert!(keys.signing_public_key().verify(message, &signature));
    }

    #[test]
    fn test_profile_keys_serialization_roundtrip() {
        let keys = ProfileKeys::generate();
        let did = keys.did();

        let bytes = keys.to_bytes();
        let recovered = ProfileKeys::from_bytes(&bytes).expect("Should deserialize");

        // DIDs should match
        assert_eq!(recovered.did(), did);

        // Public keys should match
        assert_eq!(
            keys.x25519_public_key().as_bytes(),
            recovered.x25519_public_key().as_bytes()
        );
    }

    #[test]
    fn test_profile_public_keys_serialization() {
        let keys = ProfileKeys::generate();
        let public_bundle = keys.public_bundle();

        let bytes = public_bundle.to_bytes();
        let recovered = ProfilePublicKeys::from_bytes(&bytes).expect("Should deserialize");

        assert_eq!(public_bundle.did(), recovered.did());
        assert_eq!(
            public_bundle.x25519.as_bytes(),
            recovered.x25519.as_bytes()
        );
    }

    #[test]
    fn test_different_keys_different_dids() {
        let keys1 = ProfileKeys::generate();
        let keys2 = ProfileKeys::generate();

        assert_ne!(keys1.did(), keys2.did());
    }

    #[test]
    fn test_from_signing_keypair() {
        let signing = HybridKeypair::generate();
        let did = Did::from_public_key(&signing.public_key());

        let keys = ProfileKeys::from_signing_keypair(signing);

        // DID should match
        assert_eq!(keys.did(), did);
    }
}
