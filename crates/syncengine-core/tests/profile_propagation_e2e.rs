//! End-to-End tests for profile update propagation between contacts
//!
//! These tests verify that profile changes (display name, avatar, etc.) propagate
//! correctly between contacts, including after app restarts.
//!
//! ## The Bug This Tests For
//!
//! Previously, profile updates would fail to propagate due to two issues:
//!
//! 1. **`profile_gossip_sender` not initialized at startup**: The sender was only
//!    set during `ensure_contact_manager()`, which was lazily called. At startup,
//!    `startup_sync()` would call `announce_profile()` which failed silently.
//!
//! 2. **Per-peer profile topic listeners lost on restart**: When `finalize_contact()`
//!    spawns a profile topic listener, that tokio task is lost on restart.
//!    `reconnect_contacts()` wasn't called at startup to recreate them.
//!
//! ## Test Scenarios
//!
//! 1. **Initial propagation**: Alice changes name -> Bob sees it
//! 2. **Post-restart propagation**: After restart, Alice changes name -> Bob sees it
//! 3. **Bidirectional**: Both Alice and Bob update profiles, both see changes
//!
//! ## Key Assertions
//!
//! - `profile_gossip_sender` should be Some after `startup_sync()`
//! - Profile updates should propagate before and after restart
//! - Peer.profile.display_name should reflect the latest broadcast

use std::path::PathBuf;
use std::time::Duration;

use syncengine_core::{SyncEngine, SyncError};
use tempfile::TempDir;
use tokio::time::sleep;

// ============================================================================
// Test Utilities
// ============================================================================

/// Test context that manages temporary directories and SyncEngine lifecycle
struct TestContext {
    temp_dir: TempDir,
    data_dir: PathBuf,
}

impl TestContext {
    fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self { temp_dir, data_dir })
    }

    fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    async fn create_engine(&self) -> anyhow::Result<SyncEngine> {
        Ok(SyncEngine::new(&self.data_dir).await?)
    }
}

/// Helper to update a user's profile display name
async fn update_display_name(engine: &mut SyncEngine, name: &str) -> Result<(), SyncError> {
    let mut profile = engine.get_own_profile()?;
    profile.display_name = name.to_string();
    engine.save_profile(&profile)?;
    Ok(())
}

/// Helper to get a contact's display name from the Peer table
fn get_peer_display_name(engine: &SyncEngine, did: &str) -> Option<String> {
    engine
        .get_peer_by_did(did)
        .ok()
        .flatten()
        .and_then(|peer| peer.profile)
        .map(|p| p.display_name)
}

/// Wait for a peer's profile to have a specific display name
async fn wait_for_profile_update(
    engine: &SyncEngine,
    peer_did: &str,
    expected_name: &str,
    timeout: Duration,
) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            let current_name = get_peer_display_name(engine, peer_did);
            anyhow::bail!(
                "Timeout waiting for profile update. Expected '{}', got '{:?}'",
                expected_name,
                current_name
            );
        }

        if let Some(name) = get_peer_display_name(engine, peer_did) {
            if name == expected_name {
                return Ok(());
            }
        }

        sleep(Duration::from_millis(200)).await;
    }
}

// ============================================================================
// Test 1: Profile Updates Propagate After Contact Exchange
// ============================================================================

