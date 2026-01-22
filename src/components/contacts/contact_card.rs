//! Contact Card Component
//!
//! Individual contact card showing avatar, name, and online status.

use dioxus::prelude::*;

use crate::components::images::AsyncImage;

// Embed default profile image as base64 data URI
const PROFILE_DEFAULT_BYTES: &[u8] = include_bytes!("../../../assets/profile-default.png");

fn profile_default_uri() -> String {
    use base64::Engine;
    let base64 = base64::engine::general_purpose::STANDARD.encode(PROFILE_DEFAULT_BYTES);
    format!("data:image/png;base64,{}", base64)
}

/// Contact Card
///
/// Displays a single contact with avatar, name, and online/offline status indicator.
///
/// # Example
///
/// ```rust
/// rsx! {
///     ContactCard {
///         contact_name: "Alice Smith".to_string(),
///         contact_avatar: Some("blob_id_here".to_string()),
///         is_online: true,
///         index: 0,
///         on_click: move |_| { /* Handle click */ },
///     }
/// }
/// ```
#[component]
pub fn ContactCard(
    /// Display name of contact
    contact_name: String,
    /// Optional avatar blob ID
    #[props(default = None)]
    contact_avatar: Option<String>,
    /// Whether contact is currently online
    #[props(default = false)]
    is_online: bool,
    /// Whether this contact has recent packet activity
    #[props(default = false)]
    has_activity: bool,
    /// Index for staggered animation
    #[props(default = 0)]
    index: usize,
    /// Optional click handler
    #[props(default = None)]
    on_click: Option<EventHandler<()>>,
) -> Element {
    let status_class = if is_online { "online" } else { "offline" };
    let activity_class = if has_activity { "packet-activity" } else { "" };

    let handle_click = move |_| {
        if let Some(handler) = &on_click {
            handler.call(());
        }
    };

    rsx! {
        div {
            class: "contact-card {status_class} {activity_class}",
            style: "--index: {index}",
            onclick: handle_click,

            // Circular avatar
            div { class: "contact-avatar",
                if let Some(blob_id) = contact_avatar.clone() {
                    AsyncImage {
                        blob_id: blob_id,
                        alt: contact_name.clone(),
                        class: Some("avatar-image".to_string()),
                    }
                } else {
                    // Default profile image
                    img {
                        class: "avatar-image",
                        src: "{profile_default_uri()}",
                        alt: "{contact_name}",
                    }
                }

                // Status dot indicator
                div {
                    class: "status-dot",
                    title: if is_online { "Online" } else { "Offline" },
                }
            }

            // Name below avatar
            div { class: "contact-name",
                "{contact_name}"
            }
        }
    }
}
