//! Profile Card Component
//!
//! Golden ratio card displaying user profile with avatar, bio, and quests.

use dioxus::prelude::*;
use syncengine_core::types::UserProfile;

use super::{CardGallery, CardHeader, CardOrientation, GoldenCard, MarkdownEditor, MarkdownRenderer};
use super::card_gallery::GalleryItem;
use crate::components::images::{AsyncImage, ImageOrientation, ImageUpload};
use crate::components::profile::QRSignature;

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
    /// Callback when profile is updated
    #[props(default = None)]
    on_update: Option<EventHandler<UserProfile>>,
) -> Element {
    // Start in edit mode if profile has default/empty name
    let should_start_editing = profile.display_name.is_empty();
    let mut editing = use_signal(move || should_start_editing);
    let mut draft = use_signal(|| profile.clone());

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

    rsx! {
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
                    div { class: "card-image__default card-image__avatar",
                        // Default avatar placeholder
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            view_box: "0 0 100 100",
                            class: "default-avatar-icon",
                            circle {
                                cx: "50",
                                cy: "35",
                                r: "20",
                                fill: "currentColor",
                            }
                            path {
                                d: "M 20 80 Q 20 55, 50 55 Q 80 55, 80 80",
                                fill: "currentColor",
                            }
                        }
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
            div { class: "card-content",
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
                    // Display mode
                    CardHeader {
                        title: profile.display_name.clone(),
                        subtitle: profile.subtitle.clone(),
                        link: profile.profile_link.as_ref().map(|l| format!("sync.local/{}", l)),
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

                // Bio (markdown)
                div { class: "card-markdown-section",
                    if editing() {
                        {
                            let bio_signal = use_memo(move || draft().bio.clone());
                            rsx! {
                                MarkdownEditor {
                                    content: bio_signal,
                                    on_change: move |new_bio| draft.write().bio = new_bio,
                                }
                            }
                        }
                    } else {
                        if !profile.bio.is_empty() {
                            {
                                let bio_signal = use_memo(move || profile.bio.clone());
                                rsx! {
                                    MarkdownRenderer {
                                        content: bio_signal,
                                        collapsible: false,
                                        collapsed: false,
                                    }
                                }
                            }
                        } else {
                            div { class: "card-empty-state",
                                "No bio yet..."
                            }
                        }
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
