//! Decentralized Identifier (DID) implementation
//!
//! Format: `did:sync:z{base58-blake3-hash}`
//!
//! The DID is derived from the hash of both public keys (Ed25519 + ML-DSA-65),
//! providing a stable identifier that doesn't reveal the full public key.

use crate::identity::HybridPublicKey;
use crate::SyncError;
use pqcrypto_traits::sign::PublicKey as PqPublicKey;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Decentralized Identifier (DID) for Synchronicity Engine
///
/// Format: `did:sync:z{base58-blake3-hash}`
///
/// The identifier is derived from the BLAKE3 hash of the concatenated
/// Ed25519 and ML-DSA-65 public keys, providing:
/// - Stable identity across key rotations (if key material is preserved)
/// - Privacy (doesn't reveal the full public key)
/// - Collision resistance via BLAKE3
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did(String);

impl Did {
    /// Create a DID from a hybrid public key
    ///
    /// The DID is computed as:
    /// 1. Concatenate Ed25519 public key (32 bytes) + ML-DSA-65 public key
    /// 2. Hash with BLAKE3 (32 bytes output)
    /// 3. Encode with base58
    /// 4. Prefix with "did:sync:z"
    pub fn from_public_key(public_key: &HybridPublicKey) -> Self {
        // Concatenate both public keys
        let mut key_material = Vec::new();
        key_material.extend_from_slice(public_key.ed25519().as_bytes());
        key_material.extend_from_slice(public_key.ml_dsa().as_bytes());

        // Hash with BLAKE3
        let hash = blake3::hash(&key_material);

        // Encode with base58
        let encoded = bs58::encode(hash.as_bytes()).into_string();

        // Format as DID
        Did(format!("did:sync:z{}", encoded))
    }

    /// Get the DID as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the identifier part (after "did:sync:z")
    pub fn identifier(&self) -> &str {
        // Skip "did:sync:z" prefix (10 characters)
        &self.0[10..]
    }

    /// Validate the format of a DID string without parsing
    fn validate_format(did_str: &str) -> Result<(), SyncError> {
        // Split by colons
        let parts: Vec<&str> = did_str.split(':').collect();

        if parts.len() != 3 {
            return Err(SyncError::Identity(
                "DID must have 3 parts separated by ':'".to_string(),
            ));
        }

        if parts[0] != "did" {
            return Err(SyncError::Identity(
                "DID must start with 'did:'".to_string(),
            ));
        }

        if parts[1] != "sync" {
            return Err(SyncError::Identity("DID method must be 'sync'".to_string()));
        }

        if !parts[2].starts_with('z') {
            return Err(SyncError::Identity(
                "DID identifier must start with 'z' (multibase prefix)".to_string(),
            ));
        }

        // Validate base58 encoding (excluding 'z' prefix)
        let identifier = &parts[2][1..];
        if identifier.is_empty() {
            return Err(SyncError::Identity(
                "DID identifier cannot be empty".to_string(),
            ));
        }

        bs58::decode(identifier).into_vec().map_err(|_| {
            SyncError::Identity("Invalid base58 encoding in DID identifier".to_string())
        })?;

        Ok(())
    }

    /// Parse a DID from a string
    pub fn parse(did_str: &str) -> Result<Self, SyncError> {
        Self::validate_format(did_str)?;
        Ok(Did(did_str.to_string()))
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Did {
    type Err = SyncError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl AsRef<str> for Did {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::HybridKeypair;

    #[test]
    fn test_did_format() {
        let keypair = HybridKeypair::generate();
        let public_key = keypair.public_key();
        let did = Did::from_public_key(&public_key);

        // Check format
        assert!(did.as_str().starts_with("did:sync:z"));

        // Check that identifier is valid base58
        let identifier = did.identifier();
        assert!(!identifier.is_empty());
        let decoded = bs58::decode(identifier).into_vec();
        assert!(decoded.is_ok());

        // BLAKE3 produces 32 bytes
        assert_eq!(decoded.unwrap().len(), 32);
    }

    #[test]
    fn test_did_deterministic() {
        let keypair = HybridKeypair::generate();
        let public_key = keypair.public_key();

        let did1 = Did::from_public_key(&public_key);
        let did2 = Did::from_public_key(&public_key);

        assert_eq!(did1, did2);
    }

    #[test]
    fn test_did_unique_per_key() {
        let keypair1 = HybridKeypair::generate();
        let keypair2 = HybridKeypair::generate();

        let did1 = Did::from_public_key(&keypair1.public_key());
        let did2 = Did::from_public_key(&keypair2.public_key());

        assert_ne!(did1, did2);
    }

    #[test]
    fn test_did_parse_valid() {
        let keypair = HybridKeypair::generate();
        let did = Did::from_public_key(&keypair.public_key());

        let parsed = Did::parse(did.as_str()).expect("Should parse valid DID");
        assert_eq!(did, parsed);
    }

    #[test]
    fn test_did_from_str() {
        let keypair = HybridKeypair::generate();
        let did = Did::from_public_key(&keypair.public_key());

        let parsed: Did = did.as_str().parse().expect("Should parse via FromStr");
        assert_eq!(did, parsed);
    }

    #[test]
    fn test_did_display() {
        let keypair = HybridKeypair::generate();
        let did = Did::from_public_key(&keypair.public_key());

        let display = format!("{}", did);
        assert_eq!(display, did.as_str());
    }

    #[test]
    fn test_did_parse_invalid_format() {
        // Missing parts
        assert!(Did::parse("did:sync").is_err());
        assert!(Did::parse("did").is_err());
        assert!(Did::parse("").is_err());

        // Wrong scheme
        assert!(Did::parse("uri:sync:z123").is_err());

        // Wrong method
        assert!(Did::parse("did:key:z123").is_err());

        // Missing 'z' prefix
        assert!(Did::parse("did:sync:123").is_err());

        // Empty identifier
        assert!(Did::parse("did:sync:z").is_err());

        // Invalid base58
        assert!(Did::parse("did:sync:z0OIl").is_err()); // 0, O, I, l are not valid base58
    }

    #[test]
    fn test_did_serde_roundtrip() {
        let keypair = HybridKeypair::generate();
        let did = Did::from_public_key(&keypair.public_key());

        let json = serde_json::to_string(&did).expect("Should serialize");
        let recovered: Did = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(did, recovered);
    }

    #[test]
    fn test_did_hash() {
        use std::collections::HashSet;

        let keypair1 = HybridKeypair::generate();
        let keypair2 = HybridKeypair::generate();

        let did1 = Did::from_public_key(&keypair1.public_key());
        let did2 = Did::from_public_key(&keypair2.public_key());

        let mut set = HashSet::new();
        set.insert(did1.clone());
        set.insert(did2.clone());
        set.insert(did1.clone()); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_did_identifier() {
        let keypair = HybridKeypair::generate();
        let did = Did::from_public_key(&keypair.public_key());

        let identifier = did.identifier();
        let full = did.as_str();

        assert_eq!(full, format!("did:sync:z{}", identifier));
    }

    #[test]
    fn test_did_as_ref() {
        let keypair = HybridKeypair::generate();
        let did = Did::from_public_key(&keypair.public_key());

        let as_ref: &str = did.as_ref();
        assert_eq!(as_ref, did.as_str());
    }
}
