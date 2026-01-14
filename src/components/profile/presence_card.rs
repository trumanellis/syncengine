//! Presence Card - Identity display with node signature and QR code.

use dioxus::prelude::*;

use super::QRSignature;

/// Props for the presence card component.
#[derive(Props, Clone, PartialEq)]
pub struct PresenceCardProps {
    /// Full node ID string
    pub node_id: String,
    /// Full endpoint address
    pub endpoint: String,
}

/// Presence card showing node identity information.
#[component]
pub fn PresenceCard(props: PresenceCardProps) -> Element {
    let mut copied_field: Signal<Option<&str>> = use_signal(|| None);

    // Copy handler with feedback
    let mut copy_to_clipboard = move |field: &'static str, value: String| {
        // Copy to clipboard using arboard (desktop clipboard)
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(&value);
        }

        // Show "Captured!" feedback
        copied_field.set(Some(field));

        // Reset after 2 seconds using tokio
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            copied_field.set(None);
        });
    };

    // Truncate node ID for display (first 8 + last 8 chars)
    let node_id_display = if props.node_id.len() > 16 {
        format!(
            "{}...{}",
            &props.node_id[..8],
            &props.node_id[props.node_id.len() - 8..]
        )
    } else {
        props.node_id.clone()
    };

    // Truncate endpoint for display
    let endpoint_display = if props.endpoint.len() > 40 {
        format!("{}...", &props.endpoint[..40])
    } else {
        props.endpoint.clone()
    };

    rsx! {
        div { class: "presence-card",
            h2 { class: "section-title", "PRESENCE" }

            // QR Code
            QRSignature {
                data: props.node_id.clone(),
                size: 200,
            }

            // Node Signature
            div { class: "node-signature",
                span { class: "label", "Node Signature" }
                span { class: "value mono cyan", "{node_id_display}" }
                button {
                    class: if copied_field() == Some("node") {
                        "copy-button copied"
                    } else {
                        "copy-button"
                    },
                    onclick: {
                        let node_id = props.node_id.clone();
                        move |_| copy_to_clipboard("node", node_id.clone())
                    },
                    if copied_field() == Some("node") {
                        "Captured! ✓"
                    } else {
                        "Capture"
                    }
                }
            }

            // Endpoint Address
            div { class: "endpoint-address",
                span { class: "label", "Endpoint Address" }
                span { class: "value mono cyan", "{endpoint_display}" }
                button {
                    class: if copied_field() == Some("endpoint") {
                        "copy-button copied"
                    } else {
                        "copy-button"
                    },
                    onclick: {
                        let endpoint = props.endpoint.clone();
                        move |_| copy_to_clipboard("endpoint", endpoint.clone())
                    },
                    if copied_field() == Some("endpoint") {
                        "Captured! ✓"
                    } else {
                        "Capture"
                    }
                }
            }

            // Connected Since (placeholder for now)
            div { class: "connected-since moss",
                "Connected · Resonating in the field"
            }

            // Share button (future: opens modal)
            button {
                class: "btn-primary",
                onclick: move |_| {
                    // TODO: Open modal with large QR code
                    tracing::info!("Share button clicked");
                },
                "Share Signature"
            }
        }
    }
}
