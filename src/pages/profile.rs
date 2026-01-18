//! Profile Page - Contact Command Center
//!
//! Redesigned layout emphasizing contact exchange with split-panel design.
//! Left: Avatar + Identity Sigil. Right: Inline-editable profile fields.

use dioxus::prelude::*;
use syncengine_core::types::contact::HybridContactInvite;
use syncengine_core::UserProfile;

use crate::app::Route;
use crate::components::cards::MarkdownRenderer;
use crate::components::contacts::{
    ContactsGallery, InviteCodeModal, InviterPreviewModal, PendingRequestsSection,
};
use crate::components::images::AsyncImage;
use crate::components::profile::QRSignature;
use crate::components::{NavHeader, NavLocation};
use crate::context::{use_engine, use_engine_ready};

// Embed default profile image as base64 data URI
const PROFILE_DEFAULT_BYTES: &[u8] = include_bytes!("../../assets/profile-default.png");

fn profile_default_uri() -> String {
    use base64::Engine;
    let base64 = base64::engine::general_purpose::STANDARD.encode(PROFILE_DEFAULT_BYTES);
    format!("data:image/png;base64,{}", base64)
}

/// Profile page - Contact Command Center layout
#[component]
pub fn Profile() -> Element {
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // State for loaded profile
    let mut profile: Signal<Option<UserProfile>> = use_signal(|| None);
    let mut loading = use_signal(|| true);

    // Inline editing state
    let mut editing_field: Signal<Option<String>> = use_signal(|| None);
    let mut edit_value = use_signal(String::new);

    // Modal states
    let mut show_receive_modal = use_signal(|| false);
    let mut decoded_invite: Signal<Option<HybridContactInvite>> = use_signal(|| None);

    // Invite button state
    let mut invite_copied = use_signal(|| false);

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

                        // Broadcast the updated profile to contacts
                        if let Err(e) = eng.announce_profile(None).await {
                            tracing::error!("Failed to announce profile update: {:?}", e);
                        } else {
                            tracing::info!("Profile update broadcast to contacts");
                        }
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
            // Sacred Navigation Console
            NavHeader {
                current: NavLocation::Profile,
                status: None,
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
                            // Left column: Avatar with QR overlay
                            div { class: "hero-left",
                                // Avatar with QR sigil overlay
                                div { class: "hero-avatar",
                                    if let Some(avatar_blob) = &p.avatar_blob_id {
                                        AsyncImage {
                                            blob_id: avatar_blob.clone(),
                                            alt: p.display_name.clone(),
                                            class: Some("hero-avatar-image".to_string()),
                                        }
                                    } else {
                                        img {
                                            class: "hero-avatar-image",
                                            src: "{profile_default_uri()}",
                                            alt: "{p.display_name}",
                                        }
                                    }

                                    // Identity Sigil (QR) overlaid at bottom
                                    div { class: "avatar-qr-overlay",
                                        QRSignature {
                                            data: p.peer_id.clone(),
                                            size: 120,
                                        }
                                    }
                                }

                                // Connection action buttons
                                div { class: "connection-actions",
                                    // Invite Connection button
                                    button {
                                        class: if invite_copied() { "connection-btn invite-btn copied" } else { "connection-btn invite-btn" },
                                        onclick: move |_| {
                                            spawn(async move {
                                                // Generate invite
                                                let shared = engine();
                                                let mut guard = shared.write().await;

                                                if let Some(ref mut eng) = *guard {
                                                    match eng.generate_contact_invite(24).await {
                                                        Ok(code) => {
                                                            // Copy to clipboard
                                                            match arboard::Clipboard::new() {
                                                                Ok(mut clipboard) => {
                                                                    if clipboard.set_text(&code).is_ok() {
                                                                        invite_copied.set(true);
                                                                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                                                        invite_copied.set(false);
                                                                    }
                                                                }
                                                                Err(e) => {
                                                                    tracing::warn!("Clipboard not available: {}", e);
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            tracing::error!("Failed to generate invite: {:?}", e);
                                                        }
                                                    }
                                                }
                                            });
                                        },
                                        span { class: "btn-icon", "✈" }
                                        span { class: "btn-text",
                                            if invite_copied() { "Copied!" } else { "Invite Connection" }
                                        }
                                    }

                                    // Scan Connection button
                                    button {
                                        class: "connection-btn scan-btn",
                                        onclick: move |_| {
                                            // Open receive modal for now (camera scanning TBD)
                                            show_receive_modal.set(true);
                                        },
                                        span { class: "btn-icon", "📷" }
                                        span { class: "btn-text", "Scan Connection" }
                                    }
                                }
                            }

                            // Right column: Editable identity info
                            div { class: "hero-identity",
                                // Display Name (click to edit)
                                {
                                    let field_name = "display_name".to_string();
                                    if editing_field() == Some(field_name.clone()) {
                                        let profile_for_blur = p.clone();
                                        let profile_for_key = p.clone();
                                        rsx! {
                                            input {
                                                class: "inline-edit-input name-input",
                                                r#type: "text",
                                                value: "{edit_value()}",
                                                autofocus: true,
                                                oninput: move |e| edit_value.set(e.value()),
                                                onblur: move |_| {
                                                    let mut updated = profile_for_blur.clone();
                                                    let new_val = edit_value();
                                                    if !new_val.is_empty() {
                                                        updated.display_name = new_val;
                                                        save_profile(updated);
                                                    }
                                                    editing_field.set(None);
                                                },
                                                onkeydown: move |e| {
                                                    if e.key() == Key::Enter {
                                                        let mut updated = profile_for_key.clone();
                                                        let new_val = edit_value();
                                                        if !new_val.is_empty() {
                                                            updated.display_name = new_val;
                                                            save_profile(updated);
                                                        }
                                                        editing_field.set(None);
                                                    } else if e.key() == Key::Escape {
                                                        editing_field.set(None);
                                                    }
                                                },
                                            }
                                        }
                                    } else {
                                        let name_to_edit = p.display_name.clone();
                                        rsx! {
                                            h2 {
                                                class: "hero-name editable",
                                                title: "Click to edit",
                                                onclick: move |_| {
                                                    edit_value.set(name_to_edit.clone());
                                                    editing_field.set(Some("display_name".to_string()));
                                                },
                                                "{p.display_name}"
                                            }
                                        }
                                    }
                                }

                                // Subtitle (click to edit)
                                {
                                    let field_name = "subtitle".to_string();
                                    if editing_field() == Some(field_name.clone()) {
                                        let profile_for_blur = p.clone();
                                        let profile_for_key = p.clone();
                                        rsx! {
                                            input {
                                                class: "inline-edit-input subtitle-input",
                                                r#type: "text",
                                                value: "{edit_value()}",
                                                placeholder: "Add a subtitle...",
                                                autofocus: true,
                                                oninput: move |e| edit_value.set(e.value()),
                                                onblur: move |_| {
                                                    let mut updated = profile_for_blur.clone();
                                                    let new_val = edit_value();
                                                    updated.subtitle = if new_val.is_empty() { None } else { Some(new_val) };
                                                    save_profile(updated);
                                                    editing_field.set(None);
                                                },
                                                onkeydown: move |e| {
                                                    if e.key() == Key::Enter {
                                                        let mut updated = profile_for_key.clone();
                                                        let new_val = edit_value();
                                                        updated.subtitle = if new_val.is_empty() { None } else { Some(new_val) };
                                                        save_profile(updated);
                                                        editing_field.set(None);
                                                    } else if e.key() == Key::Escape {
                                                        editing_field.set(None);
                                                    }
                                                },
                                            }
                                        }
                                    } else {
                                        let subtitle_to_edit = p.subtitle.clone().unwrap_or_default();
                                        rsx! {
                                            p {
                                                class: if p.subtitle.is_some() { "hero-subtitle editable" } else { "hero-subtitle editable placeholder" },
                                                title: "Click to edit",
                                                onclick: move |_| {
                                                    edit_value.set(subtitle_to_edit.clone());
                                                    editing_field.set(Some("subtitle".to_string()));
                                                },
                                                if let Some(sub) = &p.subtitle {
                                                    "{sub}"
                                                } else {
                                                    "Click to add subtitle..."
                                                }
                                            }
                                        }
                                    }
                                }

                                // Bio (click to edit)
                                {
                                    let field_name = "bio".to_string();
                                    if editing_field() == Some(field_name.clone()) {
                                        let profile_for_save = p.clone();
                                        let profile_for_cancel = p.clone();
                                        rsx! {
                                            div { class: "bio-edit-container",
                                                textarea {
                                                    class: "inline-edit-textarea bio-input",
                                                    value: "{edit_value()}",
                                                    placeholder: "Tell your story... (supports markdown)",
                                                    autofocus: true,
                                                    rows: "6",
                                                    oninput: move |e| edit_value.set(e.value()),
                                                    onkeydown: move |e| {
                                                        if e.key() == Key::Escape {
                                                            editing_field.set(None);
                                                        }
                                                    },
                                                }
                                                div { class: "bio-edit-actions",
                                                    button {
                                                        class: "bio-save-btn",
                                                        onclick: move |_| {
                                                            let mut updated = profile_for_save.clone();
                                                            updated.bio = edit_value();
                                                            save_profile(updated);
                                                            editing_field.set(None);
                                                        },
                                                        "Save"
                                                    }
                                                    button {
                                                        class: "bio-cancel-btn",
                                                        onclick: move |_| {
                                                            editing_field.set(None);
                                                        },
                                                        "Cancel"
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        let bio_to_edit = p.bio.clone();
                                        if !p.bio.is_empty() {
                                            let bio_content = p.bio.clone();
                                            let bio_signal = use_memo(move || bio_content.clone());
                                            rsx! {
                                                div {
                                                    class: "hero-bio editable",
                                                    title: "Click to edit",
                                                    onclick: move |_| {
                                                        edit_value.set(bio_to_edit.clone());
                                                        editing_field.set(Some("bio".to_string()));
                                                    },
                                                    MarkdownRenderer {
                                                        content: bio_signal,
                                                    }
                                                }
                                            }
                                        } else {
                                            rsx! {
                                                div {
                                                    class: "hero-bio editable placeholder",
                                                    title: "Click to edit",
                                                    onclick: move |_| {
                                                        edit_value.set(String::new());
                                                        editing_field.set(Some("bio".to_string()));
                                                    },
                                                    p { "Click to add your bio..." }
                                                }
                                            }
                                        }
                                    }
                                }

                                // DID (not editable, just display)
                                p { class: "hero-did", "{p.peer_id}" }
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

            } else {
                div { class: "error-state",
                    p { "Failed to load profile" }
                }
            }
        }
    }
}
