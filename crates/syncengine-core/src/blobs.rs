//! Blob Manager - iroh-blobs integration for all image storage
//!
//! Provides content-addressed storage for avatar images, quest images, and other binary data.
//! Blobs are identified by BLAKE3 hash, enabling deduplication and integrity verification.
//!
//! # Architecture
//!
//! - Uses iroh-blobs for content-addressed storage
//! - Memory store for testing, FsStore for production
//! - Integrates with iroh Router for P2P blob transfer
//! - BlobTickets enable sharing download instructions
//!
//! # Storage Modes
//!
//! - **Memory**: In-memory storage, lost on restart. Use for tests.
//! - **Persistent**: FsStore-based, persisted to `data_dir/blobs/`. Use for production.
//!
//! # Example
//!
//! ```ignore
//! use syncengine_core::blobs::BlobManager;
//!
//! // Production: persistent storage
//! let manager = BlobManager::new_persistent(&data_dir.join("blobs")).await?;
//!
//! // Import an avatar image (with size validation)
//! let hash = manager.import_avatar(avatar_bytes).await?;
//!
//! // Import a quest image (larger limit)
//! let hash = manager.import_image(quest_image_bytes).await?;
//!
//! // Create a ticket for P2P sharing
//! let ticket = manager.create_ticket(hash, &endpoint);
//!
//! // Another node can download using the ticket
//! let bytes = other_manager.download_blob(&ticket, &endpoint).await?;
//! ```

use std::path::Path;

use bytes::Bytes;
use iroh::Endpoint;
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::store::mem::MemStore;
use iroh_blobs::ticket::BlobTicket;
use iroh_blobs::{BlobFormat, Hash};
use tracing::{debug, info};

use crate::error::SyncError;

/// Result type for blob operations
pub type BlobResult<T> = Result<T, SyncError>;

/// Maximum avatar size: 256 KB
/// Avatars should be small, optimized images (WebP preferred)
pub const MAX_AVATAR_SIZE: usize = 256 * 1024;

/// Maximum general image size: 2 MB
/// Quest images and other content can be larger
pub const MAX_IMAGE_SIZE: usize = 2 * 1024 * 1024;

/// The underlying store type (memory or persistent)
enum StoreInner {
    Memory(MemStore),
    Persistent(FsStore),
}

impl std::fmt::Debug for StoreInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreInner::Memory(_) => write!(f, "MemStore"),
            StoreInner::Persistent(_) => write!(f, "FsStore"),
        }
    }
}

/// Manager for content-addressed blob storage.
///
/// Wraps iroh-blobs store with convenience methods for avatar
/// and other binary data management. Supports both in-memory
/// (for testing) and persistent (for production) storage.
pub struct BlobManager {
    /// The underlying blob store
    inner: StoreInner,
}

impl Clone for BlobManager {
    fn clone(&self) -> Self {
        match &self.inner {
            StoreInner::Memory(store) => Self {
                inner: StoreInner::Memory(store.clone()),
            },
            StoreInner::Persistent(store) => Self {
                inner: StoreInner::Persistent(store.clone()),
            },
        }
    }
}

impl std::fmt::Debug for BlobManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobManager")
            .field("store", &self.inner)
            .finish()
    }
}

impl BlobManager {
    /// Create a new blob manager with in-memory storage.
    ///
    /// This is suitable for development and testing. Data is lost on restart.
    /// For production, use `new_persistent()`.
    pub fn new_memory() -> Self {
        info!("Creating in-memory blob manager");
        Self {
            inner: StoreInner::Memory(MemStore::new()),
        }
    }

    /// Create a new blob manager with persistent FsStore storage.
    ///
    /// This is the recommended mode for production. Data is persisted
    /// to the filesystem and survives restarts.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory for blob storage. Will be created if it doesn't exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let blob_manager = BlobManager::new_persistent(&data_dir.join("blobs")).await?;
    /// ```
    pub async fn new_persistent(path: &Path) -> BlobResult<Self> {
        // Create directory if it doesn't exist
        tokio::fs::create_dir_all(path).await.map_err(|e| {
            SyncError::Blob(format!("Failed to create blob directory {:?}: {}", path, e))
        })?;

        info!(?path, "Creating persistent blob manager with FsStore");
        let store = FsStore::load(path).await.map_err(|e| {
            SyncError::Blob(format!("Failed to load FsStore at {:?}: {}", path, e))
        })?;

        Ok(Self {
            inner: StoreInner::Persistent(store),
        })
    }

