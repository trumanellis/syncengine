//! Automerge document wrapper for realm task management
//!
//! RealmDoc wraps an Automerge document and provides CRUD operations for tasks.
//! It handles serialization, merging, and incremental sync message generation.

use automerge::{transaction::Transactable, AutoCommit, ObjType, ReadDoc, ROOT};

use crate::{SyncError, Task, TaskId};

/// Automerge document wrapper for a realm's tasks
///
/// RealmDoc provides a high-level API for managing tasks within a realm.
/// All operations are automatically tracked by Automerge for CRDT-based
/// conflict resolution during sync.
///
/// # Example
///
/// ```
/// use syncengine_core::realm::RealmDoc;
///
/// let mut doc = RealmDoc::new();
/// let task_id = doc.add_task("Build solar dehydrator").unwrap();
/// doc.toggle_task(&task_id).unwrap();
///
/// let tasks = doc.list_tasks().unwrap();
/// assert_eq!(tasks.len(), 1);
/// assert!(tasks[0].completed);
/// ```
pub struct RealmDoc {
    doc: AutoCommit,
}

impl RealmDoc {
    /// Create a new empty realm document
    ///
    /// Initializes an Automerge document with an empty tasks map.
    pub fn new() -> Self {
        let mut doc = AutoCommit::new();
        // Initialize with tasks map at root
        doc.put_object(ROOT, "tasks", ObjType::Map).unwrap();
        Self { doc }
    }

    /// Load a realm document from saved bytes
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Serialization` if the bytes are not a valid Automerge document.
    pub fn load(data: &[u8]) -> Result<Self, SyncError> {
        let doc = AutoCommit::load(data).map_err(|e| SyncError::Serialization(e.to_string()))?;
        Ok(Self { doc })
    }

    /// Save the document to bytes
    ///
    /// Returns the full document state as bytes that can be stored or transmitted.
    pub fn save(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    /// Fork the document for concurrent editing
    ///
    /// Creates an independent copy of the document that can be edited
    /// separately and later merged back.
    pub fn fork(&mut self) -> Self {
        Self {
            doc: self.doc.fork(),
        }
    }

    /// Merge another document into this one
    ///
    /// Combines changes from another RealmDoc using Automerge's CRDT
    /// conflict resolution. The merge is commutative - the result is
    /// the same regardless of merge order.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Serialization` if the merge fails.
    pub fn merge(&mut self, other: &mut RealmDoc) -> Result<(), SyncError> {
        self.doc
            .merge(&mut other.doc)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;
        Ok(())
    }

    /// Add a new task to the realm
    ///
    /// Creates a new task with the given title and adds it to the document.
    /// Returns the TaskId of the created task.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Serialization` if the task cannot be stored.
    pub fn add_task(&mut self, title: &str) -> Result<TaskId, SyncError> {
        let task = Task::new(title);
        let task_id = task.id.clone();

        let tasks = self
            .doc
            .get(ROOT, "tasks")
            .map_err(|e| SyncError::Serialization(e.to_string()))?
            .ok_or_else(|| SyncError::Serialization("tasks map not found".into()))?;

        let (_, tasks_obj_id) = tasks;

        // Store task as JSON string in the map
        let task_json =
            serde_json::to_string(&task).map_err(|e| SyncError::Serialization(e.to_string()))?;

        self.doc
            .put(&tasks_obj_id, task_id.to_string(), task_json)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;

        Ok(task_id)
    }

    /// Add a new "quest" (rich task with metadata) to the document
    ///
    /// Returns the TaskId of the newly created quest.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Serialization` if the quest cannot be serialized.
    pub fn add_quest(
        &mut self,
        title: &str,
        subtitle: Option<String>,
        description: &str,
    ) -> Result<TaskId, SyncError> {
        let task = Task::new_quest(title, subtitle, description);
        let task_id = task.id.clone();

        let tasks = self
            .doc
            .get(ROOT, "tasks")
            .map_err(|e| SyncError::Serialization(e.to_string()))?
            .ok_or_else(|| SyncError::Serialization("tasks map not found".into()))?;

        let (_, tasks_obj_id) = tasks;

        // Store task as JSON string in the map
        let task_json =
            serde_json::to_string(&task).map_err(|e| SyncError::Serialization(e.to_string()))?;

        self.doc
            .put(&tasks_obj_id, task_id.to_string(), task_json)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;

        Ok(task_id)
    }

