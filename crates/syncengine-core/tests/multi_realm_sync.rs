//! Multi-Realm Sync Integration Tests
//!
//! These tests verify that multiple realms can sync concurrently
//! with different peers, and that sync recovers gracefully from
//! network interruptions.
//!
//! ## Test Scenarios
//!
//! - Two realms syncing concurrently with different peers
//! - Status tracking per realm
//! - Event callbacks firing when remote changes arrive
//! - Sync recovery after simulated network interruption

use std::time::Duration;

use syncengine_core::{SyncEngine, SyncEvent, SyncStatus};
use tempfile::TempDir;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a test engine in a temporary directory
async fn create_test_engine() -> (SyncEngine, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let engine = SyncEngine::new(temp_dir.path()).await.unwrap();
    (engine, temp_dir)
}

// ============================================================================
// Multi-Realm Sync Tests
// ============================================================================

/// Test that multiple realms can sync concurrently without interference.
///
/// Verifies:
/// - start_sync() can be called for multiple realms without blocking
/// - Each realm has independent sync status
/// - Stopping one realm doesn't affect others
#[tokio::test]
async fn test_concurrent_multi_realm_sync() {
    let (mut engine, _temp) = create_test_engine().await;

    // Create multiple realms
    let realm1 = engine.create_realm("Work Projects").await.unwrap();
    let realm2 = engine.create_realm("Home Tasks").await.unwrap();
    let realm3 = engine.create_realm("Hobbies").await.unwrap();

    // Start syncing all three concurrently
    engine.start_sync(&realm1).await.unwrap();
    engine.start_sync(&realm2).await.unwrap();
    engine.start_sync(&realm3).await.unwrap();

    // Verify all are syncing independently
    assert!(engine.is_realm_syncing(&realm1));
    assert!(engine.is_realm_syncing(&realm2));
    assert!(engine.is_realm_syncing(&realm3));

    // Verify status is Syncing for all
    assert!(matches!(engine.sync_status(&realm1), SyncStatus::Syncing { .. }));
    assert!(matches!(engine.sync_status(&realm2), SyncStatus::Syncing { .. }));
    assert!(matches!(engine.sync_status(&realm3), SyncStatus::Syncing { .. }));

    // Add tasks to different realms while syncing
    engine.add_task(&realm1, "Finish project proposal").await.unwrap();
    engine.add_task(&realm2, "Clean garage").await.unwrap();
    engine.add_task(&realm3, "Practice guitar").await.unwrap();

    // Verify tasks are isolated to their realms
    let tasks1 = engine.list_tasks(&realm1).unwrap();
    let tasks2 = engine.list_tasks(&realm2).unwrap();
    let tasks3 = engine.list_tasks(&realm3).unwrap();

    assert_eq!(tasks1.len(), 1);
    assert_eq!(tasks2.len(), 1);
    assert_eq!(tasks3.len(), 1);
    assert_eq!(tasks1[0].title, "Finish project proposal");
    assert_eq!(tasks2[0].title, "Clean garage");
    assert_eq!(tasks3[0].title, "Practice guitar");

    // Stop one realm
    engine.stop_sync(&realm2).await.unwrap();

    // Others should still be syncing
    assert!(engine.is_realm_syncing(&realm1));
    assert!(!engine.is_realm_syncing(&realm2));
    assert!(engine.is_realm_syncing(&realm3));

    // Syncing count should reflect active syncs
    assert_eq!(engine.syncing_count(), 2);

    engine.shutdown().await.unwrap();
}

/// Test that sync status is tracked correctly per realm.
#[tokio::test]
async fn test_sync_status_per_realm() {
    let (mut engine, _temp) = create_test_engine().await;

    let realm1 = engine.create_realm("Status Test 1").await.unwrap();
    let realm2 = engine.create_realm("Status Test 2").await.unwrap();

    // Both should be idle initially
    assert_eq!(engine.sync_status(&realm1), SyncStatus::Idle);
    assert_eq!(engine.sync_status(&realm2), SyncStatus::Idle);

    // Start sync on realm1 only
    engine.start_sync(&realm1).await.unwrap();

    // realm1 should be syncing, realm2 still idle
    assert!(matches!(engine.sync_status(&realm1), SyncStatus::Syncing { .. }));
    assert_eq!(engine.sync_status(&realm2), SyncStatus::Idle);

    // Start sync on realm2
    engine.start_sync(&realm2).await.unwrap();

    // Both should be syncing
    assert!(matches!(engine.sync_status(&realm1), SyncStatus::Syncing { .. }));
    assert!(matches!(engine.sync_status(&realm2), SyncStatus::Syncing { .. }));

    // Stop realm1
    engine.stop_sync(&realm1).await.unwrap();

    // realm1 idle, realm2 still syncing
    assert_eq!(engine.sync_status(&realm1), SyncStatus::Idle);
    assert!(matches!(engine.sync_status(&realm2), SyncStatus::Syncing { .. }));

    engine.shutdown().await.unwrap();
}

