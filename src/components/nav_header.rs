//! Navigation Header Component
//!
//! Desktop: Horizontal header with app title, nav links, status orb
//! Mobile: Hidden (replaced by MobileNav)

use dioxus::prelude::*;
use syncengine_core::{Peer, PeerStatus};

use crate::app::Route;
use crate::components::messages::MessageCompose;
use crate::components::mobile_nav::MobileNav;
use crate::components::PeerStatusDropdown;
use crate::context::{use_engine, use_engine_ready};

/// Navigation location within the application
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NavLocation {
    Field,
    Network,
    Profile,
}

impl NavLocation {
    /// Get the display name for this location
    pub fn display_name(&self) -> &'static str {
        match self {
            NavLocation::Field => "Tasks",
            NavLocation::Network => "Network",
            NavLocation::Profile => "Profile",
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
}

#[derive(Props, Clone, PartialEq)]
pub struct NavHeaderProps {
    /// Current location in the app
    pub current: NavLocation,
}

/// Navigation Header component
///
/// Desktop header with:
/// - Left: "Synchronicity Engine" title (gold serif)
/// - Center: Navigation links with Lucide icons
/// - Right: Status orb with peer count dropdown
#[component]
pub fn NavHeader(props: NavHeaderProps) -> Element {
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // State
    let mut show_dropdown = use_signal(|| false);
    let mut peers: Signal<Vec<Peer>> = use_signal(Vec::new);
    let mut syncing = use_signal(|| false);
    let mut compose_target: Signal<Option<(String, String)>> = use_signal(|| None);

    let locations = [
        NavLocation::Field,
        NavLocation::Network,
        NavLocation::Profile,
    ];

    // Load peers when engine ready
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let guard = shared.read().await;
                if let Some(ref eng) = *guard {
                    if let Ok(peer_list) = eng.list_peer_contacts() {
                        peers.set(peer_list);
                    }
                }
            });
        }
    });

    // Poll for peer updates
    use_effect(move || {
        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                if engine_ready() {
                    let shared = engine();
                    let guard = shared.read().await;
                    if let Some(ref eng) = *guard {
                        if let Ok(peer_list) = eng.list_peer_contacts() {
                            peers.set(peer_list);
                        }
                    }
                }
            }
        });
    });

    // Sync handler
    let on_sync = move |_: ()| {
        if syncing() { return; }
        syncing.set(true);

        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;
            if let Some(ref mut eng) = *guard {
                match eng.manual_sync().await {
                    Ok(_) => {
                        if let Ok(peer_list) = eng.list_peer_contacts() {
                            peers.set(peer_list);
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

    // Message handler
    let on_message = move |(did, name): (String, String)| {
        compose_target.set(Some((did, name)));
        show_dropdown.set(false);
    };

    // Send message handler
    let send_message = move |content: String| {
        if let Some((did, _name)) = compose_target() {
            spawn(async move {
                let shared = engine();
                let mut guard = shared.write().await;
                if let Some(ref mut eng) = *guard {
                    // Use send_message which properly sets recipient and routes to 1:1 topic
                    match eng.send_message(&did, &content).await {
                        Ok(seq) => {
                            tracing::info!(to = %did, sequence = seq, "Sent message");
                        }
                        Err(e) => {
                            tracing::error!(error = %e, "Failed to send message");
                        }
                    }
                }
                compose_target.set(None);
            });
        }
    };

    let peer_count = peers().len();
    let online_count = peers().iter()
        .filter(|p| matches!(p.status, PeerStatus::Online))
        .count();

    rsx! {
        header { class: "nav-header-v2",
            div { class: "nav-header-inner",
                // Left: App title
                div { class: "nav-title",
                    h1 { class: "app-title", "Synchronicity Engine" }
                }

                // Center: Navigation links
                nav { class: "nav-links-v2",
                    for location in &locations {
                        Link {
                            to: location.route(),
                            class: if *location == props.current { "nav-link-v2 active" } else { "nav-link-v2" },

                            // Icon
                            span { class: "nav-link-icon",
                                {render_nav_icon(*location)}
                            }

                            // Label
                            span { class: "nav-link-label", "{location.display_name()}" }
                        }
                    }
                }

                // Right: Status orb with peer count
                div { class: "nav-status-v2",
                    button {
                        r#type: "button",
                        class: if syncing() { "status-orb-btn syncing" } else { "status-orb-btn" },
                        onclick: move |_| show_dropdown.set(!show_dropdown()),
                        "aria-label": "Connection status - {online_count} peers online",
                        "aria-expanded": "{show_dropdown()}",

                        span { class: if syncing() { "status-orb syncing" } else { "status-orb" } }
                        span { class: "peer-count", "{peer_count}" }

                        // Dropdown chevron
                        svg {
                            class: "dropdown-chevron",
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "12",
                            height: "12",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "m6 9 6 6 6-6" }
                        }
                    }
                }
            }
        }

        // Mobile navigation (hidden on desktop via CSS)
        MobileNav {
            current: props.current,
            peer_count: peer_count,
            syncing: syncing(),
            on_status_click: move |_| show_dropdown.set(!show_dropdown()),
        }

        // Dropdown (rendered at root level so it works on both desktop and mobile)
        if show_dropdown() {
            PeerStatusDropdown {
                peers: peers(),
                syncing: syncing(),
                on_close: move |_| show_dropdown.set(false),
                on_sync: on_sync,
                on_message: on_message,
            }
        }

        // Message compose modal
        if let Some((did, name)) = compose_target() {
            MessageCompose {
                recipient_name: name,
                recipient_did: did,
                on_send: send_message,
                on_close: move |_| compose_target.set(None),
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
                width: "18",
                height: "18",
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
                width: "18",
                height: "18",
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
                width: "18",
                height: "18",
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
