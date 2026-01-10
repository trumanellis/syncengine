//! Identity module for Synchronicity Engine
//!
//! This module provides quantum-secure identity using hybrid signatures
//! combining Ed25519 (classical) and ML-DSA-65/Dilithium5 (post-quantum).
//!
//! ## Overview
//!
//! The identity system provides:
//! - **Hybrid Keypairs**: Ed25519 + ML-DSA-65 for quantum-resistant signatures
//! - **DIDs**: Decentralized identifiers in the format `did:sync:z{base58}`
//! - **Signatures**: Both classical and post-quantum signatures must verify
//!
//! ## Example
//!
//! ```rust
//! use syncengine_core::identity::{HybridKeypair, Did};
//!
//! // Generate a new identity
//! let keypair = HybridKeypair::generate();
//! let public_key = keypair.public_key();
//! let did = Did::from_public_key(&public_key);
//!
//! println!("Identity: {}", did);
//!
//! // Sign a message
//! let message = b"Hello, quantum-secure world!";
//! let signature = keypair.sign(message);
//!
//! // Verify the signature
//! assert!(public_key.verify(message, &signature));
//! ```
//!
//! ## Security Model
//!
//! The hybrid signature scheme provides:
//! - **Classical security**: Ed25519 is secure against classical computers
//! - **Post-quantum security**: ML-DSA-65 (Dilithium5) is secure against quantum computers
//! - **Defense in depth**: Both signatures must verify, so an attacker must break BOTH
//!
//! This means:
//! - If quantum computers break Ed25519, ML-DSA-65 still protects you
//! - If ML-DSA-65 is found to have a flaw, Ed25519 still protects you
//! - Signatures are larger but provide future-proof security

mod did;
mod keypair;
mod signature;

// Re-export public types
pub use did::Did;
pub use keypair::{HybridKeypair, HybridPublicKey};
pub use signature::HybridSignature;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_identity_workflow() {
        // Generate identity
        let keypair = HybridKeypair::generate();
        let public_key = keypair.public_key();
        let did = Did::from_public_key(&public_key);

        // Verify DID format
        assert!(did.as_str().starts_with("did:sync:z"));

        // Sign and verify
        let message = b"Integration test message";
        let signature = keypair.sign(message);
        assert!(public_key.verify(message, &signature));

        // Serialize and deserialize
        let pk_bytes = public_key.to_bytes();
        let recovered_pk = HybridPublicKey::from_bytes(&pk_bytes).unwrap();
        assert!(recovered_pk.verify(message, &signature));

        // DID should be the same for recovered public key
        let recovered_did = Did::from_public_key(&recovered_pk);
        assert_eq!(did, recovered_did);
    }

    #[test]
    fn test_keypair_persistence() {
        let keypair = HybridKeypair::generate();
        let message = b"Persistence test";
        let original_signature = keypair.sign(message);

        // Serialize keypair
        let bytes = keypair.to_bytes();

        // Deserialize and verify
        let recovered = HybridKeypair::from_bytes(&bytes).unwrap();
        let new_signature = recovered.sign(message);

        // Both signatures should verify with either public key
        assert!(keypair.public_key().verify(message, &new_signature));
        assert!(recovered.public_key().verify(message, &original_signature));
    }

    #[test]
    fn test_cross_verification() {
        // Create two identities
        let alice = HybridKeypair::generate();
        let bob = HybridKeypair::generate();

        let message = b"Message from Alice to Bob";

        // Alice signs
        let alice_signature = alice.sign(message);

        // Bob should be able to verify Alice's signature using Alice's public key
        assert!(alice.public_key().verify(message, &alice_signature));

        // Bob's public key should NOT verify Alice's signature
        assert!(!bob.public_key().verify(message, &alice_signature));
    }
}
