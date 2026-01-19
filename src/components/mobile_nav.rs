//! Mobile Navigation Component
//!
//! Bottom navigation bar for mobile devices (< 768px).

use dioxus::prelude::*;

use crate::components::nav_header::NavLocation;

#[derive(Props, Clone, PartialEq)]
pub struct MobileNavProps {
    /// Current active location
    pub current: NavLocation,
    /// Number of connected peers
    pub peer_count: usize,
    /// Whether currently syncing
    pub syncing: bool,
    /// Callback when status orb is clicked
    pub on_status_click: EventHandler<()>,
}

/// Mobile bottom navigation bar
///
/// Replaces header on screens < 768px.
/// Shows: Tasks | Network | Profile | Status Orb
#[component]
pub fn MobileNav(props: MobileNavProps) -> Element {
    let locations = [
        NavLocation::Field,
        NavLocation::Network,
        NavLocation::Profile,
    ];

    rsx! {
        nav { class: "mobile-nav",
            // Navigation items
            for location in &locations {
                Link {
                    to: location.route(),
                    class: if *location == props.current { "mobile-nav-item active" } else { "mobile-nav-item" },

                    // Icon
                    span { class: "mobile-nav-icon",
                        {render_nav_icon(*location)}
                    }

                    // Label (hidden by default, shown on active)
                    span { class: "mobile-nav-label", "{location.display_name()}" }
                }
            }

            // Status orb
            button {
                class: if props.syncing { "mobile-nav-status syncing" } else { "mobile-nav-status" },
                onclick: move |_| props.on_status_click.call(()),
                "aria-label": "Connection status",

                span { class: "status-orb" }
                if props.peer_count > 0 {
                    span { class: "peer-count-mini", "{props.peer_count}" }
                }
            }
        }
    }
}

/// Render Lucide icon for navigation location
fn render_nav_icon(location: NavLocation) -> Element {
    match location {
        NavLocation::Field => rsx! {
            // Lucide check-square icon (Tasks)
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "24",
                height: "24",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "m9 11 3 3L22 4" }
                path { d: "M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11" }
            }
        },
        NavLocation::Network => rsx! {
            // Lucide users icon
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "24",
                height: "24",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" }
                circle { cx: "9", cy: "7", r: "4" }
                path { d: "M22 21v-2a4 4 0 0 0-3-3.87" }
                path { d: "M16 3.13a4 4 0 0 1 0 7.75" }
            }
        },
        NavLocation::Profile => rsx! {
            // Lucide user icon
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "24",
                height: "24",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                circle { cx: "12", cy: "8", r: "5" }
                path { d: "M20 21a8 8 0 0 0-16 0" }
            }
        },
    }
}
