//! Landing page - Entry point to the Synchronicity Engine.
//!
//! "Enter the Field" - The gateway to collective intention.

use dioxus::prelude::*;

use crate::app::Route;
use crate::components::{FieldState, FieldStatus};

/// Landing page component.
#[component]
pub fn Landing() -> Element {
    let navigator = use_navigator();

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
