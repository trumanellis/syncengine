//! Benchmarks for Synchronicity Engine sync operations
//!
//! Run with: cargo bench -p syncengine-core
//!
//! These benchmarks establish performance baselines for:
//! - Document operations (add, toggle, delete, list)
//! - Save/load cycles
//! - Merge operations
//! - Incremental sync

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use syncengine_core::realm::RealmDoc;
use syncengine_core::{RealmId, TaskId};

// ============================================================================
// Document Creation Benchmarks
// ============================================================================

fn bench_document_creation(c: &mut Criterion) {
    c.bench_function("create_empty_document", |b| {
        b.iter(|| black_box(RealmDoc::new()))
    });
}

// ============================================================================
// Task Operation Benchmarks
// ============================================================================

fn bench_add_task(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_task");

    // Benchmark adding to empty doc
    group.bench_function("to_empty_doc", |b| {
        b.iter_batched(
            || RealmDoc::new(),
            |mut doc| black_box(doc.add_task("Test task").unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });

    // Benchmark adding to doc with 100 tasks
    group.bench_function("to_100_task_doc", |b| {
        b.iter_batched(
            || {
                let mut doc = RealmDoc::new();
                for i in 0..100 {
                    doc.add_task(&format!("Task {}", i)).unwrap();
                }
                doc
            },
            |mut doc| black_box(doc.add_task("New task").unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });

    // Benchmark adding to doc with 1000 tasks
    group.bench_function("to_1000_task_doc", |b| {
        b.iter_batched(
            || {
                let mut doc = RealmDoc::new();
                for i in 0..1000 {
                    doc.add_task(&format!("Task {}", i)).unwrap();
                }
                doc
            },
            |mut doc| black_box(doc.add_task("New task").unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_toggle_task(c: &mut Criterion) {
    let mut group = c.benchmark_group("toggle_task");

    // Simple toggle
    group.bench_function("single", |b| {
        b.iter_batched(
            || {
                let mut doc = RealmDoc::new();
                let id = doc.add_task("Test").unwrap();
                (doc, id)
            },
            |(mut doc, id)| black_box(doc.toggle_task(&id).unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });

    // Toggle in document with many tasks
    group.bench_function("in_100_task_doc", |b| {
        b.iter_batched(
            || {
                let mut doc = RealmDoc::new();
                let mut ids = Vec::new();
                for i in 0..100 {
                    ids.push(doc.add_task(&format!("Task {}", i)).unwrap());
                }
                (doc, ids[50].clone())
            },
            |(mut doc, id)| black_box(doc.toggle_task(&id).unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_delete_task(c: &mut Criterion) {
    c.bench_function("delete_task", |b| {
        b.iter_batched(
            || {
                let mut doc = RealmDoc::new();
                let id = doc.add_task("To delete").unwrap();
                (doc, id)
            },
            |(mut doc, id)| black_box(doc.delete_task(&id).unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });
}

fn bench_get_task(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_task");

    group.bench_function("from_10_tasks", |b| {
        let mut doc = RealmDoc::new();
        let mut ids = Vec::new();
        for i in 0..10 {
            ids.push(doc.add_task(&format!("Task {}", i)).unwrap());
        }
        let target_id = ids[5].clone();

        b.iter(|| black_box(doc.get_task(&target_id).unwrap()))
    });

    group.bench_function("from_100_tasks", |b| {
        let mut doc = RealmDoc::new();
        let mut ids = Vec::new();
        for i in 0..100 {
            ids.push(doc.add_task(&format!("Task {}", i)).unwrap());
        }
        let target_id = ids[50].clone();

        b.iter(|| black_box(doc.get_task(&target_id).unwrap()))
    });

    group.finish();
}

fn bench_list_tasks(c: &mut Criterion) {
    let mut group = c.benchmark_group("list_tasks");

    for size in [10, 50, 100, 500].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut doc = RealmDoc::new();
            for i in 0..size {
                doc.add_task(&format!("Task {}", i)).unwrap();
            }

            b.iter(|| black_box(doc.list_tasks().unwrap()))
        });
    }

    group.finish();
}

// ============================================================================
// Save/Load Benchmarks
// ============================================================================

fn bench_save_document(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_document");

    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::new("tasks", size), size, |b, &size| {
            let mut doc = RealmDoc::new();
            for i in 0..size {
                doc.add_task(&format!("Task {}", i)).unwrap();
            }

            b.iter(|| black_box(doc.save()))
        });
    }

    group.finish();
}

fn bench_load_document(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_document");

    for size in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::new("tasks", size), size, |b, &size| {
            let mut doc = RealmDoc::new();
            for i in 0..size {
                doc.add_task(&format!("Task {}", i)).unwrap();
            }
            let bytes = doc.save();

            b.iter(|| black_box(RealmDoc::load(&bytes).unwrap()))
        });
    }

    group.finish();
}

fn bench_save_load_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_load_roundtrip");

    for size in [10, 100].iter() {
        group.bench_with_input(BenchmarkId::new("tasks", size), size, |b, &size| {
            let mut doc = RealmDoc::new();
            for i in 0..size {
                doc.add_task(&format!("Task {}", i)).unwrap();
            }

            b.iter(|| {
                let bytes = doc.save();
                black_box(RealmDoc::load(&bytes).unwrap())
            })
        });
    }

    group.finish();
}

// ============================================================================
// Merge Benchmarks
// ============================================================================

fn bench_fork(c: &mut Criterion) {
    let mut group = c.benchmark_group("fork");

    for size in [10, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::new("tasks", size), size, |b, &size| {
            let mut doc = RealmDoc::new();
            for i in 0..size {
                doc.add_task(&format!("Task {}", i)).unwrap();
            }

            b.iter(|| black_box(doc.fork()))
        });
    }

    group.finish();
}

fn bench_merge(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge");

    // Merge empty into empty
    group.bench_function("empty_into_empty", |b| {
        b.iter_batched(
            || (RealmDoc::new(), RealmDoc::new()),
            |(mut doc1, mut doc2)| black_box(doc1.merge(&mut doc2).unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });

    // Merge small change
    group.bench_function("small_change", |b| {
        b.iter_batched(
            || {
                let mut doc = RealmDoc::new();
                doc.add_task("Base").unwrap();
                let mut fork = doc.fork();
                fork.add_task("New").unwrap();
                (doc, fork)
            },
            |(mut doc, mut fork)| black_box(doc.merge(&mut fork).unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });

    // Merge 10 changes
    group.bench_function("10_changes", |b| {
        b.iter_batched(
            || {
                let mut doc = RealmDoc::new();
                doc.add_task("Base").unwrap();
                let mut fork = doc.fork();
                for i in 0..10 {
                    fork.add_task(&format!("Fork task {}", i)).unwrap();
                }
                (doc, fork)
            },
            |(mut doc, mut fork)| black_box(doc.merge(&mut fork).unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });

    // Merge with conflicting changes (concurrent edits)
    group.bench_function("concurrent_edits", |b| {
        b.iter_batched(
            || {
                let base = RealmDoc::new();
                let mut fork1 = base.fork();
                let mut fork2 = base.fork();
                for i in 0..5 {
                    fork1.add_task(&format!("Fork1 task {}", i)).unwrap();
                    fork2.add_task(&format!("Fork2 task {}", i)).unwrap();
                }
                (fork1, fork2)
            },
            |(mut fork1, mut fork2)| black_box(fork1.merge(&mut fork2).unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

// ============================================================================
// Incremental Sync Benchmarks
// ============================================================================

fn bench_incremental_sync(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_sync");

    // Generate sync message
    group.bench_function("generate_sync_message", |b| {
        b.iter_batched(
            || {
                let mut doc = RealmDoc::new();
                for i in 0..10 {
                    doc.add_task(&format!("Task {}", i)).unwrap();
                }
                doc
            },
            |mut doc| black_box(doc.generate_sync_message()),
            criterion::BatchSize::SmallInput,
        )
    });

    // Apply sync message
    group.bench_function("apply_sync_message", |b| {
        // Create two docs, generate sync message from one with changes
        let mut doc1 = RealmDoc::new();
        for i in 0..10 {
            doc1.add_task(&format!("Task {}", i)).unwrap();
        }
        let bytes = doc1.save();

        // Add more to doc1, generate sync message
        for i in 10..20 {
            doc1.add_task(&format!("Task {}", i)).unwrap();
        }
        let sync_msg = doc1.generate_sync_message();

        b.iter_batched(
            || RealmDoc::load(&bytes).unwrap(),
            |mut doc2| black_box(doc2.apply_sync_message(&sync_msg).unwrap()),
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

// ============================================================================
// ID Generation Benchmarks
// ============================================================================

fn bench_id_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("id_generation");

    group.bench_function("task_id", |b| b.iter(|| black_box(TaskId::new())));

    group.bench_function("realm_id", |b| b.iter(|| black_box(RealmId::new())));

    group.finish();
}

fn bench_id_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("id_serialization");

    let task_id = TaskId::new();
    group.bench_function("task_id_to_string", |b| {
        b.iter(|| black_box(task_id.to_string_repr()))
    });

    let realm_id = RealmId::new();
    group.bench_function("realm_id_to_base58", |b| {
        b.iter(|| black_box(realm_id.to_base58()))
    });

    let realm_str = realm_id.to_base58();
    group.bench_function("realm_id_from_base58", |b| {
        b.iter(|| black_box(RealmId::from_base58(&realm_str).unwrap()))
    });

    group.finish();
}

// ============================================================================
// Full Sync Scenario Benchmarks
// ============================================================================

fn bench_two_node_sync(c: &mut Criterion) {
    let mut group = c.benchmark_group("two_node_sync");

    // Simulate two nodes syncing 100 tasks
    group.bench_function("100_tasks", |b| {
        b.iter(|| {
            // Node 1 creates document with 100 tasks
            let mut node1 = RealmDoc::new();
            for i in 0..100 {
                node1.add_task(&format!("Task {}", i)).unwrap();
            }

            // Node 1 sends full document to Node 2
            let bytes = node1.save();
            let mut node2 = RealmDoc::load(&bytes).unwrap();

            // Node 1 adds more tasks
            for i in 100..110 {
                node1.add_task(&format!("Task {}", i)).unwrap();
            }

            // Incremental sync
            let sync_msg = node1.generate_sync_message();
            node2.apply_sync_message(&sync_msg).unwrap();

            black_box((node1, node2))
        })
    });

    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(creation_benches, bench_document_creation,);

criterion_group!(
    task_op_benches,
    bench_add_task,
    bench_toggle_task,
    bench_delete_task,
    bench_get_task,
    bench_list_tasks,
);

criterion_group!(
    persistence_benches,
    bench_save_document,
    bench_load_document,
    bench_save_load_roundtrip,
);

criterion_group!(merge_benches, bench_fork, bench_merge,);

criterion_group!(sync_benches, bench_incremental_sync, bench_two_node_sync,);

criterion_group!(id_benches, bench_id_generation, bench_id_serialization,);

criterion_main!(
    creation_benches,
    task_op_benches,
    persistence_benches,
    merge_benches,
    sync_benches,
    id_benches,
);
