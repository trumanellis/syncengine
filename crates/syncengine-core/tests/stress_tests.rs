//! Stress tests for high-volume operations
//!
//! These tests verify the system handles large numbers of operations,
//! concurrent access, and sustained throughput.

use std::collections::HashSet;
use std::time::Instant;

use syncengine_core::realm::RealmDoc;
use syncengine_core::{RealmId, TaskId};

// ============================================================================
// Document Stress Tests
// ============================================================================

/// Test adding 100 tasks to a single document
#[test]
fn test_100_tasks_in_document() {
    let mut doc = RealmDoc::new();

    let start = Instant::now();

    for i in 0..100 {
        doc.add_task(&format!("Task {}", i)).unwrap();
    }

    let add_duration = start.elapsed();

    let tasks = doc.list_tasks().unwrap();
    assert_eq!(tasks.len(), 100);

    // Verify all tasks are unique
    let ids: HashSet<_> = tasks.iter().map(|t| t.id.clone()).collect();
    assert_eq!(ids.len(), 100);

    // Verify we can save and load
    let bytes = doc.save();
    let loaded = RealmDoc::load(&bytes).unwrap();
    assert_eq!(loaded.list_tasks().unwrap().len(), 100);

    println!(
        "Added 100 tasks in {:?} ({:.2} tasks/ms)",
        add_duration,
        100.0 / add_duration.as_millis() as f64
    );
}

/// Test adding 1000 tasks to a single document
#[test]
fn test_1000_tasks_in_document() {
    let mut doc = RealmDoc::new();

    let start = Instant::now();

    for i in 0..1000 {
        doc.add_task(&format!("Task {}", i)).unwrap();
    }

    let add_duration = start.elapsed();

    let tasks = doc.list_tasks().unwrap();
    assert_eq!(tasks.len(), 1000);

    // Save and load should work
    let save_start = Instant::now();
    let bytes = doc.save();
    let save_duration = save_start.elapsed();

    let load_start = Instant::now();
    let loaded = RealmDoc::load(&bytes).unwrap();
    let load_duration = load_start.elapsed();

    assert_eq!(loaded.list_tasks().unwrap().len(), 1000);

    println!(
        "1000 tasks: add {:?}, save {:?} ({} bytes), load {:?}",
        add_duration,
        save_duration,
        bytes.len(),
        load_duration
    );
}

/// Test rapidly toggling tasks
#[test]
fn test_rapid_toggles() {
    let mut doc = RealmDoc::new();

    // Add some tasks
    let mut ids = Vec::new();
    for i in 0..10 {
        ids.push(doc.add_task(&format!("Task {}", i)).unwrap());
    }

    let start = Instant::now();

    // Toggle each task 100 times
    for _ in 0..100 {
        for id in &ids {
            doc.toggle_task(id).unwrap();
        }
    }

    let duration = start.elapsed();

    // After even number of toggles, all should be incomplete
    for id in &ids {
        let task = doc.get_task(id).unwrap().unwrap();
        assert!(
            !task.completed,
            "Task should be incomplete after even toggles"
        );
    }

    println!(
        "1000 toggles in {:?} ({:.2} toggles/ms)",
        duration,
        1000.0 / duration.as_millis().max(1) as f64
    );
}

/// Test merging many small changes
#[test]
fn test_merge_many_small_changes() {
    let mut base = RealmDoc::new();
    base.add_task("Base task").unwrap();

    let start = Instant::now();

    // Create 50 forks, each with one task, merge all
    for i in 0..50 {
        let mut fork = base.fork();
        fork.add_task(&format!("Fork task {}", i)).unwrap();
        base.merge(&mut fork).unwrap();
    }

    let duration = start.elapsed();

    let tasks = base.list_tasks().unwrap();
    assert_eq!(tasks.len(), 51); // 1 base + 50 fork tasks

    println!(
        "50 fork-merge cycles in {:?} ({:.2} merges/ms)",
        duration,
        50.0 / duration.as_millis().max(1) as f64
    );
}

/// Test concurrent editing simulation
#[test]
fn test_simulated_concurrent_edits() {
    // Simulate 5 peers each making 20 changes, then merging
    let mut base = RealmDoc::new();

    let mut peers: Vec<RealmDoc> = (0..5).map(|_| base.fork()).collect();

    let start = Instant::now();

    // Each peer adds tasks
    for (peer_idx, peer) in peers.iter_mut().enumerate() {
        for task_idx in 0..20 {
            peer.add_task(&format!("Peer {} Task {}", peer_idx, task_idx))
                .unwrap();
        }
    }

    // Merge all peers into base
    for peer in &mut peers {
        base.merge(peer).unwrap();
    }

    let duration = start.elapsed();

    let tasks = base.list_tasks().unwrap();
    assert_eq!(tasks.len(), 100); // 5 peers * 20 tasks

    // All peers should converge to same state after syncing
    for peer in &mut peers {
        peer.merge(&mut base.fork()).unwrap();
        assert_eq!(peer.list_tasks().unwrap().len(), 100);
    }

    println!("5 peers x 20 edits + merge in {:?}", duration);
}