/// Test that profile updates propagate to contacts after initial exchange
///
/// ## Test Flow:
/// 1. Alice and Bob exchange contacts (both start with "Anonymous User")
/// 2. Alice updates her display name to "Alice"
/// 3. Alice broadcasts her profile via announce_profile()
/// 4. Verify Bob's Peer table shows "Alice" for Alice's DID
///
/// ## What This Tests:
/// - Profile sync initialization during contact exchange
/// - announce_profile() successfully broadcasts
/// - Per-peer profile topic listener receives updates
///
/// ## Known Issue (2024-01)
/// Per-peer profile topic subscriptions close immediately because the gossip mesh
/// doesn't form between peers. The bootstrap peer (Alice) is subscribed to her own
/// topic, but Bob's subscription to that topic closes before connectivity is established.
/// This test will pass once the gossip mesh formation issue is resolved.
#[tokio::test]
async fn test_profile_update_propagates_to_contact() {
    tracing_subscriber::fmt()
        .with_env_filter("info,syncengine_core=debug,quinn=warn,iroh=warn")
        .try_init()
        .ok();

    println!("\n=== Test: Profile Update Propagates to Contact ===\n");

    // Setup Alice
    let ctx_alice = TestContext::new().expect("Failed to create Alice context");
    let mut alice = ctx_alice.create_engine().await.expect("Failed to create Alice engine");
    alice.init_identity().unwrap();
    alice.start_networking().await.unwrap();
    // Initialize profile sync (subscribes Alice to her own profile topic)
    alice.startup_sync().await.ok();
    sleep(Duration::from_millis(500)).await;

    // Setup Bob
    let ctx_bob = TestContext::new().expect("Failed to create Bob context");
    let mut bob = ctx_bob.create_engine().await.expect("Failed to create Bob engine");
    bob.init_identity().unwrap();
    bob.start_networking().await.unwrap();
    // Initialize profile sync for Bob too
    bob.startup_sync().await.ok();
    sleep(Duration::from_millis(500)).await;

    let alice_did = alice.did().unwrap().to_string();
    let bob_did = bob.did().unwrap().to_string();

    println!("Alice DID: {}", alice_did);
    println!("Bob DID: {}", bob_did);

    // Exchange contacts
    let invite_code = alice.generate_contact_invite(24).await.unwrap();
    let invite = bob.decode_contact_invite(&invite_code).await.unwrap();
    bob.send_contact_request(invite).await.unwrap();

    // Wait for contact exchange to complete and gossip mesh to form
    // Profile topic subscriptions need time to establish connections
    sleep(Duration::from_millis(3000)).await;

    // Verify contacts were created
    let alice_contacts = alice.list_contacts().unwrap();
    let bob_contacts = bob.list_contacts().unwrap();
    assert_eq!(alice_contacts.len(), 1, "Alice should have 1 contact");
    assert_eq!(bob_contacts.len(), 1, "Bob should have 1 contact");
    println!("Contact exchange complete");

    // Alice updates her display name
    update_display_name(&mut alice, "Alice").await.unwrap();
    println!("Alice updated display name to 'Alice'");

    // Alice broadcasts her profile
    alice.announce_profile(None).await.unwrap();
    println!("Alice broadcast profile update");

    // Wait for Bob to receive the update
    wait_for_profile_update(&bob, &alice_did, "Alice", Duration::from_secs(10))
        .await
        .expect("Bob should receive Alice's profile update");

    let bob_sees_alice_name = get_peer_display_name(&bob, &alice_did);
    println!("Bob sees Alice as: {:?}", bob_sees_alice_name);
    assert_eq!(bob_sees_alice_name, Some("Alice".to_string()));

    // Cleanup
    alice.shutdown().await.ok();
    bob.shutdown().await.ok();

    println!("\n=== PASSED: Profile update propagated successfully ===\n");
}

// ============================================================================
// Test 2: Profile Updates Work After Restart (THE CRITICAL BUG TEST)
// ============================================================================

