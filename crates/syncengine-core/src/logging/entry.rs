//! Log entry types for JSONL-based logging.
//!
//! Each log entry is a self-contained JSON object that can be appended
//! to a JSONL file without risk of corruption from concurrent writes.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single log entry in JSONL format.
///
/// Each entry is independent and self-contained, making JSONL files
/// resilient to concurrent appends from multiple instances.
///
/// Named `JsonLogEntry` to avoid conflict with `profile::JsonLogEntry`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonLogEntry {
    /// ISO 8601 timestamp (e.g., "2026-01-21T14:30:45.123Z")
    pub ts: String,

    /// Log level: trace, debug, info, warn, error
    pub level: String,

    /// Instance name (e.g., "love", "joy", "peace")
    pub instance: String,

    /// Module path / target (e.g., "syncengine_core::sync::gossip")
    pub target: String,

    /// Human-readable message
    pub msg: String,

    /// Optional structured fields (span data, custom fields)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Value>,

    /// Optional span name if this entry is from within a span
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<String>,
}

impl JsonLogEntry {
    /// Create a new log entry with the current timestamp.
    pub fn new(
        level: impl Into<String>,
        instance: impl Into<String>,
        target: impl Into<String>,
        msg: impl Into<String>,
    ) -> Self {
        Self {
            ts: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            level: level.into(),
            instance: instance.into(),
            target: target.into(),
            msg: msg.into(),
            fields: None,
            span: None,
        }
    }

    /// Add structured fields to the entry.
    pub fn with_fields(mut self, fields: Value) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Add span name to the entry.
    pub fn with_span(mut self, span: impl Into<String>) -> Self {
        self.span = Some(span.into());
        self
    }

    /// Serialize to a single JSON line (no trailing newline).
    pub fn to_json_line(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parse from a JSON line.
    pub fn from_json_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line)
    }
}

/// Session metadata written at the start of a logging session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Session ID (timestamp-based)
    pub session_id: String,

    /// When the session started
    pub started_at: String,

    /// List of instances participating in this session
    pub instances: Vec<String>,

    /// Working directory
    pub cwd: Option<String>,

    /// Git branch (if in a git repo)
    pub git_branch: Option<String>,

    /// Git commit hash (short)
    pub git_commit: Option<String>,
}

impl SessionMetadata {
    /// Create new session metadata.
    pub fn new(instances: Vec<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            session_id: now.format("%Y-%m-%dT%H-%M-%S").to_string(),
            started_at: now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            instances,
            cwd: std::env::current_dir()
                .ok()
                .map(|p| p.display().to_string()),
            git_branch: None,
            git_commit: None,
        }
    }

    /// Add git information.
    pub fn with_git(mut self, branch: Option<String>, commit: Option<String>) -> Self {
        self.git_branch = branch;
        self.git_commit = commit;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_serialization() {
        let entry = JsonLogEntry::new("info", "love", "syncengine::sync", "Connected to peer");

        let json = entry.to_json_line().unwrap();
        assert!(json.contains("\"level\":\"info\""));
        assert!(json.contains("\"instance\":\"love\""));
        assert!(json.contains("\"msg\":\"Connected to peer\""));

        // Roundtrip
        let parsed = JsonLogEntry::from_json_line(&json).unwrap();
        assert_eq!(parsed.level, "info");
        assert_eq!(parsed.instance, "love");
        assert_eq!(parsed.msg, "Connected to peer");
    }

    #[test]
    fn test_log_entry_with_fields() {
        let entry = JsonLogEntry::new("debug", "joy", "syncengine::gossip", "Received message")
            .with_fields(serde_json::json!({
                "peer_id": "abc123",
                "message_size": 1024
            }));

        let json = entry.to_json_line().unwrap();
        assert!(json.contains("\"peer_id\":\"abc123\""));
        assert!(json.contains("\"message_size\":1024"));
    }

    #[test]
    fn test_session_metadata() {
        let meta = SessionMetadata::new(vec!["love".into(), "joy".into()]);

        let json = serde_json::to_string_pretty(&meta).unwrap();
        assert!(json.contains("\"love\""));
        assert!(json.contains("\"joy\""));
    }
}
