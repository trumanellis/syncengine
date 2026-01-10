//! Property-based tests for RealmDoc operations
//!
//! Uses proptest to verify invariants and properties of the RealmDoc CRDT.

use proptest::prelude::*;
use syncengine_core::realm::RealmDoc;
use syncengine_core::TaskId;

// ============================================================================
// Strategy Generators
// ============================================================================

/// Generate valid task titles (non-empty strings up to 10000 chars)
fn task_title_strategy() -> impl Strategy<Value = String> {
    // Using printable characters, avoiding empty strings
    prop::string::string_regex(".{1,1000}")
        .expect("valid regex")
        .prop_filter("non-empty", |s| !s.is_empty())
}

/// Generate a shorter task title for faster tests
fn short_title_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9 ]{1,100}")
        .expect("valid regex")
        .prop_filter("non-empty", |s| !s.is_empty())
}

/// Operations that can be performed on a RealmDoc
#[derive(Debug, Clone)]
enum DocOp {
    AddTask(String),
    ToggleTask(usize), // Index into existing tasks
    DeleteTask(usize), // Index into existing tasks
}

/// Generate a sequence of document operations
fn doc_ops_strategy(max_ops: usize) -> impl Strategy<Value = Vec<DocOp>> {
    prop::collection::vec(
        prop_oneof![
            3 => short_title_strategy().prop_map(DocOp::AddTask),
            1 => (0..100usize).prop_map(DocOp::ToggleTask),
            1 => (0..100usize).prop_map(DocOp::DeleteTask),
        ],
        0..max_ops,
    )
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    /// Any valid string can be stored and retrieved as a task title
    #[test]
    fn task_title_roundtrip(title in task_title_strategy()) {
        let mut doc = RealmDoc::new();
        let id = doc.add_task(&title).unwrap();
        let task = doc.get_task(&id).unwrap().unwrap();
        prop_assert_eq!(task.title, title);
    }

    /// Adding a task always increases the task count by 1
    #[test]
    fn add_task_increases_count(titles in prop::collection::vec(short_title_strategy(), 1..20)) {
        let mut doc = RealmDoc::new();

        for (i, title) in titles.iter().enumerate() {
            doc.add_task(title).unwrap();
            let tasks = doc.list_tasks().unwrap();
            prop_assert_eq!(tasks.len(), i + 1);
        }
    }

    /// Deleting a task decreases the task count by 1
    #[test]
    fn delete_task_decreases_count(titles in prop::collection::vec(short_title_strategy(), 1..10)) {
        let mut doc = RealmDoc::new();

        // Add all tasks
        let mut ids = Vec::new();
        for title in &titles {
            ids.push(doc.add_task(title).unwrap());
        }

        // Delete them one by one
        for (i, id) in ids.iter().enumerate() {
            let count_before = doc.list_tasks().unwrap().len();
            doc.delete_task(id).unwrap();
            let count_after = doc.list_tasks().unwrap().len();
            prop_assert_eq!(count_after, count_before - 1, "After deleting task {}", i);
        }
    }

    /// Toggle is an involution: toggle(toggle(x)) == x
    #[test]
    fn toggle_is_involution(title in short_title_strategy()) {
        let mut doc = RealmDoc::new();
        let id = doc.add_task(&title).unwrap();

        let initial_state = doc.get_task(&id).unwrap().unwrap().completed;

        doc.toggle_task(&id).unwrap();
        let after_first_toggle = doc.get_task(&id).unwrap().unwrap().completed;
        prop_assert_ne!(initial_state, after_first_toggle);

        doc.toggle_task(&id).unwrap();
        let after_second_toggle = doc.get_task(&id).unwrap().unwrap().completed;
        prop_assert_eq!(initial_state, after_second_toggle);
    }

    /// Merge is commutative: merge(A, B) == merge(B, A) in terms of final state
    #[test]
    fn merge_is_commutative(
        titles1 in prop::collection::vec(short_title_strategy(), 0..5),
        titles2 in prop::collection::vec(short_title_strategy(), 0..5)
    ) {
        // Create base document
        let mut base = RealmDoc::new();

        // Fork to create two branches
        let mut doc1 = base.fork();
        let mut doc2 = base.fork();

        // Make changes on each branch
        for title in &titles1 {
            doc1.add_task(title).unwrap();
        }
        for title in &titles2 {
            doc2.add_task(title).unwrap();
        }

        // Merge in both directions
        let mut result1 = doc1.fork();
        let mut result2 = doc2.fork();

        result1.merge(&mut doc2.fork()).unwrap();
        result2.merge(&mut doc1.fork()).unwrap();

        // Both should have the same number of tasks
        let tasks1 = result1.list_tasks().unwrap();
        let tasks2 = result2.list_tasks().unwrap();

        prop_assert_eq!(tasks1.len(), tasks2.len());
        prop_assert_eq!(tasks1.len(), titles1.len() + titles2.len());
    }

    /// Save and load preserves all data
    #[test]
    fn save_load_roundtrip(titles in prop::collection::vec(short_title_strategy(), 0..10)) {
        let mut doc = RealmDoc::new();

        let mut ids = Vec::new();
        for title in &titles {
            ids.push(doc.add_task(title).unwrap());
        }

        // Toggle some tasks
        for (i, id) in ids.iter().enumerate() {
            if i % 2 == 0 {
                doc.toggle_task(id).unwrap();
            }
        }

        // Save and reload
        let bytes = doc.save();
        let loaded = RealmDoc::load(&bytes).unwrap();

        // Verify same tasks
        let original_tasks = doc.list_tasks().unwrap();
        let loaded_tasks = loaded.list_tasks().unwrap();

        prop_assert_eq!(original_tasks.len(), loaded_tasks.len());

        for (orig, loaded) in original_tasks.iter().zip(loaded_tasks.iter()) {
            prop_assert_eq!(&orig.title, &loaded.title);
            prop_assert_eq!(orig.completed, loaded.completed);
        }
    }

    /// Incremental sync preserves all data
    #[test]
    fn incremental_sync_preserves_data(
        initial_titles in prop::collection::vec(short_title_strategy(), 1..5),
        new_titles in prop::collection::vec(short_title_strategy(), 1..5)
    ) {
        let mut doc1 = RealmDoc::new();

        // Add initial tasks
        for title in &initial_titles {
            doc1.add_task(title).unwrap();
        }

        // Save full state and load into doc2
        let full_save = doc1.save();
        let mut doc2 = RealmDoc::load(&full_save).unwrap();

        // Add more tasks to doc1
        for title in &new_titles {
            doc1.add_task(title).unwrap();
        }

        // Generate incremental sync message
        let sync_msg = doc1.generate_sync_message();

        // Apply to doc2
        doc2.apply_sync_message(&sync_msg).unwrap();

        // Both should have same tasks
        let tasks1 = doc1.list_tasks().unwrap();
        let tasks2 = doc2.list_tasks().unwrap();

        prop_assert_eq!(tasks1.len(), tasks2.len());
        prop_assert_eq!(tasks1.len(), initial_titles.len() + new_titles.len());
    }

    /// Heads change after any modification
    #[test]
    fn heads_change_on_modification(title in short_title_strategy()) {
        let mut doc = RealmDoc::new();
        let heads1 = doc.heads();

        let id = doc.add_task(&title).unwrap();
        let heads2 = doc.heads();
        prop_assert_ne!(heads1, heads2.clone(), "Heads should change after add");

        doc.toggle_task(&id).unwrap();
        let heads3 = doc.heads();
        prop_assert_ne!(heads2, heads3.clone(), "Heads should change after toggle");

        doc.delete_task(&id).unwrap();
        let heads4 = doc.heads();
        prop_assert_ne!(heads3, heads4, "Heads should change after delete");
    }

    /// Getting a nonexistent task returns None
    #[test]
    fn get_nonexistent_task_returns_none(_dummy in 0..100i32) {
        let doc = RealmDoc::new();
        let fake_id = TaskId::new();
        let result = doc.get_task(&fake_id).unwrap();
        prop_assert!(result.is_none());
    }

    /// Applying empty operations leaves document unchanged
    #[test]
    fn empty_ops_sequence(titles in prop::collection::vec(short_title_strategy(), 0..5)) {
        let mut doc = RealmDoc::new();

        for title in &titles {
            doc.add_task(title).unwrap();
        }

        let initial_save = doc.save();

        // Apply empty sync message
        let empty_msg = doc.generate_sync_message();
        doc.apply_sync_message(&empty_msg).unwrap();

        let final_save = doc.save();

        // Task count should be unchanged
        let initial_tasks = RealmDoc::load(&initial_save).unwrap().list_tasks().unwrap().len();
        let final_tasks = RealmDoc::load(&final_save).unwrap().list_tasks().unwrap().len();

        prop_assert_eq!(initial_tasks, final_tasks);
    }

    /// Fork creates an independent copy
    #[test]
    fn fork_creates_independent_copy(title in short_title_strategy()) {
        let mut doc = RealmDoc::new();
        doc.add_task(&title).unwrap();

        let mut forked = doc.fork();

        // Add different tasks to each
        let id1 = doc.add_task("original only").unwrap();
        let id2 = forked.add_task("fork only").unwrap();

        // Original should not have fork's task
        prop_assert!(doc.get_task(&id2).unwrap().is_none());

        // Fork should not have original's new task
        prop_assert!(forked.get_task(&id1).unwrap().is_none());
    }

    /// Random operation sequences do not corrupt the document
    #[test]
    fn random_ops_no_corruption(ops in doc_ops_strategy(20)) {
        let mut doc = RealmDoc::new();
        let mut task_ids: Vec<TaskId> = Vec::new();

        for op in ops {
            match op {
                DocOp::AddTask(title) => {
                    let id = doc.add_task(&title).unwrap();
                    task_ids.push(id);
                }
                DocOp::ToggleTask(idx) => {
                    if !task_ids.is_empty() {
                        let id = &task_ids[idx % task_ids.len()];
                        // Only toggle if task still exists
                        if doc.get_task(id).unwrap().is_some() {
                            doc.toggle_task(id).unwrap();
                        }
                    }
                }
                DocOp::DeleteTask(idx) => {
                    if !task_ids.is_empty() {
                        let id = &task_ids[idx % task_ids.len()];
                        // Delete is idempotent - no error if already deleted
                        let _ = doc.delete_task(id);
                    }
                }
            }
        }

        // Document should still be valid - listing should not panic
        let tasks = doc.list_tasks().unwrap();

        // Should be able to save and load
        let bytes = doc.save();
        let loaded = RealmDoc::load(&bytes).unwrap();
        let loaded_tasks = loaded.list_tasks().unwrap();

        prop_assert_eq!(tasks.len(), loaded_tasks.len());
    }

    /// Merging many forks produces consistent results
    #[test]
    fn multi_fork_merge_consistent(
        num_forks in 2..5usize,
        titles_per_fork in 1..4usize
    ) {
        let mut base = RealmDoc::new();
        let mut forks: Vec<RealmDoc> = (0..num_forks).map(|_| base.fork()).collect();

        // Add tasks to each fork
        for (fork_idx, fork) in forks.iter_mut().enumerate() {
            for task_idx in 0..titles_per_fork {
                fork.add_task(&format!("fork{}-task{}", fork_idx, task_idx)).unwrap();
            }
        }

        // Merge all into first fork
        let mut result = forks[0].fork();
        for fork in forks.iter_mut().skip(1) {
            result.merge(fork).unwrap();
        }

        // Should have all tasks from all forks
        let tasks = result.list_tasks().unwrap();
        prop_assert_eq!(tasks.len(), num_forks * titles_per_fork);
    }
}

// ============================================================================
// Standard Tests (non-property-based)
// ============================================================================

#[test]
fn test_unicode_task_titles() {
    let mut doc = RealmDoc::new();

    // Various Unicode strings
    let titles = [
        "Simple ASCII",
        "Unicode: cafe",
        "Chinese: zhong wen",
        "Emoji: star fire rocket",
        "Arabic: mrhba",
        "Math: 2x + 3 = 7",
        "Mixed: Hello shi jie 123",
    ];

    for title in &titles {
        let id = doc.add_task(title).unwrap();
        let task = doc.get_task(&id).unwrap().unwrap();
        assert_eq!(&task.title, *title);
    }
}

#[test]
fn test_special_characters() {
    let mut doc = RealmDoc::new();

    let titles = [
        "Quotes: \"hello\" 'world'",
        "Backslash: C:\\path\\file",
        "Newline in title\nshould work",
        "Tab\there",
        "Null byte works",
        "JSON-like: {\"key\": \"value\"}",
    ];

    for title in &titles {
        let id = doc.add_task(title).unwrap();
        let task = doc.get_task(&id).unwrap().unwrap();
        assert_eq!(&task.title, *title);
    }
}
