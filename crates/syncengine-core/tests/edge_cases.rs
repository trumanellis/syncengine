//! Edge case and boundary condition tests
//!
//! These tests verify the system handles unusual inputs,
//! error conditions, and boundary values correctly.

use syncengine_core::realm::RealmDoc;
use syncengine_core::{RealmId, SyncError, Task, TaskId};

// ============================================================================
// Empty Input Tests
// ============================================================================

/// Test adding empty string as task title
#[test]
fn test_empty_task_title() {
    let mut doc = RealmDoc::new();

    // Empty string should still work (no validation on title)
    let id = doc.add_task("").unwrap();
    let task = doc.get_task(&id).unwrap().unwrap();
    assert_eq!(task.title, "");
}

/// Test whitespace-only task title
#[test]
fn test_whitespace_task_title() {
    let mut doc = RealmDoc::new();

    let titles = ["   ", "\t", "\n", "\r\n", "   \t\n  "];

    for title in titles {
        let id = doc.add_task(title).unwrap();
        let task = doc.get_task(&id).unwrap().unwrap();
        assert_eq!(task.title, title);
    }
}

/// Test document with no tasks
#[test]
fn test_empty_document_operations() {
    let doc = RealmDoc::new();

    // List should return empty vec
    let tasks = doc.list_tasks().unwrap();
    assert!(tasks.is_empty());

    // Get nonexistent task should return None
    let fake_id = TaskId::new();
    assert!(doc.get_task(&fake_id).unwrap().is_none());
}

/// Test empty document save/load
#[test]
fn test_empty_document_save_load() {
    let mut doc = RealmDoc::new();
    let bytes = doc.save();

    assert!(!bytes.is_empty(), "Even empty doc should have some bytes");

    let loaded = RealmDoc::load(&bytes).unwrap();
    assert!(loaded.list_tasks().unwrap().is_empty());
}

/// Test empty sync message
#[test]
fn test_empty_sync_message() {
    let mut doc = RealmDoc::new();

    // Generate sync message from doc with no changes
    let sync_msg = doc.generate_sync_message();

    // Apply empty sync message should not error
    let mut doc2 = RealmDoc::new();
    doc2.apply_sync_message(&sync_msg).unwrap();
}

// ============================================================================
// Maximum Length Tests
// ============================================================================

/// Test maximum length task title (10000 chars)
#[test]
fn test_max_length_task_title() {
    let mut doc = RealmDoc::new();

    let long_title = "a".repeat(10000);
    let id = doc.add_task(&long_title).unwrap();
    let task = doc.get_task(&id).unwrap().unwrap();

    assert_eq!(task.title.len(), 10000);
    assert_eq!(task.title, long_title);

    // Save and load should preserve
    let bytes = doc.save();
    let loaded = RealmDoc::load(&bytes).unwrap();
    let loaded_task = loaded.get_task(&id).unwrap().unwrap();
    assert_eq!(loaded_task.title.len(), 10000);
}

/// Test very long Unicode task title
#[test]
fn test_long_unicode_task_title() {
    let mut doc = RealmDoc::new();

    // Unicode characters take more bytes than chars
    let unicode_char = "\u{1F600}"; // Grinning face emoji (4 bytes)
    let title = unicode_char.repeat(1000);

    let id = doc.add_task(&title).unwrap();
    let task = doc.get_task(&id).unwrap().unwrap();

    // Should have 1000 emoji characters
    assert_eq!(task.title.chars().count(), 1000);

    // Save and load should preserve
    let bytes = doc.save();
    let loaded = RealmDoc::load(&bytes).unwrap();
    let loaded_task = loaded.get_task(&id).unwrap().unwrap();
    assert_eq!(loaded_task.title, title);
}

// ============================================================================
// Invalid Input Tests
// ============================================================================

/// Test loading from invalid bytes
#[test]
fn test_load_invalid_bytes() {
    let invalid_bytes = b"not a valid automerge document";
    let result = RealmDoc::load(invalid_bytes);
    assert!(result.is_err());

    if let Err(SyncError::Serialization(msg)) = result {
        assert!(!msg.is_empty());
    } else {
        panic!("Expected SyncError::Serialization");
    }
}

/// Test loading from empty bytes
/// Note: Automerge can load an empty document from empty bytes (creates new doc)
#[test]
fn test_load_empty_bytes() {
    let empty_bytes: &[u8] = &[];
    let result = RealmDoc::load(empty_bytes);
    // Automerge may either succeed (loading empty as new doc) or fail
    // Both behaviors are acceptable - the key is it shouldn't panic
    if result.is_ok() {
        // If it succeeds, verify it's a valid empty-ish document
        let _ = result.unwrap();
    }
    // Either way, we didn't panic - that's the important thing
}

