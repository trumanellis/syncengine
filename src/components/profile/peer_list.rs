//! Peer List - Container for peer cards, organized by status.

use dioxus::prelude::*;
use syncengine_core::{PeerInfo, PeerStatus};

use super::PeerCard;

/// Props for the peer list component.
#[derive(Props, Clone, PartialEq)]
pub struct PeerListProps {
    /// List of all peers
    pub peers: Vec<PeerInfo>,
}

/// Peer list showing online and offline peers.
#[component]
pub fn PeerList(props: PeerListProps) -> Element {
    // Separate peers by status
    let online: Vec<_> = props
        .peers
        .iter()
        .filter(|p| p.status == PeerStatus::Online)
        .collect();

    let offline: Vec<_> = props
        .peers
        .iter()
        .filter(|p| p.status != PeerStatus::Online)
        .collect();

    // Calculate counts before consuming vectors
    let online_count = online.len();
    let offline_count = offline.len();
    let is_empty = online_count == 0 && offline_count == 0;

    rsx! {
        div { class: "peer-list",
            h2 { class: "section-title", "FIELD CONNECTIONS" }

            // Resonating Now section (online peers)
            if online_count > 0 {
                div { class: "peer-section",
                    h3 { class: "peer-section-header",
                        "Resonating Now ({online_count})"
                    }
                    for peer in online {
                        PeerCard {
                            key: "{hex::encode(peer.endpoint_id)}",
                            peer: peer.clone(),
                        }
                    }
                }
            }

            // Recently Seen section (offline peers)
            if offline_count > 0 {
                div { class: "peer-section",
                    h3 { class: "peer-section-header",
                        "Recently Seen ({offline_count})"
                    }
                    for peer in offline {
                        PeerCard {
                            key: "{hex::encode(peer.endpoint_id)}",
                            peer: peer.clone(),
                        }
                    }
                }
            }

            // Empty state
            if is_empty {
                div { class: "empty-state",
                    p { "No field connections yet" }
                    p { class: "hint", "Join a realm to connect with others" }
                }
            }
        }
    }
}
