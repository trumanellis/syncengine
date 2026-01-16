//! Pending Requests Section Component
//!
//! Displays incoming and outgoing contact requests with accept/decline actions.

use dioxus::prelude::*;
use syncengine_core::types::contact::PendingContact;

use crate::context::use_engine;

/// Pending Contact Requests Section
///
/// Shows both incoming requests (requiring user action) and outgoing requests
/// (awaiting response from other party).
///
/// # Example
///
/// ```rust
/// rsx! {
///     PendingRequestsSection {}
/// }
/// ```
#[component]
pub fn PendingRequestsSection() -> Element {
    let engine = use_engine();
    let mut pending = use_signal(|| (Vec::<PendingContact>::new(), Vec::<PendingContact>::new()));
    let mut loading = use_signal(|| true);

    // Load pending contacts on mount and poll for updates
    use_effect(move || {
        spawn(async move {
            loop {
                let shared = engine();
                let guard = shared.read().await;

                if let Some(ref eng) = *guard {
                    match eng.list_pending_contacts() {
                        Ok((incoming, outgoing)) => {
                            pending.set((incoming, outgoing));
                        }
                        Err(e) => {
                            tracing::error!("Failed to load pending contacts: {:?}", e);
                        }
                    }
                }
                loading.set(false);

                // Poll every 2 seconds for new requests
                // TODO: Replace with proper event subscription when available
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
    });

    let accept_contact = move |invite_id: [u8; 16]| {
        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;

            if let Some(ref mut eng) = *guard {
                match eng.accept_contact(&invite_id).await {
                    Ok(_) => {
                        tracing::info!("Contact accepted");
                        // Reload pending list by updating the signal
                        // In a real app, we'd listen to ContactEvent::ContactAccepted
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept contact: {:?}", e);
                    }
                }
            }
        });
    };

    let decline_contact = move |invite_id: [u8; 16]| {
        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;

            if let Some(ref mut eng) = *guard {
                match eng.decline_contact(&invite_id).await {
                    Ok(_) => {
                        tracing::info!("Contact declined");
                        // Reload pending list
                    }
                    Err(e) => {
                        tracing::error!("Failed to decline contact: {:?}", e);
                    }
                }
            }
        });
    };

    let cancel_request = move |invite_id: [u8; 16]| {
        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;

            if let Some(ref mut eng) = *guard {
                // Cancel by declining (same as declining on sender side)
                match eng.decline_contact(&invite_id).await {
                    Ok(_) => {
                        tracing::info!("Request cancelled");
                    }
                    Err(e) => {
                        tracing::error!("Failed to cancel request: {:?}", e);
                    }
                }
            }
        });
    };

    if loading() {
        return rsx! {
            div { class: "pending-requests-section loading",
                "Loading pending requests..."
            }
        };
    }

    let (incoming, outgoing) = pending();
    let total_pending = incoming.len() + outgoing.len();

    if total_pending == 0 {
        return rsx! { };
    }

    rsx! {
        div { class: "pending-requests-section",
            h3 { class: "section-title",
                "Pending Connections ({total_pending})"
            }

            if !incoming.is_empty() {
                div { class: "incoming-requests",
                    h4 { class: "subsection-title", "Incoming Requests:" }
                    {incoming.iter().map(|pending_contact| {
                        let invite_id = pending_contact.invite_id;
                        let contact_name = pending_contact.profile.display_name.clone();
                        let contact_did = pending_contact.peer_did.clone();

                        rsx! {
                            div {
                                class: "pending-card incoming",
                                key: "{contact_did}",

                                div { class: "pending-info",
                                    span { class: "pending-name", "{contact_name}" }
                                    span { class: "pending-message", " wants to connect" }
                                }

                                div { class: "pending-actions",
                                    button {
                                        class: "accept-button btn-small",
                                        onclick: move |_| {
                                            accept_contact(invite_id);
                                        },
                                        "Accept"
                                    }
                                    button {
                                        class: "decline-button btn-small",
                                        onclick: move |_| {
                                            decline_contact(invite_id);
                                        },
                                        "Decline"
                                    }
                                }
                            }
                        }
                    })}
                }
            }

            if !outgoing.is_empty() {
                div { class: "outgoing-requests",
                    h4 { class: "subsection-title", "Awaiting Response:" }
                    {outgoing.iter().map(|pending_contact| {
                        let invite_id = pending_contact.invite_id;
                        let contact_name = pending_contact.profile.display_name.clone();
                        let contact_did = pending_contact.peer_did.clone();

                        rsx! {
                            div {
                                class: "pending-card outgoing",
                                key: "{contact_did}",

                                div { class: "pending-info",
                                    span { class: "pending-name", "{contact_name}" }
                                    span { class: "pending-message", " - invitation sent" }
                                }

                                button {
                                    class: "cancel-button btn-small",
                                    onclick: move |_| {
                                        cancel_request(invite_id);
                                    },
                                    "Cancel Request"
                                }
                            }
                        }
                    })}
                }
            }
        }
    }
}
