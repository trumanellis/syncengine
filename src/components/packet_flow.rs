//! Packet Flow Section Component
//!
//! Animated visualization of packet flow in real-time.
//! Shows sender -> content -> destination with staggered glow animations.

use std::collections::VecDeque;

use dioxus::prelude::*;
use syncengine_core::DecryptionStatus;

use crate::context::use_engine;

/// Maximum number of packet entries to show
const MAX_VISIBLE_PACKETS: usize = 8;

/// How long each packet entry stays visible (ms)
const PACKET_DISPLAY_DURATION_MS: u64 = 8000;

/// A packet flow entry with animation timing
#[derive(Clone)]
struct PacketFlowEntry {
    id: String,
    sender_name: String,
    content_preview: String,
    destination_name: String,
    is_encrypted: bool,
    timestamp: i64,
}

/// Packet Flow Section
///
/// Displays an animated visualization of packets flowing through the network.
/// - Sender name glows on the left
/// - Content preview in the middle
/// - Destination name glows on the right (with 1s delay)
///
/// # Example
///
/// ```rust
/// rsx! {
///     PacketFlowSection {}
/// }
/// ```
#[component]
pub fn PacketFlowSection() -> Element {
    let engine = use_engine();
    let mut packets: Signal<VecDeque<PacketFlowEntry>> = use_signal(VecDeque::new);

    // Subscribe to packet events
    use_effect(move || {
        spawn(async move {
            let shared = engine();
            let guard = shared.read().await;

            if let Some(ref eng) = *guard {
                let mut event_rx = eng.subscribe_packet_events();
                drop(guard);

                while let Ok(event) = event_rx.recv().await {
                    let entry = PacketFlowEntry {
                        id: event.id.clone(),
                        sender_name: event.author_name.clone(),
                        content_preview: event.content_preview.clone(),
                        destination_name: event.destination_name.clone(),
                        is_encrypted: !matches!(event.decryption_status, DecryptionStatus::Decrypted | DecryptionStatus::Global),
                        timestamp: event.timestamp,
                    };

                    // Add to front of queue
                    let mut queue = packets();
                    queue.push_front(entry.clone());

                    // Keep only MAX_VISIBLE_PACKETS
                    while queue.len() > MAX_VISIBLE_PACKETS {
                        queue.pop_back();
                    }

                    packets.set(queue);

                    // Schedule removal after display duration
                    let entry_id = entry.id.clone();
                    spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(PACKET_DISPLAY_DURATION_MS)).await;
                        let mut queue = packets();
                        queue.retain(|p| p.id != entry_id);
                        packets.set(queue);
                    });
                }
            }
        });
    });

    let packet_list: Vec<PacketFlowEntry> = packets().iter().cloned().collect();

    rsx! {
        section { class: "packet-flow-section",
            h3 { class: "section-title", "Packets" }

            div { class: "packet-flow-container",
                if packet_list.is_empty() {
                    div { class: "packet-flow-empty",
                        span { class: "packet-flow-empty-icon", "~" }
                        span { class: "packet-flow-empty-text", "Awaiting transmissions..." }
                    }
                } else {
                    for (index, packet) in packet_list.iter().enumerate() {
                        {
                            let content_display = if packet.is_encrypted {
                                "[encrypted]".to_string()
                            } else {
                                format!("\"{}\"", packet.content_preview)
                            };
                            let encrypted_class = if packet.is_encrypted { "encrypted" } else { "" };

                            rsx! {
                                div {
                                    key: "{packet.id}",
                                    class: "packet-flow-entry",
                                    style: "--entry-index: {index}",

                                    // Sender (left side with glow)
                                    div { class: "packet-flow-sender",
                                        span { class: "packet-flow-name sender-glow",
                                            "{packet.sender_name}"
                                        }
                                    }

                                    // Arrow
                                    div { class: "packet-flow-arrow", "→" }

                                    // Content (middle)
                                    div { class: "packet-flow-content {encrypted_class}",
                                        "{content_display}"
                                    }

                                    // Arrow
                                    div { class: "packet-flow-arrow", "→" }

                                    // Destination (right side with delayed glow)
                                    div { class: "packet-flow-destination",
                                        span { class: "packet-flow-name destination-glow",
                                            "{packet.destination_name}"
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
}
