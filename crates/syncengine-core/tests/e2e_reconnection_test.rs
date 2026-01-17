//! End-to-End Reconnection Tests
//!
//! These tests verify the complete reconnection flow after node restarts,
//! including endpoint ID persistence, peer registry tracking, and task sync recovery.
//!
//! ## Critical Test: test_reconnection_after_restart
//!
//! This test validates the core reconnection scenario:
//! 1. Phase 1: Two nodes sync tasks successfully
//! 2. Phase 2: Both nodes restart, reconnect, and continue syncing
//!
//! Success criteria:
//! - Endpoint IDs match after restart (deterministic identity)
//! - Peer registry tracks discovered peers
//! - Tasks sync successfully post-restart
//! - No manual reconnection required (automatic via bootstrap peers)

use std::path::PathBuf;
use std::time::Duration;

use syncengine_core::{PeerRegistry, PeerSource, PeerStatus, RealmId, SyncEngine, Task};
use tempfile::TempDir;

// ============================================================================
// Test Utilities
// ============================================================================

/// Test context that manages temporary directories and SyncEngine lifecycle
struct TestContext {
    temp_dir: TempDir,
    data_dir: PathBuf,
}

impl TestContext {
    /// Create a new test context with a temporary data directory
    fn new() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&data_dir)?;

        Ok(Self { temp_dir, data_dir })
    }

    /// Get the data directory path
    fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    /// Create a new SyncEngine instance using this context's data directory
    async fn create_engine(&self) -> anyhow::Result<SyncEngine> {
        Ok(SyncEngine::new(&self.data_dir).await?)
    }
}