    /// Check if this is a persistent (FsStore) blob manager.
    pub fn is_persistent(&self) -> bool {
        matches!(self.inner, StoreInner::Persistent(_))
    }

    /// Get a reference to the underlying store as a trait object.
    ///
    /// This is needed for integrating with the iroh Router.
    pub fn store(&self) -> iroh_blobs::api::Store {
        match &self.inner {
            StoreInner::Memory(store) => store.as_ref().clone(),
            StoreInner::Persistent(store) => (*store).clone().into(),
        }
    }

    /// Get the MemStore if this is a memory-based manager.
    ///
    /// Returns None if this is a persistent manager.
    /// Used for backward compatibility with code expecting MemStore.
    pub fn mem_store(&self) -> Option<&MemStore> {
        match &self.inner {
            StoreInner::Memory(store) => Some(store),
            StoreInner::Persistent(_) => None,
        }
    }

    /// Get the FsStore if this is a persistent manager.
    ///
    /// Returns None if this is a memory-based manager.
    pub fn fs_store(&self) -> Option<&FsStore> {
        match &self.inner {
            StoreInner::Memory(_) => None,
            StoreInner::Persistent(store) => Some(store),
        }
    }

    /// Import an avatar image with size validation.
    ///
    /// Avatars are limited to 256 KB. Use WebP format for best results.
    ///
    /// Returns the BLAKE3 hash of the imported data.
    pub async fn import_avatar(&self, data: impl Into<Bytes>) -> BlobResult<Hash> {
        let data: Bytes = data.into();
        let len = data.len();

        if len > MAX_AVATAR_SIZE {
            return Err(SyncError::Blob(format!(
                "Avatar too large: {} bytes (max {} bytes / {} KB). Use WebP format and resize.",
                len,
                MAX_AVATAR_SIZE,
                MAX_AVATAR_SIZE / 1024
            )));
        }

        self.import_bytes_internal(data).await
    }

    /// Import a general image with size validation.
    ///
    /// General images (quest images, etc.) are limited to 2 MB.
    ///
    /// Returns the BLAKE3 hash of the imported data.
    pub async fn import_image(&self, data: impl Into<Bytes>) -> BlobResult<Hash> {
        let data: Bytes = data.into();
        let len = data.len();

        if len > MAX_IMAGE_SIZE {
            return Err(SyncError::Blob(format!(
                "Image too large: {} bytes (max {} bytes / {} MB)",
                len,
                MAX_IMAGE_SIZE,
                MAX_IMAGE_SIZE / (1024 * 1024)
            )));
        }

        self.import_bytes_internal(data).await
    }

    /// Import raw bytes without size validation.
    ///
    /// Use `import_avatar()` or `import_image()` for size-validated imports.
    /// This method is for internal use or when you need to bypass size limits.
    ///
    /// Returns the BLAKE3 hash of the imported data.
    pub async fn import_bytes(&self, data: impl Into<Bytes>) -> BlobResult<Hash> {
        self.import_bytes_internal(data.into()).await
    }

    /// Internal import implementation.
    async fn import_bytes_internal(&self, data: Bytes) -> BlobResult<Hash> {
        let len = data.len();

        let blobs = match &self.inner {
            StoreInner::Memory(store) => store.blobs(),
            StoreInner::Persistent(store) => store.blobs(),
        };

        let tag = blobs
            .add_bytes(data)
            .temp_tag()
            .await
            .map_err(|e| SyncError::Blob(format!("Failed to import blob: {}", e)))?;

        let hash = tag.hash();
        debug!(?hash, len, "Imported blob");
        Ok(hash)
    }