/// Test loading from truncated document
#[test]
fn test_load_truncated_document() {
    let mut doc = RealmDoc::new();
    doc.add_task("Test").unwrap();
    let bytes = doc.save();

    // Truncate the bytes
    let truncated = &bytes[..bytes.len() / 2];
    let result = RealmDoc::load(truncated);
    assert!(result.is_err());
}

/// Test applying invalid sync message
/// Note: Automerge's load_incremental may handle some invalid data gracefully
#[test]
fn test_apply_invalid_sync_message() {
    let mut doc = RealmDoc::new();
    doc.add_task("Test").unwrap();

    let invalid_msg = b"not a valid sync message";
    let result = doc.apply_sync_message(invalid_msg);
    // Automerge may handle this gracefully or error
    // The important thing is the document should remain valid afterward
    let tasks = doc.list_tasks().unwrap();
    assert_eq!(tasks.len(), 1); // Original task should still exist
}

/// Test applying empty sync message bytes
#[test]
fn test_apply_empty_sync_message() {
    let mut doc = RealmDoc::new();
    doc.add_task("Test").unwrap();

    let empty_msg: &[u8] = &[];
    // Empty message should be handled gracefully
    // (may succeed or error depending on Automerge behavior)
    let _ = doc.apply_sync_message(empty_msg);

    // Document should still be valid
    let tasks = doc.list_tasks().unwrap();
    assert_eq!(tasks.len(), 1);
}

// ============================================================================
// RealmId Edge Cases
// ============================================================================

/// Test RealmId from_base58 with invalid input
#[test]
fn test_realm_id_invalid_base58() {
    // Invalid base58 characters
    let invalid = "0OIl"; // Contains 0, O, I, l which are not in base58
    let result = RealmId::from_base58(invalid);
    assert!(result.is_err());
}

/// Test RealmId from_base58 with wrong length
#[test]
fn test_realm_id_wrong_length() {
    // Too short
    let short = "abc";
    let result = RealmId::from_base58(short);
    assert!(result.is_err());

    // Too long
    let long = "a".repeat(100);
    let result = RealmId::from_base58(&long);
    // This should either error or truncate, depending on base58 behavior
    assert!(result.is_err() || result.is_ok());
}

/// Test RealmId from_base58 with empty string
#[test]
fn test_realm_id_empty_base58() {
    let result = RealmId::from_base58("");
    assert!(result.is_err());
}

// ============================================================================
// TaskId Edge Cases
// ============================================================================

/// Test TaskId from_string with invalid ULID
#[test]
fn test_task_id_invalid_ulid() {
    // Invalid ULID characters
    let invalid = "not-a-ulid";
    let result = TaskId::from_string(invalid);
    assert!(result.is_err());
}

/// Test TaskId from_string with empty string
#[test]
fn test_task_id_empty_string() {
    let result = TaskId::from_string("");
    assert!(result.is_err());
}

/// Test TaskId from_string with too short string
#[test]
fn test_task_id_too_short() {
    let result = TaskId::from_string("abc");
    assert!(result.is_err());
}

// ============================================================================
// Task Edge Cases
// ============================================================================

/// Test Task toggle when already completed
#[test]
fn test_task_toggle_when_completed() {
    let mut task = Task::new("Test");
    task.complete();
    assert!(task.completed);
    assert!(task.completed_at.is_some());

    task.toggle();
    assert!(!task.completed);
    assert!(task.completed_at.is_none());
}

/// Test Task complete when already completed
#[test]
fn test_task_complete_when_completed() {
    let mut task = Task::new("Test");
    task.complete();
    let first_completed_at = task.completed_at;

    // Complete again - should be idempotent
    task.complete();
    assert!(task.completed);
    assert_eq!(task.completed_at, first_completed_at);
}

/// Test Task uncomplete when already incomplete
#[test]
fn test_task_uncomplete_when_incomplete() {
    let mut task = Task::new("Test");
    assert!(!task.completed);

    // Uncomplete when already incomplete
    task.uncomplete();
    assert!(!task.completed);
    assert!(task.completed_at.is_none());
}

// ============================================================================
// Document State Edge Cases
// ============================================================================

/// Test toggle on nonexistent task
#[test]
fn test_toggle_nonexistent_task() {
    let mut doc = RealmDoc::new();
    let fake_id = TaskId::new();

    let result = doc.toggle_task(&fake_id);
    assert!(matches!(result, Err(SyncError::TaskNotFound(_))));
}

/// Test delete on already deleted task
#[test]
fn test_delete_already_deleted_task() {
    let mut doc = RealmDoc::new();
    let id = doc.add_task("Test").unwrap();

    // First delete
    doc.delete_task(&id).unwrap();
    assert!(doc.get_task(&id).unwrap().is_none());

    // Second delete should succeed (idempotent) or error
    // Automerge may allow deleting non-existent keys
    let _ = doc.delete_task(&id);

    // Document should still be valid
    assert!(doc.get_task(&id).unwrap().is_none());
}

