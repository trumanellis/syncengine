//! Sacred Navigation Console - Unified Header Component
//!
//! A mystical terminal-style navigation header that spans all pages.
//! Features sacred geometry accents and cyber-mystical aesthetic.

use dioxus::prelude::*;

use crate::app::Route;

/// Navigation location within the application
#[derive(Clone, Copy, PartialEq)]
pub enum NavLocation {
    Field,
    Network,
    Profile,
}

impl NavLocation {
    /// Get the sacred name for this location
    pub fn sacred_name(&self) -> &'static str {
        match self {
            NavLocation::Field => "The Field",
            NavLocation::Network => "The Network",
            NavLocation::Profile => "Identity",
        }
    }

    /// Get the route for this location
    pub fn route(&self) -> Route {
        match self {
            NavLocation::Field => Route::Field {},
            NavLocation::Network => Route::Network {},
            NavLocation::Profile => Route::Profile {},
        }
    }

    /// Get the sigil (icon) for this location
    pub fn sigil(&self) -> &'static str {
        match self {
            NavLocation::Field => "◈",  // Sacred geometry diamond
            NavLocation::Network => "∴", // Therefore symbol / three dots in triangle
            NavLocation::Profile => "⬡", // Hexagon
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct NavHeaderProps {
    /// Current location in the app
    pub current: NavLocation,
    /// Optional status indicator (e.g., "field resonating", "3 connections")
    #[props(default = None)]
    pub status: Option<String>,
}

/// Sacred Navigation Console component
///
/// A unified header that appears across all main pages (Field, Network, Profile).
/// Features:
/// - Sacred geometry divider lines
/// - Current location with glowing sigil
/// - Navigation to other sections
/// - Optional status indicator
/// - Mystical terminal aesthetic
#[component]
pub fn NavHeader(props: NavHeaderProps) -> Element {
    let locations = [
        NavLocation::Field,
        NavLocation::Network,
        NavLocation::Profile,
    ];

    rsx! {
        header { class: "nav-header",
            // Sacred geometry border accent (top line)
            div { class: "nav-border-accent" }

            div { class: "nav-inner",
                // Left: Current location with sigil
                div { class: "nav-current-location",
                    span { class: "nav-sigil pulsing", "{props.current.sigil()}" }
                    h1 { class: "nav-location-name", "{props.current.sacred_name()}" }
                }

                // Center: Status indicator (if provided)
                if let Some(status_text) = &props.status {
                    div { class: "nav-status",
                        span { class: "nav-status-dot" }
                        span { class: "nav-status-text", "{status_text}" }
                    }
                }

                // Right: Navigation links to other locations
                nav { class: "nav-links",
                    for location in &locations {
                        if *location != props.current {
                            Link {
                                to: location.route(),
                                class: "nav-link",
                                title: format!("Navigate to {}", location.sacred_name()),
                                span { class: "nav-link-sigil", "{location.sigil()}" }
                                span { class: "nav-link-text", "{location.sacred_name()}" }
                            }
                        }
                    }
                }
            }

            // Sacred geometry border accent (bottom line with notches)
            div { class: "nav-border-accent-bottom" }
        }
    }
}
