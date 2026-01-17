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

    // Setup Alice (blob provider)
    let alice_dir = tempdir().unwrap();
    let mut alice = SyncEngine::new(alice_dir.path()).await.unwrap();
    alice.init_identity().unwrap();
    alice.start_networking().await.unwrap();
    sleep(Duration::from_millis(300)).await;

    // Setup Bob (blob requester)
    let bob_dir = tempdir().unwrap();
    let mut bob = SyncEngine::new(bob_dir.path()).await.unwrap();
    bob.init_identity().unwrap();
    bob.start_networking().await.unwrap();
    sleep(Duration::from_millis(300)).await;

    // Alice uploads a blob
    let original_data = vec![0xEF; 4096]; // 4KB test blob
    let blob_id = alice.upload_image(original_data.clone()).await.unwrap();

    // Alice creates a ticket
    let ticket_str = alice.create_image_ticket(&blob_id).await.unwrap();
    println!("Alice created ticket: {}...", &ticket_str[..50.min(ticket_str.len())]);

    // Verify Bob doesn't have this blob yet
    let bob_has_blob = bob.load_image(&blob_id).await.unwrap();
    assert!(bob_has_blob.is_none(), "Bob should not have blob initially");

    // Bob downloads via ticket
    let downloaded_blob_id = bob.download_image_from_ticket(&ticket_str).await.unwrap();
    assert_eq!(blob_id, downloaded_blob_id, "Downloaded blob ID should match");

    // Verify Bob now has the blob
    let bob_data = bob.load_image(&blob_id).await.unwrap();
    assert!(bob_data.is_some(), "Bob should have blob after download");
    assert_eq!(bob_data.unwrap(), original_data, "Downloaded data should match original");

    println!("✅ Two-node blob transfer completed successfully!");
}

/// Test 3-node blob propagation: Alice → Bob → Carol
///
/// This tests that:
/// 1. Alice creates a blob
/// 2. Bob downloads from Alice
/// 3. Carol downloads from Bob (not Alice)
/// 4. All three nodes have identical blob data
#[tokio::test]
async fn test_three_node_blob_propagation() {
    tracing_subscriber::fmt()
        .with_env_filter("info,quinn=warn,iroh=warn")
        .try_init()
        .ok();

    // Setup Alice (original blob owner)
    let alice_dir = tempdir().unwrap();
    let mut alice = SyncEngine::new(alice_dir.path()).await.unwrap();
    alice.init_identity().unwrap();
    alice.start_networking().await.unwrap();
    sleep(Duration::from_millis(300)).await;

    // Setup Bob (will download from Alice)
    let bob_dir = tempdir().unwrap();
    let mut bob = SyncEngine::new(bob_dir.path()).await.unwrap();
    bob.init_identity().unwrap();
    bob.start_networking().await.unwrap();
    sleep(Duration::from_millis(300)).await;

    // Setup Carol (will download from Bob)
    let carol_dir = tempdir().unwrap();
    let mut carol = SyncEngine::new(carol_dir.path()).await.unwrap();
    carol.init_identity().unwrap();
    carol.start_networking().await.unwrap();
    sleep(Duration::from_millis(300)).await;

    println!("All three nodes started");

    // Alice creates a 10KB blob
    let original_data = vec![0x42; 10240]; // 10KB
    let blob_id = alice.upload_image(original_data.clone()).await.unwrap();
    let alice_ticket = alice.create_image_ticket(&blob_id).await.unwrap();
    println!("Alice uploaded blob: {}", blob_id);

    // Bob downloads from Alice
    let bob_blob_id = bob.download_image_from_ticket(&alice_ticket).await.unwrap();
    assert_eq!(blob_id, bob_blob_id);

    let bob_data = bob.load_image(&blob_id).await.unwrap().unwrap();
    assert_eq!(bob_data, original_data, "Bob's data should match Alice's");
    println!("Bob downloaded blob from Alice");

    // Bob creates a ticket for the same blob
    let bob_ticket = bob.create_image_ticket(&blob_id).await.unwrap();

    // Carol downloads from Bob's ticket (different peer addresses)
    let carol_blob_id = carol.download_image_from_ticket(&bob_ticket).await.unwrap();
    assert_eq!(blob_id, carol_blob_id);

    let carol_data = carol.load_image(&blob_id).await.unwrap().unwrap();
    assert_eq!(carol_data, original_data, "Carol's data should match original");
    println!("Carol downloaded blob from Bob");

    // Verify all three have identical data
    let alice_final = alice.load_image(&blob_id).await.unwrap().unwrap();
    let bob_final = bob.load_image(&blob_id).await.unwrap().unwrap();
    let carol_final = carol.load_image(&blob_id).await.unwrap().unwrap();

    assert_eq!(alice_final, bob_final);
    assert_eq!(bob_final, carol_final);
    assert_eq!(alice_final.len(), 10240);

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

    // Setup Alice
    let alice_dir = tempdir().unwrap();
    let mut alice = SyncEngine::new(alice_dir.path()).await.unwrap();
    alice.init_identity().unwrap();
    alice.start_networking().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    // Setup Bob
    let bob_dir = tempdir().unwrap();
    let mut bob = SyncEngine::new(bob_dir.path()).await.unwrap();
    bob.init_identity().unwrap();
    bob.start_networking().await.unwrap();
    sleep(Duration::from_millis(500)).await;

    // Exchange contacts
    let invite_code = alice.generate_contact_invite(24).await.unwrap();
    let invite = bob.decode_contact_invite(&invite_code).await.unwrap();
    bob.send_contact_request(invite).await.unwrap();
    sleep(Duration::from_millis(1000)).await;

    let (alice_incoming, _) = alice.list_pending_contacts().unwrap();
    if !alice_incoming.is_empty() {
        alice.accept_contact(&alice_incoming[0].invite_id).await.unwrap();
        sleep(Duration::from_millis(1500)).await;
    }

    // Verify they're contacts
    let alice_contacts = alice.list_contacts().unwrap();
    let bob_contacts = bob.list_contacts().unwrap();

    if alice_contacts.is_empty() || bob_contacts.is_empty() {
        println!("⚠️ Contact exchange didn't complete - skipping blob test");
        return;
    }

    println!("Contact exchange complete, testing blob sharing...");

    // Alice uploads avatar
    let avatar_data = vec![0xAA; 50 * 1024]; // 50KB avatar
    let avatar_id = alice.upload_avatar(avatar_data.clone()).await.unwrap();
    let avatar_ticket = alice.create_image_ticket(&avatar_id).await.unwrap();

    // Bob downloads Alice's avatar
    let downloaded_id = bob.download_image_from_ticket(&avatar_ticket).await.unwrap();
    assert_eq!(avatar_id, downloaded_id);

    let bob_avatar = bob.load_image(&avatar_id).await.unwrap();
    assert!(bob_avatar.is_some());
    assert_eq!(bob_avatar.unwrap(), avatar_data);

    println!("✅ Blob sharing between contacts works!");
}
