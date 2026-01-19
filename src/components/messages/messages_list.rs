//! Messages List Component
//!
//! Displays received direct messages from contacts.
//! Follows Design System v2: minimal terminal aesthetic.

use dioxus::prelude::*;

/// A received message with sender info
#[derive(Clone, Debug, PartialEq)]
pub struct ReceivedMessage {
    /// Sender's DID
    pub sender_did: String,
    /// Sender's display name (if known)
    pub sender_name: Option<String>,
    /// Message content
    pub content: String,
    /// Timestamp (milliseconds)
    pub timestamp: i64,
    /// Packet sequence number
    pub sequence: u64,
}

/// Format timestamp as relative time
fn format_time(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp_millis();
    let elapsed_ms = now - timestamp;
    let elapsed_secs = elapsed_ms / 1000;

    if elapsed_secs < 60 {
        "Just now".to_string()
    } else if elapsed_secs < 3600 {
        format!("{}m ago", elapsed_secs / 60)
    } else if elapsed_secs < 86400 {
        format!("{}h ago", elapsed_secs / 3600)
    } else {
        format!("{}d ago", elapsed_secs / 86400)
    }
}

/// Truncate DID for display
fn truncate_did(did: &str) -> String {
    if did.starts_with("did:sync:") && did.len() > 20 {
        format!("{}…", &did[..20])
    } else if did.len() > 16 {
        format!("{}…", &did[..16])
    } else {
        did.to_string()
    }
}

/// Messages list showing received direct messages
#[component]
pub fn MessagesList(
    /// List of received messages
    messages: Vec<ReceivedMessage>,
    /// Loading state
    #[props(default = false)]
    loading: bool,
) -> Element {
    if loading {
        return rsx! {
            div { class: "messages-list loading",
                div { class: "loading-spinner" }
                p { class: "loading-text", "Loading messages..." }
            }
        };
    }

    if messages.is_empty() {
        return rsx! {
            div { class: "messages-list empty",
                p { class: "empty-state-message", "No messages yet" }
                p { class: "empty-state-hint", "Messages from contacts will appear here." }
            }
        };
    }

    rsx! {
        div { class: "messages-list",
            for msg in messages.iter().rev() {
                div {
                    key: "{msg.sender_did}-{msg.sequence}",
                    class: "message-item",

                    // Sender info
                    div { class: "message-header",
                        span { class: "message-sender",
                            if let Some(ref name) = msg.sender_name {
                                "{name}"
                            } else {
                                "{truncate_did(&msg.sender_did)}"
                            }
                        }
                        span { class: "message-time", "{format_time(msg.timestamp)}" }
                    }

                    // Message content
                    div { class: "message-content",
                        "{msg.content}"
                    }
                }
            }
        }
    }
}
