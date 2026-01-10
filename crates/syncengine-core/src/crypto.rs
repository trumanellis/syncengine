//! Encryption layer using ChaCha20-Poly1305 AEAD
//!
//! Provides symmetric encryption for realm data using the ChaCha20-Poly1305
//! authenticated encryption cipher. Each realm has its own encryption key.

use crate::error::SyncError;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;

/// Nonce size for ChaCha20-Poly1305 (12 bytes)
pub const NONCE_SIZE: usize = 12;

/// Encryption utilities for realm data using ChaCha20-Poly1305 AEAD.
///
/// This struct holds a cipher instance initialized with a symmetric key.
/// All encrypted data includes a random nonce prepended to the ciphertext.
///
/// # Wire Format
///
/// Encrypted data format: `[nonce (12 bytes)] + [ciphertext + auth_tag (16 bytes)]`
///
/// # Example
///
/// ```
/// use syncengine_core::crypto::RealmCrypto;
///
/// let key = RealmCrypto::generate_key();
/// let crypto = RealmCrypto::new(&key);
///
/// let plaintext = b"Hello, World!";
/// let ciphertext = crypto.encrypt(plaintext).unwrap();
/// let decrypted = crypto.decrypt(&ciphertext).unwrap();
///
/// assert_eq!(plaintext.as_slice(), decrypted.as_slice());
/// ```
pub struct RealmCrypto {
    cipher: ChaCha20Poly1305,
}

impl RealmCrypto {
    /// Create a new RealmCrypto instance with the given 32-byte key.
    ///
    /// # Arguments
    ///
    /// * `key` - A 32-byte symmetric key
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            cipher: ChaCha20Poly1305::new(key.into()),
        }
    }

    /// Generate a new random 32-byte encryption key.
    ///
    /// Uses the system's cryptographically secure random number generator.
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        rand::rng().fill_bytes(&mut key);
        key
    }

    /// Encrypt data using ChaCha20-Poly1305 AEAD.
    ///
    /// The output format is: `[nonce (12 bytes)] + [ciphertext + tag]`
    ///
    /// A random nonce is generated for each encryption operation to ensure
    /// that the same plaintext produces different ciphertext each time.
    ///
    /// # Arguments
    ///
    /// * `plaintext` - The data to encrypt
    ///
    /// # Returns
    ///
    /// The encrypted data with prepended nonce, or an error if encryption fails.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, SyncError> {
        // Generate random nonce (12 bytes for ChaCha20-Poly1305)
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| SyncError::Crypto(format!("Encryption failed: {}", e)))?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data using ChaCha20-Poly1305 AEAD.
    ///
    /// Expects format: `[nonce (12 bytes)] + [ciphertext + tag]`
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - The encrypted data with prepended nonce
    ///
    /// # Returns
    ///
    /// The decrypted plaintext, or an error if decryption fails
    /// (e.g., wrong key, tampered data, or malformed input).
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SyncError> {
        if ciphertext.len() < NONCE_SIZE {
            return Err(SyncError::Crypto(
                "Data too short to contain nonce".to_string(),
            ));
        }

        // Extract nonce
        let nonce = Nonce::from_slice(&ciphertext[..NONCE_SIZE]);

        // Extract ciphertext
        let encrypted = &ciphertext[NONCE_SIZE..];

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, encrypted)
            .map_err(|e| SyncError::Crypto(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }

    /// Encrypt data using a provided nonce.
    ///
    /// Unlike [`encrypt`], this method does NOT prepend the nonce to the output.
    /// The caller is responsible for storing the nonce separately.
    ///
    /// # Arguments
    ///
    /// * `plaintext` - The data to encrypt
    /// * `nonce` - A 12-byte nonce (must be unique per key)
    ///
    /// # Returns
    ///
    /// The ciphertext with authentication tag (no nonce prepended).
    pub fn encrypt_with_nonce(
        &self,
        plaintext: &[u8],
        nonce: &[u8; NONCE_SIZE],
    ) -> Result<Vec<u8>, SyncError> {
        let nonce = Nonce::from_slice(nonce);
        self.cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| SyncError::Crypto(format!("Encryption failed: {}", e)))
    }

    /// Decrypt data using a provided nonce.
    ///
    /// Unlike [`decrypt`], this method expects the nonce to be provided separately,
    /// not prepended to the ciphertext.
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - The encrypted data with authentication tag (no nonce)
    /// * `nonce` - The 12-byte nonce used for encryption
    ///
    /// # Returns
    ///
    /// The decrypted plaintext, or an error if decryption fails.
    pub fn decrypt_with_nonce(
        &self,
        ciphertext: &[u8],
        nonce: &[u8; NONCE_SIZE],
    ) -> Result<Vec<u8>, SyncError> {
        let nonce = Nonce::from_slice(nonce);
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| SyncError::DecryptionFailed(format!("{}", e)))
    }

    /// Generate a random 12-byte nonce.
    ///
    /// Uses the system's cryptographically secure random number generator.
    pub fn generate_nonce() -> [u8; NONCE_SIZE] {
        let mut nonce = [0u8; NONCE_SIZE];
        rand::rng().fill_bytes(&mut nonce);
        nonce
    }
}

