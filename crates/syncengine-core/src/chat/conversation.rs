//! Conversation abstraction for chat sessions
//!
//! A [`Conversation`] represents the message history with a specific contact,
//! aggregating messages from both directions (sent and received).

use super::message::ChatMessage;

/// A conversation with a specific contact.
///
/// Aggregates all messages exchanged with a contact, sorted chronologically.
/// Messages include both those we sent and those we received.
///
/// # Example
///
/// ```ignore
/// let conversation = Conversation::new(
///     "did:sync:friend123".to_string(),
///     Some("Alice".to_string()),
/// );
///
/// // Add messages
/// conversation.add_message(received_msg);
/// conversation.add_message(sent_msg);
///
/// // Get sorted messages
/// for msg in conversation.messages() {
///     println!("{}: {}", msg.display_sender(), msg.content);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Conversation {
    /// Contact's DID
    pub contact_did: String,
    /// Contact's display name (if known)
    pub contact_name: Option<String>,
    /// Messages in chronological order (oldest first)
    messages: Vec<ChatMessage>,
    /// Last message timestamp (for sorting conversations)
    pub last_activity: i64,
    /// Highest sequence number we've seen from the contact
    last_received_sequence: u64,
    /// Highest sequence number we've sent to this contact
    last_sent_sequence: u64,
}

impl Conversation {
    /// Create a new empty conversation.
    ///
    /// # Arguments
    ///
    /// * `contact_did` - The contact's decentralized identifier
    /// * `contact_name` - Optional display name for the contact
    pub fn new(contact_did: String, contact_name: Option<String>) -> Self {
        Self {
            contact_did,
            contact_name,
            messages: Vec::new(),
            last_activity: 0,
            last_received_sequence: 0,
            last_sent_sequence: 0,
        }
    }

    /// Add a message to the conversation.
    ///
    /// Messages are kept sorted by timestamp. Duplicates (same id) are ignored.
    pub fn add_message(&mut self, message: ChatMessage) {
        // Check for duplicate
        if self.messages.iter().any(|m| m.id == message.id) {
            return;
        }

        // Update sequence tracking
        if message.is_mine {
            if message.sequence > self.last_sent_sequence {
                self.last_sent_sequence = message.sequence;
            }
        } else if message.sequence > self.last_received_sequence {
            self.last_received_sequence = message.sequence;
        }

        // Update last activity
        if message.timestamp > self.last_activity {
            self.last_activity = message.timestamp;
        }

        // Insert in sorted order by timestamp
        let pos = self
            .messages
            .iter()
            .position(|m| m.timestamp > message.timestamp)
            .unwrap_or(self.messages.len());
        self.messages.insert(pos, message);
    }

    /// Get all messages in chronological order.
    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    /// Get messages since a specific sequence number (from the contact).
    ///
    /// Returns messages from the contact with sequence > since_seq.
    pub fn messages_since(&self, since_seq: u64) -> Vec<&ChatMessage> {
        self.messages
            .iter()
            .filter(|m| !m.is_mine && m.sequence > since_seq)
            .collect()
    }

    /// Get the number of messages in the conversation.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if the conversation is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get the most recent message, if any.
    pub fn last_message(&self) -> Option<&ChatMessage> {
        self.messages.last()
    }

    /// Get a preview of the conversation (last message content truncated).
    ///
    /// Returns the last message content truncated to `max_len` characters.
    pub fn preview(&self, max_len: usize) -> Option<String> {
        self.last_message().map(|m| {
            if m.content.len() > max_len {
                format!("{}...", &m.content[..max_len])
            } else {
                m.content.clone()
            }
        })
    }

