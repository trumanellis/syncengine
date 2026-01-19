//! Profile Card Component
//!
//! Golden ratio card displaying user profile with avatar and connection info.

use dioxus::prelude::*;
use syncengine_core::types::UserProfile;

use super::{CardGallery, CardHeader, CardOrientation, GoldenCard};
use super::card_gallery::GalleryItem;
use crate::components::images::{AsyncImage, ImageOrientation, ImageUpload};
use crate::components::profile::QRSignature;

// Embed default profile image as base64 data URI
const PROFILE_DEFAULT_BYTES: &[u8] = include_bytes!("../../../assets/profile-default.png");

fn profile_default_uri() -> String {
    use base64::Engine;
    let base64 = base64::engine::general_purpose::STANDARD.encode(PROFILE_DEFAULT_BYTES);
    format!("data:image/png;base64,{}", base64)
}

/// Truncate DID for display, preserving the prefix
fn truncate_did(did: &str) -> String {
    if did.starts_with("did:sync:") {
        let suffix = &did[9..];
        if suffix.len() > 12 {
            format!("did:sync:{}…", &suffix[..12])
        } else {
            did.to_string()
        }
    } else if did.len() > 20 {
        format!("{}…", &did[..20])
    } else {
        did.to_string()
    }
}

/// Profile card with editable fields and QR code overlay
///
/// # Examples
///
/// ```rust
/// rsx! {
///     ProfileCard {
///         profile: user_profile,
///         editable: true,
///         show_qr: true,
///         did: Some("did:sync:abc123...".to_string()),
///         status: Some("online".to_string()),
///         last_seen: Some("Just now".to_string()),
///         on_update: move |updated| {
///             // Save updated profile
///         },
///     }
/// }
/// ```
#[component]
pub fn ProfileCard(
    /// Current profile data
    profile: UserProfile,
    /// Enable edit mode
    #[props(default = false)]
    editable: bool,
    /// Show QR code overlay on avatar
    #[props(default = false)]
    show_qr: bool,
    /// Compact mode for smaller cards (reduces font sizes)
    #[props(default = false)]
    compact: bool,
    /// DID to display (truncated automatically)
    #[props(default = None)]
    did: Option<String>,
    /// Connection status (e.g., "online", "offline", "unknown")
    #[props(default = None)]
    status: Option<String>,
    /// Formatted last seen time (e.g., "Just now", "5m ago")
    #[props(default = None)]
    last_seen: Option<String>,
    /// Callback when profile is updated
    #[props(default = None)]
    on_update: Option<EventHandler<UserProfile>>,
) -> Element {
    // Start in edit mode if profile has default/empty name
    let should_start_editing = profile.display_name.is_empty();
    let mut editing = use_signal(move || should_start_editing);
    let mut draft = use_signal(|| profile.clone());

    // Track profile prop as a signal for reactive memos (display mode)
    let mut display_profile = use_signal(|| profile.clone());

    // Sync display_profile when prop changes (detected via updated_at timestamp)
    if display_profile().updated_at != profile.updated_at {
        display_profile.set(profile.clone());
    }

    // Clone profile for closures
    let profile_for_cancel = profile.clone();

    // Save handler
    let save_changes = move |_| {
        if let Some(handler) = &on_update {
            let mut updated = draft();
            updated.touch(); // Update timestamp
            handler.call(updated.clone());
            editing.set(false);
        }
    };

    // Cancel handler
    let cancel_edit = move |_| {
        draft.set(profile_for_cancel.clone());
        editing.set(false);
    };

    // Determine card classes based on compact mode
    let card_class = if compact { "profile-card profile-card--compact" } else { "profile-card" };
    let content_class = if compact { "card-content card-content--compact" } else { "card-content" };

    // Format status class for styling
    let status_class = status.as_ref().map(|s| {
        match s.as_str() {
            "online" => "status-indicator status-online",
            "offline" => "status-indicator status-offline",
            _ => "status-indicator status-unknown",
        }
    });

    rsx! {
        div { class: "{card_class}",
            GoldenCard {
                orientation: CardOrientation::Landscape,
                interactive: !editing(),

                // Left: Avatar image area (38.2%)
                div { class: "card-image-area",
                // Avatar image
                if let Some(blob_id) = &draft().avatar_blob_id {
                    AsyncImage {
                        blob_id: blob_id.clone(),
                        alt: draft().display_name.clone(),
                        class: Some("card-image__avatar".to_string()),
                    }
                } else {
                    // Default profile image
                    img {
                        class: "card-image__default card-image__avatar",
                        src: "{profile_default_uri()}",
                        alt: "Profile",
                    }
                }

                // QR code overlay
                if show_qr {
                    div { class: "card-image__overlay",
                        QRSignature {
                            data: format!("iroh://{}", profile.peer_id),
                            size: 120,
                        }
                    }
                }

                // Upload button (edit mode only)
                if editing() {
                    div { class: "card-image__upload-icon",
                        ImageUpload {
                            orientation: ImageOrientation::Portrait,
                            icon_only: true,
                            on_upload: move |blob_id| {
                                draft.write().avatar_blob_id = Some(blob_id);
                            },
                        }
                    }
                }
            }

            // Right: Content area (61.8%)
            div { class: "{content_class}",
                // Header: Name, subtitle, link
                if editing() {
                    // Editable header
                    div { class: "card-header card-header--editing",
                        input {
                            class: "editable-input editable-title",
                            r#type: "text",
                            value: "{draft().display_name}",
                            oninput: move |e| draft.write().display_name = e.value(),
                            placeholder: "Full name",
                        }
                        input {
                            class: "editable-input editable-subtitle",
                            r#type: "text",
                            value: "{draft().subtitle.clone().unwrap_or_default()}",
                            oninput: move |e| {
                                let val = e.value();
                                draft.write().subtitle = if val.is_empty() { None } else { Some(val) };
                            },
                            placeholder: "Profile Subtitle",
                        }
                        input {
                            class: "editable-input editable-link",
                            r#type: "text",
                            value: "{draft().profile_link.clone().unwrap_or_default()}",
                            oninput: move |e| {
                                let val = e.value();
                                draft.write().profile_link = if val.is_empty() { None } else { Some(val) };
                            },
                            placeholder: "Profile Link",
                        }
                    }
                } else {
                    // Display mode - name only in header
                    CardHeader {
                        title: profile.display_name.clone(),
                        subtitle: None,
                        link: profile.profile_link.as_ref().map(|l| format!("sync.local/{}", l)),
                    }
                }

                // Tagline (subtitle) - shown after name, before connection info
                if let Some(ref tagline) = profile.subtitle {
                    p { class: "card-tagline", "{tagline}" }
                }

                // Connection info (DID, status, last seen) - only shown when provided
                if did.is_some() || status.is_some() || last_seen.is_some() {
                    div { class: "card-connection-info",
                        // DID
                        if let Some(ref did_str) = did {
                            div { class: "connection-did",
                                span { class: "did-label", "DID: " }
                                span { class: "did-value", "{truncate_did(did_str)}" }
                            }
                        }

                        // Status and last seen row
                        div { class: "connection-status-row",
                            // Status indicator
                            if let Some(ref status_str) = status {
                                if let Some(ref class_str) = status_class {
                                    span { class: "{class_str}", "{status_str}" }
                                }
                            }

                            // Last seen
                            if let Some(ref seen) = last_seen {
                                span { class: "last-seen", "Last seen: {seen}" }
                            }
                        }
                    }
                }

                // Gallery: Top quests (TODO: load actual quest data)
                if !profile.top_quests.is_empty() {
                    CardGallery {
                        title: "Featured Quests".to_string(),
                        items: profile.top_quests.iter().map(|quest_id| {
                            GalleryItem {
                                id: quest_id.clone(),
                                image_url: None, // TODO: Load quest image
                                label: Some(quest_id.clone()),
                            }
                        }).collect::<Vec<_>>(),
                    }
                }

                // Action buttons
                if editable {
                    div { class: "card-actions",
                        if editing() {
                            button {
                                class: "btn-primary",
                                onclick: save_changes,
                                "Save Changes"
                            }
                            button {
                                class: "btn-secondary",
                                onclick: cancel_edit,
                                "Cancel"
                            }
                        } else {
                            button {
                                class: "btn-primary",
                                onclick: move |_| editing.set(true),
                                "Edit Profile"
                            }
                        }
                    }
                }
            }
            }
        }
    }
}
