//! Contacts Gallery Component
//!
//! Grid display of all accepted contacts with real-time online/offline status.

use dioxus::prelude::*;
use syncengine_core::sync::ContactEvent;
use syncengine_core::types::contact::{ContactInfo, ContactStatus};

use super::ContactCard;
use crate::context::use_engine;

/// Contacts Gallery
///
/// Displays all accepted contacts in a grid layout with real-time status updates.
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
    let mut contacts = use_signal(|| Vec::<ContactInfo>::new());
    let mut loading = use_signal(|| true);

    // Load contacts on mount and poll for updates
    use_effect(move || {
        spawn(async move {
            loop {
                let shared = engine();
                let guard = shared.read().await;

                if let Some(ref eng) = *guard {
                    match eng.list_contacts() {
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
                                    contacts.write().push(contact);
                                }
                                ContactEvent::ContactOnline { did } => {
                                    if let Some(c) = contacts.write().iter_mut().find(|c| c.peer_did == did) {
                                        c.status = ContactStatus::Online;
                                    }
                                }
                                ContactEvent::ContactOffline { did } => {
                                    if let Some(c) = contacts.write().iter_mut().find(|c| c.peer_did == did) {
                                        c.status = ContactStatus::Offline;
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
        .filter(|c| matches!(c.status, ContactStatus::Online))
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
                    let contact_did = contact.peer_did.clone();
                    let contact_did_for_click = contact_did.clone(); // Clone for closure
                    let contact_name_display = contact.profile.display_name.clone();
                    let contact_avatar_display = contact.profile.avatar_blob_id.clone();
                    let is_online_display = matches!(contact.status, ContactStatus::Online);

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