    /// Get the display name for this conversation.
    ///
    /// Returns contact_name if available, otherwise truncates the DID.
    pub fn display_name(&self) -> String {
        if let Some(ref name) = self.contact_name {
            name.clone()
        } else {
            // Truncate DID for display
            let did = &self.contact_did;
            if let Some(last_colon) = did.rfind(':') {
                let suffix = &did[last_colon + 1..];
                if suffix.len() > 12 {
                    format!("{}...", &suffix[..12])
                } else {
                    suffix.to_string()
                }
            } else {
                did.chars().take(16).collect()
            }
        }
    }

    /// Get the highest received sequence number from the contact.
    pub fn last_received_sequence(&self) -> u64 {
        self.last_received_sequence
    }

    /// Get the highest sent sequence number to this contact.
    pub fn last_sent_sequence(&self) -> u64 {
        self.last_sent_sequence
    }

    /// Get unread message count (messages received after our last sent message).
    ///
    /// This is a heuristic - messages received after our last reply are considered unread.
    pub fn unread_count(&self) -> usize {
        if self.messages.is_empty() {
            return 0;
        }

        // Find the timestamp of our last sent message
        let last_sent_time = self
            .messages
            .iter()
            .filter(|m| m.is_mine)
            .map(|m| m.timestamp)
            .max()
            .unwrap_or(0);

        // Count messages from contact after that time
        self.messages
            .iter()
            .filter(|m| !m.is_mine && m.timestamp > last_sent_time)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_message(sender: &str, content: &str, timestamp: i64, seq: u64, is_mine: bool) -> ChatMessage {
        ChatMessage::new(
            sender.to_string(),
            None,
            content.to_string(),
            timestamp,
            seq,
            is_mine,
        )
    }

    #[test]
    fn test_conversation_creation() {
        let convo = Conversation::new(
            "did:sync:friend".to_string(),
            Some("Friend".to_string()),
        );

        assert_eq!(convo.contact_did, "did:sync:friend");
        assert_eq!(convo.contact_name, Some("Friend".to_string()));
        assert!(convo.is_empty());
        assert_eq!(convo.len(), 0);
    }

    #[test]
    fn test_add_message() {
        let mut convo = Conversation::new("did:sync:friend".to_string(), None);

        let msg = make_message("did:sync:friend", "Hello!", 1000, 1, false);
        convo.add_message(msg);

        assert_eq!(convo.len(), 1);
        assert_eq!(convo.messages()[0].content, "Hello!");
    }

    #[test]
    fn test_messages_sorted_by_timestamp() {
        let mut convo = Conversation::new("did:sync:friend".to_string(), None);

        // Add messages out of order
        convo.add_message(make_message("did:sync:friend", "Third", 3000, 3, false));
        convo.add_message(make_message("did:sync:friend", "First", 1000, 1, false));
        convo.add_message(make_message("did:sync:friend", "Second", 2000, 2, false));

        let messages = convo.messages();
        assert_eq!(messages[0].content, "First");
        assert_eq!(messages[1].content, "Second");
        assert_eq!(messages[2].content, "Third");
    }

    #[test]
    fn test_duplicate_messages_ignored() {
        let mut convo = Conversation::new("did:sync:friend".to_string(), None);

        let msg = make_message("did:sync:friend", "Hello!", 1000, 1, false);
        convo.add_message(msg.clone());
        convo.add_message(msg); // Duplicate

        assert_eq!(convo.len(), 1);
    }

    #[test]
    fn test_messages_since() {
        let mut convo = Conversation::new("did:sync:friend".to_string(), None);

        convo.add_message(make_message("did:sync:friend", "One", 1000, 1, false));
        convo.add_message(make_message("did:sync:friend", "Two", 2000, 2, false));
        convo.add_message(make_message("did:sync:friend", "Three", 3000, 3, false));
        convo.add_message(make_message("did:sync:me", "My reply", 2500, 10, true));

        let since = convo.messages_since(1);
        assert_eq!(since.len(), 2); // Messages with seq 2 and 3
        assert_eq!(since[0].content, "Two");
        assert_eq!(since[1].content, "Three");
    }

    #[test]
    fn test_last_activity_updated() {
        let mut convo = Conversation::new("did:sync:friend".to_string(), None);

        convo.add_message(make_message("did:sync:friend", "Old", 1000, 1, false));
        assert_eq!(convo.last_activity, 1000);

        convo.add_message(make_message("did:sync:friend", "New", 2000, 2, false));
        assert_eq!(convo.last_activity, 2000);
    }

    #[test]
    fn test_last_message() {
        let mut convo = Conversation::new("did:sync:friend".to_string(), None);

        assert!(convo.last_message().is_none());

        convo.add_message(make_message("did:sync:friend", "First", 1000, 1, false));
        convo.add_message(make_message("did:sync:friend", "Last", 2000, 2, false));

        assert_eq!(convo.last_message().unwrap().content, "Last");
    }

    #[test]
    fn test_preview() {
        let mut convo = Conversation::new("did:sync:friend".to_string(), None);

        assert!(convo.preview(50).is_none());

        convo.add_message(make_message(
            "did:sync:friend",
            "This is a very long message that should be truncated",
            1000,
            1,
            false,
        ));

        let preview = convo.preview(20).unwrap();
        assert_eq!(preview, "This is a very long ...");
    }

    #[test]
    fn test_display_name_with_name() {
        let convo = Conversation::new(
            "did:sync:friend".to_string(),
            Some("Alice".to_string()),
        );
        assert_eq!(convo.display_name(), "Alice");
    }

    #[test]
    fn test_display_name_truncates_did() {
        let convo = Conversation::new(
            "did:sync:abcdefghijklmnopqrstuvwxyz".to_string(),
            None,
        );
        assert_eq!(convo.display_name(), "abcdefghijkl...");
    }

    #[test]
    fn test_sequence_tracking() {
        let mut convo = Conversation::new("did:sync:friend".to_string(), None);

        convo.add_message(make_message("did:sync:friend", "Recv 1", 1000, 5, false));
        convo.add_message(make_message("did:sync:friend", "Recv 2", 2000, 10, false));
        convo.add_message(make_message("did:sync:me", "Sent 1", 1500, 3, true));
        convo.add_message(make_message("did:sync:me", "Sent 2", 2500, 7, true));

        assert_eq!(convo.last_received_sequence(), 10);
        assert_eq!(convo.last_sent_sequence(), 7);
    }

    #[test]
    fn test_unread_count() {
        let mut convo = Conversation::new("did:sync:friend".to_string(), None);

        // No messages = no unread
        assert_eq!(convo.unread_count(), 0);

        // Received message, no sent = 1 unread
        convo.add_message(make_message("did:sync:friend", "Hello", 1000, 1, false));
        assert_eq!(convo.unread_count(), 1);

        // We reply = 0 unread
        convo.add_message(make_message("did:sync:me", "Hi!", 2000, 1, true));
        assert_eq!(convo.unread_count(), 0);

        // They send more = unread increases
        convo.add_message(make_message("did:sync:friend", "How are you?", 3000, 2, false));
        convo.add_message(make_message("did:sync:friend", "Still there?", 4000, 3, false));
        assert_eq!(convo.unread_count(), 2);
    }

    #[test]
    fn test_mixed_sent_received() {
        let mut convo = Conversation::new("did:sync:friend".to_string(), None);

        // Interleaved conversation
        convo.add_message(make_message("did:sync:friend", "Hi", 1000, 1, false));
        convo.add_message(make_message("did:sync:me", "Hello!", 1100, 1, true));
        convo.add_message(make_message("did:sync:friend", "How are you?", 1200, 2, false));
        convo.add_message(make_message("did:sync:me", "Good, you?", 1300, 2, true));

        assert_eq!(convo.len(), 4);

        let messages = convo.messages();
        assert!(!messages[0].is_mine);
        assert!(messages[1].is_mine);
        assert!(!messages[2].is_mine);
        assert!(messages[3].is_mine);
    }
}
