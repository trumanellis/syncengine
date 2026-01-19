//! Network Page - Contacts You Mirror
//!
//! Displays your contacts' profiles for P2P redundancy.
//! Now uses the unified Peer table as the single source of truth.

use dioxus::prelude::*;
use syncengine_core::Peer;

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
fn format_relative_time(timestamp: u64) -> String {
    let now = chrono::Utc::now().timestamp() as u64;
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

/// Network page - Contacts You Mirror
#[component]
pub fn Network() -> Element {
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // State for contacts (using unified Peer table)
    let mut contacts: Signal<Vec<Peer>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut syncing = use_signal(|| false);

    // Load contacts when engine becomes ready
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let guard = shared.read().await;

                if let Some(ref eng) = *guard {
                    // Use unified Peer table - single source of truth
                    if let Ok(contact_list) = eng.list_peer_contacts() {
                        contacts.set(contact_list);
                    }

                    loading.set(false);
                }
            });
        }
    });

    // Handler for manual sync button
    let on_sync_click = move |_: ()| {
        if syncing() {
            return; // Prevent double-clicks
        }

        syncing.set(true);

        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;

            if let Some(ref mut eng) = *guard {
                match eng.manual_sync().await {
                    Ok(contacts_count) => {
                        tracing::info!("Manual sync completed: {} contacts", contacts_count);

                        // Refresh the contacts list after sync
                        if let Ok(contact_list) = eng.list_peer_contacts() {
                            contacts.set(contact_list);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Manual sync failed: {:?}", e);
                    }
                }
            }

            syncing.set(false);
        });
    };

    let contact_count = contacts().len();
    let status_text = format!("{} mirrored", contact_count);
    let action_text = if syncing() { "Syncing..." } else { "Sync" };

    rsx! {
        div { class: "network-page",
            // Compact Navigation Header with Sync action
            NavHeader {
                current: NavLocation::Network,
                status: Some(status_text),
                action_text: Some(action_text.to_string()),
                action_loading: syncing(),
                on_action: on_sync_click,
            }

            if loading() {
                div { class: "loading-state",
                    div { class: "loading-orb" }
                    p { "Loading contacts..." }
                }
            } else {
                div { class: "network-content",
                    // Profiles You Mirror
                    section { class: "network-section mirrored-profiles-section",
                        h2 { class: "section-title", "Profiles You Mirror" }
                        p { class: "section-subtitle", "Full profiles of contacts you're carrying in the network" }

                        if contacts().is_empty() {
                            div { class: "empty-state",
                                p { "You are not mirroring any profiles yet. Add contacts to automatically mirror their profiles." }
                            }
                        } else {
                            div { class: "mirrored-profiles-grid",
                                for contact in contacts() {
                                    ContactProfileCard { peer: contact }
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

#[derive(Props, Clone, PartialEq)]
struct ContactProfileCardProps {
    peer: Peer,
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

/// Full profile card for contacts - shows complete profile information
#[component]
fn ContactProfileCard(props: ContactProfileCardProps) -> Element {
    let mut expanded = use_signal(|| false);

    let display_name = props.peer.display_name();
    let profile = props.peer.profile.as_ref();
    let subtitle = profile.and_then(|p| p.subtitle.clone());
    let bio = profile.map(|p| p.bio.clone()).unwrap_or_default();
    let avatar_blob_id = profile.and_then(|p| p.avatar_blob_id.clone());
    let last_seen = format_relative_time(props.peer.last_seen);

    let did_display = if let Some(ref did) = props.peer.did {
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
    } else {
        format!("peer_{}", hex::encode(&props.peer.endpoint_id[..4]))
    };

    // Determine if bio should show expand option
    let bio_preview = truncate_bio(&bio, 150);
    let has_more_bio = bio.len() > 150;

    // Status indicator
    let status_class = match props.peer.status {
        syncengine_core::PeerStatus::Online => "status-online",
        syncengine_core::PeerStatus::Offline => "status-offline",
        syncengine_core::PeerStatus::Unknown => "status-unknown",
    };

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
                }

                // Status badge
                span { class: "mirrored-relationship-badge {status_class}",
                    "{props.peer.status}"
                }
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
                        if has_more_bio {
                            button {
                                class: "bio-toggle",
                                onclick: move |_| expanded.set(false),
                                "Show less"
                            }
                        }
                    } else {
                        p { class: "bio-text bio-preview", "{bio_preview}" }
                        if has_more_bio {
                            button {
                                class: "bio-toggle",
                                onclick: move |_| expanded.set(true),
                                "Show more"
                            }
                        }
                    }
                }
            }

            // Footer with metadata
            div { class: "mirrored-footer",
                span { class: "mirrored-meta", "Last seen: {last_seen}" }
            }
        }
    }
}