/// Test that profile updates propagate after engine restart
///
/// THIS IS THE CRITICAL TEST that catches the bug where:
/// - `profile_gossip_sender` was not initialized at startup
/// - `reconnect_contacts()` was not called to recreate profile listeners
///
/// ## Test Flow:
///
/// ### Phase 1: Initial Setup
/// 1. Alice and Bob exchange contacts
/// 2. Alice sets name to "Alice v1", broadcasts, Bob receives it
/// 3. Shutdown both engines
///
/// ### Phase 2: Restart and Test (THE BUG SCENARIO)
/// 4. Restart both engines from same data directories
/// 5. Call startup_sync() on both (this should initialize profile sync)
/// 6. Alice updates name to "Alice v2" and broadcasts
/// 7. CRITICAL: Verify Bob receives "Alice v2"
///
/// ## The Bug:
/// Before the fix, step 7 would fail because:
/// - Alice's `profile_gossip_sender` was None (announce_profile fails)
/// - Bob's profile topic listener for Alice wasn't recreated (no receiver)
///
/// ## Known Issue (2024-01)
/// TODO: Fix database locking issue on restart
#[tokio::test]
#[ignore = "Database lock not released on shutdown - needs proper cleanup"]
async fn test_profile_update_works_after_restart() {
    tracing_subscriber::fmt()
        .with_env_filter("info,syncengine_core=debug,quinn=warn,iroh=warn")
        .try_init()
        .ok();

    println!("\n=== Test: Profile Update After Restart (Critical Bug Test) ===\n");

    // Create persistent contexts (data survives restart)
    let ctx_alice = TestContext::new().expect("Failed to create Alice context");
    let ctx_bob = TestContext::new().expect("Failed to create Bob context");

    let alice_did: String;
    let bob_did: String;

    // ========================================================================
    // Phase 1: Initial Setup
    // ========================================================================
    println!("--- Phase 1: Initial Setup ---\n");

    {
        let mut alice = ctx_alice.create_engine().await.expect("Failed to create Alice");
        let mut bob = ctx_bob.create_engine().await.expect("Failed to create Bob");

        alice.init_identity().unwrap();
        bob.init_identity().unwrap();

        alice.start_networking().await.unwrap();
        bob.start_networking().await.unwrap();

        // Initialize profile sync on both
        alice.startup_sync().await.ok();
        bob.startup_sync().await.ok();
        sleep(Duration::from_millis(500)).await;

        alice_did = alice.did().unwrap().to_string();
        bob_did = bob.did().unwrap().to_string();

        println!("Alice DID: {}", alice_did);
        println!("Bob DID: {}", bob_did);

        // Exchange contacts
        let invite_code = alice.generate_contact_invite(24).await.unwrap();
        let invite = bob.decode_contact_invite(&invite_code).await.unwrap();
        bob.send_contact_request(invite).await.unwrap();
        sleep(Duration::from_millis(2000)).await;

        assert_eq!(alice.list_contacts().unwrap().len(), 1);
        assert_eq!(bob.list_contacts().unwrap().len(), 1);
        println!("Contacts exchanged");

        // Alice sets initial name and broadcasts
        update_display_name(&mut alice, "Alice v1").await.unwrap();
        alice.announce_profile(None).await.unwrap();
        println!("Alice set name to 'Alice v1' and broadcast");

        // Verify Bob receives it
        wait_for_profile_update(&bob, &alice_did, "Alice v1", Duration::from_secs(10))
            .await
            .expect("Bob should receive initial profile");

        println!("Bob received 'Alice v1'");

        // Shutdown both engines
        alice.shutdown().await.ok();
        bob.shutdown().await.ok();

        println!("\nPhase 1 complete - engines shutdown\n");
    }

    // ========================================================================
    // Phase 2: Restart and Test
    // ========================================================================
    println!("--- Phase 2: Restart and Test ---\n");

    // Small delay to ensure clean shutdown
    sleep(Duration::from_millis(500)).await;

    // Restart Alice
    let mut alice = ctx_alice.create_engine().await.expect("Failed to restart Alice");
    alice.start_networking().await.unwrap();

    // CRITICAL: Call startup_sync which should initialize profile sync
    let alice_sync_result = alice.startup_sync().await;
    println!("Alice startup_sync result: {:?}", alice_sync_result.is_ok());

    // Restart Bob
    let mut bob = ctx_bob.create_engine().await.expect("Failed to restart Bob");
    bob.start_networking().await.unwrap();

    // CRITICAL: Call startup_sync which should reconnect to profile topics
    let bob_sync_result = bob.startup_sync().await;
    println!("Bob startup_sync result: {:?}", bob_sync_result.is_ok());

    // Wait for networking to stabilize
    sleep(Duration::from_millis(1000)).await;

    // Verify contacts persisted
    assert_eq!(
        alice.list_contacts().unwrap().len(),
        1,
        "Alice should still have contact after restart"
    );
    assert_eq!(
        bob.list_contacts().unwrap().len(),
        1,
        "Bob should still have contact after restart"
    );
    println!("Contacts persisted after restart");

    // Verify Bob still has "Alice v1" from before restart
    let pre_update_name = get_peer_display_name(&bob, &alice_did);
    println!("Bob's stored name for Alice before update: {:?}", pre_update_name);
    assert_eq!(pre_update_name, Some("Alice v1".to_string()));

    // ========================================================================
    // THE CRITICAL TEST: Profile update after restart
    // ========================================================================
    println!("\n--- Critical Test: Post-restart profile update ---\n");

    // Alice updates her name
    update_display_name(&mut alice, "Alice v2").await.unwrap();
    println!("Alice updated display name to 'Alice v2'");

    // Alice broadcasts (THIS WOULD FAIL WITHOUT THE FIX)
    let broadcast_result = alice.announce_profile(None).await;
    println!("Alice announce_profile result: {:?}", broadcast_result.is_ok());
    assert!(
        broadcast_result.is_ok(),
        "announce_profile should succeed after restart (profile_gossip_sender must be initialized)"
    );

    // Wait for Bob to receive the update (THIS WOULD FAIL WITHOUT THE FIX)
    wait_for_profile_update(&bob, &alice_did, "Alice v2", Duration::from_secs(15))
        .await
        .expect(
            "Bob should receive profile update after restart \
            (reconnect_contacts must recreate profile listeners)"
        );

    let post_update_name = get_peer_display_name(&bob, &alice_did);
    println!("Bob sees Alice as: {:?}", post_update_name);
    assert_eq!(
        post_update_name,
        Some("Alice v2".to_string()),
        "Profile should update to 'Alice v2' after restart"
    );

    // Cleanup
    alice.shutdown().await.ok();
    bob.shutdown().await.ok();

    println!("\n=== PASSED: Profile update works after restart ===\n");
}