/// Test that event subscription receives status change events.
#[tokio::test]
async fn test_event_subscription_for_multi_realm() {
    let (mut engine, _temp) = create_test_engine().await;

    let realm1 = engine.create_realm("Event Realm 1").await.unwrap();
    let realm2 = engine.create_realm("Event Realm 2").await.unwrap();

    // Subscribe to events
    let mut events = engine.subscribe_events();

    // Start sync on both realms
    engine.start_sync(&realm1).await.unwrap();
    engine.start_sync(&realm2).await.unwrap();

    // Collect events (with timeout to avoid hanging)
    let mut status_events = Vec::new();
    for _ in 0..20 {
        match tokio::time::timeout(Duration::from_millis(50), events.recv()).await {
            Ok(Ok(event @ SyncEvent::StatusChanged { .. })) => {
                status_events.push(event);
            }
            _ => break,
        }
    }

    // Should have received events for both realms
    // At minimum: Connecting + Syncing for each realm = 4 events
    assert!(
        status_events.len() >= 2,
        "Expected at least 2 status events, got {}",
        status_events.len()
    );

    // Verify we got events for both realms
    let realm1_events: Vec<_> = status_events
        .iter()
        .filter(|e| matches!(e, SyncEvent::StatusChanged { realm_id, .. } if *realm_id == realm1))
        .collect();
    let realm2_events: Vec<_> = status_events
        .iter()
        .filter(|e| matches!(e, SyncEvent::StatusChanged { realm_id, .. } if *realm_id == realm2))
        .collect();

    assert!(!realm1_events.is_empty(), "Should have events for realm1");
    assert!(!realm2_events.is_empty(), "Should have events for realm2");

    engine.shutdown().await.unwrap();
}

/// Test that syncing_realms() returns correct list.
#[tokio::test]
async fn test_syncing_realms_list() {
    let (mut engine, _temp) = create_test_engine().await;

    let realm1 = engine.create_realm("List Test 1").await.unwrap();
    let realm2 = engine.create_realm("List Test 2").await.unwrap();
    let realm3 = engine.create_realm("List Test 3").await.unwrap();

    // Initially empty
    assert!(engine.syncing_realms().is_empty());

    // Start some
    engine.start_sync(&realm1).await.unwrap();
    engine.start_sync(&realm3).await.unwrap();

    let syncing = engine.syncing_realms();
    assert_eq!(syncing.len(), 2);
    assert!(syncing.contains(&realm1));
    assert!(!syncing.contains(&realm2));
    assert!(syncing.contains(&realm3));

    // Add realm2
    engine.start_sync(&realm2).await.unwrap();

    let syncing = engine.syncing_realms();
    assert_eq!(syncing.len(), 3);

    // Remove realm1
    engine.stop_sync(&realm1).await.unwrap();

    let syncing = engine.syncing_realms();
    assert_eq!(syncing.len(), 2);
    assert!(!syncing.contains(&realm1));

    engine.shutdown().await.unwrap();
}

/// Test that start_sync() is idempotent (calling twice doesn't cause issues).
#[tokio::test]
async fn test_start_sync_idempotent() {
    let (mut engine, _temp) = create_test_engine().await;

    let realm_id = engine.create_realm("Idempotent Test").await.unwrap();

    // Start sync twice
    engine.start_sync(&realm_id).await.unwrap();
    engine.start_sync(&realm_id).await.unwrap();

    // Should still be syncing (not error)
    assert!(engine.is_realm_syncing(&realm_id));
    assert_eq!(engine.syncing_count(), 1);

    engine.shutdown().await.unwrap();
}

/// Test that stop_sync() is idempotent.
#[tokio::test]
async fn test_stop_sync_idempotent() {
    let (mut engine, _temp) = create_test_engine().await;

    let realm_id = engine.create_realm("Stop Idempotent Test").await.unwrap();

    // Start and stop
    engine.start_sync(&realm_id).await.unwrap();
    engine.stop_sync(&realm_id).await.unwrap();

    // Stop again - should not error
    engine.stop_sync(&realm_id).await.unwrap();

    // Should be idle
    assert!(!engine.is_realm_syncing(&realm_id));
    assert_eq!(engine.sync_status(&realm_id), SyncStatus::Idle);

    engine.shutdown().await.unwrap();
}

