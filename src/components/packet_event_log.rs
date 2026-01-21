//! Packet Event Log Component
//!
//! Displays a collapsible log of packet events for Indra's Network visualization.
//! Shows packet flow: Author -> (Relay) -> Destination

use dioxus::prelude::*;
use syncengine_core::{DecryptionStatus, PacketDirection, PacketEvent};

/// Format timestamp as relative time string (for milliseconds)
fn format_time(timestamp_ms: i64) -> String {
    let now = chrono::Utc::now().timestamp_millis();
    let elapsed_ms = now - timestamp_ms;
    let elapsed_secs = (elapsed_ms / 1000) as u64;

    if elapsed_secs < 60 {
        "now".to_string()
    } else if elapsed_secs < 3600 {
        format!("{}m", elapsed_secs / 60)
    } else if elapsed_secs < 86400 {
        format!("{}h", elapsed_secs / 3600)
    } else {
        format!("{}d", elapsed_secs / 86400)
    }
}

/// Get the visual indicator for decryption status
fn decrypt_indicator(status: &DecryptionStatus) -> (&'static str, &'static str) {
    match status {
        DecryptionStatus::Decrypted => ("\u{2713}", "decrypt-success"),     // ✓
        DecryptionStatus::Global => ("\u{25CB}", "decrypt-global"),          // ○
        DecryptionStatus::CannotDecrypt { .. } => ("\u{2717}", "decrypt-fail"), // ✗
        DecryptionStatus::NotAttempted => ("\u{2014}", "decrypt-pending"),   // —
    }
}

/// Packet Event Log Component
///
/// Displays packet events for a peer showing the path:
/// - Direct: Author -> Destination
/// - Relayed: Author -> Relay -> Destination
///
/// # Example
///
/// ```rust
/// rsx! {
///     PacketEventLog {
///         peer_did: "did:sync:abc123".to_string(),
///         events: vec![...],
///         expanded: false,
///     }
/// }
/// ```
#[component]
pub fn PacketEventLog(
    /// The peer's DID for filtering
    peer_did: String,
    /// Packet events to display
    events: Vec<PacketEvent>,
    /// Whether the log is initially expanded
    #[props(default = false)]
    expanded: bool,
) -> Element {
    let mut is_expanded = use_signal(|| expanded);

    let event_count = events.len();
    let recent_events: Vec<_> = events.into_iter().rev().take(10).collect();

    rsx! {
        div { class: "packet-event-log",
            // Collapsible header
            div {
                class: "packet-log-header",
                onclick: move |_| is_expanded.toggle(),

                span { class: "packet-log-toggle",
                    if is_expanded() { "\u{25BC}" } else { "\u{25B6}" }
                }
                span { class: "packet-log-title", "Packet Log" }
                span { class: "packet-log-count", "({event_count})" }
            }

            // Event rows (when expanded)
            if is_expanded() {
                div { class: "packet-log-events",
                    if recent_events.is_empty() {
                        div { class: "packet-log-empty",
                            "No packets yet"
                        }
                    } else {
                        for event in recent_events {
                            {
                                let direction_class = match event.direction {
                                    PacketDirection::Incoming => "incoming",
                                    PacketDirection::Outgoing => "outgoing",
                                };
                                let delivered_class = if event.is_delivered {
                                    "delivered"
                                } else {
                                    ""
                                };
                                let (decrypt_icon, decrypt_class) = decrypt_indicator(&event.decryption_status);

                                // Build the path display: Author -> (Relay) -> Destination
                                let path_display = if let Some(ref relay_name) = event.relay_name {
                                    format!("{} \u{2192} {} \u{2192} {}",
                                        event.author_name, relay_name, event.destination_name)
                                } else {
                                    format!("{} \u{2192} {}",
                                        event.author_name, event.destination_name)
                                };

                                // Format content preview: "quoted text..." or [encrypted]
                                let content_display = if matches!(event.decryption_status, DecryptionStatus::Decrypted) {
                                    format!("\"{}\"", event.content_preview)
                                } else {
                                    event.content_preview.clone()
                                };

                                rsx! {
                                    div {
                                        key: "{event.id}",
                                        class: "packet-event-row {direction_class} {delivered_class}",

                                        // Path: Author -> Relay -> Destination
                                        span { class: "packet-path", "{path_display}" }

                                        // Content preview (quoted if decrypted)
                                        span { class: "packet-content", "{content_display}" }

                                        // Decrypt status
                                        span { class: "packet-decrypt {decrypt_class}", "{decrypt_icon}" }

                                        // Time
                                        span { class: "packet-time", "{format_time(event.timestamp)}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Compact packet indicator showing event count and status
#[component]
pub fn PacketIndicator(
    /// Number of total events
    event_count: usize,
    /// Number of undelivered events
    #[props(default = 0)]
    pending_count: usize,
) -> Element {
    if event_count == 0 {
        return rsx! {};
    }

    rsx! {
        span {
            class: "packet-indicator",
            title: "Packet events",

            if pending_count > 0 {
                span { class: "packet-pending", "{pending_count}" }
            }
            span { class: "packet-total", "/{event_count}" }
        }
    }
}
