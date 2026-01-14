//! Peer Card - Individual peer display with status and metrics.

use dioxus::prelude::*;
use syncengine_core::{PeerInfo, PeerStatus};

/// Props for the peer card component.
#[derive(Props, Clone, PartialEq)]
pub struct PeerCardProps {
    /// Peer information
    pub peer: PeerInfo,
}

/// Format timestamp as relative time string.
fn format_relative_time(timestamp: u64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

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

/// Peer card showing connection status and metrics.
#[component]
pub fn PeerCard(props: PeerCardProps) -> Element {
    let is_online = props.peer.status == PeerStatus::Online;

    // Format peer ID (first 4 bytes as hex)
    let peer_id_short = hex::encode(&props.peer.endpoint_id[..4]);

    // Format last seen
    let last_seen = if is_online {
        "Now".to_string()
    } else {
        format_relative_time(props.peer.last_seen)
    };

    rsx! {
        div { class: "peer-card",
            // Status and name row
            div { class: "peer-status",
                span {
                    class: if is_online {
                        "peer-status-dot online"
                    } else {
                        "peer-status-dot offline"
                    },
                }

                span {
                    class: if props.peer.nickname.is_some() {
                        "peer-name"
                    } else {
                        "peer-id"
                    },
                    if let Some(ref name) = props.peer.nickname {
                        "{name}"
                    } else {
                        "{peer_id_short}"
                    }
                }

                span { class: "peer-last-seen", "Last seen: {last_seen}" }
            }

            // Metrics row
            div { class: "peer-metrics",
                if props.peer.connection_attempts > 0 {
                    span {
                        "Connection: {props.peer.success_rate() * 100.0:.0}%"
                    }
                }

                span {
                    "{props.peer.shared_realms.len()} realms in common"
                }
            }

            // Actions row (placeholder for future features)
            div { class: "peer-actions",
                button {
                    class: "btn-ghost",
                    onclick: move |_| {
                        tracing::info!("View peer details: {:?}", props.peer.endpoint_id);
                    },
                    "View"
                }
                button {
                    class: "btn-ghost",
                    onclick: move |_| {
                        tracing::info!("Set nickname for peer: {:?}", props.peer.endpoint_id);
                    },
                    "Set Nickname"
                }
            }
        }
    }
}
