use std::sync::Arc;

use dioxus::prelude::*;
use tokio::sync::RwLock;

use crate::context::{get_data_dir, get_init_connect, get_init_profile_name, SharedEngine};
use crate::pages::{Field, Landing, Network, Profile, RealmView};
use crate::theme::GLOBAL_STYLES;

/// Application routes.
///
/// - `/` - Landing page with "Enter the Field" button
/// - `/field` - Main app view with realm sidebar and task list
/// - `/realms/:id` - Direct link to a specific realm
/// - `/profile` - Profile page with identity, peers, and stats
/// - `/network` - Network page with peers, pinners, and what you pin
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    Landing {},
    #[route("/field")]
    Field {},
    #[route("/realms/:id")]
    RealmView { id: String },
    #[route("/profile")]
    Profile {},
    #[route("/network")]
    Network {},
}

/// Root application component.
///
/// Provides global styles, engine context, and routing.
#[component]
pub fn App() -> Element {
    // Initialize shared engine state
    let engine: Signal<SharedEngine> = use_signal(|| Arc::new(RwLock::new(None)));
    let mut engine_ready: Signal<bool> = use_signal(|| false);

    // Provide engine context to all child components
    use_context_provider(|| engine);
    use_context_provider(|| engine_ready);

    // Initialize engine on mount
    use_effect(move || {
        spawn(async move {
            let data_dir = get_data_dir();
            match syncengine_core::SyncEngine::new(&data_dir).await {
                Ok(mut eng) => {
                    // Initialize identity for signing sync messages
                    // This is required for P2P sync to work
                    if let Err(e) = eng.init_identity() {
                        tracing::error!("Failed to initialize identity: {}", e);
                    }

                    // Initialize profile keys for the packet layer (direct messaging)
                    // This is required for send_message() and get_conversation() to work
                    if let Err(e) = eng.init_profile_keys() {
                        tracing::error!("Failed to initialize profile keys: {}", e);
                    }

                    // Set initial profile name if provided via --init-profile-name
                    // Only sets if the current display_name is empty or default (first launch)
                    if let Some(init_name) = get_init_profile_name() {
                        match eng.get_own_profile() {
                            Ok(mut profile) => {
                                if profile.display_name.is_empty() || profile.display_name == "Anonymous User" {
                                    profile.display_name = init_name.clone();
                                    if let Err(e) = eng.save_profile(&profile) {
                                        tracing::error!("Failed to set initial profile name: {}", e);
                                    } else {
                                        tracing::info!("Profile name set to '{}'", init_name);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Could not get profile for initial name setup: {}", e);
                            }
                        }
                    }

                    // Bootstrap auto-connect: connect with other named instances
                    // This allows ./se love joy peace to auto-connect all three instances
                    if let Some(bootstrap_peers) = get_init_connect() {
                        // Get our profile name to identify ourselves
                        let our_name = eng.get_own_profile()
                            .map(|p| p.display_name.to_lowercase())
                            .unwrap_or_default();

                        if !our_name.is_empty() && our_name != "anonymous user" {
                            // Bootstrap directory for sharing invites between instances
                            let bootstrap_dir = dirs::data_dir()
                                .unwrap_or_else(|| std::path::PathBuf::from("."))
                                .join("syncengine-bootstrap");

                            // Create directory if it doesn't exist
                            if let Err(e) = std::fs::create_dir_all(&bootstrap_dir) {
                                tracing::warn!("Could not create bootstrap directory: {}", e);
                            } else {
                                // Generate and write our invite for others to find
                                match eng.generate_contact_invite(24).await {
                                    Ok(invite_str) => {
                                        let our_invite_path = bootstrap_dir.join(format!("{}.invite", our_name));
                                        if let Err(e) = std::fs::write(&our_invite_path, &invite_str) {
                                            tracing::warn!("Could not write bootstrap invite: {}", e);
                                        } else {
                                            tracing::info!("Bootstrap invite written to {:?}", our_invite_path);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Could not generate bootstrap invite: {}", e);
                                    }
                                }

                                // Small delay to let other instances write their invites
                                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                                // Read invites from other bootstrap peers and send contact requests
                                for peer_name in &bootstrap_peers {
                                    let peer_name_lower = peer_name.to_lowercase();

                                    // Skip ourselves
                                    if peer_name_lower == our_name {
                                        continue;
                                    }

                                    let peer_invite_path = bootstrap_dir.join(format!("{}.invite", peer_name_lower));

                                    // Try to read the peer's invite file
                                    if let Ok(invite_str) = std::fs::read_to_string(&peer_invite_path) {
                                        // Decode and send contact request
                                        match eng.decode_contact_invite(&invite_str).await {
                                            Ok(invite) => {
                                                match eng.send_contact_request(invite).await {
                                                    Ok(()) => {
                                                        tracing::info!(
                                                            "Bootstrap: sent contact request to '{}'",
                                                            peer_name
                                                        );
                                                    }
                                                    Err(e) => {
                                                        // May fail if already connected or request pending
                                                        tracing::debug!(
                                                            "Bootstrap: could not send request to '{}': {}",
                                                            peer_name, e
                                                        );
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                tracing::debug!(
                                                    "Bootstrap: could not decode invite from '{}': {}",
                                                    peer_name, e
                                                );
                                            }
                                        }
                                    } else {
                                        tracing::debug!(
                                            "Bootstrap: no invite found for '{}' (may not be running yet)",
                                            peer_name
                                        );
                                    }
                                }
                            }
                        }
                    }

                    // Perform immediate startup sync with known peers
                    // Uses jitter to avoid the simultaneous wake-up problem
                    match eng.startup_sync().await {
                        Ok(result) => {
                            tracing::info!(
                                "Startup sync complete: {} succeeded, {} attempted, {} skipped (backoff), jitter={}ms",
                                result.peers_succeeded,
                                result.peers_attempted,
                                result.peers_skipped_backoff,
                                result.jitter_delay_ms
                            );
                        }
                        Err(e) => {
                            // Non-fatal - app continues, will retry via background task
                            tracing::warn!("Startup sync failed (will retry in background): {}", e);
                        }
                    }

                    let shared = engine();
                    let mut guard = shared.write().await;
                    *guard = Some(eng);
                    drop(guard);
                    engine_ready.set(true);
                    tracing::info!("SyncEngine initialized with identity");
                }
                Err(e) => {
                    tracing::error!("Failed to initialize SyncEngine: {}", e);
                }
            }
        });
    });

    rsx! {
        style { {GLOBAL_STYLES} }
        Router::<Route> {}
    }
}