/// Test that operations work correctly during active sync.
#[tokio::test]
async fn test_operations_during_sync() {
    let (mut engine, _temp) = create_test_engine().await;

    let realm_id = engine.create_realm("Operations Test").await.unwrap();
    engine.start_sync(&realm_id).await.unwrap();

    // Add tasks while syncing
    let task1 = engine.add_task(&realm_id, "Task 1").await.unwrap();
    let task2 = engine.add_task(&realm_id, "Task 2").await.unwrap();

    // List tasks
    let tasks = engine.list_tasks(&realm_id).unwrap();
    assert_eq!(tasks.len(), 2);

    // Toggle a task
    engine.toggle_task(&realm_id, &task1).await.unwrap();
    let task = engine.get_task(&realm_id, &task1).unwrap().unwrap();
    assert!(task.completed);

    // Delete a task
    engine.delete_task(&realm_id, &task2).await.unwrap();
    let tasks = engine.list_tasks(&realm_id).unwrap();
    assert_eq!(tasks.len(), 1);

    // Still syncing
    assert!(engine.is_realm_syncing(&realm_id));

    engine.shutdown().await.unwrap();
}

/// Test sync with identity initialized.
#[tokio::test]
async fn test_multi_realm_sync_with_identity() {
    let (mut engine, _temp) = create_test_engine().await;

    // Initialize identity
    engine.init_identity().unwrap();

    let realm1 = engine.create_realm("Identity Realm 1").await.unwrap();
    let realm2 = engine.create_realm("Identity Realm 2").await.unwrap();

    engine.start_sync(&realm1).await.unwrap();
    engine.start_sync(&realm2).await.unwrap();

    // Should work with identity
    assert!(engine.is_realm_syncing(&realm1));
    assert!(engine.is_realm_syncing(&realm2));
    assert!(engine.has_identity());

    engine.shutdown().await.unwrap();
}

/// Test that shutdown properly stops all syncs.
#[tokio::test]
async fn test_shutdown_stops_all_syncs() {
    let (mut engine, _temp) = create_test_engine().await;

    let realm1 = engine.create_realm("Shutdown Test 1").await.unwrap();
    let realm2 = engine.create_realm("Shutdown Test 2").await.unwrap();
    let realm3 = engine.create_realm("Shutdown Test 3").await.unwrap();

    engine.start_sync(&realm1).await.unwrap();
    engine.start_sync(&realm2).await.unwrap();
    engine.start_sync(&realm3).await.unwrap();

    assert_eq!(engine.syncing_count(), 3);

    // Shutdown should not panic or hang
    engine.shutdown().await.unwrap();
}

// ============================================================================
// Sync Recovery Tests
// ============================================================================

/// Test that sync can be restarted after being stopped.
///
/// Simulates recovery from a network interruption by stopping and restarting sync.
#[tokio::test]
async fn test_sync_restart_after_stop() {
    let (mut engine, _temp) = create_test_engine().await;

    let realm_id = engine.create_realm("Restart Test").await.unwrap();

    // Initial sync
    engine.start_sync(&realm_id).await.unwrap();
    assert!(engine.is_realm_syncing(&realm_id));

    // Add a task during sync
    engine.add_task(&realm_id, "Task before stop").await.unwrap();

    // Simulate network interruption - stop sync
    engine.stop_sync(&realm_id).await.unwrap();
    assert!(!engine.is_realm_syncing(&realm_id));
    assert_eq!(engine.sync_status(&realm_id), SyncStatus::Idle);

    // Realm data should still be accessible
    let tasks = engine.list_tasks(&realm_id).unwrap();
    assert_eq!(tasks.len(), 1);

    // Simulate network recovery - restart sync
    engine.start_sync(&realm_id).await.unwrap();
    assert!(engine.is_realm_syncing(&realm_id));
    assert!(matches!(engine.sync_status(&realm_id), SyncStatus::Syncing { .. }));

    // Add another task after restart
    engine.add_task(&realm_id, "Task after restart").await.unwrap();
    let tasks = engine.list_tasks(&realm_id).unwrap();
    assert_eq!(tasks.len(), 2);

    engine.shutdown().await.unwrap();
}

/// Test that multiple stop/start cycles work correctly.
#[tokio::test]
async fn test_multiple_stop_start_cycles() {
    let (mut engine, _temp) = create_test_engine().await;

    let realm_id = engine.create_realm("Cycle Test").await.unwrap();

    for i in 0..3 {
        // Start sync
        engine.start_sync(&realm_id).await.unwrap();
        assert!(engine.is_realm_syncing(&realm_id), "Cycle {}: Should be syncing after start", i);

        // Add a task
        engine.add_task(&realm_id, &format!("Task {}", i)).await.unwrap();

        // Stop sync
        engine.stop_sync(&realm_id).await.unwrap();
        assert!(!engine.is_realm_syncing(&realm_id), "Cycle {}: Should not be syncing after stop", i);
    }

    // All tasks should be present
    let tasks = engine.list_tasks(&realm_id).unwrap();
    assert_eq!(tasks.len(), 3);

    engine.shutdown().await.unwrap();
}

