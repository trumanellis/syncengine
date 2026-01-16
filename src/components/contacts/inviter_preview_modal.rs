//! Inviter Preview Modal Component
//!
//! Modal showing profile preview for incoming contact request with accept/decline actions.

use dioxus::prelude::*;

use crate::components::images::AsyncImage;

/// Inviter Preview Modal
///
/// Shows a preview of the inviter's profile when receiving a contact request.
/// Allows user to accept or decline the connection.
///
/// # Example
///
/// ```rust
/// rsx! {
///     InviterPreviewModal {
///         inviter_name: "Alice Smith".to_string(),
///         inviter_subtitle: Some("Developer".to_string()),
///         inviter_bio: "Loves building cool things...".to_string(),
///         inviter_avatar: Some("blob_id_here".to_string()),
///         on_close: move |_| show_preview.set(false),
///         on_accept: move |_| { /* Accept contact */ },
///         on_decline: move |_| { /* Decline contact */ },
///     }
/// }
/// ```
#[component]
pub fn InviterPreviewModal(
    /// Display name of inviter
    inviter_name: String,
    /// Optional subtitle
    #[props(default = None)]
    inviter_subtitle: Option<String>,
    /// Bio excerpt
    inviter_bio: String,
    /// Optional avatar blob ID
    #[props(default = None)]
    inviter_avatar: Option<String>,
    /// Show/hide modal
    #[props(default = true)]
    show: bool,
    /// Callback when modal is closed
    on_close: EventHandler<()>,
    /// Callback when accept is clicked
    on_accept: EventHandler<()>,
    /// Callback when decline is clicked
    on_decline: EventHandler<()>,
) -> Element {
    if !show {
        return rsx! { };
    }

    let handle_close = move |_| on_close.call(());
    let handle_accept = move |_| on_accept.call(());
    let handle_decline = move |_| on_decline.call(());

    rsx! {
        div {
            class: "modal-overlay",
            onclick: handle_close,

            div {
                class: "inviter-preview-modal",
                onclick: move |e| e.stop_propagation(),

                h2 { class: "modal-title", "Contact Request" }

                // Profile preview
                div { class: "profile-preview",
                    // Avatar
                    div { class: "preview-avatar",
                        if let Some(blob_id) = inviter_avatar.clone() {
                            AsyncImage {
                                blob_id: blob_id,
                                alt: inviter_name.clone(),
                                class: Some("avatar-image".to_string()),
                            }
                        } else {
                            div { class: "avatar-placeholder",
                                "{inviter_name.chars().next().unwrap_or('?')}"
                            }
                        }
                    }

                    // Profile info
                    div { class: "profile-header",
                        h3 { class: "inviter-name", "{inviter_name}" }
                        if let Some(subtitle) = inviter_subtitle {
                            p { class: "inviter-subtitle", "{subtitle}" }
                        }
                    }

                    p { class: "inviter-bio", "{inviter_bio}" }
                }

                p { class: "request-message",
                    "This user wants to connect with you"
                }

                // Actions
                div { class: "modal-actions",
                    button {
                        class: "accept-button btn-primary",
                        onclick: handle_accept,
                        "Accept"
                    }
                    button {
                        class: "decline-button btn-secondary",
                        onclick: handle_decline,
                        "Decline"
                    }
                }
            }
        }
    }
}