    /// Get bytes from the blob store by hash.
    ///
    /// Returns `None` if the blob doesn't exist locally.
    pub async fn get_bytes(&self, hash: &Hash) -> BlobResult<Option<Bytes>> {
        let blobs = match &self.inner {
            StoreInner::Memory(store) => store.blobs(),
            StoreInner::Persistent(store) => store.blobs(),
        };

        // First check if we have the blob
        let has_blob = blobs
            .has(*hash)
            .await
            .map_err(|e| SyncError::Blob(format!("Failed to check blob: {}", e)))?;

        if !has_blob {
            return Ok(None);
        }

        // Read the blob data
        let data = blobs
            .get_bytes(*hash)
            .await
            .map_err(|e| SyncError::Blob(format!("Failed to get blob: {}", e)))?;

        Ok(Some(data))
    }

    /// Check if a blob exists in the local store.
    pub async fn has_blob(&self, hash: &Hash) -> BlobResult<bool> {
        let blobs = match &self.inner {
            StoreInner::Memory(store) => store.blobs(),
            StoreInner::Persistent(store) => store.blobs(),
        };

        blobs
            .has(*hash)
            .await
            .map_err(|e| SyncError::Blob(format!("Failed to check blob: {}", e)))
    }

    /// Create a blob ticket for sharing.
    ///
    /// The ticket contains the hash and the endpoint address,
    /// allowing others to download the blob from this node via P2P.
    pub fn create_ticket(&self, hash: Hash, endpoint: &Endpoint) -> BlobTicket {
        BlobTicket::new(endpoint.addr(), hash, BlobFormat::Raw)
    }

    /// Download a blob from a peer using a ticket.
    ///
    /// This initiates a P2P download of the blob specified in the ticket.
    /// The blob will be stored locally after download.
    ///
    /// Returns the hash of the downloaded blob.
    pub async fn download_blob(
        &self,
        ticket: &BlobTicket,
        endpoint: &Endpoint,
    ) -> BlobResult<Hash> {
        let hash = ticket.hash();
        let node_addr = ticket.addr();

        debug!(?hash, peer = %node_addr.id, "Downloading blob from peer");

        let store = self.store();
        let downloader = store.downloader(endpoint);

        // Download the blob
        let mut stream = downloader
            .download(hash, vec![node_addr.id])
            .stream()
            .await
            .map_err(|e| SyncError::Blob(format!("Failed to start download: {}", e)))?;

        // Wait for download to complete
        use n0_future::StreamExt;
        while let Some(event) = stream.next().await {
            debug!(?hash, ?event, "Download progress");
        }

        info!(?hash, "Blob download complete");
        Ok(hash)
    }

    /// Download a blob from a ticket string.
    ///
    /// Convenience method that parses the ticket and downloads.
    pub async fn download_from_ticket_str(
        &self,
        ticket_str: &str,
        endpoint: &Endpoint,
    ) -> BlobResult<Hash> {
        let ticket = Self::parse_ticket(ticket_str)?;
        self.download_blob(&ticket, endpoint).await
    }

    /// Delete a blob from the local store.
    ///
    /// Returns `Ok(())` even if the blob doesn't exist.
    /// Note: iroh-blobs uses tags for retention - blobs without tags are garbage collected.
    pub async fn delete_blob(&self, hash: &Hash) -> BlobResult<()> {
        // iroh-blobs uses tags for retention - blobs without tags are garbage collected
        // For now, we just return Ok since explicit deletion isn't directly supported
        debug!(?hash, "Delete blob requested (will be GC'd when tag removed)");
        Ok(())
    }

    /// Get the size of a blob in bytes.
    ///
    /// Returns `None` if the blob doesn't exist.
    pub async fn blob_size(&self, hash: &Hash) -> BlobResult<Option<u64>> {
        let blobs = match &self.inner {
            StoreInner::Memory(store) => store.blobs(),
            StoreInner::Persistent(store) => store.blobs(),
        };

        let status = blobs
            .status(*hash)
            .await
            .map_err(|e| SyncError::Blob(format!("Failed to get blob status: {}", e)))?;

        use iroh_blobs::api::blobs::BlobStatus;
        match status {
            BlobStatus::Complete { size, .. } => Ok(Some(size)),
            BlobStatus::Partial { .. } => Ok(None), // Partial blobs don't have full size yet
            BlobStatus::NotFound => Ok(None),
        }
    }