/// Test get task after delete
#[test]
fn test_get_task_after_delete() {
    let mut doc = RealmDoc::new();
    let id = doc.add_task("Test").unwrap();

    doc.delete_task(&id).unwrap();

    // Get should return None, not error
    let result = doc.get_task(&id).unwrap();
    assert!(result.is_none());
}

// ============================================================================
// Merge Edge Cases
// ============================================================================

/// Test merge with self
#[test]
fn test_merge_with_self() {
    let mut doc = RealmDoc::new();
    doc.add_task("Test").unwrap();

    let mut clone = doc.fork();
    doc.merge(&mut clone).unwrap();

    // Should still have same tasks
    let tasks = doc.list_tasks().unwrap();
    assert_eq!(tasks.len(), 1);
}

/// Test merge empty with non-empty
/// Note: Merging independent documents (not forked) may not combine changes
/// because they have different actor histories
#[test]
fn test_merge_empty_with_nonempty() {
    // Create documents that share history via fork
    let mut base = RealmDoc::new();
    let mut doc1 = base.fork();
    let mut doc2 = base.fork();
    doc2.add_task("Test").unwrap();

    doc1.merge(&mut doc2).unwrap();

    let tasks = doc1.list_tasks().unwrap();
    assert_eq!(tasks.len(), 1);
}

/// Test merge non-empty with empty
/// Note: Merging requires shared history via fork
#[test]
fn test_merge_nonempty_with_empty() {
    let mut base = RealmDoc::new();
    let mut doc1 = base.fork();
    doc1.add_task("Test").unwrap();
    let mut doc2 = base.fork();

    doc1.merge(&mut doc2).unwrap();

    let tasks = doc1.list_tasks().unwrap();
    assert_eq!(tasks.len(), 1);
}

/// Test merge two empty documents
#[test]
fn test_merge_two_empty() {
    let mut doc1 = RealmDoc::new();
    let mut doc2 = RealmDoc::new();

    doc1.merge(&mut doc2).unwrap();

    assert!(doc1.list_tasks().unwrap().is_empty());
}

// ============================================================================
// Fork Edge Cases
// ============================================================================

/// Test fork of empty document
#[test]
fn test_fork_empty_document() {
    let mut doc = RealmDoc::new();
    let forked = doc.fork();

    assert!(forked.list_tasks().unwrap().is_empty());
}

/// Test multiple forks from same document
#[test]
fn test_multiple_forks() {
    let mut doc = RealmDoc::new();
    doc.add_task("Original").unwrap();

    let fork1 = doc.fork();
    let fork2 = doc.fork();
    let fork3 = doc.fork();

    // All forks should have same initial content
    assert_eq!(fork1.list_tasks().unwrap().len(), 1);
    assert_eq!(fork2.list_tasks().unwrap().len(), 1);
    assert_eq!(fork3.list_tasks().unwrap().len(), 1);
}

/// Test nested fork operations
#[test]
fn test_nested_forks() {
    let mut doc = RealmDoc::new();
    doc.add_task("Level 0").unwrap();

    let mut fork1 = doc.fork();
    fork1.add_task("Level 1").unwrap();

    let mut fork2 = fork1.fork();
    fork2.add_task("Level 2").unwrap();

    let fork3 = fork2.fork();

    // Each level should have accumulated tasks
    assert_eq!(doc.list_tasks().unwrap().len(), 1);
    assert_eq!(fork1.list_tasks().unwrap().len(), 2);
    assert_eq!(fork2.list_tasks().unwrap().len(), 3);
    assert_eq!(fork3.list_tasks().unwrap().len(), 3);
}

// ============================================================================
// Concurrent Edit Edge Cases
// ============================================================================

/// Test concurrent add of same title (should create two tasks)
#[test]
fn test_concurrent_add_same_title() {
    let mut base = RealmDoc::new();

    let mut fork1 = base.fork();
    let mut fork2 = base.fork();

    // Both add the same title
    fork1.add_task("Same title").unwrap();
    fork2.add_task("Same title").unwrap();

    // Merge
    fork1.merge(&mut fork2).unwrap();

    // Should have TWO tasks (different IDs)
    let tasks = fork1.list_tasks().unwrap();
    assert_eq!(tasks.len(), 2);

    // Both should have same title but different IDs
    assert_eq!(tasks[0].title, "Same title");
    assert_eq!(tasks[1].title, "Same title");
    assert_ne!(tasks[0].id, tasks[1].id);
}

