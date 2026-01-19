//! Network Page
//!
//! Displays your contacts' profiles.
//! Uses the unified Peer table as the single source of truth.

use dioxus::prelude::*;
use syncengine_core::{ContactEvent, Peer, UserProfile};

use crate::components::cards::ProfileCard;
use crate::components::{NavHeader, NavLocation};
use crate::context::{use_engine, use_engine_ready};

/// Format timestamp as relative time string
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

/// Convert a Peer to UserProfile for display in ProfileCard
fn peer_to_user_profile(peer: &Peer) -> UserProfile {
    let profile = peer.profile.as_ref();

    UserProfile {
        peer_id: hex::encode(&peer.endpoint_id),
        display_name: peer.display_name(),
        subtitle: profile.and_then(|p| p.subtitle.clone()),
        profile_link: None, // Contacts don't have profile links
        avatar_blob_id: profile.and_then(|p| p.avatar_blob_id.clone()),
        bio: profile.map(|p| p.bio.clone()).unwrap_or_default(),
        top_quests: vec![], // TODO: load from contact's profile
        created_at: 0,
        updated_at: peer.last_seen as i64,
    }
}

/// Network page - displays contacts.
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
            // Initial load
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

            // Subscribe to contact events for real-time profile updates
            spawn(async move {
                let shared = engine();
                let mut guard = shared.write().await;

                if let Some(ref mut eng) = *guard {
                    // Subscribe to contact events
                    match eng.subscribe_contact_events().await {
                        Ok(mut event_rx) => {
                            // Release the lock before the event loop
                            drop(guard);

                            tracing::debug!("Network page subscribed to contact events");

                            // Listen for profile updates
                            while let Ok(event) = event_rx.recv().await {
                                if let ContactEvent::ProfileUpdated { did } = event {
                                    tracing::debug!(did = %did, "Profile updated event received, refreshing contacts");

                                    // Refresh contacts list
                                    let shared = engine();
                                    let guard = shared.read().await;
                                    if let Some(ref eng) = *guard {
                                        if let Ok(contact_list) = eng.list_peer_contacts() {
                                            contacts.set(contact_list);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Failed to subscribe to contact events");
                        }
                    }
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
                    // Mirrored profiles grid
                    section { class: "network-section mirrored-profiles-section",
                        if contacts().is_empty() {
                            div { class: "empty-state",
                                p { "You are not mirroring any profiles yet. Add contacts to automatically mirror their profiles." }
                            }
                        } else {
                            div { class: "mirrored-profiles-grid",
                                for contact in contacts() {
                                    ProfileCard {
                                        profile: peer_to_user_profile(&contact),
                                        editable: false,
                                        show_qr: false,
                                        compact: true,
                                        did: contact.did.clone(),
                                        status: Some(contact.status.to_string()),
                                        last_seen: Some(format_relative_time(contact.last_seen)),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}