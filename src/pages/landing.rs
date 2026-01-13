//! Landing page - Entry point to the Synchronicity Engine.
//!
//! "Enter the Field" - The gateway to collective intention.
//!
//! If the user already has realms (returning user), auto-redirects to Field.

use dioxus::prelude::*;

use crate::app::Route;
use crate::components::{FieldState, FieldStatus};
use crate::context::{use_engine, use_engine_ready};

/// Landing page component.
///
/// Auto-redirects to Field if user already has realms (returning user).
#[component]
pub fn Landing() -> Element {
    let navigator = use_navigator();
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // Auto-redirect returning users to the Field
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let guard = shared.read().await;
                if let Some(ref eng) = *guard {
                    // Check if user has any realms (returning user)
                    if let Ok(realms) = eng.list_realms().await {
                        if !realms.is_empty() {
                            tracing::info!(
                                "Returning user detected ({} realms), auto-navigating to Field",
                                realms.len()
                            );
                            navigator.push(Route::Field {});
                        }
                    }
                }
            });
        }
    });

    let enter_field = move |_| {
        navigator.push(Route::Field {});
    };

    rsx! {
        main { class: "landing",
            // Sacred geometry background
            div { class: "seed-of-life-bg" }

            header { class: "landing-header",
                h1 { class: "page-title", "Synchronicity Engine" }
                p { class: "tagline",
                    "a decentralized organism of collective awakening"
                }

                div { style: "margin-top: 2rem;",
                    FieldStatus { status: FieldState::Resonating }
                }

                button {
                    class: "btn-enter",
                    onclick: enter_field,
                    "Enter the Field"
                }
            }

            section { class: "vision-section",
                h2 { class: "section-header", "The Vision" }
                p { class: "body-text", style: "margin-top: 1rem;",
                    "The "
                    span { class: "sacred-term", "Synchronicity Engine" }
                    " is a "
                    span { class: "tech-term", "decentralized, peer-to-peer" }
                    " organism for collective awakening. Share "
                    span { class: "sacred-term", "intentions" }
                    ", collaborate on tasks, and weave the fabric of mutual support. "
                    "No central servers. No gatekeepers. Just peers synchronizing across space and time."
                }
            }
        }
    }
}
