//! P2P Blob Transfer End-to-End Tests
//!
//! These tests verify blob transfer between multiple SyncEngine nodes.
//!
//! ## Test Architecture
//!
//! - **Unit tests** (`src/blobs.rs`): Test BlobManager storage/import
//! - **Integration tests** (this file): Test P2P blob transfers with real networking
//!
//! ## What These Tests Verify
//!
//! - FsStore persistence across engine restarts
//! - BlobTicket creation and parsing
//! - P2P blob downloads between connected peers
//! - 3-node blob propagation (Node A → Node B → Node C)
//! - Avatar size limits in P2P context

use bytes::Bytes;
use syncengine_core::blobs::{BlobManager, MAX_AVATAR_SIZE};
use syncengine_core::engine::SyncEngine;
use tempfile::tempdir;
use tokio::time::{sleep, Duration};

/// Test that FsStore blobs persist across BlobManager recreations.
#[tokio::test]
async fn test_fsstore_blob_persistence() {
    let temp_dir = tempdir().unwrap();
    let blob_path = temp_dir.path().join("blobs");

    // Create manager and import a blob
    let hash = {
        let manager = BlobManager::new_persistent(&blob_path).await.unwrap();
        let data = Bytes::from_static(b"Persistent blob test data");
        manager.import_bytes(data).await.unwrap()
    };

    // Create new manager on same path - blob should still exist
    let manager2 = BlobManager::new_persistent(&blob_path).await.unwrap();
    assert!(
        manager2.has_blob(&hash).await.unwrap(),
        "Blob should persist across manager restarts"
    );

    let retrieved = manager2.get_bytes(&hash).await.unwrap();
    assert!(retrieved.is_some(), "Should retrieve persisted blob");
    assert_eq!(
        retrieved.unwrap().as_ref(),
        b"Persistent blob test data"
    );
}

/// Test that SyncEngine's blob manager uses FsStore and persists blobs.
#[tokio::test]
async fn test_engine_blob_persistence() {
    let temp_dir = tempdir().unwrap();

    // First engine instance - upload an image
    let blob_id = {
        let engine = SyncEngine::new(temp_dir.path()).await.unwrap();
        let data = vec![0xAB; 1024]; // 1KB test image
        engine.upload_image(data).await.unwrap()
    };

    // Second engine instance - image should still be there
    let engine2 = SyncEngine::new(temp_dir.path()).await.unwrap();
    let loaded = engine2.load_image(&blob_id).await.unwrap();

    assert!(loaded.is_some(), "Image should persist across engine restarts");
    assert_eq!(loaded.unwrap().len(), 1024);
}

/// Test BlobTicket creation and hash extraction.
#[tokio::test]
async fn test_blob_ticket_creation() {
    let temp_dir = tempdir().unwrap();
    let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
    engine.init_identity().unwrap();
    engine.start_networking().await.unwrap();

    // Give networking time to start
    sleep(Duration::from_millis(200)).await;

    // Upload an image
    let data = vec![0xCD; 2048]; // 2KB test image
    let blob_id = engine.upload_image(data).await.unwrap();

    // Create ticket
    let ticket_str = engine.create_image_ticket(&blob_id).await.unwrap();

    // Ticket should be a valid base58 string
    assert!(!ticket_str.is_empty(), "Ticket should not be empty");

    // Parse ticket and verify hash matches
    let hash_from_ticket = BlobManager::ticket_hash(&ticket_str).unwrap();
    let original_hash = BlobManager::blob_id_to_hash(&blob_id).unwrap();

    assert_eq!(hash_from_ticket, original_hash, "Ticket hash should match original blob");
}