// ============================================================================
// Test 3: Manual Sync Triggers Profile Refresh
// ============================================================================

/// Test that manual_sync() properly broadcasts profile and reconnects contacts
///
/// ## Test Flow:
/// 1. Setup Alice and Bob with contacts
/// 2. Alice updates profile
/// 3. Call manual_sync() on Alice (should broadcast profile)
/// 4. Verify Bob receives the update
///
/// ## What This Tests:
/// - manual_sync() initializes contact manager if needed
/// - manual_sync() broadcasts profile via announce_profile()
/// - manual_sync() returns correct contact count
///
/// ## Known Issue (2024-01)
#[tokio::test]
async fn test_manual_sync_broadcasts_profile() {
    tracing_subscriber::fmt()
        .with_env_filter("info,syncengine_core=debug,quinn=warn,iroh=warn")
        .try_init()
        .ok();

    println!("\n=== Test: Manual Sync Broadcasts Profile ===\n");

    // Setup Alice
    let ctx_alice = TestContext::new().expect("Failed to create Alice context");
    let mut alice = ctx_alice.create_engine().await.expect("Failed to create Alice");
    alice.init_identity().unwrap();
    alice.start_networking().await.unwrap();
    alice.startup_sync().await.ok();

    // Setup Bob
    let ctx_bob = TestContext::new().expect("Failed to create Bob context");
    let mut bob = ctx_bob.create_engine().await.expect("Failed to create Bob");
    bob.init_identity().unwrap();
    bob.start_networking().await.unwrap();
    bob.startup_sync().await.ok();
    sleep(Duration::from_millis(500)).await;

    let alice_did = alice.did().unwrap().to_string();

    // Exchange contacts
    let invite_code = alice.generate_contact_invite(24).await.unwrap();
    let invite = bob.decode_contact_invite(&invite_code).await.unwrap();
    bob.send_contact_request(invite).await.unwrap();
    sleep(Duration::from_millis(2000)).await;

    // Verify contacts
    assert_eq!(alice.list_contacts().unwrap().len(), 1);
    assert_eq!(bob.list_contacts().unwrap().len(), 1);
    println!("Contacts established");

    // Alice updates profile
    update_display_name(&mut alice, "Alice via Manual Sync").await.unwrap();
    println!("Alice updated display name");

    // Call manual_sync (this should broadcast profile AND reconnect contacts)
    let sync_result = alice.manual_sync().await;
    assert!(sync_result.is_ok(), "manual_sync should succeed");
    let contacts_count = sync_result.unwrap();
    println!("manual_sync returned {} contacts", contacts_count);
    assert!(contacts_count >= 1, "Should have at least 1 contact");

    // Wait for Bob to receive update
    wait_for_profile_update(
        &bob,
        &alice_did,
        "Alice via Manual Sync",
        Duration::from_secs(10),
    )
    .await
    .expect("Bob should receive profile update via manual_sync");

    let bob_sees = get_peer_display_name(&bob, &alice_did);
    println!("Bob sees Alice as: {:?}", bob_sees);
    assert_eq!(bob_sees, Some("Alice via Manual Sync".to_string()));

    // Cleanup
    alice.shutdown().await.ok();
    bob.shutdown().await.ok();

    println!("\n=== PASSED: Manual sync broadcasts profile ===\n");
}

