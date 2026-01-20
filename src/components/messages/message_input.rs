//! Inline Message Input Component
//!
//! Fixed-position input bar at the bottom of a conversation view.
//! Follows DESIGN_SYSTEM.md cyber-mystical terminal aesthetic.

use dioxus::prelude::*;

/// Inline message input bar for sending messages
#[component]
pub fn MessageInput(
    /// Handler called when message is sent (receives message content)
    on_send: EventHandler<String>,
    /// Placeholder text
    #[props(default = "Type a message...".to_string())]
    placeholder: String,
    /// Whether input is disabled (e.g., during sending)
    #[props(default = false)]
    disabled: bool,
) -> Element {
    let mut message_content = use_signal(String::new);
    let mut is_sending = use_signal(|| false);

    let handle_send = move |_| {
        let content = message_content();
        if content.trim().is_empty() || is_sending() {
            return;
        }

        is_sending.set(true);
        on_send.call(content);

        // Clear input after sending
        message_content.set(String::new());

        // Reset sending state after brief delay
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            is_sending.set(false);
        });
    };

    // Handle Enter key (without Shift) to send
    let handle_keydown = move |e: KeyboardEvent| {
        if e.key() == Key::Enter && !e.modifiers().shift() {
            e.prevent_default();
            let content = message_content();
            if !content.trim().is_empty() && !is_sending() {
                is_sending.set(true);
                on_send.call(content);
                message_content.set(String::new());

                spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    is_sending.set(false);
                });
            }
        }
    };

    let is_disabled = disabled || is_sending();
    let can_send = !message_content().trim().is_empty() && !is_disabled;

    rsx! {
        div { class: "message-input-bar",
            // Text input
            textarea {
                class: "message-input-textarea",
                placeholder: "{placeholder}",
                value: "{message_content}",
                oninput: move |e| message_content.set(e.value()),
                onkeydown: handle_keydown,
                disabled: is_disabled,
                rows: 1,
            }

            // Send button
            button {
                class: if can_send { "message-send-btn message-send-btn-active" } else { "message-send-btn" },
                onclick: handle_send,
                disabled: !can_send,
                title: "Send message (Enter)",

                // Send icon (arrow)
                svg {
                    class: "send-icon",
                    width: "20",
                    height: "20",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    // Arrow pointing right
                    line { x1: "22", y1: "2", x2: "11", y2: "13" }
                    polygon { points: "22 2 15 22 11 13 2 9 22 2" }
                }
            }
        }
    }
}
