//! Profile Page - Contact Command Center
//!
//! Redesigned layout emphasizing contact exchange with split-panel design.
//! Left: Identity beacon (compact). Right: Contact flows (hero).

use dioxus::prelude::*;
use syncengine_core::types::contact::HybridContactInvite;
use syncengine_core::UserProfile;

use crate::app::Route;
use crate::components::cards::{MarkdownRenderer, ProfileCard};
use crate::components::contacts::{
    ContactsGallery, GenerateInviteButton, InviteCodeModal, InviterPreviewModal,
    PendingRequestsSection,
};
use crate::components::images::AsyncImage;
use crate::components::profile::QRSignature;
use crate::context::{use_engine, use_engine_ready};

/// Profile page - Contact Command Center layout
#[component]
pub fn Profile() -> Element {
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // State for loaded profile
    let mut profile: Signal<Option<UserProfile>> = use_signal(|| None);
    let mut loading = use_signal(|| true);

    // Identity beacon expansion state
    let mut identity_expanded = use_signal(|| false);

    // Modal states
    let mut show_receive_modal = use_signal(|| false);
    let mut show_edit_modal = use_signal(|| false);
    let mut decoded_invite: Signal<Option<HybridContactInvite>> = use_signal(|| None);

    // Load profile when engine becomes ready
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let guard = shared.read().await;

                if let Some(ref eng) = *guard {
                    match eng.get_own_profile() {
                        Ok(prof) => {
                            profile.set(Some(prof));
                            loading.set(false);
                        }
                        Err(e) => {
                            tracing::error!("Failed to load profile: {:?}", e);
                            loading.set(false);
                        }
                    }
                }
            });
        }
    });

    // Save profile handler
    let save_profile = move |updated: UserProfile| {
        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;
            if let Some(ref mut eng) = *guard {
                match eng.save_profile(&updated) {
                    Ok(_) => {
                        tracing::info!("Profile saved successfully");
                        profile.set(Some(updated));
                    }
                    Err(e) => {
                        tracing::error!("Failed to save profile: {:?}", e);
                    }
                }
            }
        });
    };

    rsx! {
        div { class: "profile-page",
            // Header with back navigation
            header { class: "profile-header",
                Link {
                    to: Route::Field {},
                    button {
                        class: "back-link",
                        title: "Return to Field",
                        "← field"
                    }
                }
                h1 { class: "profile-title", "Identity & Connections" }
            }

            // Main content
            if loading() {
                div { class: "loading-state",
                    div { class: "loading-orb" }
                    p { "synchronicities are forming..." }
                }
            } else if let Some(p) = profile() {
                div { class: "profile-content",
                    // HERO: Identity Profile Card
                    section { class: "identity-hero",
                        div { class: "identity-card",
                            // Avatar
                            div { class: "hero-avatar",
                                if let Some(avatar_blob) = &p.avatar_blob_id {
                                    AsyncImage {
                                        blob_id: avatar_blob.clone(),
                                        alt: p.display_name.clone(),
                                        class: Some("hero-avatar-image".to_string()),
                                    }
                                } else {
                                    span { class: "hero-avatar-initial",
                                        "{p.display_name.chars().next().unwrap_or('?').to_uppercase()}"
                                    }
                                }
                            }

                            // Identity Info
                            div { class: "hero-identity",
                                h2 { class: "hero-name", "{p.display_name}" }
                                if let Some(subtitle) = &p.subtitle {
                                    p { class: "hero-subtitle", "{subtitle}" }
                                }

                                if !p.bio.is_empty() {
                                    div { class: "hero-bio",
                                        {
                                            let bio_content = p.bio.clone();
                                            let bio_signal = use_memo(move || bio_content.clone());
                                            rsx! {
                                                MarkdownRenderer {
                                                    content: bio_signal,
                                                }
                                            }
                                        }
                                    }
                                }

                                p { class: "hero-did", "{p.peer_id}" }
                            }

                            // QR Code (always visible)
                            div { class: "hero-qr",
                                p { class: "qr-label", "Identity Sigil" }
                                QRSignature {
                                    data: p.peer_id.clone(),
                                    size: 140,
                                }
                                p { class: "qr-hint", "Others can scan to verify" }
                            }

                            // Edit button
                            button {
                                class: "hero-edit-btn",
                                onclick: move |_| show_edit_modal.set(true),
                                "✎ Edit Identity"
                            }
                        }
                    }

                    // Contact Exchange Section
                    section { class: "contact-exchange",
                        h3 { class: "section-title", "Connections" }

                        div { class: "exchange-actions",
                            // Share invite
                            div { class: "exchange-card share-card",
                                div { class: "card-icon", "⬡" }
                                h4 { class: "card-title", "Share Invite" }
                                p { class: "card-description",
                                    "Generate an invitation code for others to connect"
                                }
                                GenerateInviteButton {}
                            }

                            // Receive invite
                            div { class: "exchange-card receive-card",
                                div { class: "card-icon", "⬢" }
                                h4 { class: "card-title", "Receive Invite" }
                                p { class: "card-description",
                                    "Enter an invitation code to connect"
                                }
                                button {
                                    class: "exchange-btn",
                                    onclick: move |_| show_receive_modal.set(true),
                                    "Enter Code"
                                }
                            }
                        }
                    }

                    // Pending requests
                    section { class: "pending-section",
                        PendingRequestsSection {}
                    }

                    // Contacts gallery
                    section { class: "contacts-section",
                        ContactsGallery {}
                    }
                }

                // Modals (only render when shown)
                if show_receive_modal() {
                    InviteCodeModal {
                        show: true,
                        on_close: move |_| show_receive_modal.set(false),
                        on_invite_decoded: move |invite: HybridContactInvite| {
                            tracing::info!("Received invite from: {}", invite.display_name);
                            // Store the decoded invite and show preview modal
                            decoded_invite.set(Some(invite));
                            show_receive_modal.set(false);
                        }
                    }
                }

                // Inviter Preview Modal (shown after decoding invite)
                if let Some(invite) = decoded_invite() {
                    InviterPreviewModal {
                        inviter_name: invite.display_name.clone(),
                        inviter_subtitle: None,
                        inviter_bio: "Full profile will be available when you connect.".to_string(),
                        inviter_avatar: None,
                        show: true,
                        on_close: move |_| decoded_invite.set(None),
                        on_accept: move |_| {
                            let invite_to_accept = invite.clone();
                            spawn(async move {
                                let shared = engine();
                                let mut guard = shared.write().await;
                                if let Some(ref mut eng) = *guard {
                                    match eng.send_contact_request(invite_to_accept).await {
                                        Ok(_) => {
                                            tracing::info!("Contact request sent successfully");
                                            decoded_invite.set(None);
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to send contact request: {:?}", e);
                                        }
                                    }
                                }
                            });
                        },
                        on_decline: move |_| {
                            tracing::info!("Contact request declined");
                            decoded_invite.set(None);
                        }
                    }
                }

                if show_edit_modal() {
                    div {
                        class: "modal-overlay",
                        onclick: move |_| show_edit_modal.set(false),

                        div {
                            class: "profile-edit-modal",
                            onclick: move |e| e.stop_propagation(),

                            button {
                                class: "modal-close-btn",
                                onclick: move |_| show_edit_modal.set(false),
                                "×"
                            }

                            ProfileCard {
                                profile: p.clone(),
                                editable: true,
                                show_qr: false,
                                on_update: save_profile,
                            }
                        }
                    }
                }
            } else {
                div { class: "error-state",
                    p { "Failed to load profile" }
                }
            }
        }
    }
}