/// Test incremental sync with many changes
#[test]
fn test_incremental_sync_100_changes() {
    let mut doc1 = RealmDoc::new();

    // Initial state
    for i in 0..10 {
        doc1.add_task(&format!("Initial task {}", i)).unwrap();
    }

    // Create doc2 from full save
    let bytes = doc1.save();
    let mut doc2 = RealmDoc::load(&bytes).unwrap();

    let start = Instant::now();

    // Add 100 more tasks to doc1, sync incrementally
    for i in 0..100 {
        doc1.add_task(&format!("New task {}", i)).unwrap();

        // Sync every 10 tasks
        if (i + 1) % 10 == 0 {
            let sync_msg = doc1.generate_sync_message();
            doc2.apply_sync_message(&sync_msg).unwrap();
        }
    }

    // Final sync
    let sync_msg = doc1.generate_sync_message();
    doc2.apply_sync_message(&sync_msg).unwrap();

    let duration = start.elapsed();

    assert_eq!(doc1.list_tasks().unwrap().len(), 110);
    assert_eq!(doc2.list_tasks().unwrap().len(), 110);

    println!("100 adds with 10 incremental syncs in {:?}", duration);
}

// ============================================================================
// Type ID Generation Stress Tests
// ============================================================================

/// Test that TaskId generation is unique under rapid creation
#[test]
fn test_task_id_uniqueness_under_stress() {
    let mut ids = HashSet::new();

    for _ in 0..10000 {
        let id = TaskId::new();
        assert!(
            ids.insert(id.to_string_repr()),
            "Duplicate TaskId generated!"
        );
    }

    assert_eq!(ids.len(), 10000);
}

/// Test that RealmId generation is unique under rapid creation
#[test]
fn test_realm_id_uniqueness_under_stress() {
    let mut ids = HashSet::new();

    for _ in 0..10000 {
        let id = RealmId::new();
        assert!(ids.insert(id.to_base58()), "Duplicate RealmId generated!");
    }

    assert_eq!(ids.len(), 10000);
}

// ============================================================================
// Document Size Stress Tests
// ============================================================================

/// Test with very long task titles
#[test]
fn test_long_task_titles() {
    let mut doc = RealmDoc::new();

    // Add tasks with increasingly long titles
    let sizes = [100, 500, 1000, 5000, 10000];

    for size in sizes {
        let title = "x".repeat(size);
        let id = doc.add_task(&title).unwrap();
        let task = doc.get_task(&id).unwrap().unwrap();
        assert_eq!(task.title.len(), size);
    }

    // Verify all tasks stored correctly
    let tasks = doc.list_tasks().unwrap();
    assert_eq!(tasks.len(), sizes.len());

    // Save and load should work
    let bytes = doc.save();
    let loaded = RealmDoc::load(&bytes).unwrap();
    let loaded_tasks = loaded.list_tasks().unwrap();
    assert_eq!(loaded_tasks.len(), sizes.len());

    // Verify title lengths preserved
    for (orig, loaded) in tasks.iter().zip(loaded_tasks.iter()) {
        assert_eq!(orig.title.len(), loaded.title.len());
    }
}

/// Test document with many toggles in history
#[test]
fn test_toggle_history_stress() {
    let mut doc = RealmDoc::new();
    let id = doc.add_task("Toggle me").unwrap();

    let start = Instant::now();

    // Toggle 1000 times
    for _ in 0..1000 {
        doc.toggle_task(&id).unwrap();
    }

    let toggle_duration = start.elapsed();

    // After even number of toggles, should be incomplete
    let task = doc.get_task(&id).unwrap().unwrap();
    assert!(!task.completed);

    // Save should still work (Automerge compacts history)
    let save_start = Instant::now();
    let bytes = doc.save();
    let save_duration = save_start.elapsed();

    println!(
        "1000 toggles: toggle {:?}, save {:?} ({} bytes)",
        toggle_duration,
        save_duration,
        bytes.len()
    );

    // Document should be loadable
    let loaded = RealmDoc::load(&bytes).unwrap();
    let loaded_task = loaded.get_task(&id).unwrap().unwrap();
    assert_eq!(task.completed, loaded_task.completed);
}

// ============================================================================
// Parallel Document Operations (Simulated)
// ============================================================================

