//! Network Page
//!
//! Unified view for contacts, mirrors, and messaging.
//! Contacts = Mirrors = Peers you communicate with.
//! Follows Design System v2: minimal terminal aesthetic.

use dioxus::prelude::*;
use syncengine_core::{ContactEvent, Peer};

use crate::components::images::AsyncImage;
use crate::components::messages::{ChatBubbleMessage, ConversationView};
use crate::components::{NavHeader, NavLocation};
use crate::context::{use_engine, use_engine_ready};

/// Contact with conversation preview info
#[derive(Clone)]
struct ContactWithPreview {
    peer: Peer,
    last_message_time: Option<i64>,
    last_message_preview: Option<String>,
    unread_count: u32,
}

/// Currently selected contact
#[derive(Clone)]
struct SelectedContact {
    did: String,
    name: String,
}

/// Format timestamp as relative time string (for milliseconds)
fn format_relative_time(timestamp_ms: i64) -> String {
    let now = chrono::Utc::now().timestamp_millis();
    let elapsed_ms = now - timestamp_ms;
    let elapsed_secs = (elapsed_ms / 1000) as u64;

    if elapsed_secs < 60 {
        "Just now".to_string()
    } else if elapsed_secs < 3600 {
        format!("{}m ago", elapsed_secs / 60)
    } else if elapsed_secs < 86400 {
        format!("{}h ago", elapsed_secs / 3600)
    } else {
        format!("{}d ago", elapsed_secs / 86400)
    }
}