/// Convenience functions for stateless encryption/decryption.
///
/// These functions create a temporary cipher instance for each operation.
/// For repeated operations with the same key, prefer using [`RealmCrypto`].
pub mod stateless {
    use super::*;

    /// Encrypt data using ChaCha20-Poly1305 AEAD.
    ///
    /// Format: `[nonce (12 bytes)] + [ciphertext + tag]`
    pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, SyncError> {
        RealmCrypto::new(key).encrypt(plaintext)
    }

    /// Decrypt data using ChaCha20-Poly1305 AEAD.
    ///
    /// Expects format: `[nonce (12 bytes)] + [ciphertext + tag]`
    pub fn decrypt(key: &[u8; 32], ciphertext: &[u8]) -> Result<Vec<u8>, SyncError> {
        RealmCrypto::new(key).decrypt(ciphertext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key1 = RealmCrypto::generate_key();
        let key2 = RealmCrypto::generate_key();

        // Keys should be different (random)
        assert_ne!(key1, key2);

        // Keys should be 32 bytes
        assert_eq!(key1.len(), 32);
        assert_eq!(key2.len(), 32);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = RealmCrypto::generate_key();
        let crypto = RealmCrypto::new(&key);

        let plaintext = b"Hello, World!";
        let ciphertext = crypto.encrypt(plaintext).unwrap();
        let decrypted = crypto.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let key = RealmCrypto::generate_key();
        let crypto = RealmCrypto::new(&key);

        let plaintext = b"";
        let ciphertext = crypto.encrypt(plaintext).unwrap();
        let decrypted = crypto.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_large_data() {
        let key = RealmCrypto::generate_key();
        let crypto = RealmCrypto::new(&key);

        // 1MB of data
        let plaintext: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
        let ciphertext = crypto.encrypt(&plaintext).unwrap();
        let decrypted = crypto.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_ciphertext_includes_nonce() {
        let key = RealmCrypto::generate_key();
        let crypto = RealmCrypto::new(&key);

        let plaintext = b"Test";
        let ciphertext = crypto.encrypt(plaintext).unwrap();

        // Ciphertext should be longer than plaintext by at least nonce + tag
        // Nonce = 12 bytes, Tag = 16 bytes
        assert!(ciphertext.len() >= plaintext.len() + NONCE_SIZE + 16);
    }

    #[test]
    fn test_same_plaintext_different_ciphertext() {
        let key = RealmCrypto::generate_key();
        let crypto = RealmCrypto::new(&key);

        let plaintext = b"Deterministic test";
        let ciphertext1 = crypto.encrypt(plaintext).unwrap();
        let ciphertext2 = crypto.encrypt(plaintext).unwrap();

        // Different nonces should produce different ciphertext
        assert_ne!(ciphertext1, ciphertext2);

        // But both should decrypt to the same plaintext
        assert_eq!(crypto.decrypt(&ciphertext1).unwrap(), plaintext.as_slice());
        assert_eq!(crypto.decrypt(&ciphertext2).unwrap(), plaintext.as_slice());
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = RealmCrypto::generate_key();
        let key2 = RealmCrypto::generate_key();

        let crypto1 = RealmCrypto::new(&key1);
        let crypto2 = RealmCrypto::new(&key2);

        let plaintext = b"Secret";
        let ciphertext = crypto1.encrypt(plaintext).unwrap();

        let result = crypto2.decrypt(&ciphertext);
        assert!(result.is_err());

        if let Err(SyncError::Crypto(msg)) = result {
            assert!(msg.contains("Decryption failed"));
        } else {
            panic!("Expected Crypto error");
        }
    }

    #[test]
    fn test_tampered_data_fails() {
        let key = RealmCrypto::generate_key();
        let crypto = RealmCrypto::new(&key);

        let plaintext = b"Original message";
        let mut ciphertext = crypto.encrypt(plaintext).unwrap();

        // Tamper with the ciphertext (not the nonce)
        if ciphertext.len() > NONCE_SIZE {
            ciphertext[NONCE_SIZE] ^= 0xFF;
        }

        let result = crypto.decrypt(&ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_nonce_fails() {
        let key = RealmCrypto::generate_key();
        let crypto = RealmCrypto::new(&key);

        let plaintext = b"Original message";
        let mut ciphertext = crypto.encrypt(plaintext).unwrap();

        // Tamper with the nonce
        ciphertext[0] ^= 0xFF;

        let result = crypto.decrypt(&ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_truncated_data_fails() {
        let key = RealmCrypto::generate_key();
        let crypto = RealmCrypto::new(&key);

        let plaintext = b"Original message";
        let ciphertext = crypto.encrypt(plaintext).unwrap();

        // Truncate to less than nonce size
        let truncated = &ciphertext[..5];
        let result = crypto.decrypt(truncated);
        assert!(result.is_err());

        if let Err(SyncError::Crypto(msg)) = result {
            assert!(msg.contains("too short"));
        } else {
            panic!("Expected Crypto error about data being too short");
        }
    }

    #[test]
    fn test_stateless_encrypt_decrypt() {
        let key = RealmCrypto::generate_key();

        let plaintext = b"Stateless test";
        let ciphertext = stateless::encrypt(&key, plaintext).unwrap();
        let decrypted = stateless::decrypt(&key, &ciphertext).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_stateless_wrong_key_fails() {
        let key1 = RealmCrypto::generate_key();
        let key2 = RealmCrypto::generate_key();

        let plaintext = b"Secret";
        let ciphertext = stateless::encrypt(&key1, plaintext).unwrap();

        let result = stateless::decrypt(&key2, &ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_decrypt_with_nonce_roundtrip() {
        let key = RealmCrypto::generate_key();
        let crypto = RealmCrypto::new(&key);
        let nonce = RealmCrypto::generate_nonce();

        let plaintext = b"Test with explicit nonce";
        let ciphertext = crypto.encrypt_with_nonce(plaintext, &nonce).unwrap();
        let decrypted = crypto.decrypt_with_nonce(&ciphertext, &nonce).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_with_nonce_no_prepended_nonce() {
        let key = RealmCrypto::generate_key();
        let crypto = RealmCrypto::new(&key);
        let nonce = RealmCrypto::generate_nonce();

        let plaintext = b"Test";
        let ciphertext = crypto.encrypt_with_nonce(plaintext, &nonce).unwrap();

        // Ciphertext should NOT have nonce prepended
        // It should be: plaintext length + 16 bytes (auth tag)
        assert_eq!(ciphertext.len(), plaintext.len() + 16);
    }

    #[test]
    fn test_decrypt_with_nonce_wrong_nonce_fails() {
        let key = RealmCrypto::generate_key();
        let crypto = RealmCrypto::new(&key);
        let nonce1 = RealmCrypto::generate_nonce();
        let nonce2 = RealmCrypto::generate_nonce();

        let plaintext = b"Test with nonce";
        let ciphertext = crypto.encrypt_with_nonce(plaintext, &nonce1).unwrap();

        // Try to decrypt with wrong nonce
        let result = crypto.decrypt_with_nonce(&ciphertext, &nonce2);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_nonce_produces_different_values() {
        let nonce1 = RealmCrypto::generate_nonce();
        let nonce2 = RealmCrypto::generate_nonce();

        // Nonces should be different (random)
        assert_ne!(nonce1, nonce2);

        // Nonces should be 12 bytes
        assert_eq!(nonce1.len(), 12);
        assert_eq!(nonce2.len(), 12);
    }

    #[test]
    fn test_same_nonce_same_ciphertext() {
        let key = RealmCrypto::generate_key();
        let crypto = RealmCrypto::new(&key);
        let nonce = [0x42u8; NONCE_SIZE]; // Fixed nonce for deterministic test

        let plaintext = b"Deterministic test";
        let ciphertext1 = crypto.encrypt_with_nonce(plaintext, &nonce).unwrap();
        let ciphertext2 = crypto.encrypt_with_nonce(plaintext, &nonce).unwrap();

        // Same key + nonce + plaintext = same ciphertext
        assert_eq!(ciphertext1, ciphertext2);
    }
}
