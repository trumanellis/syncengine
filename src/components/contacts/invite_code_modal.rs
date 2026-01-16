//! Invite Code Modal Component
//!
//! Modal for pasting and decoding contact invitation codes.

use dioxus::prelude::*;
use syncengine_core::types::contact::HybridContactInvite;

use crate::context::use_engine;

/// Invite Code Modal
///
/// Modal dialog for entering and decoding contact invitation codes.
///
/// # Example
///
/// ```rust
/// rsx! {
///     InviteCodeModal {
///         show: show_modal(),
///         on_close: move |_| show_modal.set(false),
///         on_invite_decoded: move |invite| {
///             // Handle decoded invite
///         }
///     }
/// }
/// ```
#[component]
pub fn InviteCodeModal(
    /// Whether to show the modal
    show: bool,
    /// Callback when modal is closed
    on_close: EventHandler<()>,
    /// Callback when invite is successfully decoded
    on_invite_decoded: EventHandler<HybridContactInvite>,
) -> Element {
    let mut invite_input = use_signal(|| String::new());
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);
    let engine = use_engine();

    let decode_invite = move |_| {
        let code = invite_input();
        if code.is_empty() {
            error.set(Some("Please enter an invitation code".to_string()));
            return;
        }

        loading.set(true);
        error.set(None);

        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;

            if let Some(ref mut eng) = *guard {
                match eng.decode_contact_invite(&code).await {
                    Ok(invite) => {
                        on_invite_decoded.call(invite);
                        invite_input.set(String::new());
                    }
                    Err(e) => {
                        error.set(Some(format!("Invalid invite code: {}", e)));
                    }
                }
            }
            loading.set(false);
        });
    };

    let handle_close = move |_| {
        invite_input.set(String::new());
        error.set(None);
        on_close.call(());
    };

    if !show {
        return rsx! { };
    }

    rsx! {
        div {
            class: "modal-overlay",
            onclick: handle_close,

            div {
                class: "invite-code-modal",
                onclick: move |e| e.stop_propagation(),

                h2 { class: "modal-title", "Add New Contact" }

                p { class: "modal-description",
                    "Paste or enter the invitation code:"
                }

                input {
                    class: if error().is_some() { "invite-input invalid" } else { "invite-input" },
                    r#type: "text",
                    value: "{invite_input()}",
                    oninput: move |e| invite_input.set(e.value()),
                    placeholder: "sync-contact:...",
                    autofocus: true,
                }

                if let Some(err) = error() {
                    p { class: "error-text", "⚠ {err}" }
                }

                div { class: "modal-actions",
                    button {
                        class: "decode-button btn-primary",
                        onclick: decode_invite,
                        disabled: loading(),

                        if loading() {
                            "Decoding..."
                        } else {
                            "Decode Invite"
                        }
                    }

                    button {
                        class: "cancel-button btn-secondary",
                        onclick: handle_close,
                        "Cancel"
                    }
                }
            }
        }
    }
}
