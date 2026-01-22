//! Bio Card Component
//!
//! A contact display card following the GoldenCard pattern with landscape orientation.
//! Features a 38.2% / 61.8% left/right split with profile image and contact details.

use dioxus::prelude::*;

use crate::components::cards::{CardOrientation, GoldenCard};
use crate::components::images::AsyncImage;
use syncengine_core::types::peer::{Peer, PeerStatus};

// Embed default profile image as base64 data URI
const PROFILE_DEFAULT_BYTES: &[u8] = include_bytes!("../../../assets/profile-default.png");

fn profile_default_uri() -> String {
    use base64::Engine;
    let base64 = base64::engine::general_purpose::STANDARD.encode(PROFILE_DEFAULT_BYTES);
    format!("data:image/png;base64,{}", base64)
}

/// Format timestamp as relative time string
fn format_relative_time(timestamp: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let elapsed = now.saturating_sub(timestamp);

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

/// Bio Card
///
/// A contact display card using the GoldenCard landscape layout with:
/// - Left (38.2%): Profile avatar with status indicator overlay
/// - Right (61.8%): Name, tagline, last seen, and shared realms
///
/// # Example
///
/// ```rust
/// rsx! {
///     BioCard {
///         peer: my_peer,
///         interactive: true,
///         onclick: move |peer| { /* Handle click */ },
///     }
/// }
/// ```
#[component]
pub fn BioCard(
    /// The peer to display
    peer: Peer,
    /// Enable hover effects and cursor pointer
    #[props(default = false)]
    interactive: bool,
    /// Optional click handler
    #[props(default = None)]
    onclick: Option<EventHandler<Peer>>,
    /// Mutual peers (contacts we share in common with this peer)
    /// Computed dynamically via engine.compute_mutual_peers_with()
    #[props(default = Vec::new())]
    mutual_peers: Vec<String>,
) -> Element {
    let is_online = peer.status == PeerStatus::Online;
    let status_class = if is_online { "online" } else { "offline" };

    // Extract display info
    let display_name = peer.display_name();
    let subtitle = peer.profile.as_ref().and_then(|p| p.subtitle.clone());
    let avatar_blob_id = peer.profile.as_ref().and_then(|p| p.avatar_blob_id.clone());

    // Format last seen
    let last_seen_text = if is_online {
        "Just now".to_string()
    } else {
        format_relative_time(peer.last_seen)
    };

    // Format shared realms count
    let realm_count = peer.shared_realms.len();
    let realms_text = match realm_count {
        0 => "No shared realms".to_string(),
        1 => "1 shared realm".to_string(),
        n => format!("{} shared realms", n),
    };

    // Format mutual peers count
    let mutual_count = mutual_peers.len();
    let mutual_text = match mutual_count {
        0 => "No mutual contacts".to_string(),
        1 => "1 mutual contact".to_string(),
        n => format!("{} mutual contacts", n),
    };

    // Clone peer for the click handler
    let peer_for_click = peer.clone();

    let handle_click = move |_| {
        if let Some(handler) = &onclick {
            handler.call(peer_for_click.clone());
        }
    };

    rsx! {
        div {
            class: "bio-card",
            onclick: handle_click,

            GoldenCard {
                orientation: CardOrientation::Landscape,
                interactive: interactive,

                // Left: Avatar with status overlay
                div { class: "card-image-area bio-card__avatar-area",
                    if let Some(blob_id) = avatar_blob_id {
                        AsyncImage {
                            blob_id: blob_id,
                            alt: display_name.clone(),
                            class: Some("card-image__avatar".to_string()),
                        }
                    } else {
                        img {
                            class: "card-image__avatar",
                            src: "{profile_default_uri()}",
                            alt: "{display_name}",
                        }
                    }

                    // Status dot overlay
                    div {
                        class: "bio-card__status-dot {status_class}",
                        title: if is_online { "Online" } else { "Offline" },
                    }
                }

                // Right: Contact details
                div { class: "card-content bio-card__content",
                    // Header: Name + Tagline
                    div { class: "bio-card__header",
                        h3 { class: "bio-card__name", "{display_name}" }

                        if let Some(ref tagline) = subtitle {
                            span { class: "bio-card__tagline", "{tagline}" }
                        }
                    }

                    // Stats
                    div { class: "bio-card__stats",
                        div { class: "bio-card__stat",
                            span { class: "bio-card__stat-label", "Last seen" }
                            span { class: "bio-card__stat-value", "{last_seen_text}" }
                        }

                        div { class: "bio-card__stat",
                            span { class: "bio-card__stat-label", "Realms" }
                            span { class: "bio-card__stat-value", "{realms_text}" }
                        }

                        div { class: "bio-card__stat",
                            span { class: "bio-card__stat-label", "Mutual" }
                            span { class: "bio-card__stat-value", "{mutual_text}" }
                        }
                    }
                }
            }
        }
    }
}