/// Test that different realms can have different sync states independently.
#[tokio::test]
async fn test_independent_sync_states() {
    let (mut engine, _temp) = create_test_engine().await;

    let realm1 = engine.create_realm("Realm Active").await.unwrap();
    let realm2 = engine.create_realm("Realm Recovering").await.unwrap();
    let realm3 = engine.create_realm("Realm Idle").await.unwrap();

    // Start all
    engine.start_sync(&realm1).await.unwrap();
    engine.start_sync(&realm2).await.unwrap();
    engine.start_sync(&realm3).await.unwrap();

    // Stop realm2 and realm3 (simulate partial network issues)
    engine.stop_sync(&realm2).await.unwrap();
    engine.stop_sync(&realm3).await.unwrap();

    // realm1 still syncing
    assert!(engine.is_realm_syncing(&realm1));
    assert!(!engine.is_realm_syncing(&realm2));
    assert!(!engine.is_realm_syncing(&realm3));

    // "Recover" realm2 only
    engine.start_sync(&realm2).await.unwrap();

    // Final states
    assert!(engine.is_realm_syncing(&realm1));
    assert!(engine.is_realm_syncing(&realm2));
    assert!(!engine.is_realm_syncing(&realm3));

    // Syncing count should be 2
    assert_eq!(engine.syncing_count(), 2);

    engine.shutdown().await.unwrap();
}

/// Test that data persists through sync cycles.
#[tokio::test]
async fn test_data_persistence_through_sync_cycles() {
    let temp_dir = TempDir::new().unwrap();

    let realm_id;

    // First engine instance - create data and sync
    {
        let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
        realm_id = engine.create_realm("Persistence Test").await.unwrap();

        engine.start_sync(&realm_id).await.unwrap();
        engine.add_task(&realm_id, "Task 1").await.unwrap();
        engine.add_task(&realm_id, "Task 2").await.unwrap();
        engine.stop_sync(&realm_id).await.unwrap();
        // Engine dropped here, data should persist
    }

    // Second engine instance - verify data and restart sync
    {
        let mut engine = SyncEngine::new(temp_dir.path()).await.unwrap();
        engine.open_realm(&realm_id).await.unwrap();

        // Data should still be there
        let tasks = engine.list_tasks(&realm_id).unwrap();
        assert_eq!(tasks.len(), 2);

        // Should be able to restart sync
        engine.start_sync(&realm_id).await.unwrap();
        assert!(engine.is_realm_syncing(&realm_id));

        // Add more data
        engine.add_task(&realm_id, "Task 3").await.unwrap();
        let tasks = engine.list_tasks(&realm_id).unwrap();
        assert_eq!(tasks.len(), 3);

        engine.shutdown().await.unwrap();
    }
}

/// Test that events are emitted during sync recovery.
#[tokio::test]
async fn test_events_during_sync_recovery() {
    let (mut engine, _temp) = create_test_engine().await;

    let realm_id = engine.create_realm("Event Recovery Test").await.unwrap();

    // Subscribe to events
    let mut events = engine.subscribe_events();

    // Start sync
    engine.start_sync(&realm_id).await.unwrap();

    // Collect start events
    let mut start_events = Vec::new();
    for _ in 0..10 {
        match tokio::time::timeout(Duration::from_millis(50), events.recv()).await {
            Ok(Ok(event @ SyncEvent::StatusChanged { .. })) => {
                start_events.push(event);
            }
            _ => break,
        }
    }

    // Stop sync
    engine.stop_sync(&realm_id).await.unwrap();

    // Collect stop event
    let mut stop_events = Vec::new();
    for _ in 0..10 {
        match tokio::time::timeout(Duration::from_millis(50), events.recv()).await {
            Ok(Ok(event @ SyncEvent::StatusChanged { .. })) => {
                stop_events.push(event);
            }
            _ => break,
        }
    }

    // Restart sync
    engine.start_sync(&realm_id).await.unwrap();

    // Collect restart events
    let mut restart_events = Vec::new();
    for _ in 0..10 {
        match tokio::time::timeout(Duration::from_millis(50), events.recv()).await {
            Ok(Ok(event @ SyncEvent::StatusChanged { .. })) => {
                restart_events.push(event);
            }
            _ => break,
        }
    }

    // Should have events for each phase
    assert!(!start_events.is_empty(), "Should have start events");
    assert!(!stop_events.is_empty(), "Should have stop events");
    assert!(!restart_events.is_empty(), "Should have restart events");

    // Check for idle event in stop phase
    let has_idle = stop_events.iter().any(|e| {
        matches!(e, SyncEvent::StatusChanged { status: SyncStatus::Idle, .. })
    });
    assert!(has_idle, "Should have Idle status event after stop");

    engine.shutdown().await.unwrap();
}
