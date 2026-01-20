//! Message Bubble Component
//!
//! Chat-style message bubbles with visual distinction between sent and received.
//! Follows DESIGN_SYSTEM.md cyber-mystical terminal aesthetic.

use dioxus::prelude::*;

/// A chat message for display in a bubble
#[derive(Clone, Debug, PartialEq)]
pub struct ChatBubbleMessage {
    /// Unique message ID
    pub id: String,
    /// Message content
    pub content: String,
    /// Sender's display name (for received messages)
    pub sender_name: Option<String>,
    /// Timestamp in milliseconds
    pub timestamp: i64,
    /// Whether this message was sent by us
    pub is_mine: bool,
}

/// Format timestamp as relative time
fn format_time(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp_millis();
    let elapsed_ms = now - timestamp;
    let elapsed_secs = elapsed_ms / 1000;

    if elapsed_secs < 60 {
        "Just now".to_string()
    } else if elapsed_secs < 3600 {
        format!("{}m", elapsed_secs / 60)
    } else if elapsed_secs < 86400 {
        format!("{}h", elapsed_secs / 3600)
    } else {
        format!("{}d", elapsed_secs / 86400)
    }
}

/// Individual message bubble component
#[component]
pub fn MessageBubble(message: ChatBubbleMessage) -> Element {
    let bubble_class = if message.is_mine {
        "message-bubble message-bubble-sent"
    } else {
        "message-bubble message-bubble-received"
    };

    let alignment_class = if message.is_mine {
        "message-row message-row-sent"
    } else {
        "message-row message-row-received"
    };

    rsx! {
        div { class: "{alignment_class}",
            div { class: "{bubble_class}",
                // Show sender name for received messages
                if !message.is_mine {
                    if let Some(ref name) = message.sender_name {
                        div { class: "message-bubble-sender", "{name}" }
                    }
                }

                // Message content
                div { class: "message-bubble-content", "{message.content}" }

                // Timestamp
                div { class: "message-bubble-time", "{format_time(message.timestamp)}" }
            }
        }
    }
}

/// Group of message bubbles (for consecutive messages from same sender)
#[component]
pub fn MessageBubbleGroup(
    /// Messages to display (should be from same sender)
    messages: Vec<ChatBubbleMessage>,
    /// Sender display name (for received messages)
    sender_name: Option<String>,
    /// Whether these are our messages
    is_mine: bool,
) -> Element {
    if messages.is_empty() {
        return rsx! {};
    }

    let group_class = if is_mine {
        "message-bubble-group message-bubble-group-sent"
    } else {
        "message-bubble-group message-bubble-group-received"
    };

    rsx! {
        div { class: "{group_class}",
            // Show sender name once at top for received messages
            if !is_mine {
                if let Some(ref name) = sender_name {
                    div { class: "message-group-sender", "{name}" }
                }
            }

            for msg in messages {
                MessageBubble { message: msg }
            }
        }
    }
}
