//! Conversation View Component
//!
//! Full chat view for a specific contact, showing message history
//! with chat-style bubbles and inline input.
//! Follows DESIGN_SYSTEM.md cyber-mystical terminal aesthetic.

use dioxus::prelude::*;

use super::message_bubble::{ChatBubbleMessage, MessageBubble};
use super::message_input::MessageInput;

/// Conversation view showing chat with a specific contact
#[component]
pub fn ConversationView(
    /// Contact's DID
    contact_did: String,
    /// Contact's display name
    contact_name: String,
    /// Messages in the conversation (chronological order)
    messages: Vec<ChatBubbleMessage>,
    /// Handler for sending messages
    on_send: EventHandler<String>,
    /// Handler for going back (closing conversation)
    on_back: EventHandler<()>,
    /// Whether currently sending a message
    #[props(default = false)]
    sending: bool,
    /// Loading state
    #[props(default = false)]
    loading: bool,
) -> Element {
    // Note: Auto-scroll would require platform-specific implementation
    // For desktop, Dioxus handles scroll positioning natively

    rsx! {
        div { class: "conversation-view",
            // Header with contact info and back button
            header { class: "conversation-header",
                button {
                    class: "conversation-back-btn",
                    onclick: move |_| on_back.call(()),
                    title: "Back to contacts",
                    // Back arrow
                    svg {
                        width: "20",
                        height: "20",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        polyline { points: "15 18 9 12 15 6" }
                    }
                }

                div { class: "conversation-contact-info",
                    h2 { class: "conversation-contact-name", "{contact_name}" }
                    p { class: "conversation-contact-did",
                        "{truncate_did(&contact_did)}"
                    }
                }
            }

            // Messages area
            div {
                class: "conversation-messages",

                if loading {
                    div { class: "conversation-loading",
                        div { class: "loading-spinner" }
                        p { "Loading messages..." }
                    }
                } else if messages.is_empty() {
                    div { class: "conversation-empty",
                        p { class: "empty-icon", "∿" }
                        p { class: "empty-text", "No messages yet" }
                        p { class: "empty-hint",
                            "Send a message to start the conversation."
                        }
                    }
                } else {
                    for msg in &messages {
                        MessageBubble {
                            key: "{msg.id}",
                            message: msg.clone()
                        }
                    }
                }

                // Scroll anchor at bottom
                div { class: "scroll-anchor" }
            }

            // Message input at bottom
            div { class: "conversation-input-container",
                MessageInput {
                    on_send: on_send,
                    disabled: sending,
                    placeholder: format!("Message {}...", contact_name),
                }
            }
        }
    }
}

/// Truncate DID for display
fn truncate_did(did: &str) -> String {
    if did.starts_with("did:sync:") && did.len() > 24 {
        format!("{}…", &did[..24])
    } else if did.len() > 20 {
        format!("{}…", &did[..20])
    } else {
        did.to_string()
    }
}
