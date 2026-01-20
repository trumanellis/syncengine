//! Network Page
//!
//! Unified view for contacts, mirrors, and messaging.
//! Contacts = Mirrors = Peers you communicate with.
//! Follows Design System v2: minimal terminal aesthetic.

use dioxus::prelude::*;
use syncengine_core::profile::{PacketEnvelope, PacketPayload};
use syncengine_core::{ContactEvent, Did, Peer, PeerStatus};

use crate::components::images::AsyncImage;
use crate::components::messages::{MessageCompose, MessagesList, ReceivedMessage};
use crate::components::{NavHeader, NavLocation};
use crate::context::{use_engine, use_engine_ready};

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

/// Extract messages from packet envelopes
fn extract_messages(
    packets: &[PacketEnvelope],
    sender_did: &str,
    sender_name: Option<String>,
) -> Vec<ReceivedMessage> {
    packets
        .iter()
        .filter_map(|env| {
            // Try to decode as DirectMessage
            if env.is_global() {
                if let Ok(payload) = env.decode_global_payload() {
                    if let PacketPayload::DirectMessage { content } = payload {
                        return Some(ReceivedMessage {
                            sender_did: sender_did.to_string(),
                            sender_name: sender_name.clone(),
                            content,
                            timestamp: env.timestamp,
                            sequence: env.sequence,
                        });
                    }
                }
            }
            None
        })
        .collect()
}

