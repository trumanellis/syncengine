//! Core types for Synchronicity Engine

use rand::RngCore;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Unique identifier for a realm (gossip topic)
///
/// A realm represents a shared space where tasks are synchronized
/// between peers using iroh-gossip.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RealmId(pub [u8; 32]);

impl RealmId {
    /// Create a new random RealmId
    pub fn new() -> Self {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Create a RealmId from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the raw bytes of the RealmId
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to base58 string for display/storage
    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }

    /// Parse from base58 string
    pub fn from_base58(s: &str) -> Result<Self, bs58::decode::Error> {
        let bytes = bs58::decode(s).into_vec()?;
        if bytes.len() != 32 {
            return Err(bs58::decode::Error::BufferTooSmall);
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }
}

impl Default for RealmId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RealmId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "realm_{}", bs58::encode(&self.0[..8]).into_string())
    }
}

/// Unique identifier for a task
///
/// Uses ULID for time-ordered unique identifiers that sort lexicographically.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Ulid);

impl TaskId {
    /// Create a new TaskId with current timestamp
    pub fn new() -> Self {
        Self(Ulid::new())
    }

    /// Create a TaskId from a ULID
    pub fn from_ulid(ulid: Ulid) -> Self {
        Self(ulid)
    }

    /// Get the underlying ULID
    pub fn as_ulid(&self) -> &Ulid {
        &self.0
    }

    /// Convert to string representation
    pub fn to_string_repr(&self) -> String {
        self.0.to_string()
    }

    /// Parse from string representation
    pub fn from_string(s: &str) -> Result<Self, ulid::DecodeError> {
        let ulid = Ulid::from_string(s)?;
        Ok(Self(ulid))
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "task_{}", self.0)
    }
}

/// Basic realm information
///
/// Contains metadata about a realm without the full task list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealmInfo {
    /// Unique identifier for the realm
    pub id: RealmId,
    /// Human-readable name
    pub name: String,
    /// Whether this realm is shared with other peers
    pub is_shared: bool,
    /// Unix timestamp of creation
    pub created_at: i64,
}

impl RealmInfo {
    /// Create a new realm with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: RealmId::new(),
            name: name.into(),
            is_shared: false,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

/// Task in a realm
///
/// Represents a single task item that can be synchronized between peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier for the task
    pub id: TaskId,
    /// Task title/description
    pub title: String,
    /// Whether the task is completed
    pub completed: bool,
    /// Unix timestamp of creation
    pub created_at: i64,
    /// Unix timestamp of completion (if completed)
    pub completed_at: Option<i64>,
}

impl Task {
    /// Create a new task with the given title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: TaskId::new(),
            title: title.into(),
            completed: false,
            created_at: chrono::Utc::now().timestamp(),
            completed_at: None,
        }
    }

    /// Mark the task as completed
    pub fn complete(&mut self) {
        if !self.completed {
            self.completed = true;
            self.completed_at = Some(chrono::Utc::now().timestamp());
        }
    }

    /// Mark the task as incomplete
    pub fn uncomplete(&mut self) {
        self.completed = false;
        self.completed_at = None;
    }

    /// Toggle the completion state
    pub fn toggle(&mut self) {
        if self.completed {
            self.uncomplete();
        } else {
            self.complete();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realm_id_new() {
        let realm1 = RealmId::new();
        let realm2 = RealmId::new();
        // Should generate different IDs
        assert_ne!(realm1, realm2);
    }

    #[test]
    fn test_realm_id_display() {
        let realm = RealmId::new();
        let display = format!("{}", realm);
        assert!(display.starts_with("realm_"));
    }

    #[test]
    fn test_realm_id_base58_roundtrip() {
        let realm = RealmId::new();
        let encoded = realm.to_base58();
        let decoded = RealmId::from_base58(&encoded).expect("Failed to decode");
        assert_eq!(realm, decoded);
    }

    #[test]
    fn test_task_id_new() {
        let task1 = TaskId::new();
        let task2 = TaskId::new();
        // Should generate different IDs
        assert_ne!(task1, task2);
    }

    #[test]
    fn test_task_id_display() {
        let task = TaskId::new();
        let display = format!("{}", task);
        assert!(display.starts_with("task_"));
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new("Test task");
        assert_eq!(task.title, "Test task");
        assert!(!task.completed);
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_task_complete() {
        let mut task = Task::new("Test task");
        task.complete();
        assert!(task.completed);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_task_uncomplete() {
        let mut task = Task::new("Test task");
        task.complete();
        task.uncomplete();
        assert!(!task.completed);
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_task_toggle() {
        let mut task = Task::new("Test task");
        assert!(!task.completed);

        task.toggle();
        assert!(task.completed);

        task.toggle();
        assert!(!task.completed);
    }

    #[test]
    fn test_realm_info_new() {
        let realm = RealmInfo::new("My Realm");
        assert_eq!(realm.name, "My Realm");
        assert!(!realm.is_shared);
    }
}