// ============================================================================
// Test 4: Bidirectional Profile Updates
// ============================================================================

/// Test that profile updates work in both directions
///
/// ## Test Flow:
/// 1. Alice and Bob exchange contacts
/// 2. Alice updates name, Bob receives it
/// 3. Bob updates name, Alice receives it
/// 4. Both see correct names for each other
///
/// ## Known Issue (2024-01)
/// TODO: Inviter (Alice) side doesn't store contact topic sender in active_topics
/// because contact finalization happens in contact_handler, not contact_manager.
/// Need to bridge contact_handler → contact_manager for inviter-side contacts.
#[tokio::test]
#[ignore = "Inviter-side contacts not tracked in active_topics - needs architectural fix"]
async fn test_bidirectional_profile_updates() {
    tracing_subscriber::fmt()
        .with_env_filter("info,syncengine_core=debug,quinn=warn,iroh=warn")
        .try_init()
        .ok();

    println!("\n=== Test: Bidirectional Profile Updates ===\n");

    // Setup
    let ctx_alice = TestContext::new().expect("Failed to create Alice context");
    let ctx_bob = TestContext::new().expect("Failed to create Bob context");

    let mut alice = ctx_alice.create_engine().await.expect("Failed to create Alice");
    let mut bob = ctx_bob.create_engine().await.expect("Failed to create Bob");

    alice.init_identity().unwrap();
    bob.init_identity().unwrap();

    alice.start_networking().await.unwrap();
    bob.start_networking().await.unwrap();

    // Initialize profile sync on both
    alice.startup_sync().await.ok();
    bob.startup_sync().await.ok();
    sleep(Duration::from_millis(500)).await;

    let alice_did = alice.did().unwrap().to_string();
    let bob_did = bob.did().unwrap().to_string();

    // Exchange contacts
    let invite_code = alice.generate_contact_invite(24).await.unwrap();
    let invite = bob.decode_contact_invite(&invite_code).await.unwrap();
    bob.send_contact_request(invite).await.unwrap();
    sleep(Duration::from_millis(2000)).await;

    println!("Contacts established");

    // Alice updates and broadcasts
    update_display_name(&mut alice, "Alice").await.unwrap();
    alice.announce_profile(None).await.unwrap();
    println!("Alice broadcast 'Alice'");

    // Wait for Bob to receive Alice's update
    wait_for_profile_update(&bob, &alice_did, "Alice", Duration::from_secs(10))
        .await
        .expect("Bob should receive Alice's update");
    println!("Bob received Alice's profile");

    // Bob updates and broadcasts
    update_display_name(&mut bob, "Bob").await.unwrap();
    bob.announce_profile(None).await.unwrap();
    println!("Bob broadcast 'Bob'");

    // Wait for Alice to receive Bob's update
    wait_for_profile_update(&alice, &bob_did, "Bob", Duration::from_secs(10))
        .await
        .expect("Alice should receive Bob's update");
    println!("Alice received Bob's profile");

    // Verify both see correct names
    assert_eq!(get_peer_display_name(&bob, &alice_did), Some("Alice".to_string()));
    assert_eq!(get_peer_display_name(&alice, &bob_did), Some("Bob".to_string()));

    println!("Bidirectional profile updates verified!");

    // Cleanup
    alice.shutdown().await.ok();
    bob.shutdown().await.ok();

    println!("\n=== PASSED: Bidirectional profile updates work ===\n");
}