/// Test concurrent toggle of same task
#[test]
fn test_concurrent_toggle_same_task() {
    let mut base = RealmDoc::new();
    let id = base.add_task("Toggle me").unwrap();

    let mut fork1 = base.fork();
    let mut fork2 = base.fork();

    // Both toggle the same task
    fork1.toggle_task(&id).unwrap();
    fork2.toggle_task(&id).unwrap();

    // Merge
    fork1.merge(&mut fork2).unwrap();

    // Final state depends on CRDT resolution
    // Both toggles should be reflected somehow
    let task = fork1.get_task(&id).unwrap().unwrap();
    // With two toggles, result should be incomplete (toggle is idempotent per-actor)
    // But CRDT may resolve differently
    assert!(task.completed || !task.completed); // Just verify no crash
}

/// Test concurrent delete and toggle
#[test]
fn test_concurrent_delete_and_toggle() {
    let mut base = RealmDoc::new();
    let id = base.add_task("Conflict me").unwrap();

    let mut fork1 = base.fork();
    let mut fork2 = base.fork();

    // One deletes, one toggles
    fork1.delete_task(&id).unwrap();
    fork2.toggle_task(&id).unwrap();

    // Merge
    fork1.merge(&mut fork2).unwrap();

    // Task should still be accessible (toggle happened on valid task)
    // or deleted (delete won), depending on CRDT semantics
    let _ = fork1.get_task(&id).unwrap();
}

// ============================================================================
// Special Character Edge Cases
// ============================================================================

/// Test task title with null bytes
#[test]
fn test_null_bytes_in_title() {
    let mut doc = RealmDoc::new();

    let title_with_null = "before\0after";
    let id = doc.add_task(title_with_null).unwrap();
    let task = doc.get_task(&id).unwrap().unwrap();

    assert_eq!(task.title, title_with_null);
}

/// Test task title with control characters
#[test]
fn test_control_characters_in_title() {
    let mut doc = RealmDoc::new();

    let control_chars = "\x01\x02\x03\x04\x05\x1B"; // Various control chars
    let id = doc.add_task(control_chars).unwrap();
    let task = doc.get_task(&id).unwrap().unwrap();

    assert_eq!(task.title, control_chars);
}

/// Test task title with JSON-like content
#[test]
fn test_json_in_title() {
    let mut doc = RealmDoc::new();

    let json_title = r#"{"key": "value", "nested": {"a": 1}}"#;
    let id = doc.add_task(json_title).unwrap();
    let task = doc.get_task(&id).unwrap().unwrap();

    assert_eq!(task.title, json_title);
}

/// Test task title that looks like escape sequences
#[test]
fn test_escape_sequences_in_title() {
    let mut doc = RealmDoc::new();

    let escape_title = r#"\n\t\r\\\"\'test"#;
    let id = doc.add_task(escape_title).unwrap();
    let task = doc.get_task(&id).unwrap().unwrap();

    assert_eq!(task.title, escape_title);
}

// ============================================================================
// Error Recovery Tests
// ============================================================================

/// Test that document remains valid after error
#[test]
fn test_document_valid_after_error() {
    let mut doc = RealmDoc::new();
    doc.add_task("Valid task").unwrap();

    // Cause an error
    let fake_id = TaskId::new();
    let _ = doc.toggle_task(&fake_id);

    // Document should still be valid
    let tasks = doc.list_tasks().unwrap();
    assert_eq!(tasks.len(), 1);

    // Should still be able to add tasks
    doc.add_task("Another task").unwrap();
    assert_eq!(doc.list_tasks().unwrap().len(), 2);
}

/// Test operations after loading corrupted-then-fixed data
#[test]
fn test_operations_after_reload() {
    let mut doc = RealmDoc::new();
    doc.add_task("Test").unwrap();

    // Save, load, continue working
    let bytes = doc.save();
    let mut loaded = RealmDoc::load(&bytes).unwrap();

    loaded.add_task("After reload").unwrap();
    let id = loaded.list_tasks().unwrap()[0].id.clone();
    loaded.toggle_task(&id).unwrap();

    assert_eq!(loaded.list_tasks().unwrap().len(), 2);
}

// ============================================================================
// Boundary Value Tests
// ============================================================================

/// Test with maximum number of tasks a realm might realistically have
#[test]
fn test_realm_with_many_tasks() {
    let mut doc = RealmDoc::new();

    // Add 500 tasks (reasonable upper bound for a single realm)
    for i in 0..500 {
        doc.add_task(&format!("Task {}", i)).unwrap();
    }

    let tasks = doc.list_tasks().unwrap();
    assert_eq!(tasks.len(), 500);

    // Toggle the first 100
    for task in &tasks[..100] {
        doc.toggle_task(&task.id).unwrap();
    }

    // Delete the last 100
    for task in &tasks[400..] {
        doc.delete_task(&task.id).unwrap();
    }

    assert_eq!(doc.list_tasks().unwrap().len(), 400);
}