    /// Get a task by its ID
    ///
    /// Returns `None` if the task does not exist.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Serialization` if the task data is corrupted.
    pub fn get_task(&self, id: &TaskId) -> Result<Option<Task>, SyncError> {
        let tasks = self
            .doc
            .get(ROOT, "tasks")
            .map_err(|e| SyncError::Serialization(e.to_string()))?;

        if let Some((_, tasks_obj_id)) = tasks {
            if let Some((value, _)) = self
                .doc
                .get(&tasks_obj_id, id.to_string())
                .map_err(|e| SyncError::Serialization(e.to_string()))?
            {
                let json = value
                    .to_str()
                    .ok_or_else(|| SyncError::Serialization("task value is not a string".into()))?;
                let task: Task = serde_json::from_str(json)
                    .map_err(|e| SyncError::Serialization(e.to_string()))?;
                return Ok(Some(task));
            }
        }
        Ok(None)
    }

    /// List all tasks in the realm
    ///
    /// Returns tasks sorted by creation time (oldest first).
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Serialization` if any task data is corrupted.
    pub fn list_tasks(&self) -> Result<Vec<Task>, SyncError> {
        let mut tasks = Vec::new();

        if let Some((_, tasks_obj_id)) = self
            .doc
            .get(ROOT, "tasks")
            .map_err(|e| SyncError::Serialization(e.to_string()))?
        {
            let keys = self.doc.keys(&tasks_obj_id);
            for key in keys {
                if let Some((value, _)) = self
                    .doc
                    .get(&tasks_obj_id, &key)
                    .map_err(|e| SyncError::Serialization(e.to_string()))?
                {
                    if let Some(json) = value.to_str() {
                        if let Ok(task) = serde_json::from_str::<Task>(json) {
                            tasks.push(task);
                        }
                    }
                }
            }
        }

        // Sort by created_at for consistent ordering
        tasks.sort_by_key(|t| t.created_at);
        Ok(tasks)
    }

    /// Toggle the completion state of a task
    ///
    /// # Errors
    ///
    /// Returns `SyncError::TaskNotFound` if the task does not exist.
    /// Returns `SyncError::Serialization` if the operation fails.
    pub fn toggle_task(&mut self, id: &TaskId) -> Result<(), SyncError> {
        let mut task = self
            .get_task(id)?
            .ok_or_else(|| SyncError::TaskNotFound(id.to_string()))?;

        task.toggle();

        let tasks = self
            .doc
            .get(ROOT, "tasks")
            .map_err(|e| SyncError::Serialization(e.to_string()))?
            .ok_or_else(|| SyncError::Serialization("tasks map not found".into()))?;

        let (_, tasks_obj_id) = tasks;

        let task_json =
            serde_json::to_string(&task).map_err(|e| SyncError::Serialization(e.to_string()))?;

        self.doc
            .put(&tasks_obj_id, id.to_string(), task_json)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;

        Ok(())
    }

    /// Delete a task from the realm
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Serialization` if the operation fails.
    pub fn delete_task(&mut self, id: &TaskId) -> Result<(), SyncError> {
        let tasks = self
            .doc
            .get(ROOT, "tasks")
            .map_err(|e| SyncError::Serialization(e.to_string()))?
            .ok_or_else(|| SyncError::Serialization("tasks map not found".into()))?;

        let (_, tasks_obj_id) = tasks;

        self.doc
            .delete(&tasks_obj_id, id.to_string())
            .map_err(|e| SyncError::Serialization(e.to_string()))?;

        Ok(())
    }

    /// Generate an incremental sync message
    ///
    /// Returns the changes since the last save, suitable for
    /// sending to peers for incremental synchronization.
    pub fn generate_sync_message(&mut self) -> Vec<u8> {
        self.doc.save_incremental()
    }

    /// Apply a sync message from a peer
    ///
    /// Loads incremental changes from another peer into this document.
    ///
    /// # Errors
    ///
    /// Returns `SyncError::Serialization` if the sync message is invalid.
    pub fn apply_sync_message(&mut self, data: &[u8]) -> Result<(), SyncError> {
        self.doc
            .load_incremental(data)
            .map_err(|e| SyncError::Serialization(e.to_string()))?;
        Ok(())
    }

    /// Get the document heads (change hashes)
    ///
    /// Returns the current heads of the document DAG, useful for
    /// determining sync state between peers.
    pub fn heads(&mut self) -> Vec<automerge::ChangeHash> {
        self.doc.get_heads()
    }
}

