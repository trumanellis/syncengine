//! Network Page - Field Topology
//!
//! Displays the P2P network state with unified peer list:
//! - Connections: All known peers (contacts and discovered) in one list
//! - Souls Carrying Your Light: Peers who are pinning your profile
//! - Souls You Carry: Profiles YOU are pinning for others

use dioxus::prelude::*;
use syncengine_core::{NetworkStats, Peer, PeerStatus, PinnerInfo, ProfilePin};

use crate::app::Route;
use crate::context::{use_engine, use_engine_ready};

/// Format timestamp as relative time string.
fn format_relative_time(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let elapsed = (now - timestamp).max(0);

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

/// Format timestamp for Unix timestamps (u64)
fn format_relative_time_u64(timestamp: u64) -> String {
    format_relative_time(timestamp as i64)
}

/// Network page - Field Topology layout
#[component]
pub fn Network() -> Element {
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // State for network data - using unified Peer type
    let mut stats: Signal<NetworkStats> = use_signal(NetworkStats::default);
    let mut peers: Signal<Vec<Peer>> = use_signal(Vec::new);
    let mut pinners: Signal<Vec<PinnerInfo>> = use_signal(Vec::new);
    let mut pins: Signal<Vec<ProfilePin>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);

    // Load network data when engine becomes ready
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let guard = shared.read().await;

                if let Some(ref eng) = *guard {
                    // Get network stats
                    stats.set(eng.network_stats());

                    // Get unified peer list (contacts + discovered in one call)
                    if let Ok(peer_list) = eng.list_peers() {
                        peers.set(peer_list);
                    }

                    // Get pinners (who pins us)
                    if let Ok(pinner_list) = eng.list_profile_pinners() {
                        pinners.set(pinner_list);
                    }

                    // Get pins (what we pin)
                    if let Ok(pin_list) = eng.list_pinned_profiles() {
                        // Filter out our own profile from the "Souls You Carry" list
                        let others: Vec<_> = pin_list
                            .into_iter()
                            .filter(|p| !p.is_own())
                            .collect();
                        pins.set(others);
                    }

                    loading.set(false);
                }
            });
        }
    });

    // Calculate stats from unified peers
    let peer_list = peers();
    let total_people = peer_list.len();
    let online_count = peer_list.iter().filter(|p| p.status == PeerStatus::Online).count();
    let contact_count = peer_list.iter().filter(|p| p.is_contact()).count();

    rsx! {
        div { class: "network-page",
            // Header with back navigation
            header { class: "network-header",
                Link {
                    to: Route::Field {},
                    button {
                        class: "back-link",
                        title: "Return to Field",
                        "← field"
                    }
                }
                h1 { class: "network-title", "The Network" }
            }

            if loading() {
                div { class: "loading-state",
                    div { class: "loading-orb" }
                    p { "Loading network data..." }
                }
            } else {
                div { class: "network-content",
                    // Stats Cards
                    section { class: "network-stats",
                        StatCard {
                            label: "People",
                            value: total_people,
                            sublabel: format!("{} online", online_count),
                        }
                        StatCard {
                            label: "Contacts",
                            value: contact_count,
                            sublabel: "verified".to_string(),
                        }
                        StatCard {
                            label: "Pinning You",
                            value: stats().pinners_count,
                            sublabel: "people".to_string(),
                        }
                        StatCard {
                            label: "You Pin",
                            value: stats().pinning_count,
                            sublabel: "profiles".to_string(),
                        }
                    }

                    // Unified Connections Section
                    section { class: "network-section",
                        h2 { class: "section-title", "Connections" }
                        p { class: "section-subtitle", "All known network participants" }

                        if peers().is_empty() {
                            div { class: "empty-state",
                                p { "No connections yet. Share your invite code or join a realm to discover peers." }
                            }
                        } else {
                            div { class: "peer-list",
                                // Sort: contacts first, then by online status, then by last seen
                                {
                                    let mut sorted_peers: Vec<_> = peers().clone();
                                    sorted_peers.sort_by(|a, b| {
                                        // Contacts first
                                        let contact_order = b.is_contact().cmp(&a.is_contact());
                                        if contact_order != std::cmp::Ordering::Equal {
                                            return contact_order;
                                        }
                                        // Then online first
                                        let status_a = matches!(a.status, PeerStatus::Online);
                                        let status_b = matches!(b.status, PeerStatus::Online);
                                        let online_order = status_b.cmp(&status_a);
                                        if online_order != std::cmp::Ordering::Equal {
                                            return online_order;
                                        }
                                        // Then by most recently seen
                                        b.last_seen.cmp(&a.last_seen)
                                    });
                                    rsx! {
                                        for peer in sorted_peers {
                                            UnifiedPeerCard { peer: UnifiedPeerDisplayData::from(&peer) }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Who Pins You Section
                    section { class: "network-section",
                        h2 { class: "section-title", "Who Pins Your Profile" }
                        p { class: "section-subtitle", "Peers caching your profile for P2P redundancy" }

                        if pinners().is_empty() {
                            div { class: "empty-state",
                                p { "No one is pinning your profile yet. Once you connect with others, they may pin your profile." }
                            }
                        } else {
                            div { class: "pinner-list",
                                for pinner in pinners() {
                                    PinnerCard { pinner: pinner }
                                }
                            }
                        }
                    }

                    // What You Pin Section
                    section { class: "network-section",
                        h2 { class: "section-title", "Profiles You Pin" }
                        p { class: "section-subtitle", "Profiles you are caching for others" }

                        if pins().is_empty() {
                            div { class: "empty-state",
                                p { "You are not pinning any profiles yet. Add contacts to automatically pin their profiles." }
                            }
                        } else {
                            div { class: "pin-list",
                                for pin in pins() {
                                    YourPinCard { pin: PinDisplayData::from(&pin) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Subcomponents
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Props, Clone, PartialEq)]
struct StatCardProps {
    label: String,
    value: usize,
    sublabel: String,
}

#[component]
fn StatCard(props: StatCardProps) -> Element {
    rsx! {
        div { class: "stat-card",
            div { class: "stat-value", "{props.value}" }
            div { class: "stat-label", "{props.label}" }
            div { class: "stat-sublabel", "{props.sublabel}" }
        }
    }
}

/// Extracted peer data for display (unified contacts + discovered)
#[derive(Clone, PartialEq)]
struct UnifiedPeerDisplayData {
    display_name: String,
    peer_id_short: String,
    did_short: Option<String>,
    is_contact: bool,
    is_favorite: bool,
    is_online: bool,
    last_seen: u64,
    shared_realms_count: usize,
    success_rate: Option<u32>,
}

impl From<&Peer> for UnifiedPeerDisplayData {
    fn from(peer: &Peer) -> Self {
        // Get display name: profile name > nickname > truncated endpoint
        let display_name = peer.display_name();

        // Format peer ID (first 4 bytes as hex)
        let peer_id_short = format!("{}", hex::encode(&peer.endpoint_id[..4]));

        // Format DID if available
        let did_short = peer.did.as_ref().map(|did| {
            if did.starts_with("did:sync:") {
                let suffix = &did[9..];
                if suffix.len() > 8 {
                    format!("{}...", &suffix[..8])
                } else {
                    suffix.to_string()
                }
            } else if did.len() > 12 {
                format!("{}...", &did[..12])
            } else {
                did.clone()
            }
        });

        // Calculate success rate
        let success_rate = if peer.connection_attempts > 0 {
            Some((peer.success_rate() * 100.0) as u32)
        } else {
            None
        };

        Self {
            display_name,
            peer_id_short,
            did_short,
            is_contact: peer.is_contact(),
            is_favorite: peer.is_favorite(),
            is_online: peer.status == PeerStatus::Online,
            last_seen: peer.last_seen,
            shared_realms_count: peer.shared_realms.len(),
            success_rate,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct UnifiedPeerCardProps {
    peer: UnifiedPeerDisplayData,
}

#[component]
fn UnifiedPeerCard(props: UnifiedPeerCardProps) -> Element {
    let status_class = if props.peer.is_online { "status-online" } else { "status-offline" };
    let status_label = if props.peer.is_online { "online" } else { "offline" };

    // Type label (Contact vs Discovered)
    let type_label = if props.peer.is_contact { "Contact" } else { "Discovered" };
    let type_class = if props.peer.is_contact { "type-contact" } else { "type-discovered" };

    // Format last seen
    let last_seen = if props.peer.is_online {
        "Now".to_string()
    } else {
        format_relative_time_u64(props.peer.last_seen)
    };

    // Format success rate
    let success_rate = props.peer.success_rate
        .map(|r| format!("{}%", r))
        .unwrap_or_else(|| "--".to_string());

    rsx! {
        div { class: "network-card unified-peer-card",
            div { class: "card-header",
                span { class: "status-dot {status_class}" }
                span { class: "card-name", "{props.peer.display_name}" }
                if props.peer.is_favorite {
                    span { class: "favorite-badge", "★" }
                }
                span { class: "card-type {type_class}", "{type_label}" }
                span { class: "card-status", "{status_label}" }
            }

            div { class: "card-details",
                if let Some(ref did) = props.peer.did_short {
                    span { class: "detail-item",
                        span { class: "detail-label", "DID: " }
                        span { class: "detail-value did-value", "{did}" }
                    }
                } else {
                    span { class: "detail-item",
                        span { class: "detail-label", "ID: " }
                        span { class: "detail-value", "{props.peer.peer_id_short}" }
                    }
                }
                if props.peer.shared_realms_count > 0 {
                    span { class: "detail-item",
                        span { class: "detail-label", "Realms: " }
                        span { class: "detail-value", "{props.peer.shared_realms_count}" }
                    }
                }
                span { class: "detail-item",
                    span { class: "detail-label", "Success: " }
                    span { class: "detail-value", "{success_rate}" }
                }
                span { class: "detail-item",
                    span { class: "detail-label", "Seen: " }
                    span { class: "detail-value", "{last_seen}" }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct PinnerCardProps {
    pinner: PinnerInfo,
}

#[component]
fn PinnerCard(props: PinnerCardProps) -> Element {
    // Format DID (first 16 chars after prefix)
    let display_name = props.pinner.display_name.clone().unwrap_or_else(|| {
        let did = &props.pinner.pinner_did;
        if did.starts_with("did:sync:") {
            let suffix = &did[9..];
            if suffix.len() > 12 {
                format!("{}...", &suffix[..12])
            } else {
                suffix.to_string()
            }
        } else if did.len() > 16 {
            format!("{}...", &did[..16])
        } else {
            did.clone()
        }
    });

    // Format relationship
    let relationship = match props.pinner.relationship.as_str() {
        "contact" => "Contact",
        r if r.starts_with("realm_member") => "Realm Member",
        "manual" => "Manual",
        _ => "Unknown",
    };

    let pinned_since = format_relative_time(props.pinner.pinned_at);

    rsx! {
        div { class: "network-card pinner-card",
            div { class: "card-header",
                span { class: "card-name", "{display_name}" }
                span { class: "card-relationship", "{relationship}" }
            }

            div { class: "card-details",
                span { class: "detail-item",
                    span { class: "detail-label", "Since: " }
                    span { class: "detail-value", "{pinned_since}" }
                }
            }
        }
    }
}

/// Extracted pin data for display (avoids PartialEq issues with SignedProfile)
#[derive(Clone, PartialEq)]
struct PinDisplayData {
    did: String,
    display_name: String,
    relationship: String,
    pinned_at: i64,
}

impl From<&ProfilePin> for PinDisplayData {
    fn from(pin: &ProfilePin) -> Self {
        let relationship = match &pin.relationship {
            syncengine_core::PinRelationship::Contact => "Contact".to_string(),
            syncengine_core::PinRelationship::RealmMember { realm_id } => {
                format!("Realm: {}", &realm_id.to_string()[..8])
            }
            syncengine_core::PinRelationship::Manual => "Manual".to_string(),
            syncengine_core::PinRelationship::Own => "Self".to_string(),
        };
        Self {
            did: pin.did.clone(),
            display_name: pin.signed_profile.profile.display_name.clone(),
            relationship,
            pinned_at: pin.pinned_at,
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct YourPinCardProps {
    pin: PinDisplayData,
}

#[component]
fn YourPinCard(props: YourPinCardProps) -> Element {
    let engine = use_engine();

    let display_name = props.pin.display_name.clone();
    let relationship = props.pin.relationship.clone();
    let pinned_since = format_relative_time(props.pin.pinned_at);
    let did_for_unpin = props.pin.did.clone();

    rsx! {
        div { class: "network-card your-pin-card",
            div { class: "card-header",
                span { class: "card-name", "{display_name}" }
                span { class: "card-relationship", "{relationship}" }
            }

            div { class: "card-details",
                span { class: "detail-item",
                    span { class: "detail-label", "Pinned: " }
                    span { class: "detail-value", "{pinned_since}" }
                }

                button {
                    class: "unpin-btn",
                    title: "Stop pinning this profile",
                    onclick: move |_| {
                        let did = did_for_unpin.clone();
                        spawn(async move {
                            let shared = engine();
                            let guard = shared.read().await;
                            if let Some(ref eng) = *guard {
                                match eng.unpin_profile(&did) {
                                    Ok(_) => {
                                        tracing::info!("Unpinned profile: {}", did);
                                        // TODO: Refresh the page
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to unpin profile: {:?}", e);
                                    }
                                }
                            }
                        });
                    },
                    "Unpin"
                }
            }
        }
    }
}
