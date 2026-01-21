//! Chat message types for display and storage
//!
//! This module provides the [`ChatMessage`] struct which represents
//! a decrypted, display-ready chat message extracted from packet envelopes.

use serde::{Deserialize, Serialize};

/// A decrypted chat message ready for display.
///
/// This is a high-level abstraction over [`PacketEnvelope`] containing
/// a [`PacketPayload::DirectMessage`]. It includes resolved sender
/// information and is ready for UI rendering.
///
/// # Example
///
/// ```ignore
/// let message = ChatMessage {
///     id: "did:sync:abc123:42".to_string(),
///     sender_did: "did:sync:abc123".to_string(),
///     sender_name: Some("Love".to_string()),
///     content: "Hello!".to_string(),
///     timestamp: 1705123456789,
///     sequence: 42,
///     is_mine: false,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Unique identifier (sender_did + ":" + sequence)
    pub id: String,
    /// Sender's DID
    pub sender_did: String,
    /// Sender's display name (if known from contacts)
    pub sender_name: Option<String>,
    /// Message content (decrypted)
    pub content: String,
    /// Unix timestamp in milliseconds when the message was created
    pub timestamp: i64,
    /// Packet sequence number within sender's log
    pub sequence: u64,
    /// Whether this message was sent by us
    pub is_mine: bool,
}

impl ChatMessage {
    /// Create a new ChatMessage.
    ///
    /// # Arguments
    ///
    /// * `sender_did` - The sender's decentralized identifier
    /// * `sender_name` - Optional display name for the sender
    /// * `content` - The decrypted message content
    /// * `timestamp` - Unix timestamp in milliseconds
    /// * `sequence` - Packet sequence number
    /// * `is_mine` - Whether this message was sent by us
    pub fn new(
        sender_did: String,
        sender_name: Option<String>,
        content: String,
        timestamp: i64,
        sequence: u64,
        is_mine: bool,
    ) -> Self {
        let id = format!("{}:{}", sender_did, sequence);
        Self {
            id,
            sender_did,
            sender_name,
            content,
            timestamp,
            sequence,
            is_mine,
        }
    }

    /// Get the display name for the sender.
    ///
    /// Returns the sender_name if available, otherwise truncates the DID.
    pub fn display_sender(&self) -> String {
        if let Some(ref name) = self.sender_name {
            name.clone()
        } else {
            // Truncate DID for display: "did:sync:abc123xyz" -> "abc123..."
            let did = &self.sender_did;
            if let Some(last_colon) = did.rfind(':') {
                let suffix = &did[last_colon + 1..];
                if suffix.len() > 8 {
                    format!("{}...", &suffix[..8])
                } else {
                    suffix.to_string()
                }
            } else {
                did.chars().take(12).collect()
            }
        }
    }

    /// Format the timestamp as a relative time string.
    ///
    /// Returns strings like "Just now", "5m ago", "2h ago", "Yesterday", etc.
    pub fn relative_time(&self) -> String {
        let now = chrono::Utc::now().timestamp_millis();
        let diff_ms = now - self.timestamp;
        let diff_secs = diff_ms / 1000;

        if diff_secs < 60 {
            "Just now".to_string()
        } else if diff_secs < 3600 {
            format!("{}m ago", diff_secs / 60)
        } else if diff_secs < 86400 {
            format!("{}h ago", diff_secs / 3600)
        } else if diff_secs < 172800 {
            "Yesterday".to_string()
        } else {
            format!("{}d ago", diff_secs / 86400)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_creation() {
        let msg = ChatMessage::new(
            "did:sync:abc123".to_string(),
            Some("Love".to_string()),
            "Hello, world!".to_string(),
            1705123456789,
            42,
            false,
        );

        assert_eq!(msg.id, "did:sync:abc123:42");
        assert_eq!(msg.sender_did, "did:sync:abc123");
        assert_eq!(msg.sender_name, Some("Love".to_string()));
        assert_eq!(msg.content, "Hello, world!");
        assert_eq!(msg.timestamp, 1705123456789);
        assert_eq!(msg.sequence, 42);
        assert!(!msg.is_mine);
    }

    #[test]
    fn test_display_sender_with_name() {
        let msg = ChatMessage::new(
            "did:sync:abc123".to_string(),
            Some("Love".to_string()),
            "Hi".to_string(),
            0,
            0,
            false,
        );
        assert_eq!(msg.display_sender(), "Love");
    }

    #[test]
    fn test_display_sender_truncates_did() {
        let msg = ChatMessage::new(
            "did:sync:abcdefghijklmnop".to_string(),
            None,
            "Hi".to_string(),
            0,
            0,
            false,
        );
        // Should truncate to first 8 chars of the suffix after last colon
        assert_eq!(msg.display_sender(), "abcdefgh...");
    }

    #[test]
    fn test_display_sender_short_did() {
        let msg = ChatMessage::new(
            "did:sync:abc".to_string(),
            None,
            "Hi".to_string(),
            0,
            0,
            false,
        );
        // Short suffix, no truncation needed
        assert_eq!(msg.display_sender(), "abc");
    }

    #[test]
    fn test_is_mine_flag() {
        let my_msg = ChatMessage::new(
            "did:sync:me".to_string(),
            None,
            "My message".to_string(),
            0,
            0,
            true,
        );
        assert!(my_msg.is_mine);

        let their_msg = ChatMessage::new(
            "did:sync:them".to_string(),
            None,
            "Their message".to_string(),
            0,
            0,
            false,
        );
        assert!(!their_msg.is_mine);
    }

    #[test]
    fn test_message_equality() {
        let msg1 = ChatMessage::new(
            "did:sync:abc".to_string(),
            None,
            "Hello".to_string(),
            1000,
            1,
            false,
        );
        let msg2 = ChatMessage::new(
            "did:sync:abc".to_string(),
            None,
            "Hello".to_string(),
            1000,
            1,
            false,
        );
        assert_eq!(msg1, msg2);
    }
}
