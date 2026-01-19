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
            NavLocation::Network => "Your Network",
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
    /// Optional action button text (renders a button in the header)
    #[props(default = None)]
    pub action_text: Option<String>,
    /// Whether the action is currently in progress (shows loading state)
    #[props(default = false)]
    pub action_loading: bool,
    /// Callback when action button is clicked
    #[props(default = None)]
    pub on_action: Option<EventHandler<()>>,
}

/// Sacred Navigation Console component
///
/// A unified header that appears across all main pages (Field, Network, Profile).
/// Features:
/// - Compact, single-line layout
/// - Current location with glowing sigil
/// - Navigation to other sections
/// - Optional status indicator
/// - Optional action button
#[component]
pub fn NavHeader(props: NavHeaderProps) -> Element {
    let locations = [
        NavLocation::Field,
        NavLocation::Network,
        NavLocation::Profile,
    ];

    rsx! {
        header { class: "nav-header compact",
            div { class: "nav-inner",
                // Left: Current location with sigil
                div { class: "nav-current-location",
                    span { class: "nav-sigil", "{props.current.sigil()}" }
                    span { class: "nav-location-name", "{props.current.sacred_name()}" }
                }

                // Center: Status indicator (if provided)
                if let Some(status_text) = &props.status {
                    div { class: "nav-status",
                        span { class: "nav-status-dot" }
                        span { class: "nav-status-text", "{status_text}" }
                    }
                }

                // Action button (if provided)
                if let Some(action_text) = &props.action_text {
                    button {
                        class: if props.action_loading { "nav-action-btn loading" } else { "nav-action-btn" },
                        disabled: props.action_loading,
                        onclick: move |_| {
                            if let Some(handler) = &props.on_action {
                                handler.call(());
                            }
                        },
                        span { class: "action-icon", if props.action_loading { "⟳" } else { "↻" } }
                        span { class: "action-text", "{action_text}" }
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
                            }
                        }
                    }
                }
            }
        }
    }
}