/// Test that creating many documents is fast
#[test]
fn test_create_100_documents() {
    let start = Instant::now();

    let docs: Vec<RealmDoc> = (0..100).map(|_| RealmDoc::new()).collect();

    let duration = start.elapsed();

    assert_eq!(docs.len(), 100);

    println!(
        "Created 100 empty documents in {:?} ({:.2} docs/ms)",
        duration,
        100.0 / duration.as_millis().max(1) as f64
    );
}

/// Test merging a chain of forked documents
#[test]
fn test_chain_merge() {
    // Create a base document
    let mut base = RealmDoc::new();
    base.add_task("Base task").unwrap();

    // Create forked documents from the base
    let mut forks: Vec<RealmDoc> = (0..10)
        .map(|i| {
            let mut fork = base.fork();
            fork.add_task(&format!("Fork {} task", i)).unwrap();
            fork
        })
        .collect();

    let start = Instant::now();

    // Merge all forks into the base
    for fork in &mut forks {
        base.merge(fork).unwrap();
    }

    let duration = start.elapsed();

    // Base should have original task + all fork tasks
    let tasks = base.list_tasks().unwrap();
    assert_eq!(tasks.len(), 11); // 1 base + 10 forks

    println!("Chain merge of 10 forked documents in {:?}", duration);
}

// ============================================================================
// Memory Efficiency Tests
// ============================================================================

/// Test that we can handle operations without excessive memory growth
#[test]
fn test_memory_stability() {
    // This test checks that repeated save/load cycles don't cause unbounded growth
    let mut doc = RealmDoc::new();

    for i in 0..100 {
        doc.add_task(&format!("Task {}", i)).unwrap();
    }

    let initial_size = doc.save().len();

    // Do many save/load cycles
    for _ in 0..10 {
        let bytes = doc.save();
        doc = RealmDoc::load(&bytes).unwrap();
    }

    let final_size = doc.save().len();

    // Size should be similar (within 20% tolerance for overhead)
    let growth_ratio = final_size as f64 / initial_size as f64;
    assert!(
        growth_ratio < 1.2,
        "Document grew too much: {} -> {} ({:.2}x)",
        initial_size,
        final_size,
        growth_ratio
    );

    println!(
        "Document size: initial {} bytes, final {} bytes ({:.2}x)",
        initial_size, final_size, growth_ratio
    );
}

// ============================================================================
// Throughput Measurement Tests
// ============================================================================

/// Measure add task throughput
#[test]
fn test_add_task_throughput() {
    let iterations = 500; // Reduced for faster test runs
    let mut doc = RealmDoc::new();

    let start = Instant::now();

    for i in 0..iterations {
        doc.add_task(&format!("Task {}", i)).unwrap();
    }

    let duration = start.elapsed();
    let throughput = iterations as f64 / duration.as_secs_f64();

    println!(
        "Add task throughput: {:.0} ops/sec ({} tasks in {:?})",
        throughput, iterations, duration
    );

    // Relaxed threshold - Automerge operations are relatively slow
    assert!(
        throughput > 10.0,
        "Add task throughput too low: {:.0} ops/sec",
        throughput
    );
}

/// Measure toggle task throughput
#[test]
fn test_toggle_task_throughput() {
    let mut doc = RealmDoc::new();
    let id = doc.add_task("Toggle test").unwrap();

    let iterations = 100; // Small number - toggles are expensive in Automerge

    let start = Instant::now();

    for _ in 0..iterations {
        doc.toggle_task(&id).unwrap();
    }

    let duration = start.elapsed();
    let throughput = iterations as f64 / duration.as_secs_f64();

    println!(
        "Toggle task throughput: {:.0} ops/sec ({} toggles in {:?})",
        throughput, iterations, duration
    );

    // Very relaxed threshold - Automerge toggles rebuild the task JSON
    // which is inherently slow. This is a known limitation.
    assert!(
        throughput > 5.0,
        "Toggle task throughput too low: {:.0} ops/sec",
        throughput
    );
}

/// Measure merge throughput
#[test]
fn test_merge_throughput() {
    let mut base = RealmDoc::new();
    base.add_task("Base").unwrap();

    let iterations = 100;

    let start = Instant::now();

    for i in 0..iterations {
        let mut fork = base.fork();
        fork.add_task(&format!("Fork {}", i)).unwrap();
        base.merge(&mut fork).unwrap();
    }

    let duration = start.elapsed();
    let throughput = iterations as f64 / duration.as_secs_f64();

    println!(
        "Merge throughput: {:.0} ops/sec ({} merges in {:?})",
        throughput, iterations, duration
    );

    assert!(
        throughput > 10.0,
        "Merge throughput too low: {:.0} ops/sec",
        throughput
    );
}
