use std::sync::Arc;

use dioxus::prelude::*;
use tokio::sync::RwLock;

use crate::context::{get_data_dir, SharedEngine};
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