    /// Parse a hash from a hex string.
    pub fn parse_hash(hex_str: &str) -> BlobResult<Hash> {
        let bytes = hex::decode(hex_str)
            .map_err(|e| SyncError::Blob(format!("Invalid hash hex: {}", e)))?;

        if bytes.len() != 32 {
            return Err(SyncError::Blob(format!(
                "Invalid hash length: expected 32 bytes, got {}",
                bytes.len()
            )));
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash::from_bytes(arr))
    }

    /// Format a hash as a hex string.
    pub fn hash_to_hex(hash: &Hash) -> String {
        hex::encode(hash.as_bytes())
    }

    /// Parse a blob ticket from a base58-encoded string.
    ///
    /// This extracts the hash and peer addresses from a ticket.
    pub fn parse_ticket(ticket_str: &str) -> BlobResult<BlobTicket> {
        ticket_str
            .parse::<BlobTicket>()
            .map_err(|e| SyncError::Blob(format!("Invalid blob ticket: {}", e)))
    }

    /// Extract the hash from a blob ticket string.
    ///
    /// This is useful for checking if we already have a blob locally
    /// before attempting to download.
    pub fn ticket_hash(ticket_str: &str) -> BlobResult<Hash> {
        let ticket = Self::parse_ticket(ticket_str)?;
        Ok(ticket.hash())
    }

    /// Convert an iroh Hash to a hex string (for storage compatibility).
    ///
    /// This allows storing blob references as strings in redb or other storage.
    pub fn hash_to_blob_id(hash: &Hash) -> String {
        Self::hash_to_hex(hash)
    }

    /// Convert a hex blob ID string back to an iroh Hash.
    ///
    /// This allows retrieving blobs stored with string IDs.
    pub fn blob_id_to_hash(blob_id: &str) -> BlobResult<Hash> {
        Self::parse_hash(blob_id)
    }
}

/// Protocol handler for blob transfers.
///
/// This wraps the BlobsProtocol for integration with the iroh Router.
pub struct BlobProtocolHandler {
    store: iroh_blobs::api::Store,
}

impl BlobProtocolHandler {
    /// Create a new blob protocol handler from a BlobManager.
    pub fn from_manager(manager: &BlobManager) -> Self {
        Self {
            store: manager.store(),
        }
    }

    /// Create a new blob protocol handler from a MemStore (legacy).
    pub fn new(store: &MemStore) -> Self {
        Self {
            store: store.as_ref().clone(),
        }
    }

    /// Create a new blob protocol handler from an FsStore.
    pub fn from_fs_store(store: &FsStore) -> Self {
        Self {
            store: (*store).clone().into(),
        }
    }

    /// Get the iroh-blobs protocol handler for router integration.
    pub fn protocol(&self) -> iroh_blobs::BlobsProtocol {
        iroh_blobs::BlobsProtocol::new(&self.store, None)
    }