/// Network page - displays contacts with messaging capability.
#[component]
pub fn Network() -> Element {
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // State
    let mut contacts: Signal<Vec<Peer>> = use_signal(Vec::new);
    let mut messages: Signal<Vec<ReceivedMessage>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut syncing = use_signal(|| false);

    // Message compose modal state
    let mut compose_target: Signal<Option<(String, String)>> = use_signal(|| None); // (did, name)

    // Load contacts and messages when engine becomes ready
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let guard = shared.read().await;

                if let Some(ref eng) = *guard {
                    // Load contacts
                    if let Ok(contact_list) = eng.list_peer_contacts() {
                        // Also load messages from each contact's mirror
                        let mut all_messages = Vec::new();

                        for contact in &contact_list {
                            if let Some(ref did_str) = contact.did {
                                if let Ok(did) = did_str.parse::<Did>() {
                                    // Get packets from mirror
                                    if let Ok(packets) = eng.mirror_packets_since(&did, 0) {
                                        let msgs = extract_messages(
                                            &packets,
                                            did_str,
                                            Some(contact.display_name()),
                                        );
                                        all_messages.extend(msgs);
                                    }
                                }
                            }
                        }

                        // Sort messages by timestamp
                        all_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

                        contacts.set(contact_list);
                        messages.set(all_messages);
                    }

                    loading.set(false);
                }
            });

            // Subscribe to contact events for real-time updates
            spawn(async move {
                let shared = engine();
                let mut guard = shared.write().await;

                if let Some(ref mut eng) = *guard {
                    match eng.subscribe_contact_events().await {
                        Ok(mut event_rx) => {
                            drop(guard);

                            while let Ok(event) = event_rx.recv().await {
                                match event {
                                    ContactEvent::ProfileUpdated { did } => {
                                        // Refresh contacts and messages
                                        let shared = engine();
                                        let guard = shared.read().await;
                                        if let Some(ref eng) = *guard {
                                            if let Ok(contact_list) = eng.list_peer_contacts() {
                                                // Reload messages
                                                let mut all_messages = Vec::new();
                                                for contact in &contact_list {
                                                    if let Some(ref did_str) = contact.did {
                                                        if let Ok(did) = did_str.parse::<Did>() {
                                                            if let Ok(packets) =
                                                                eng.mirror_packets_since(&did, 0)
                                                            {
                                                                let msgs = extract_messages(
                                                                    &packets,
                                                                    did_str,
                                                                    Some(contact.display_name()),
                                                                );
                                                                all_messages.extend(msgs);
                                                            }
                                                        }
                                                    }
                                                }
                                                all_messages
                                                    .sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                                                contacts.set(contact_list);
                                                messages.set(all_messages);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Failed to subscribe to contact events");
                        }
                    }
                }
            });

            // Poll for new messages periodically
            spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                    let shared = engine();
                    let guard = shared.read().await;
                    if let Some(ref eng) = *guard {
                        if let Ok(contact_list) = eng.list_peer_contacts() {
                            let mut all_messages = Vec::new();
                            for contact in &contact_list {
                                if let Some(ref did_str) = contact.did {
                                    if let Ok(did) = did_str.parse::<Did>() {
                                        if let Ok(packets) = eng.mirror_packets_since(&did, 0) {
                                            let msgs = extract_messages(
                                                &packets,
                                                did_str,
                                                Some(contact.display_name()),
                                            );
                                            all_messages.extend(msgs);
                                        }
                                    }
                                }
                            }
                            all_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
                            messages.set(all_messages);
                        }
                    }
                }
            });
        }
    });

    // Handler for sending a message
    let send_message = move |content: String| {
        if let Some((did, name)) = compose_target() {
            spawn(async move {
                let shared = engine();
                let mut guard = shared.write().await;

                if let Some(ref mut eng) = *guard {
                    // Create a DirectMessage packet addressed to the specific contact
                    // This routes the packet via the 1:1 contact topic, not the global topic
                    let payload = syncengine_core::PacketPayload::DirectMessage { content };

                    // Parse the DID and create an Individual address for direct routing
                    let address = match syncengine_core::Did::parse(&did) {
                        Ok(parsed_did) => syncengine_core::PacketAddress::Individual(parsed_did),
                        Err(e) => {
                            tracing::error!(error = %e, did = %did, "Invalid DID for message");
                            compose_target.set(None);
                            return;
                        }
                    };

                    match eng.create_and_broadcast_packet(payload, address).await {
                        Ok(seq) => {
                            tracing::info!(
                                to = %did,
                                sequence = seq,
                                "Sent message via 1:1 contact topic"
                            );
                        }
                        Err(e) => {
                            tracing::error!(error = %e, to = %did, "Failed to send message");
                        }
                    }
                }

                // Close modal
                compose_target.set(None);
            });
        }
    };

    // Handler for manual sync
    let on_sync_click = move |_: ()| {
        if syncing() {
            return;
        }

        syncing.set(true);

        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;

            if let Some(ref mut eng) = *guard {
                match eng.manual_sync().await {
                    Ok(_) => {
                        if let Ok(contact_list) = eng.list_peer_contacts() {
                            contacts.set(contact_list);
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

    let contact_count = contacts().len();
    let message_count = messages().len();
    let online_count = contacts()
        .iter()
        .filter(|c| matches!(c.status, PeerStatus::Online))
        .count();

    let status_text = format!("{} contacts · {} online", contact_count, online_count);
    let action_text = if syncing() { "Syncing..." } else { "Sync" };

    rsx! {
        div { class: "network-page",
            NavHeader {
                current: NavLocation::Network,
            }

            if loading() {
                div { class: "loading",
                    div { class: "loading-spinner" }
                    p { class: "loading-text", "Loading..." }
                }
            } else {
                div { class: "network-content",
                    // Two-column layout: contacts on left, messages on right
                    div { class: "network-grid",
                        // Contacts section
                        section { class: "network-section contacts-section",
                            header { class: "section-header",
                                h2 { class: "card-title", "Contacts" }
                            }

                            if contacts().is_empty() {
                                div { class: "empty-state",
                                    p { class: "empty-state-message",
                                        "No contacts yet"
                                    }
                                    p { class: "empty-state-hint",
                                        "Add contacts to start messaging."
                                    }
                                }
                            } else {
                                div { class: "contacts-list",
                                    for contact in contacts() {
                                        {
                                            let contact_did = contact.did.clone().unwrap_or_default();
                                            let contact_name = contact.display_name();
                                            let contact_did_for_click = contact_did.clone();
                                            let contact_name_for_click = contact_name.clone();
                                            let is_online = matches!(contact.status, PeerStatus::Online);
                                            let avatar_blob_id = contact.profile.as_ref().and_then(|p| p.avatar_blob_id.clone());
                                            let first_char = contact_name.chars().next().unwrap_or('?').to_uppercase().to_string();

                                            rsx! {
                                                div {
                                                    key: "{contact_did}",
                                                    class: "contact-row",

                                                    // Avatar
                                                    if let Some(ref blob_id) = avatar_blob_id {
                                                        AsyncImage {
                                                            blob_id: blob_id.clone(),
                                                            alt: contact_name.clone(),
                                                            class: Some("contact-avatar".to_string()),
                                                        }
                                                    } else {
                                                        div { class: "contact-avatar-placeholder",
                                                            span { "{first_char}" }
                                                        }
                                                    }

                                                    // Info
                                                    div { class: "contact-info",
                                                        div { class: "contact-name", "{contact_name}" }
                                                        div { class: "contact-meta",
                                                            div { class: "contact-status",
                                                                span {
                                                                    class: if is_online { "status-dot online" } else { "status-dot" }
                                                                }
                                                                span { class: "contact-status-text",
                                                                    if is_online { "online" } else { "offline" }
                                                                }
                                                            }
                                                            span { "· {format_relative_time(contact.last_seen)}" }
                                                        }
                                                    }

                                                    // Message button
                                                    div { class: "contact-actions",
                                                        button {
                                                            class: "btn btn-primary btn-message",
                                                            onclick: move |_| {
                                                                compose_target.set(Some((
                                                                    contact_did_for_click.clone(),
                                                                    contact_name_for_click.clone(),
                                                                )));
                                                            },
                                                            "Message"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Messages section
                        section { class: "network-section messages-section",
                            header { class: "section-header",
                                h2 { class: "card-title", "Messages" }
                                if message_count > 0 {
                                    span { class: "message-count", "{message_count}" }
                                }
                            }

                            MessagesList {
                                messages: messages(),
                                loading: loading(),
                            }
                        }
                    }
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
}