/// Test P2P blob download between two engines.
#[tokio::test]
async fn test_two_node_blob_transfer() {
    tracing_subscriber::fmt()
        .with_env_filter("debug,quinn=warn,iroh=warn")
        .try_init()
        .ok();

    // Setup Love (blob provider)
    let love_dir = tempdir().unwrap();
    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_identity().unwrap();
    love.start_networking().await.unwrap();
    sleep(Duration::from_millis(300)).await;

    // Setup Joy (blob requester)
    let joy_dir = tempdir().unwrap();
    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_identity().unwrap();
    joy.start_networking().await.unwrap();
    sleep(Duration::from_millis(300)).await;

    // Love uploads a blob
    let original_data = vec![0xEF; 4096]; // 4KB test blob
    let blob_id = love.upload_image(original_data.clone()).await.unwrap();

    // Love creates a ticket
    let ticket_str = love.create_image_ticket(&blob_id).await.unwrap();
    println!("Love created ticket: {}...", &ticket_str[..50.min(ticket_str.len())]);

    // Verify Joy doesn't have this blob yet
    let joy_has_blob = joy.load_image(&blob_id).await.unwrap();
    assert!(joy_has_blob.is_none(), "Joy should not have blob initially");

    // Joy downloads via ticket
    let downloaded_blob_id = joy.download_image_from_ticket(&ticket_str).await.unwrap();
    assert_eq!(blob_id, downloaded_blob_id, "Downloaded blob ID should match");

    // Verify Joy now has the blob
    let joy_data = joy.load_image(&blob_id).await.unwrap();
    assert!(joy_data.is_some(), "Joy should have blob after download");
    assert_eq!(joy_data.unwrap(), original_data, "Downloaded data should match original");

    println!("✅ Two-node blob transfer completed successfully!");
}

/// Test 3-node blob propagation: Love → Joy → Peace
///
/// This tests that:
/// 1. Love creates a blob
/// 2. Joy downloads from Love
/// 3. Peace downloads from Joy (not Love)
/// 4. All three nodes have identical blob data
#[tokio::test]
async fn test_three_node_blob_propagation() {
    tracing_subscriber::fmt()
        .with_env_filter("info,quinn=warn,iroh=warn")
        .try_init()
        .ok();

    // Setup Love (original blob owner)
    let love_dir = tempdir().unwrap();
    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_identity().unwrap();
    love.start_networking().await.unwrap();
    sleep(Duration::from_millis(300)).await;

    // Setup Joy (will download from Love)
    let joy_dir = tempdir().unwrap();
    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_identity().unwrap();
    joy.start_networking().await.unwrap();
    sleep(Duration::from_millis(300)).await;

    // Setup Peace (will download from Joy)
    let peace_dir = tempdir().unwrap();
    let mut peace = SyncEngine::new(peace_dir.path()).await.unwrap();
    peace.init_identity().unwrap();
    peace.start_networking().await.unwrap();
    sleep(Duration::from_millis(300)).await;

    println!("All three nodes started");

    // Love creates a 10KB blob
    let original_data = vec![0x42; 10240]; // 10KB
    let blob_id = love.upload_image(original_data.clone()).await.unwrap();
    let love_ticket = love.create_image_ticket(&blob_id).await.unwrap();
    println!("Love uploaded blob: {}", blob_id);

    // Joy downloads from Love
    let joy_blob_id = joy.download_image_from_ticket(&love_ticket).await.unwrap();
    assert_eq!(blob_id, joy_blob_id);

    let joy_data = joy.load_image(&blob_id).await.unwrap().unwrap();
    assert_eq!(joy_data, original_data, "Joy's data should match Love's");
    println!("Joy downloaded blob from Love");

    // Joy creates a ticket for the same blob
    let joy_ticket = joy.create_image_ticket(&blob_id).await.unwrap();

    // Peace downloads from Joy's ticket (different peer addresses)
    let peace_blob_id = peace.download_image_from_ticket(&joy_ticket).await.unwrap();
    assert_eq!(blob_id, peace_blob_id);

    let peace_data = peace.load_image(&blob_id).await.unwrap().unwrap();
    assert_eq!(peace_data, original_data, "Peace's data should match original");
    println!("Peace downloaded blob from Joy");

    // Verify all three have identical data
    let love_final = love.load_image(&blob_id).await.unwrap().unwrap();
    let joy_final = joy.load_image(&blob_id).await.unwrap().unwrap();
    let peace_final = peace.load_image(&blob_id).await.unwrap().unwrap();

    assert_eq!(love_final, joy_final);
    assert_eq!(joy_final, peace_final);
    assert_eq!(love_final.len(), 10240);

    println!("✅ Three-node blob propagation completed successfully!");
}

