//! Message Compose Modal
//!
//! Modal dialog for composing and sending direct messages to contacts.
//! Follows Design System v2: minimal terminal aesthetic.

use dioxus::prelude::*;

/// Message compose modal for sending direct messages
#[component]
pub fn MessageCompose(
    /// Recipient's display name
    recipient_name: String,
    /// Recipient's DID
    recipient_did: String,
    /// Handler called when message is sent (receives message content)
    on_send: EventHandler<String>,
    /// Handler called when modal is closed
    on_close: EventHandler<()>,
) -> Element {
    let mut message_content = use_signal(String::new);
    let mut sending = use_signal(|| false);

    let handle_send = move |_| {
        let content = message_content();
        if content.trim().is_empty() {
            return;
        }

        sending.set(true);
        on_send.call(content);
    };

    let handle_close = move |_| {
        on_close.call(());
    };

    // Handle keyboard shortcuts
    let handle_keydown = move |e: KeyboardEvent| {
        if e.key() == Key::Escape {
            on_close.call(());
        } else if e.key() == Key::Enter && e.modifiers().ctrl() {
            // Ctrl+Enter to send
            let content = message_content();
            if !content.trim().is_empty() {
                sending.set(true);
                on_send.call(content);
            }
        }
    };

    rsx! {
        div {
            class: "modal-overlay",
            onclick: handle_close,
            onkeydown: handle_keydown,

            div {
                class: "modal message-compose-modal",
                onclick: move |e| e.stop_propagation(),

                // Header
                header { class: "modal-header",
                    h2 { class: "modal-title", "Send Message" }
                    button {
                        class: "btn-icon modal-close",
                        onclick: handle_close,
                        "×"
                    }
                }

                // Recipient info
                div { class: "message-recipient",
                    span { class: "label", "To: " }
                    span { class: "recipient-name", "{recipient_name}" }
                }

                // Message input
                div { class: "message-input-container",
                    textarea {
                        class: "input message-textarea",
                        placeholder: "Type your message...",
                        value: "{message_content}",
                        oninput: move |e| message_content.set(e.value()),
                        disabled: sending(),
                        rows: 4,
                    }
                }

                // Actions
                div { class: "modal-actions",
                    button {
                        class: "btn btn-secondary",
                        onclick: handle_close,
                        disabled: sending(),
                        "Cancel"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: handle_send,
                        disabled: sending() || message_content().trim().is_empty(),
                        if sending() {
                            "Sending..."
                        } else {
                            "Send"
                        }
                    }
                }

                // Note about encryption
                p { class: "message-note",
                    "Messages are signed with your identity. Encryption coming soon."
                }
            }
        }
    }
}
