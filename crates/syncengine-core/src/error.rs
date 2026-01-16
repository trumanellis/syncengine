//! Error types for Synchronicity Engine

use thiserror::Error;

/// Main error type for Synchronicity Engine operations
#[derive(Error, Debug)]
pub enum SyncError {
    /// Realm was not found in storage
    #[error("Realm not found: {0}")]
    RealmNotFound(String),

    /// Task was not found in the specified realm
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    /// Error during gossip protocol operations
    #[error("Gossip error: {0}")]
    Gossip(String),

    /// Error during storage operations (redb)
    #[error("Storage error: {0}")]
    Storage(String),

    /// Database creation/opening error
    #[error("Database error: {0}")]
    Database(#[from] redb::DatabaseError),

    /// Transaction error
    #[error("Transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),

    /// Table error
    #[error("Table error: {0}")]
    Table(#[from] redb::TableError),

    /// Storage operation error
    #[error("Storage operation error: {0}")]
    StorageOp(#[from] redb::StorageError),

    /// Commit error
    #[error("Commit error: {0}")]
    Commit(#[from] redb::CommitError),

    /// Error during serialization/deserialization
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Cryptographic operation failed
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Network-related error
    #[error("Network error: {0}")]
    Network(String),

    /// General I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Automerge document error
    #[error("Automerge error: {0}")]
    Automerge(String),

    /// Invalid invite format or data
    #[error("Invalid invite: {0}")]
    InvalidInvite(String),

    /// Peer connection failed
    #[error("Peer connection error: {0}")]
    PeerConnection(String),

    /// Signature verification failed
    #[error("Signature invalid: {0}")]
    SignatureInvalid(String),

    /// Decryption failed (wrong key, tampered data, or malformed input)
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    /// Envelope protocol version not supported
    #[error("Envelope version {0} is not supported")]
    EnvelopeVersionUnsupported(u8),

    /// Identity-related error (keys, signatures, DIDs)
    #[error("Identity error: {0}")]
    Identity(String),

    /// Invalid DID format
    #[error("Invalid DID format: {0}")]
    InvalidDidFormat(String),

    /// Operation not allowed on Private realm
    #[error("Operation not allowed on Private realm: {0}")]
    PrivateRealmOperation(String),

    /// Contact not found in storage
    #[error("Contact not found: {0}")]
    ContactNotFound(String),

    /// Invalid operation for current state
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Result type alias using SyncError
pub type SyncResult<T> = Result<T, SyncError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = SyncError::RealmNotFound("test-realm".to_string());
        assert_eq!(format!("{}", err), "Realm not found: test-realm");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let sync_err: SyncError = io_err.into();
        assert!(matches!(sync_err, SyncError::Io(_)));
    }
}