    /// Get the ALPN for blob protocol.
    pub fn alpn() -> &'static [u8] {
        iroh_blobs::ALPN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_import_and_get_bytes() {
        let manager = BlobManager::new_memory();
        let data = Bytes::from_static(b"Hello, World!");

        // Import
        let hash = manager.import_bytes(data.clone()).await.unwrap();

        // Verify hash is deterministic
        let hash2 = manager.import_bytes(data.clone()).await.unwrap();
        assert_eq!(hash, hash2);

        // Get bytes
        let retrieved = manager.get_bytes(&hash).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().as_ref(), b"Hello, World!");
    }

    #[tokio::test]
    async fn test_get_nonexistent_blob() {
        let manager = BlobManager::new_memory();
        let fake_hash = Hash::from_bytes([0u8; 32]);

        let result = manager.get_bytes(&fake_hash).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_has_blob() {
        let manager = BlobManager::new_memory();
        let data = Bytes::from_static(b"Test data");

        let hash = manager.import_bytes(data).await.unwrap();
        assert!(manager.has_blob(&hash).await.unwrap());

        let fake_hash = Hash::from_bytes([1u8; 32]);
        assert!(!manager.has_blob(&fake_hash).await.unwrap());
    }

    #[tokio::test]
    async fn test_blob_size() {
        let manager = BlobManager::new_memory();
        let data = Bytes::from_static(b"Test data for size check");

        let hash = manager.import_bytes(data.clone()).await.unwrap();
        let size = manager.blob_size(&hash).await.unwrap();

        assert!(size.is_some());
        assert_eq!(size.unwrap(), data.len() as u64);
    }

    #[tokio::test]
    async fn test_hash_hex_roundtrip() {
        let manager = BlobManager::new_memory();
        let data = Bytes::from_static(b"Hash roundtrip test");

        let hash = manager.import_bytes(data).await.unwrap();

        // Convert to hex and back
        let hex_str = BlobManager::hash_to_hex(&hash);
        let parsed = BlobManager::parse_hash(&hex_str).unwrap();

        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_parse_invalid_hash() {
        // Invalid hex
        let result = BlobManager::parse_hash("not-valid-hex");
        assert!(result.is_err());

        // Wrong length
        let result = BlobManager::parse_hash("abcd");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_large_blob() {
        let manager = BlobManager::new_memory();

        // Create a 1MB blob
        let data = Bytes::from(vec![0xABu8; 1024 * 1024]);

        let hash = manager.import_bytes(data.clone()).await.unwrap();
        let retrieved = manager.get_bytes(&hash).await.unwrap().unwrap();

        assert_eq!(retrieved.len(), data.len());
        assert_eq!(&retrieved[..100], &data[..100]); // Check first 100 bytes
    }

    #[tokio::test]
    async fn test_avatar_size_limit() {
        let manager = BlobManager::new_memory();

        // Small avatar should work
        let small_avatar = vec![0u8; 100 * 1024]; // 100 KB
        let result = manager.import_avatar(small_avatar).await;
        assert!(result.is_ok());

        // Avatar at limit should work
        let at_limit = vec![0u8; MAX_AVATAR_SIZE];
        let result = manager.import_avatar(at_limit).await;
        assert!(result.is_ok());

        // Avatar over limit should fail
        let too_large = vec![0u8; MAX_AVATAR_SIZE + 1];
        let result = manager.import_avatar(too_large).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Avatar too large"));
    }

    #[tokio::test]
    async fn test_image_size_limit() {
        let manager = BlobManager::new_memory();

        // Image at limit should work
        let at_limit = vec![0u8; MAX_IMAGE_SIZE];
        let result = manager.import_image(at_limit).await;
        assert!(result.is_ok());

        // Image over limit should fail
        let too_large = vec![0u8; MAX_IMAGE_SIZE + 1];
        let result = manager.import_image(too_large).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Image too large"));
    }

    #[tokio::test]
    async fn test_persistent_store() {
        let temp_dir = tempfile::tempdir().unwrap();
        let blob_path = temp_dir.path().join("blobs");

        // Create persistent manager
        let manager = BlobManager::new_persistent(&blob_path).await.unwrap();
        assert!(manager.is_persistent());

        // Import some data
        let data = Bytes::from_static(b"Persistent test data");
        let hash = manager.import_bytes(data.clone()).await.unwrap();

        // Verify it exists
        assert!(manager.has_blob(&hash).await.unwrap());

        // Get the data back
        let retrieved = manager.get_bytes(&hash).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().as_ref(), b"Persistent test data");
    }

    #[tokio::test]
    async fn test_blob_id_conversion() {
        let manager = BlobManager::new_memory();
        let data = Bytes::from_static(b"Blob ID test");

        let hash = manager.import_bytes(data).await.unwrap();

        // Convert to blob ID (hex string)
        let blob_id = BlobManager::hash_to_blob_id(&hash);

        // Convert back to hash
        let recovered_hash = BlobManager::blob_id_to_hash(&blob_id).unwrap();

        assert_eq!(hash, recovered_hash);
    }

    #[tokio::test]
    async fn test_memory_store_access() {
        let manager = BlobManager::new_memory();
        assert!(manager.mem_store().is_some());
        assert!(manager.fs_store().is_none());
        assert!(!manager.is_persistent());
    }
}
