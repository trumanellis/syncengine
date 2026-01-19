//! Network Page - Profiles You Mirror
//!
//! Displays the profiles you are mirroring for P2P redundancy.

use dioxus::prelude::*;
use syncengine_core::ProfilePin;

use crate::components::images::AsyncImage;
use crate::components::{NavHeader, NavLocation};
use crate::context::{use_engine, use_engine_ready};

// Embed default profile image as base64 data URI
const PROFILE_DEFAULT_BYTES: &[u8] = include_bytes!("../../assets/profile-default.png");

fn profile_default_uri() -> String {
    use base64::Engine;
    let base64 = base64::engine::general_purpose::STANDARD.encode(PROFILE_DEFAULT_BYTES);
    format!("data:image/png;base64,{}", base64)
}

/// Format timestamp as relative time string.
fn format_relative_time(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let elapsed = (now - timestamp).max(0);

    if elapsed < 60 {
        "Just now".to_string()
    } else if elapsed < 3600 {
        format!("{}m ago", elapsed / 60)
    } else if elapsed < 86400 {
        format!("{}h ago", elapsed / 3600)
    } else {
        format!("{}d ago", elapsed / 86400)
    }
}

/// Network page - Profiles You Mirror
#[component]
pub fn Network() -> Element {
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // State for mirrored profiles only
    let mut pins: Signal<Vec<ProfilePin>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);

    // Load mirrored profiles when engine becomes ready
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let guard = shared.read().await;

                if let Some(ref eng) = *guard {
                    // Get pins (profiles we mirror)
                    if let Ok(pin_list) = eng.list_pinned_profiles() {
                        // Filter out our own profile
                        let others: Vec<_> = pin_list
                            .into_iter()
                            .filter(|p| !p.is_own())
                            .collect();
                        pins.set(others);
                    }

                    loading.set(false);
                }
            });
        }
    });

    let pin_count = pins().len();

    rsx! {
        div { class: "network-page",
            // Sacred Navigation Console
            NavHeader {
                current: NavLocation::Network,
                status: Some(format!("{} profiles mirrored", pin_count)),
            }

            if loading() {
                div { class: "loading-state",
                    div { class: "loading-orb" }
                    p { "Loading mirrored profiles..." }
                }
            } else {
                div { class: "network-content",
                    // Profiles You Mirror
                    section { class: "network-section mirrored-profiles-section",
                        h2 { class: "section-title", "Profiles You Mirror" }
                        p { class: "section-subtitle", "Full profiles of peers you're carrying in the network" }

                        if pins().is_empty() {
                            div { class: "empty-state",
                                p { "You are not mirroring any profiles yet. Add contacts to automatically mirror their profiles." }
                            }
                        } else {
                            div { class: "mirrored-profiles-grid",
                                for pin in pins() {
                                    YourPinCard { pin: PinDisplayData::from(&pin) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Subcomponents
// ═══════════════════════════════════════════════════════════════════════════════

/// Full profile data extracted from ProfilePin for display
/// (avoids PartialEq issues with SignedProfile while exposing all fields)
#[derive(Clone, PartialEq)]
struct PinDisplayData {
    did: String,
    display_name: String,
    subtitle: Option<String>,
    bio: String,
    avatar_blob_id: Option<String>,
    profile_link: Option<String>,
    relationship: String,
    pinned_at: i64,
    last_updated: i64,
}

impl From<&ProfilePin> for PinDisplayData {
    fn from(pin: &ProfilePin) -> Self {
        let relationship = match &pin.relationship {
            syncengine_core::PinRelationship::Contact => "Contact".to_string(),
            syncengine_core::PinRelationship::RealmMember { realm_id } => {
                format!("Realm: {}", &realm_id.to_string()[..8])
            }
            syncengine_core::PinRelationship::Manual => "Manual".to_string(),
            syncengine_core::PinRelationship::Own => "Self".to_string(),
        };
        let profile = &pin.signed_profile.profile;
        Self {
            did: pin.did.clone(),
            display_name: profile.display_name.clone(),
            subtitle: profile.subtitle.clone(),
            bio: profile.bio.clone(),
            avatar_blob_id: profile.avatar_blob_id.clone(),
            profile_link: profile.profile_link.clone(),
            relationship,
            pinned_at: pin.pinned_at,
            last_updated: pin.last_updated,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct YourPinCardProps {
    pin: PinDisplayData,
}

/// Truncate bio text to a maximum length, adding ellipsis if needed
fn truncate_bio(bio: &str, max_chars: usize) -> String {
    if bio.len() <= max_chars {
        bio.to_string()
    } else {
        // Find a good break point (space or newline)
        let truncated = &bio[..max_chars];
        if let Some(last_space) = truncated.rfind(|c: char| c.is_whitespace()) {
            format!("{}…", &truncated[..last_space])
        } else {
            format!("{}…", truncated)
        }
    }
}

/// Full profile card for mirrored peers - shows complete profile information
#[component]
fn YourPinCard(props: YourPinCardProps) -> Element {
    let engine = use_engine();
    let mut expanded = use_signal(|| false);

    let display_name = props.pin.display_name.clone();
    let subtitle = props.pin.subtitle.clone();
    let bio = props.pin.bio.clone();
    let avatar_blob_id = props.pin.avatar_blob_id.clone();
    let profile_link = props.pin.profile_link.clone();
    let relationship = props.pin.relationship.clone();
    let pinned_since = format_relative_time(props.pin.pinned_at);
    let last_mirrored = format_relative_time(props.pin.last_updated);
    let did_for_unpin = props.pin.did.clone();
    let did_display = {
        let did = &props.pin.did;
        if did.starts_with("did:sync:") {
            let suffix = &did[9..];
            if suffix.len() > 16 {
                format!("did:sync:{}…", &suffix[..16])
            } else {
                did.clone()
            }
        } else if did.len() > 24 {
            format!("{}…", &did[..24])
        } else {
            did.clone()
        }
    };

    // Determine if bio should show expand option
    let bio_preview = truncate_bio(&bio, 150);
    let has_more_bio = bio.len() > 150;

    rsx! {
        div { class: "network-card mirrored-profile-card",
            // Profile header with avatar
            div { class: "mirrored-profile-header",
                // Avatar
                div { class: "mirrored-avatar",
                    if let Some(ref blob_id) = avatar_blob_id {
                        AsyncImage {
                            blob_id: blob_id.clone(),
                            alt: display_name.clone(),
                            class: Some("mirrored-avatar-img".to_string()),
                        }
                    } else {
                        img {
                            class: "mirrored-avatar-img mirrored-avatar-default",
                            src: "{profile_default_uri()}",
                            alt: "Profile",
                        }
                    }
                }

                // Name and subtitle
                div { class: "mirrored-identity",
                    h3 { class: "mirrored-name", "{display_name}" }
                    if let Some(ref sub) = subtitle {
                        p { class: "mirrored-subtitle", "{sub}" }
                    }
                    if let Some(ref link) = profile_link {
                        span { class: "mirrored-link", "sync.local/{link}" }
                    }
                }

                // Relationship badge
                span { class: "mirrored-relationship-badge", "{relationship}" }
            }

            // DID (truncated)
            div { class: "mirrored-did",
                span { class: "did-label", "DID: " }
                span { class: "did-value", "{did_display}" }
            }

            // Bio section
            if !bio.is_empty() {
                div { class: "mirrored-bio",
                    if expanded() {
                        p { class: "bio-text bio-full", "{bio}" }
                    } else {
                        p { class: "bio-text bio-preview", "{bio_preview}" }
                    }
                    if has_more_bio {
                        button {
                            class: "bio-expand-btn",
                            onclick: move |_| expanded.set(!expanded()),
                            if expanded() { "Show less" } else { "Show more" }
                        }
                    }
                }
            } else {
                div { class: "mirrored-bio mirrored-bio-empty",
                    p { class: "bio-placeholder", "No bio provided" }
                }
            }

            // Mirror timestamps
            div { class: "mirrored-timestamps",
                div { class: "timestamp-item",
                    span { class: "timestamp-label", "Mirroring since: " }
                    span { class: "timestamp-value", "{pinned_since}" }
                }
                div { class: "timestamp-item",
                    span { class: "timestamp-label", "Last synced: " }
                    span { class: "timestamp-value timestamp-mirrored", "{last_mirrored}" }
                }
            }

            // Actions
            div { class: "mirrored-actions",
                button {
                    class: "unpin-btn",
                    title: "Stop mirroring this profile",
                    onclick: move |_| {
                        let did = did_for_unpin.clone();
                        spawn(async move {
                            let shared = engine();
                            let guard = shared.read().await;
                            if let Some(ref eng) = *guard {
                                match eng.unpin_profile(&did) {
                                    Ok(_) => {
                                        tracing::info!("Unpinned profile: {}", did);
                                        // TODO: Refresh the page
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to unpin profile: {:?}", e);
                                    }
                                }
                            }
                        });
                    },
                    "Stop Mirroring"
                }
            }
        }
    }
}
