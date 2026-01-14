//! Stewardship Stats - Activity metrics display.

use dioxus::prelude::*;

/// Props for the stewardship stats component.
#[derive(Props, Clone, PartialEq)]
pub struct StewardshipStatsProps {
    /// Number of realms joined
    pub realm_count: u32,
    /// Number of intentions created
    pub intention_count: u32,
    /// Number of intentions manifested (completed)
    pub manifested_count: u32,
}

/// Stewardship stats card showing activity metrics.
#[component]
pub fn StewardshipStats(props: StewardshipStatsProps) -> Element {
    rsx! {
        div { class: "stewardship-stats",
            h2 { class: "section-title", "STEWARDSHIP" }
            p { class: "subtitle", "Your presence in the field" }

            // Stats grid
            div { class: "stats-grid",
                div { class: "stat-box",
                    div { class: "stat-value", "{props.realm_count}" }
                    div { class: "stat-label", "Realms" }
                }
                div { class: "stat-box",
                    div { class: "stat-value", "{props.intention_count}" }
                    div { class: "stat-label", "Intentions" }
                }
                div { class: "stat-box",
                    div { class: "stat-value", "{props.manifested_count}" }
                    div { class: "stat-label", "Manifested" }
                }
            }

            // Future: Recent activity feed
            // button { class: "btn-badge", "View All Intentions" }
        }
    }
}