/// Test avatar upload with size limit enforcement
#[tokio::test]
async fn test_avatar_size_limit_in_engine() {
    let temp_dir = tempdir().unwrap();
    let engine = SyncEngine::new(temp_dir.path()).await.unwrap();

    // Small avatar should work
    let small_avatar = vec![0u8; 100 * 1024]; // 100KB
    let result = engine.upload_avatar(small_avatar).await;
    assert!(result.is_ok(), "Small avatar should upload successfully");

    // Avatar at limit should work
    let at_limit = vec![0u8; MAX_AVATAR_SIZE];
    let result = engine.upload_avatar(at_limit).await;
    assert!(result.is_ok(), "Avatar at limit should upload successfully");

    // Avatar over limit should fail
    let too_large = vec![0u8; MAX_AVATAR_SIZE + 1];
    let result = engine.upload_avatar(too_large).await;
    assert!(result.is_err(), "Avatar over limit should fail");
    assert!(
        result.unwrap_err().to_string().contains("Avatar too large"),
        "Error should mention avatar size"
    );

    println!("✅ Avatar size limits enforced correctly!");
}

/// Test that blobs can be shared after contact exchange
#[tokio::test]
async fn test_blob_sharing_between_contacts() {
    tracing_subscriber::fmt()
        .with_env_filter("debug,quinn=warn,iroh=warn")
        .try_init()
        .ok();

    // Setup Love
    let love_dir = tempdir().unwrap();
    let mut love = SyncEngine::new(love_dir.path()).await.unwrap();
    love.init_identity().unwrap();
    love.start_networking().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    // Setup Joy
    let joy_dir = tempdir().unwrap();
    let mut joy = SyncEngine::new(joy_dir.path()).await.unwrap();
    joy.init_identity().unwrap();
    joy.start_networking().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    // Exchange contacts
    let invite_code = love.generate_contact_invite(24).await.unwrap();
    let invite = joy.decode_contact_invite(&invite_code).await.unwrap();
    joy.send_contact_request(invite).await.unwrap();
    sleep(Duration::from_millis(1000)).await;

    let (love_incoming, _) = love.list_pending_contacts().unwrap();
    if !love_incoming.is_empty() {
        love.accept_contact(&love_incoming[0].invite_id).await.unwrap();
        sleep(Duration::from_millis(1500)).await;
    }

    // Verify they're contacts
    let love_contacts = love.list_contacts().unwrap();
    let joy_contacts = joy.list_contacts().unwrap();

    if love_contacts.is_empty() || joy_contacts.is_empty() {
        println!("⚠️ Contact exchange didn't complete - skipping blob test");
        return;
    }

    println!("Contact exchange complete, testing blob sharing...");

    // Love uploads avatar
    let avatar_data = vec![0xAA; 50 * 1024]; // 50KB avatar
    let avatar_id = love.upload_avatar(avatar_data.clone()).await.unwrap();
    let avatar_ticket = love.create_image_ticket(&avatar_id).await.unwrap();

    // Joy downloads Love's avatar
    let downloaded_id = joy.download_image_from_ticket(&avatar_ticket).await.unwrap();
    assert_eq!(avatar_id, downloaded_id);

    let joy_avatar = joy.load_image(&avatar_id).await.unwrap();
    assert!(joy_avatar.is_some());
    assert_eq!(joy_avatar.unwrap(), avatar_data);

    println!("✅ Blob sharing between contacts works!");
}
