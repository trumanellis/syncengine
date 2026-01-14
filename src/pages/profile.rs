//! Profile Page - View and edit user identity with ProfileCard
//!
//! Displays user's profile with avatar, bio, quest gallery using golden rectangle design.

use dioxus::prelude::*;
use syncengine_core::UserProfile;

use crate::app::Route;
use crate::components::cards::ProfileCard;
use crate::context::{use_engine, use_engine_ready};

/// Profile page showing identity with ProfileCard
#[component]
pub fn Profile() -> Element {
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // State for loaded profile
    let mut profile: Signal<Option<UserProfile>> = use_signal(|| None);
    let mut loading = use_signal(|| true);

    // Load profile when engine becomes ready
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let guard = shared.read().await;

                if let Some(ref eng) = *guard {
                    // Load profile using DID (available immediately, no waiting needed)
                    match eng.get_own_profile() {
                        Ok(prof) => {
                            profile.set(Some(prof));
                            loading.set(false);
                        }
                        Err(e) => {
                            tracing::error!("Failed to load profile: {:?}", e);
                            loading.set(false);
                        }
                    }
                }
            });
        }
    });

    // Save profile handler
    let save_profile = move |updated: UserProfile| {
        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;
            if let Some(ref mut eng) = *guard {
                match eng.save_profile(&updated) {
                    Ok(_) => {
                        tracing::info!("Profile saved successfully");
                        profile.set(Some(updated));
                    }
                    Err(e) => {
                        tracing::error!("Failed to save profile: {:?}", e);
                    }
                }
            }
        });
    };

    rsx! {
        div { class: "profile-page",
            // Header with back navigation
            header { class: "profile-header",
                Link {
                    to: Route::Field {},
                    button {
                        class: "back-button",
                        title: "Return to Field",
                        "← Back to Field"
                    }
                }
            }

            // Profile content
            div { class: "profile-content",
                if loading() {
                    div { class: "loading",
                        "Loading profile..."
                    }
                } else if let Some(p) = profile() {
                    ProfileCard {
                        profile: p,
                        editable: true,
                        show_qr: true,
                        on_update: save_profile,
                    }
                } else {
                    div { class: "error",
                        "Failed to load profile"
                    }
                }
            }
        }
    }
}
