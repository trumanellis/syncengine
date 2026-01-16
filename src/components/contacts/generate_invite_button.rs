//! Generate Invite Button Component
//!
//! Button to generate contact invites with text code overlay.

use dioxus::prelude::*;

use crate::context::use_engine;

/// Generate Contact Invite Button
///
/// Displays a button that generates a contact invite when clicked.
/// Shows QR code overlay when invite is generated.
///
/// # Example
///
/// ```rust
/// rsx! {
///     GenerateInviteButton {
///         on_invite_generated: move |code| {
///             // Handle generated invite code
///         }
///     }
/// }
/// ```
#[component]
pub fn GenerateInviteButton(
    /// Optional callback when invite is generated
    #[props(default = None)]
    on_invite_generated: Option<EventHandler<String>>,
) -> Element {
    let engine = use_engine();
    let mut invite_code = use_signal(|| Option::<String>::None);
    let mut generating = use_signal(|| false);
    let mut error = use_signal(|| Option::<String>::None);
    let mut copied = use_signal(|| false);

    let generate_invite = move |_| {
        generating.set(true);
        error.set(None);

        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;

            if let Some(ref mut eng) = *guard {
                match eng.generate_contact_invite(24).await {
                    Ok(code) => {
                        invite_code.set(Some(code.clone()));
                        if let Some(handler) = &on_invite_generated {
                            handler.call(code);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to generate invite: {:?}", e);
                        error.set(Some(format!("Failed to generate invite: {}", e)));
                    }
                }
            }
            generating.set(false);
        });
    };

    let close_overlay = move |_| {
        invite_code.set(None);
        error.set(None);
    };

    rsx! {
        button {
            class: "generate-invite-button",
            onclick: generate_invite,
            disabled: generating(),

            if generating() {
                "Generating Invite..."
            } else {
                "Generate Contact Invite"
            }
        }

        // Error message
        if let Some(err) = error() {
            div { class: "error-message",
                "{err}"
            }
        }

        // Invite code overlay (shown when invite_code is Some)
        if let Some(code) = invite_code() {
            div {
                class: "invite-overlay",
                onclick: close_overlay,

                div {
                    class: "invite-content",
                    onclick: move |e| e.stop_propagation(),

                    h2 { class: "overlay-title", "Contact Invite" }

                    p { class: "overlay-description",
                        "Share this invitation code with someone to connect."
                    }

                    div { class: "invite-code-display",
                        pre { class: "invite-code-text", "{code}" }
                    }

                    button {
                        class: if copied() { "copy-code-button copied" } else { "copy-code-button" },
                        onclick: move |_| {
                            let code_to_copy = code.clone();
                            spawn(async move {
                                // Use arboard for cross-platform clipboard access
                                match arboard::Clipboard::new() {
                                    Ok(mut clipboard) => {
                                        if clipboard.set_text(&code_to_copy).is_ok() {
                                            copied.set(true);
                                            // Wait briefly to show feedback
                                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                            // Auto-close the overlay
                                            invite_code.set(None);
                                            error.set(None);
                                            copied.set(false);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Clipboard not available: {}", e);
                                        // Still show feedback and close even if clipboard fails
                                        copied.set(true);
                                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                        invite_code.set(None);
                                        error.set(None);
                                        copied.set(false);
                                    }
                                }
                            });
                        },
                        if copied() {
                            "✓ Copied!"
                        } else {
                            "Copy Code"
                        }
                    }

                    button {
                        class: "close-overlay-button",
                        onclick: close_overlay,
                        "Close"
                    }
                }
            }
        }
    }
}
