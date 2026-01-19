//! Peer Status Dropdown Component
//!
//! Shows connected peers with status and message capability.

use dioxus::prelude::*;
use syncengine_core::{Peer, PeerStatus};

/// Format timestamp as relative time string
fn format_relative_time(timestamp: u64) -> String {
    let now = chrono::Utc::now().timestamp() as u64;
    let elapsed = now.saturating_sub(timestamp);

    if elapsed < 60 {
        "Just now".to_string()
    } else if elapsed < 3600 {
        format!("{}m ago", elapsed / 60)
    } else if elapsed < 86400 {
        format!("{}h ago", elapsed / 3600)
    } else {
        format!("{}d ago", elapsed / 86400)
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PeerStatusDropdownProps {
    /// List of connected peers
    pub peers: Vec<Peer>,
    /// Whether currently syncing
    pub syncing: bool,
    /// Callback when dropdown is closed
    pub on_close: EventHandler<()>,
    /// Callback when Sync Now is clicked
    pub on_sync: EventHandler<()>,
    /// Callback when Message button is clicked for a peer
    pub on_message: EventHandler<(String, String)>, // (did, name)
}

/// Peer Status Dropdown
///
/// Displays list of connected peers with:
/// - Avatar (or initial)
/// - Display name
/// - Status dot (online/offline)
/// - Last seen time
/// - Message button
#[component]
pub fn PeerStatusDropdown(props: PeerStatusDropdownProps) -> Element {
    let online_count = props.peers.iter()
        .filter(|p| matches!(p.status, PeerStatus::Online))
        .count();

    rsx! {
        // Backdrop to close dropdown when clicking outside
        div {
            class: "peer-dropdown-backdrop",
            onclick: move |_| props.on_close.call(()),
        }

        div { class: "peer-dropdown",
            // Header
            header { class: "peer-dropdown-header",
                h3 { class: "peer-dropdown-title",
                    "Connected"
                    span { class: "peer-count-badge", "({online_count})" }
                }
            }

            // Peer list
            div { class: "peer-dropdown-list",
                if props.peers.is_empty() {
                    div { class: "peer-dropdown-empty",
                        p { "No peers connected" }
                        p { class: "peer-dropdown-hint", "Share your invite code to connect with others" }
                    }
                } else {
                    for peer in &props.peers {
                        {
                            let peer_did = peer.did.clone().unwrap_or_default();
                            let peer_name = peer.display_name();
                            let peer_did_for_click = peer_did.clone();
                            let peer_name_for_click = peer_name.clone();
                            let is_online = matches!(peer.status, PeerStatus::Online);
                            let first_char = peer_name.chars().next().unwrap_or('?').to_uppercase().to_string();
                            let last_seen = format_relative_time(peer.last_seen);

                            rsx! {
                                div {
                                    key: "{peer_did}",
                                    class: "peer-dropdown-item",

                                    // Avatar placeholder
                                    div { class: "peer-avatar-small",
                                        span { "{first_char}" }
                                    }

                                    // Info
                                    div { class: "peer-info",
                                        span { class: "peer-name", "{peer_name}" }
                                        div { class: "peer-meta",
                                            span {
                                                class: if is_online { "status-dot online" } else { "status-dot" }
                                            }
                                            span { class: "peer-last-seen", "{last_seen}" }
                                        }
                                    }

                                    // Message button
                                    button {
                                        class: "peer-message-btn",
                                        title: "Send message",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            props.on_message.call((
                                                peer_did_for_click.clone(),
                                                peer_name_for_click.clone(),
                                            ));
                                        },
                                        // Lucide message-square icon
                                        svg {
                                            xmlns: "http://www.w3.org/2000/svg",
                                            width: "16",
                                            height: "16",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "2",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            path { d: "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Footer with Sync Now button
            footer { class: "peer-dropdown-footer",
                button {
                    class: if props.syncing { "btn btn-primary sync-btn syncing" } else { "btn btn-primary sync-btn" },
                    disabled: props.syncing,
                    onclick: move |_| props.on_sync.call(()),
                    if props.syncing {
                        span { class: "sync-spinner" }
                        "Syncing..."
                    } else {
                        // Lucide refresh-cw icon
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            width: "14",
                            height: "14",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" }
                            path { d: "M21 3v5h-5" }
                            path { d: "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" }
                            path { d: "M8 16H3v5" }
                        }
                        "Sync Now"
                    }
                }
            }
        }
    }
}