impl Default for RealmDoc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realm_doc_new() {
        let doc = RealmDoc::new();
        assert!(doc.list_tasks().unwrap().is_empty());
    }

    #[test]
    fn test_add_task() {
        let mut doc = RealmDoc::new();
        let id = doc.add_task("Test task").unwrap();
        let task = doc.get_task(&id).unwrap().unwrap();
        assert_eq!(task.title, "Test task");
        assert!(!task.completed);
    }

    #[test]
    fn test_list_tasks() {
        let mut doc = RealmDoc::new();
        doc.add_task("Task 1").unwrap();
        doc.add_task("Task 2").unwrap();
        let tasks = doc.list_tasks().unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn test_toggle_task() {
        let mut doc = RealmDoc::new();
        let id = doc.add_task("Test").unwrap();

        // Toggle to completed
        doc.toggle_task(&id).unwrap();
        let task = doc.get_task(&id).unwrap().unwrap();
        assert!(task.completed);

        // Toggle back to incomplete
        doc.toggle_task(&id).unwrap();
        let task = doc.get_task(&id).unwrap().unwrap();
        assert!(!task.completed);
    }

    #[test]
    fn test_delete_task() {
        let mut doc = RealmDoc::new();
        let id = doc.add_task("Test").unwrap();
        doc.delete_task(&id).unwrap();
        assert!(doc.get_task(&id).unwrap().is_none());
    }

    #[test]
    fn test_save_and_load() {
        let mut doc = RealmDoc::new();
        doc.add_task("Persisted task").unwrap();
        let bytes = doc.save();

        let loaded = RealmDoc::load(&bytes).unwrap();
        let tasks = loaded.list_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Persisted task");
    }

    #[test]
    fn test_fork_and_merge() {
        let mut doc1 = RealmDoc::new();
        doc1.add_task("Original").unwrap();

        let mut doc2 = doc1.fork();
        doc2.add_task("From fork").unwrap();

        doc1.merge(&mut doc2).unwrap();
        let tasks = doc1.list_tasks().unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn test_concurrent_edits_merge() {
        let mut doc1 = RealmDoc::new();
        let mut doc2 = doc1.fork();

        doc1.add_task("From doc1").unwrap();
        doc2.add_task("From doc2").unwrap();

        doc1.merge(&mut doc2).unwrap();

        let tasks = doc1.list_tasks().unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn test_merge_is_commutative() {
        // Create base document
        let mut base = RealmDoc::new();
        base.add_task("Base task").unwrap();

        // Fork to create two branches
        let mut doc1 = base.fork();
        let mut doc2 = base.fork();

        // Make different edits on each branch
        doc1.add_task("From branch 1").unwrap();
        doc2.add_task("From branch 2").unwrap();

        // Merge in both directions
        let mut result1 = doc1.fork();
        let mut result2 = doc2.fork();

        result1.merge(&mut doc2.fork()).unwrap();
        result2.merge(&mut doc1.fork()).unwrap();

        // Both should have the same tasks
        let tasks1 = result1.list_tasks().unwrap();
        let tasks2 = result2.list_tasks().unwrap();

        assert_eq!(tasks1.len(), tasks2.len());
        assert_eq!(tasks1.len(), 3);
    }

    #[test]
    fn test_get_nonexistent_task() {
        let doc = RealmDoc::new();
        let fake_id = TaskId::new();
        let result = doc.get_task(&fake_id).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_toggle_nonexistent_task() {
        let mut doc = RealmDoc::new();
        let fake_id = TaskId::new();
        let result = doc.toggle_task(&fake_id);
        assert!(matches!(result, Err(SyncError::TaskNotFound(_))));
    }

    #[test]
    fn test_incremental_sync() {
        let mut doc1 = RealmDoc::new();
        doc1.add_task("Initial task").unwrap();

        // Save full state
        let full_save = doc1.save();

        // Load into new doc
        let mut doc2 = RealmDoc::load(&full_save).unwrap();

        // Add more tasks to doc1
        doc1.add_task("New task").unwrap();

        // Generate incremental sync message
        let sync_msg = doc1.generate_sync_message();

        // Apply to doc2
        doc2.apply_sync_message(&sync_msg).unwrap();

        // Both should have same tasks
        let tasks1 = doc1.list_tasks().unwrap();
        let tasks2 = doc2.list_tasks().unwrap();

        assert_eq!(tasks1.len(), 2);
        assert_eq!(tasks2.len(), 2);
    }

    #[test]
    fn test_heads_change_on_edit() {
        let mut doc = RealmDoc::new();
        let heads1 = doc.heads();

        doc.add_task("Task").unwrap();
        let heads2 = doc.heads();

        // Heads should change after edit
        assert_ne!(heads1, heads2);
    }

    #[test]
    fn test_default_impl() {
        let doc: RealmDoc = Default::default();
        assert!(doc.list_tasks().unwrap().is_empty());
    }
}
