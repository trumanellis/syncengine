//! Contacts Gallery Component
//!
//! Grid display of all accepted contacts with real-time online/offline status.
//! Now uses the unified Peer type for consistency with the rest of the system.

use dioxus::prelude::*;
use syncengine_core::sync::ContactEvent;
use syncengine_core::types::contact::ContactStatus;
use syncengine_core::{Peer, PeerStatus};

use super::ContactCard;
use crate::context::use_engine;

/// Contacts Gallery
///
/// Displays all accepted contacts (peers with is_contact() == true) in a grid layout
/// with real-time status updates.
///
/// # Example
///
/// ```rust
/// rsx! {
///     ContactsGallery {}
/// }
/// ```
#[component]
pub fn ContactsGallery() -> Element {
    let engine = use_engine();
    let mut contacts = use_signal(|| Vec::<Peer>::new());
    let mut loading = use_signal(|| true);

    // Load contacts on mount and poll for updates
    use_effect(move || {
        spawn(async move {
            loop {
                let shared = engine();
                let guard = shared.read().await;

                if let Some(ref eng) = *guard {
                    // Use the new unified peer list, filtered to contacts only
                    match eng.list_peer_contacts() {
                        Ok(loaded_contacts) => {
                            contacts.set(loaded_contacts);
                        }
                        Err(e) => {
                            tracing::error!("Failed to load contacts: {:?}", e);
                        }
                    }
                }
                loading.set(false);

                // Poll every 2 seconds for new contacts
                // The event subscription should handle most updates, this is a fallback
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
    });

    // Subscribe to contact events for real-time updates
    use_effect(move || {
        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;

            if let Some(ref mut eng) = *guard {
                match eng.subscribe_contact_events().await {
                    Ok(mut event_rx) => {
                        drop(guard); // Release the lock before entering loop

                        while let Ok(event) = event_rx.recv().await {
                            match event {
                                ContactEvent::ContactAccepted { contact } => {
                                    // Convert ContactInfo to Peer for the list
                                    // We'll reload the full list to get the proper Peer object
                                    let shared = engine();
                                    let guard = shared.read().await;
                                    if let Some(ref eng) = *guard {
                                        if let Ok(updated_list) = eng.list_peer_contacts() {
                                            contacts.set(updated_list);
                                        }
                                    }
                                    let _ = contact; // Suppress unused warning
                                }
                                ContactEvent::ContactOnline { did } => {
                                    if let Some(c) = contacts.write().iter_mut().find(|c| c.did.as_deref() == Some(&did)) {
                                        c.status = PeerStatus::Online;
                                    }
                                }
                                ContactEvent::ContactOffline { did } => {
                                    if let Some(c) = contacts.write().iter_mut().find(|c| c.did.as_deref() == Some(&did)) {
                                        c.status = PeerStatus::Offline;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to subscribe to contact events: {:?}", e);
                    }
                }
            }
        });
    });

    if loading() {
        return rsx! {
            div { class: "contacts-gallery loading",
                "Loading contacts..."
            }
        };
    }

    let contact_list = contacts();
    let online_count = contact_list
        .iter()
        .filter(|c| matches!(c.status, PeerStatus::Online))
        .count();

    if contact_list.is_empty() {
        return rsx! {
            div { class: "contacts-gallery-empty",
                h3 { class: "section-title", "Contacts" }

                div { class: "empty-state",
                    div { class: "empty-icon", "⬡" }
                    p { "No contacts yet." }
                    p { class: "empty-hint",
                        "Share your invitation code to connect with others."
                    }
                }
            }
        };
    }

    rsx! {
        div { class: "contacts-gallery",
            h3 { class: "section-title",
                "Contacts ({online_count} online)"
            }

            div { class: "contact-grid",
                {contact_list.iter().enumerate().map(|(index, contact)| {
                    let contact_did = contact.did.clone().unwrap_or_else(|| format!("peer_{}", hex::encode(&contact.endpoint_id[..4])));
                    let contact_did_for_click = contact_did.clone();
                    let contact_name_display = contact.display_name();
                    let contact_avatar_display = contact.profile.as_ref().and_then(|p| p.avatar_blob_id.clone());
                    let is_online_display = matches!(contact.status, PeerStatus::Online);

                    rsx! {
                        ContactCard {
                            key: "{contact_did}",
                            contact_name: contact_name_display,
                            contact_avatar: contact_avatar_display,
                            is_online: is_online_display,
                            index: index,
                            on_click: move |_| {
                                tracing::info!("Clicked contact: {}", contact_did_for_click);
                            },
                        }
                    }
                })}
            }
        }
    }
}