/// Network page - displays contacts with messaging capability.
#[component]
pub fn Network() -> Element {
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // State
    let mut contacts: Signal<Vec<ContactWithPreview>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut syncing = use_signal(|| false);

    // Conversation state
    let mut selected_contact: Signal<Option<SelectedContact>> = use_signal(|| None);
    let mut conversation_messages: Signal<Vec<ChatBubbleMessage>> = use_signal(Vec::new);
    let mut conversation_loading = use_signal(|| false);
    let mut sending = use_signal(|| false);

    // Load contacts and messages when engine becomes ready
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let guard = shared.read().await;

                if let Some(ref eng) = *guard {
                    // Load contacts with preview info
                    if let Ok(contact_list) = eng.list_peer_contacts() {
                        let mut contacts_with_preview = Vec::new();

                        for contact in contact_list {
                            if let Some(ref did_str) = contact.did {
                                // Get conversation to extract last message info
                                let (last_time, last_preview) = match eng.get_conversation(did_str) {
                                    Ok(convo) => {
                                        let last_msg = convo.messages().last();
                                        if let Some(msg) = last_msg {
                                            let preview = if msg.content.len() > 50 {
                                                format!("{}...", &msg.content[..47])
                                            } else {
                                                msg.content.clone()
                                            };
                                            (Some(msg.timestamp), Some(preview))
                                        } else {
                                            (None, None)
                                        }
                                    }
                                    Err(_) => (None, None),
                                };

                                contacts_with_preview.push(ContactWithPreview {
                                    peer: contact,
                                    last_message_time: last_time,
                                    last_message_preview: last_preview,
                                    unread_count: 0, // TODO: implement unread tracking
                                });
                            }
                        }

                        // Sort by last message time (most recent first)
                        contacts_with_preview.sort_by(|a, b| {
                            b.last_message_time.cmp(&a.last_message_time)
                        });

                        contacts.set(contacts_with_preview);
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
                                    ContactEvent::ProfileUpdated { did: _did } => {
                                        // Refresh contacts list
                                        let shared = engine();
                                        let guard = shared.read().await;
                                        if let Some(ref eng) = *guard {
                                            if let Ok(contact_list) = eng.list_peer_contacts() {
                                                let mut contacts_with_preview = Vec::new();

                                                for contact in contact_list {
                                                    if let Some(ref did_str) = contact.did {
                                                        let (last_time, last_preview) = match eng.get_conversation(did_str) {
                                                            Ok(convo) => {
                                                                let last_msg = convo.messages().last();
                                                                if let Some(msg) = last_msg {
                                                                    let preview = if msg.content.len() > 50 {
                                                                        format!("{}...", &msg.content[..47])
                                                                    } else {
                                                                        msg.content.clone()
                                                                    };
                                                                    (Some(msg.timestamp), Some(preview))
                                                                } else {
                                                                    (None, None)
                                                                }
                                                            }
                                                            Err(_) => (None, None),
                                                        };

                                                        contacts_with_preview.push(ContactWithPreview {
                                                            peer: contact,
                                                            last_message_time: last_time,
                                                            last_message_preview: last_preview,
                                                            unread_count: 0,
                                                        });
                                                    }
                                                }

                                                contacts_with_preview.sort_by(|a, b| {
                                                    b.last_message_time.cmp(&a.last_message_time)
                                                });

                                                contacts.set(contacts_with_preview);
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

            // Poll for new messages in current conversation periodically
            spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                    // If a contact is selected, refresh their conversation
                    if let Some(ref contact) = selected_contact() {
                        let contact_did = contact.did.clone();

                        let shared = engine();
                        let guard = shared.read().await;
                        if let Some(ref eng) = *guard {
                            if let Ok(convo) = eng.get_conversation(&contact_did) {
                                let messages: Vec<ChatBubbleMessage> = convo
                                    .messages()
                                    .iter()
                                    .map(|msg| ChatBubbleMessage {
                                        id: msg.id.clone(),
                                        content: msg.content.clone(),
                                        sender_name: msg.sender_name.clone(),
                                        timestamp: msg.timestamp,
                                        is_mine: msg.is_mine,
                                    })
                                    .collect();

                                conversation_messages.set(messages);
                            }
                        }
                    }
                }
            });
        }
    });

    // Load conversation when contact is selected
    use_effect(move || {
        if let Some(ref contact) = selected_contact() {
            let contact_did = contact.did.clone();

            spawn(async move {
                conversation_loading.set(true);

                let shared = engine();
                let guard = shared.read().await;

                if let Some(ref eng) = *guard {
                    match eng.get_conversation(&contact_did) {
                        Ok(convo) => {
                            let messages: Vec<ChatBubbleMessage> = convo
                                .messages()
                                .iter()
                                .map(|msg| ChatBubbleMessage {
                                    id: msg.id.clone(),
                                    content: msg.content.clone(),
                                    sender_name: msg.sender_name.clone(),
                                    timestamp: msg.timestamp,
                                    is_mine: msg.is_mine,
                                })
                                .collect();

                            conversation_messages.set(messages);
                        }
                        Err(e) => {
                            tracing::error!(error = %e, did = %contact_did, "Failed to load conversation");
                            conversation_messages.set(Vec::new());
                        }
                    }
                }

                conversation_loading.set(false);
            });
        }
    });

    // Handler for sending a message
    let send_message = move |content: String| {
        if let Some(ref contact) = selected_contact() {
            let contact_did = contact.did.clone();

            spawn(async move {
                sending.set(true);

                let shared = engine();
                let mut guard = shared.write().await;

                if let Some(ref mut eng) = *guard {
                    match eng.send_message(&contact_did, &content).await {
                        Ok(seq) => {
                            tracing::info!(
                                to = %contact_did,
                                sequence = seq,
                                "Sent message"
                            );

                            // Optimistically add message to conversation
                            let new_msg = ChatBubbleMessage {
                                id: format!("sent-{}", seq),
                                content,
                                sender_name: None,
                                timestamp: chrono::Utc::now().timestamp_millis(),
                                is_mine: true,
                            };

                            let mut msgs = conversation_messages();
                            msgs.push(new_msg);
                            conversation_messages.set(msgs);
                        }
                        Err(e) => {
                            tracing::error!(error = %e, to = %contact_did, "Failed to send message");
                        }
                    }
                }

                sending.set(false);
            });
        }
    };

    // Handler for manual sync
    let _on_sync_click = move |_: ()| {
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
                            let mut contacts_with_preview = Vec::new();

                            for contact in contact_list {
                                if let Some(ref did_str) = contact.did {
                                    let (last_time, last_preview) = match eng.get_conversation(did_str) {
                                        Ok(convo) => {
                                            let last_msg = convo.messages().last();
                                            if let Some(msg) = last_msg {
                                                let preview = if msg.content.len() > 50 {
                                                    format!("{}...", &msg.content[..47])
                                                } else {
                                                    msg.content.clone()
                                                };
                                                (Some(msg.timestamp), Some(preview))
                                            } else {
                                                (None, None)
                                            }
                                        }
                                        Err(_) => (None, None),
                                    };

                                    contacts_with_preview.push(ContactWithPreview {
                                        peer: contact,
                                        last_message_time: last_time,
                                        last_message_preview: last_preview,
                                        unread_count: 0,
                                    });
                                }
                            }

                            contacts_with_preview.sort_by(|a, b| {
                                b.last_message_time.cmp(&a.last_message_time)
                            });

                            contacts.set(contacts_with_preview);
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
                    div { class: "network-chat-layout",
                        // Contacts Sidebar
                        aside {
                            class: if selected_contact().is_some() {
                                "contacts-sidebar mobile-hidden"
                            } else {
                                "contacts-sidebar"
                            },

                            header { class: "sidebar-header",
                                h2 { class: "sidebar-title", "Contacts" }
                                if contact_count > 0 {
                                    span { class: "contact-count-badge", "{contact_count}" }
                                }
                            }

                            if contacts().is_empty() {
                                div { class: "empty-state",
                                    p { class: "empty-state-message", "No contacts yet" }
                                    p { class: "empty-state-hint", "Add contacts to start messaging." }
                                }
                            } else {
                                div { class: "contacts-list",
                                    for contact in contacts() {
                                        {
                                            let did = contact.peer.did.clone().unwrap_or_default();
                                            let name = contact.peer.display_name();
                                            let is_selected = selected_contact()
                                                .as_ref()
                                                .map(|s| s.did == did)
                                                .unwrap_or(false);
                                            let has_unread = contact.unread_count > 0;
                                            let did_clone = did.clone();
                                            let name_clone = name.clone();
                                            let avatar_blob_id = contact.peer.profile
                                                .as_ref()
                                                .and_then(|p| p.avatar_blob_id.clone());
                                            let first_char = name.chars().next()
                                                .unwrap_or('?')
                                                .to_uppercase()
                                                .to_string();

                                            rsx! {
                                                div {
                                                    key: "{did}",
                                                    class: if is_selected {
                                                        "contact-row contact-row-selected"
                                                    } else {
                                                        "contact-row"
                                                    },
                                                    onclick: move |_| {
                                                        selected_contact.set(Some(SelectedContact {
                                                            did: did_clone.clone(),
                                                            name: name_clone.clone(),
                                                        }));
                                                    },

                                                    // Avatar
                                                    div { class: "contact-row-avatar",
                                                        if let Some(ref blob_id) = avatar_blob_id {
                                                            AsyncImage {
                                                                blob_id: blob_id.clone(),
                                                                alt: name.clone(),
                                                                class: Some("avatar-image".to_string()),
                                                            }
                                                        } else {
                                                            div { class: "avatar-placeholder",
                                                                "{first_char}"
                                                            }
                                                        }
                                                    }

                                                    // Info
                                                    div { class: "contact-row-info",
                                                        div { class: "contact-row-header",
                                                            span { class: "contact-row-name", "{name}" }
                                                            if has_unread {
                                                                span { class: "unread-dot" }
                                                            }
                                                            if let Some(time) = contact.last_message_time {
                                                                span { class: "contact-row-time",
                                                                    "{format_relative_time(time)}"
                                                                }
                                                            }
                                                        }
                                                        if let Some(ref preview) = contact.last_message_preview {
                                                            p { class: "contact-row-preview", "{preview}" }
                                                        }
                                                    }

                                                    // Chevron (mobile indicator)
                                                    div { class: "contact-row-chevron",
                                                        svg {
                                                            width: "16",
                                                            height: "16",
                                                            view_box: "0 0 24 24",
                                                            fill: "none",
                                                            stroke: "currentColor",
                                                            stroke_width: "2",
                                                            polyline { points: "9 18 15 12 9 6" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Conversation Panel
                        main {
                            class: if selected_contact().is_some() {
                                "conversation-panel"
                            } else {
                                "conversation-panel mobile-hidden"
                            },

                            if let Some(ref contact) = selected_contact() {
                                ConversationView {
                                    contact_did: contact.did.clone(),
                                    contact_name: contact.name.clone(),
                                    messages: conversation_messages(),
                                    on_send: send_message,
                                    on_back: move |_| selected_contact.set(None),
                                    sending: sending(),
                                    loading: conversation_loading(),
                                }
                            } else {
                                // Empty state
                                div { class: "empty-conversation",
                                    p { class: "empty-icon", "~" }
                                    p { class: "empty-text", "Select a contact to begin transmission" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