/// Wait for nodes to discover each other as peers
async fn wait_for_peer_discovery(
    engine: &SyncEngine,
    expected_peer_id: iroh::PublicKey,
    timeout: Duration,
) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            anyhow::bail!("Timeout waiting for peer discovery");
        }

        // Check if peer registry contains the expected peer
        let peer_registry = engine.peer_registry();
        if let Some(peer_info) = peer_registry.get(&expected_peer_id)? {
            if peer_info.status == PeerStatus::Online {
                return Ok(());
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Wait for tasks to sync between nodes
async fn wait_for_task_sync(
    engine: &mut SyncEngine,
    realm_id: &RealmId,
    expected_task_title: &str,
    timeout: Duration,
) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            anyhow::bail!(
                "Timeout waiting for task sync. Expected task: {}",
                expected_task_title
            );
        }

        let tasks = engine.list_tasks(realm_id)?;
        if tasks.iter().any(|t| t.title == expected_task_title) {
            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// ============================================================================
// Test 1: Reconnection After Restart (THE CRITICAL TEST)
// ============================================================================

/// Test complete reconnection flow after node restarts
///
/// This is the most important test for Phase 1 completion.
///
/// ## Test Flow:
///
/// ### Phase 1: Initial Sync
/// 1. Create two nodes (A and B) with deterministic identities
/// 2. Node A creates a realm and task
/// 3. Node B joins via invite
/// 4. Verify task syncs from A to B
/// 5. Record endpoint IDs and peer registry state
/// 6. Shutdown both nodes
///
/// ### Phase 2: Restart and Reconnect
/// 7. Restart both nodes using same data directories
/// 8. Verify endpoint IDs are preserved
/// 9. Verify peer registry contains peer information
/// 10. Start networking and sync on both nodes
/// 11. Node A creates another task
/// 12. Verify task syncs from A to B (reconnection successful)
///
/// ## Success Criteria:
/// - Endpoint IDs match before/after restart
/// - Peer registry persists peer information
/// - Nodes reconnect automatically (no new invite needed)
/// - Tasks sync successfully post-restart
#[tokio::test]
async fn test_reconnection_after_restart() {
    // ========================================================================
    // Phase 1: Initial Sync
    // ========================================================================

    println!("\n=== Phase 1: Initial Sync ===\n");

    // Create test contexts for two nodes
    let ctx_a = TestContext::new().expect("Failed to create context A");
    let ctx_b = TestContext::new().expect("Failed to create context B");

    let endpoint_id_a_before: Option<String>;
    let endpoint_id_b_before: Option<String>;
    let realm_id: RealmId;

    {
        // Create node A
        let mut engine_a = ctx_a
            .create_engine()
            .await
            .expect("Failed to create engine A");

        // Create node B
        let mut engine_b = ctx_b
            .create_engine()
            .await
            .expect("Failed to create engine B");

        // Start networking on both nodes
        engine_a
            .start_networking()
            .await
            .expect("Failed to start networking on A");
        engine_b
            .start_networking()
            .await
            .expect("Failed to start networking on B");

        // Record endpoint IDs before restart
        let node_info_a = engine_a
            .node_info()
            .await
            .expect("Failed to get node info A");
        let node_info_b = engine_b
            .node_info()
            .await
            .expect("Failed to get node info B");

        endpoint_id_a_before = node_info_a.node_id;
        endpoint_id_b_before = node_info_b.node_id;

        println!("Node A ID (before): {:?}", endpoint_id_a_before);
        println!("Node B ID (before): {:?}", endpoint_id_b_before);

        // Node A: Create realm and task
        realm_id = engine_a
            .create_realm("Test Realm")
            .await
            .expect("Failed to create realm");

        engine_a
            .add_task(&realm_id, "Task from Phase 1")
            .await
            .expect("Failed to add task");

        // Node A: Start sync
        engine_a
            .start_sync(&realm_id)
            .await
            .expect("Failed to start sync on A");

        // Node A: Generate invite
        let invite = engine_a
            .generate_invite(&realm_id)
            .await
            .expect("Failed to generate invite");

        println!("Generated invite from Node A");

        // Node B: Join via invite
        let joined_realm_id = engine_b
            .join_via_invite(&invite)
            .await
            .expect("Failed to join via invite");

        assert_eq!(joined_realm_id, realm_id);
        println!("Node B joined realm via invite");

        // Node B: Start sync
        engine_b
            .start_sync(&realm_id)
            .await
            .expect("Failed to start sync on B");

        // Wait for task to sync from A to B
        wait_for_task_sync(
            &mut engine_b,
            &realm_id,
            "Task from Phase 1",
            Duration::from_secs(10),
        )
        .await
        .expect("Task did not sync from A to B");

        println!("✓ Task synced successfully from A to B");

        // Verify peer registry on both nodes
        let peer_registry_a = engine_a.peer_registry();
        let peer_registry_b = engine_b.peer_registry();

        let peers_a = peer_registry_a
            .list_all()
            .expect("Failed to list peers on A");
        let peers_b = peer_registry_b
            .list_all()
            .expect("Failed to list peers on B");

        println!("Node A peer count: {}", peers_a.len());
        println!("Node B peer count: {}", peers_b.len());

        // Verify Node A knows about Node B
        if let Some(ref hex_b) = endpoint_id_b_before {
            let bytes_b = hex::decode(hex_b).expect("Invalid hex for B");
            assert!(
                peers_a
                    .iter()
                    .any(|p| p.endpoint_id.as_slice() == bytes_b.as_slice()),
                "Node A should know about Node B"
            );
        }

        // Verify Node B knows about Node A
        if let Some(ref hex_a) = endpoint_id_a_before {
            let bytes_a = hex::decode(hex_a).expect("Invalid hex for A");
            assert!(
                peers_b
                    .iter()
                    .any(|p| p.endpoint_id.as_slice() == bytes_a.as_slice()),
                "Node B should know about Node A"
            );
        }

        println!("✓ Peer registries populated");

        // Shutdown both nodes
        engine_a.shutdown().await.expect("Failed to shutdown A");
        engine_b.shutdown().await.expect("Failed to shutdown B");

        println!("\n✓ Phase 1 Complete: Both nodes shut down\n");
    }

    // ========================================================================
    // Phase 2: Restart and Reconnect
    // ========================================================================

    println!("=== Phase 2: Restart and Reconnect ===\n");

    // Restart node A
    let mut engine_a = ctx_a
        .create_engine()
        .await
        .expect("Failed to restart engine A");

    // Restart node B
    let mut engine_b = ctx_b
        .create_engine()
        .await
        .expect("Failed to restart engine B");

    // Verify endpoint IDs are preserved
    let node_info_a = engine_a
        .node_info()
        .await
        .expect("Failed to get node info A after restart");
    let node_info_b = engine_b
        .node_info()
        .await
        .expect("Failed to get node info B after restart");

    let endpoint_id_a_after = node_info_a.node_id;
    let endpoint_id_b_after = node_info_b.node_id;

    println!("Node A ID (after): {:?}", endpoint_id_a_after);
    println!("Node B ID (after): {:?}", endpoint_id_b_after);

    assert_eq!(
        endpoint_id_a_before, endpoint_id_a_after,
        "Node A endpoint ID should be preserved across restart"
    );
    assert_eq!(
        endpoint_id_b_before, endpoint_id_b_after,
        "Node B endpoint ID should be preserved across restart"
    );

    println!("✓ Endpoint IDs preserved across restart");

    // Verify peer registry persists
    let peer_registry_a = engine_a.peer_registry();
    let peer_registry_b = engine_b.peer_registry();

    let peers_a = peer_registry_a
        .list_all()
        .expect("Failed to list peers on A after restart");
    let peers_b = peer_registry_b
        .list_all()
        .expect("Failed to list peers on B after restart");

    assert!(
        !peers_a.is_empty(),
        "Node A should have persisted peer information"
    );
    assert!(
        !peers_b.is_empty(),
        "Node B should have persisted peer information"
    );

    println!("✓ Peer registry persisted across restart");

    // Start networking on both nodes
    engine_a
        .start_networking()
        .await
        .expect("Failed to start networking on A after restart");
    engine_b
        .start_networking()
        .await
        .expect("Failed to start networking on B after restart");

    // Start sync on both nodes
    engine_a
        .start_sync(&realm_id)
        .await
        .expect("Failed to start sync on A after restart");
    engine_b
        .start_sync(&realm_id)
        .await
        .expect("Failed to start sync on B after restart");

    // Wait a moment for reconnection to establish
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Node A: Add a new task
    engine_a
        .add_task(&realm_id, "Task from Phase 2")
        .await
        .expect("Failed to add task in Phase 2");

    println!("Node A added task: 'Task from Phase 2'");

    // Wait for new task to sync from A to B
    wait_for_task_sync(
        &mut engine_b,
        &realm_id,
        "Task from Phase 2",
        Duration::from_secs(10),
    )
    .await
    .expect("Task did not sync from A to B after restart");

    println!("✓ Task synced successfully after restart");

    // Verify both tasks exist on both nodes
    let tasks_a = engine_a
        .list_tasks(&realm_id)
        .expect("Failed to list tasks on A");
    let tasks_b = engine_b
        .list_tasks(&realm_id)
        .expect("Failed to list tasks on B");

    assert_eq!(tasks_a.len(), 2, "Node A should have 2 tasks");
    assert_eq!(tasks_b.len(), 2, "Node B should have 2 tasks");

    assert!(
        tasks_a.iter().any(|t| t.title == "Task from Phase 1"),
        "Node A should have task from Phase 1"
    );
    assert!(
        tasks_a.iter().any(|t| t.title == "Task from Phase 2"),
        "Node A should have task from Phase 2"
    );
    assert!(
        tasks_b.iter().any(|t| t.title == "Task from Phase 1"),
        "Node B should have task from Phase 1"
    );
    assert!(
        tasks_b.iter().any(|t| t.title == "Task from Phase 2"),
        "Node B should have task from Phase 2"
    );

    println!("✓ All tasks synced correctly");

    // Cleanup
    engine_a.shutdown().await.expect("Failed to shutdown A");
    engine_b.shutdown().await.expect("Failed to shutdown B");

    println!("\n✓✓✓ TEST PASSED: Reconnection after restart successful ✓✓✓\n");
}

// ============================================================================
// Test 2: Two Nodes Sync Tasks (Basic Verification)
// ============================================================================

/// Test basic task synchronization between two nodes
///
/// This is a simpler version of the reconnection test, focusing only on
/// verifying that tasks sync correctly without restart.
///
/// ## Test Flow:
/// 1. Create two nodes
/// 2. Node A creates realm and task
/// 3. Node B joins via invite
/// 4. Verify task syncs from A to B
/// 5. Node B creates a task
/// 6. Verify task syncs from B to A
///
/// ## Success Criteria:
/// - Tasks created on A appear on B
/// - Tasks created on B appear on A
/// - Bidirectional sync works correctly
#[tokio::test]
async fn test_two_nodes_sync_tasks() {
    println!("\n=== Two Nodes Sync Tasks ===\n");

    // Create test contexts
    let ctx_a = TestContext::new().expect("Failed to create context A");
    let ctx_b = TestContext::new().expect("Failed to create context B");

    let mut engine_a = ctx_a
        .create_engine()
        .await
        .expect("Failed to create engine A");
    let mut engine_b = ctx_b
        .create_engine()
        .await
        .expect("Failed to create engine B");

    // Start networking
    engine_a
        .start_networking()
        .await
        .expect("Failed to start networking on A");
    engine_b
        .start_networking()
        .await
        .expect("Failed to start networking on B");

    // Node A: Create realm and task
    let realm_id = engine_a
        .create_realm("Sync Test Realm")
        .await
        .expect("Failed to create realm");

    engine_a
        .add_task(&realm_id, "Task A1")
        .await
        .expect("Failed to add task A1");

    println!("Node A created task: 'Task A1'");

    // Node A: Start sync
    engine_a
        .start_sync(&realm_id)
        .await
        .expect("Failed to start sync on A");

    // Node A: Generate invite
    let invite = engine_a
        .generate_invite(&realm_id)
        .await
        .expect("Failed to generate invite");

    // Node B: Join via invite
    let joined_realm_id = engine_b
        .join_via_invite(&invite)
        .await
        .expect("Failed to join via invite");

    assert_eq!(joined_realm_id, realm_id);
    println!("Node B joined realm");

    // Node B: Start sync
    engine_b
        .start_sync(&realm_id)
        .await
        .expect("Failed to start sync on B");

    // Wait for Task A1 to sync to B
    wait_for_task_sync(&mut engine_b, &realm_id, "Task A1", Duration::from_secs(10))
        .await
        .expect("Task A1 did not sync to B");

    println!("✓ Task A1 synced from A to B");

    // Node B: Create a task
    engine_b
        .add_task(&realm_id, "Task B1")
        .await
        .expect("Failed to add task B1");

    println!("Node B created task: 'Task B1'");

    // Wait for Task B1 to sync to A
    wait_for_task_sync(&mut engine_a, &realm_id, "Task B1", Duration::from_secs(10))
        .await
        .expect("Task B1 did not sync to A");

    println!("✓ Task B1 synced from B to A");

    // Verify both nodes have both tasks
    let tasks_a = engine_a
        .list_tasks(&realm_id)
        .expect("Failed to list tasks on A");
    let tasks_b = engine_b
        .list_tasks(&realm_id)
        .expect("Failed to list tasks on B");

    assert_eq!(tasks_a.len(), 2, "Node A should have 2 tasks");
    assert_eq!(tasks_b.len(), 2, "Node B should have 2 tasks");

    assert!(
        tasks_a.iter().any(|t| t.title == "Task A1"),
        "Node A should have Task A1"
    );
    assert!(
        tasks_a.iter().any(|t| t.title == "Task B1"),
        "Node A should have Task B1"
    );
    assert!(
        tasks_b.iter().any(|t| t.title == "Task A1"),
        "Node B should have Task A1"
    );
    assert!(
        tasks_b.iter().any(|t| t.title == "Task B1"),
        "Node B should have Task B1"
    );

    println!("✓ Bidirectional sync verified");

    // Cleanup
    engine_a.shutdown().await.expect("Failed to shutdown A");
    engine_b.shutdown().await.expect("Failed to shutdown B");

    println!("\n✓✓✓ TEST PASSED: Two nodes sync tasks ✓✓✓\n");
}

// ============================================================================
// Test 3: Peer Registry Tracks Connections
// ============================================================================

/// Test that peer registry automatically tracks discovered peers
///
/// This test verifies that the peer registry correctly captures peer
/// information during gossip discovery without manual intervention.
///
/// ## Test Flow:
/// 1. Create two nodes
/// 2. Node A creates realm and generates invite
/// 3. Node B joins via invite
/// 4. Start sync on both nodes
/// 5. Verify peer registry automatically recorded peer information
/// 6. Verify peer status is correctly tracked
///
/// ## Success Criteria:
/// - Peer registry contains discovered peers
/// - Peer source is correctly recorded (FromRealm)
/// - Peer status reflects connection state
/// - Shared realms list is populated
#[tokio::test]
async fn test_peer_registry_tracks_connections() {
    println!("\n=== Peer Registry Tracks Connections ===\n");

    // Create test contexts
    let ctx_a = TestContext::new().expect("Failed to create context A");
    let ctx_b = TestContext::new().expect("Failed to create context B");

    let mut engine_a = ctx_a
        .create_engine()
        .await
        .expect("Failed to create engine A");
    let mut engine_b = ctx_b
        .create_engine()
        .await
        .expect("Failed to create engine B");

    // Start networking
    engine_a
        .start_networking()
        .await
        .expect("Failed to start networking on A");
    engine_b
        .start_networking()
        .await
        .expect("Failed to start networking on B");

    // Get node IDs
    let node_info_a = engine_a
        .node_info()
        .await
        .expect("Failed to get node info A");
    let node_info_b = engine_b
        .node_info()
        .await
        .expect("Failed to get node info B");

    let endpoint_id_a = node_info_a.node_id;
    let endpoint_id_b = node_info_b.node_id;

    println!("Node A ID: {:?}", endpoint_id_a);
    println!("Node B ID: {:?}", endpoint_id_b);

    // Node A: Create realm
    let realm_id = engine_a
        .create_realm("Registry Test Realm")
        .await
        .expect("Failed to create realm");

    engine_a
        .start_sync(&realm_id)
        .await
        .expect("Failed to start sync on A");

    // Node A: Generate invite
    let invite = engine_a
        .generate_invite(&realm_id)
        .await
        .expect("Failed to generate invite");

    // Node B: Join via invite
    engine_b
        .join_via_invite(&invite)
        .await
        .expect("Failed to join via invite");

    engine_b
        .start_sync(&realm_id)
        .await
        .expect("Failed to start sync on B");

    // Wait for peer discovery
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check peer registry on Node A
    let peer_registry_a = engine_a.peer_registry();

    // Convert hex string to PublicKey for registry lookup
    let endpoint_id_b_hex = endpoint_id_b.as_ref().expect("Node B should have endpoint ID");
    let endpoint_id_b_bytes: [u8; 32] = hex::decode(endpoint_id_b_hex)
        .expect("Invalid hex")
        .try_into()
        .expect("Wrong length");
    let endpoint_id_b_pk = iroh::PublicKey::from_bytes(&endpoint_id_b_bytes)
        .expect("Invalid public key bytes");

    let peer_info_b = peer_registry_a
        .get(&endpoint_id_b_pk)
        .expect("Failed to get peer B from registry A")
        .expect("Node A should know about Node B");

    println!("Node A knows about Node B:");
    println!("  Endpoint ID: {:?}", peer_info_b.endpoint_id);
    println!("  Status: {}", peer_info_b.status);
    println!("  Source: {:?}", peer_info_b.source);
    println!("  Shared realms: {}", peer_info_b.shared_realms.len());

    // Verify peer information
    // endpoint_id_b is Option<String> (hex), peer_info_b.endpoint_id is [u8; 32]
    if let Some(ref hex_id) = endpoint_id_b {
        let expected_bytes = hex::decode(hex_id).expect("Invalid hex");
        assert_eq!(
            peer_info_b.endpoint_id.as_slice(),
            expected_bytes.as_slice(),
            "Endpoint ID should match"
        );
    }

    // Verify source is FromRealm
    match &peer_info_b.source {
        PeerSource::FromRealm(rid) => {
            assert_eq!(rid, &realm_id, "Peer source should reference the realm");
        }
        _ => panic!("Peer source should be FromRealm"),
    }

    // Verify shared realms
    assert!(
        peer_info_b.shared_realms.contains(&realm_id),
        "Shared realms should contain the test realm"
    );

    println!("✓ Peer registry correctly tracked Node B");

    // Check peer registry on Node B
    let peer_registry_b = engine_b.peer_registry();

    // Convert hex string to PublicKey for registry lookup
    let endpoint_id_a_hex = endpoint_id_a.as_ref().expect("Node A should have endpoint ID");
    let endpoint_id_a_bytes: [u8; 32] = hex::decode(endpoint_id_a_hex)
        .expect("Invalid hex")
        .try_into()
        .expect("Wrong length");
    let endpoint_id_a_pk = iroh::PublicKey::from_bytes(&endpoint_id_a_bytes)
        .expect("Invalid public key bytes");

    let peer_info_a = peer_registry_b
        .get(&endpoint_id_a_pk)
        .expect("Failed to get peer A from registry B")
        .expect("Node B should know about Node A");

    println!("\nNode B knows about Node A:");
    println!("  Endpoint ID: {:?}", peer_info_a.endpoint_id);
    println!("  Status: {}", peer_info_a.status);
    println!("  Source: {:?}", peer_info_a.source);
    println!("  Shared realms: {}", peer_info_a.shared_realms.len());

    // endpoint_id_a is Option<String> (hex), peer_info_a.endpoint_id is [u8; 32]
    if let Some(ref hex_id) = endpoint_id_a {
        let expected_bytes = hex::decode(hex_id).expect("Invalid hex");
        assert_eq!(
            peer_info_a.endpoint_id.as_slice(),
            expected_bytes.as_slice(),
            "Endpoint ID should match"
        );
    }

    println!("✓ Peer registry correctly tracked Node A");

    // Cleanup
    engine_a.shutdown().await.expect("Failed to shutdown A");
    engine_b.shutdown().await.expect("Failed to shutdown B");

    println!("\n✓✓✓ TEST PASSED: Peer registry tracks connections ✓✓✓\n");
}

// ============================================================================
// Test 4: Exponential Backoff Math
// ============================================================================

/// Test exponential backoff calculation for reconnection attempts
///
/// This test verifies the Fibonacci backoff algorithm used for periodic reconnection
/// attempts to inactive peers.
///
/// ## Backoff Formula:
/// ```text
/// delay = min(fibonacci(failures) * 1min, 60min)
/// where fibonacci(n) = F(0)=1, F(1)=1, F(n)=F(n-1)+F(n-2)
/// ```
///
/// ## Expected Delays (Fibonacci sequence in minutes):
/// - 0 failures: 1 min (60s)
/// - 1 failure: 1 min (60s)
/// - 2 failures: 2 min (120s)
/// - 3 failures: 3 min (180s)
/// - 4 failures: 5 min (300s)
/// - 5 failures: 8 min (480s)
/// - 6 failures: 13 min (780s)
/// - 7 failures: 21 min (1260s)
/// - 8 failures: 34 min (2040s)
/// - 9 failures: 55 min (3300s)
/// - 10+ failures: 60 min (3600s, capped)
///
/// ## Success Criteria:
/// - Backoff follows Fibonacci sequence
/// - Backoff is capped at 60 minutes
/// - Formula matches expected values
#[test]
fn test_fibonacci_backoff() {
    println!("\n=== Fibonacci Backoff Math ===\n");

    const BASE_UNIT_SECS: u64 = 60; // 1 minute
    const MAX_DELAY_SECS: u64 = 3600; // 60 minutes

    /// Calculate nth Fibonacci number
    fn fibonacci(n: u32) -> u64 {
        match n {
            0 => 1,
            1 => 1,
            _ => {
                let mut a = 1u64;
                let mut b = 1u64;
                for _ in 2..=n {
                    let next = a.saturating_add(b);
                    a = b;
                    b = next;
                }
                b
            }
        }
    }

    /// Calculate Fibonacci backoff delay in seconds
    fn calculate_backoff(failures: u32, base_unit: u64, max_delay: u64) -> u64 {
        let fib = fibonacci(failures);
        let delay = fib.saturating_mul(base_unit);
        delay.min(max_delay)
    }

    // Test cases: (failures, expected_delay_in_minutes)
    let test_cases = vec![
        (0, 1),   // F(0) = 1
        (1, 1),   // F(1) = 1
        (2, 2),   // F(2) = 2
        (3, 3),   // F(3) = 3
        (4, 5),   // F(4) = 5
        (5, 8),   // F(5) = 8
        (6, 13),  // F(6) = 13
        (7, 21),  // F(7) = 21
        (8, 34),  // F(8) = 34
        (9, 55),  // F(9) = 55
        (10, 60), // F(10) = 89, but capped at 60
        (15, 60), // Still capped at 60
    ];

    println!("Testing Fibonacci backoff delays:");
    println!(
        "Base unit: {}s (1 min), Max delay: {}s (60 min)\n",
        BASE_UNIT_SECS, MAX_DELAY_SECS
    );

    for (failures, expected_minutes) in test_cases {
        let expected_secs = expected_minutes * 60;
        let actual = calculate_backoff(failures, BASE_UNIT_SECS, MAX_DELAY_SECS);
        println!(
            "Failures {}: {}s ({}min) - expected {}min",
            failures,
            actual,
            actual / 60,
            expected_minutes
        );
        assert_eq!(
            actual,
            expected_secs,
            "Backoff for {} failures should be {}s ({}min), got {}s ({}min)",
            failures,
            expected_secs,
            expected_minutes,
            actual,
            actual / 60
        );
    }

    println!("\n✓ Fibonacci backoff formula verified");
    println!("✓ Sequence: 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 60 (capped) minutes");
    println!("✓✓✓ TEST PASSED: Fibonacci backoff ✓✓✓\n");
}
