//! Invite Panel Components for Synchronicity Engine.
//!
//! Components for generating and sharing realm invites:
//! - `InvitePanel` - Generate invite and display as QR code
//! - `JoinRealmModal` - Modal for joining via invite ticket
//! - `QrCodeDisplay` - QR code renderer using qrcode crate
//!
//! Uses sacred terminology:
//! - "Summon Others" instead of "Invite"
//! - "Enter invite sigil" as placeholder
//! - "Join the Field" as button text
//! - "Sigil copied to clipboard" as feedback

use dioxus::prelude::*;
use dioxus::events::MouseData;
use syncengine_core::{InviteTicket, RealmId};

use crate::context::use_engine;

/// Generate QR code data URL from a string.
///
/// Returns a base64-encoded PNG data URL that can be used as an img src.
/// Returns None if QR code generation fails.
fn generate_qr_data_url(data: &str) -> Option<String> {
    use qrcode::QrCode;
    use qrcode::render::svg;

    let code = QrCode::new(data.as_bytes()).ok()?;

    // Render as SVG for crisp scaling
    let svg_string = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#f5f5f5")) // White modules on void
        .light_color(svg::Color("#0a0a0a")) // Void background
        .build();

    // Encode as base64 data URL
    let encoded = base64_encode(&svg_string);
    Some(format!("data:image/svg+xml;base64,{}", encoded))
}

/// Simple base64 encoding without external dependency.
fn base64_encode(data: &str) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut encoder = base64_encoder(&mut buf);
        encoder.write_all(data.as_bytes()).ok();
    }
    String::from_utf8(buf).unwrap_or_default()
}

/// Create a base64 encoder.
fn base64_encoder(output: &mut Vec<u8>) -> impl std::io::Write + '_ {
    Base64Writer::new(output)
}

/// Simple base64 writer for encoding data.
struct Base64Writer<'a> {
    output: &'a mut Vec<u8>,
    buffer: [u8; 3],
    buffer_len: usize,
}

impl<'a> Base64Writer<'a> {
    fn new(output: &'a mut Vec<u8>) -> Self {
        Self {
            output,
            buffer: [0; 3],
            buffer_len: 0,
        }
    }

    fn flush_buffer(&mut self) {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        if self.buffer_len == 0 {
            return;
        }

        let b0 = self.buffer[0];
        let b1 = if self.buffer_len > 1 { self.buffer[1] } else { 0 };
        let b2 = if self.buffer_len > 2 { self.buffer[2] } else { 0 };

        self.output.push(ALPHABET[(b0 >> 2) as usize]);
        self.output.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize]);

        if self.buffer_len > 1 {
            self.output.push(ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize]);
        } else {
            self.output.push(b'=');
        }

        if self.buffer_len > 2 {
            self.output.push(ALPHABET[(b2 & 0x3f) as usize]);
        } else {
            self.output.push(b'=');
        }

        self.buffer_len = 0;
    }
}

impl std::io::Write for Base64Writer<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for &byte in buf {
            self.buffer[self.buffer_len] = byte;
            self.buffer_len += 1;

            if self.buffer_len == 3 {
                self.flush_buffer();
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush_buffer();
        Ok(())
    }
}

impl Drop for Base64Writer<'_> {
    fn drop(&mut self) {
        self.flush_buffer();
    }
}

/// Format seconds into a human-readable countdown string.
fn format_countdown(seconds: i64) -> String {
    if seconds <= 0 {
        return "expired".to_string();
    }

    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// QR Code display component.
///
/// Generates and displays a QR code from the provided data string.
/// Falls back to displaying the raw text if QR generation fails.
#[component]
pub fn QrCodeDisplay(
    /// The data to encode as a QR code
    data: String,
    /// Optional size in pixels (default: 200)
    #[props(default = 200)]
    size: u32,
) -> Element {
    // Clone data for use in both memo and fallback
    let data_for_memo = data.clone();
    let data_for_fallback = data.clone();

    // Generate QR code data URL
    let qr_url = use_memo(move || generate_qr_data_url(&data_for_memo));

    match qr_url() {
        Some(url) => {
            rsx! {
                div { class: "qr-code-container",
                    img {
                        class: "qr-code-image",
                        src: "{url}",
                        alt: "Invite QR Code",
                        width: "{size}",
                        height: "{size}"
                    }
                }
            }
        }
        None => {
            rsx! {
                div { class: "qr-code-fallback",
                    p { class: "qr-fallback-label", "sigil generation failed" }
                    code { class: "qr-fallback-text", "{data_for_fallback}" }
                }
            }
        }
    }
}

/// Invite Panel component for generating and displaying realm invites.
///
/// Displays a "Summon Others" button that generates an invite ticket.
/// Once generated, shows:
/// - QR code representation
/// - Copyable invite link
/// - Expiration countdown (if applicable)
#[component]
pub fn InvitePanel(
    /// The realm to generate invites for
    realm_id: RealmId,
    /// Callback when an invite is successfully created
    #[props(default)]
    on_invite_created: Option<EventHandler<InviteTicket>>,
) -> Element {
    let mut invite_ticket: Signal<Option<InviteTicket>> = use_signal(|| None);
    let mut invite_string: Signal<Option<String>> = use_signal(|| None);
    let mut loading: Signal<bool> = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut copied: Signal<bool> = use_signal(|| false);
    let mut expiry_seconds: Signal<Option<i64>> = use_signal(|| None);

    // Get shared engine from context
    let engine = use_engine();

    // Generate invite handler
    let generate_invite = move |_| {
        let realm_id = realm_id.clone();
        let on_created = on_invite_created.clone();

        spawn(async move {
            loading.set(true);
            error.set(None);

            // Use shared engine from context
            let shared = engine();
            let mut guard = shared.write().await;

            if let Some(ref mut eng) = *guard {
                match eng.generate_invite(&realm_id).await {
                    Ok(ticket) => {
                        // Calculate expiry if set
                        if let Some(exp) = ticket.expires_at {
                            let now = chrono::Utc::now().timestamp();
                            expiry_seconds.set(Some(exp - now));
                        }

                        // Encode the ticket
                        match ticket.encode() {
                            Ok(encoded) => {
                                invite_string.set(Some(encoded));

                                // Call callback if provided
                                if let Some(handler) = &on_created {
                                    handler.call(ticket.clone());
                                }

                                invite_ticket.set(Some(ticket));
                            }
                            Err(e) => {
                                error.set(Some(format!("Failed to encode sigil: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to summon sigil: {}", e)));
                    }
                }
            } else {
                error.set(Some("Engine not initialized".to_string()));
            }

            loading.set(false);
        });
    };

    // Copy to clipboard handler
    let copy_to_clipboard = move |_| {
        if let Some(ref invite) = invite_string() {
            let invite_text = invite.clone();

            spawn(async move {
                // Use arboard for cross-platform clipboard access
                match arboard::Clipboard::new() {
                    Ok(mut clipboard) => {
                        if clipboard.set_text(&invite_text).is_ok() {
                            copied.set(true);
                            // Reset copied state after 2 seconds
                            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                            copied.set(false);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Clipboard not available: {}", e);
                        // Still show feedback even if clipboard fails
                        copied.set(true);
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        copied.set(false);
                    }
                }
            });
        }
    };

    // Expiry countdown effect
    use_effect(move || {
        if expiry_seconds().is_some() {
            spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    if let Some(secs) = expiry_seconds() {
                        if secs > 0 {
                            expiry_seconds.set(Some(secs - 1));
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            });
        }
    });

    rsx! {
        div { class: "invite-panel",
            h3 { class: "invite-panel-header",
                span { class: "sacred-term", "Summon Others" }
            }

            // Error display
            if let Some(err) = error() {
                div { class: "invite-error",
                    span { "{err}" }
                    button {
                        class: "error-dismiss",
                        onclick: move |_| error.set(None),
                        "dismiss"
                    }
                }
            }

            // Generate button (shown when no invite)
            if invite_string().is_none() && !loading() {
                button {
                    class: "btn-primary invite-generate-btn",
                    onclick: generate_invite,
                    "Generate Invite Sigil"
                }
            }

            // Loading state
            if loading() {
                div { class: "invite-loading",
                    span { class: "loading-text", "summoning sigil..." }
                }
            }

            // Invite display (shown when invite exists)
            if let Some(ref invite) = invite_string() {
                div { class: "invite-display",
                    // QR Code
                    QrCodeDisplay {
                        data: invite.clone(),
                        size: 200
                    }

                    // Invite text
                    div { class: "invite-ticket-container",
                        label { class: "input-label", "invite sigil" }
                        div { class: "invite-ticket-text",
                            code { class: "invite-ticket-code", "{invite}" }
                        }
                    }

                    // Copy button
                    button {
                        class: if copied() { "btn-primary invite-copy-btn copied" } else { "btn-primary invite-copy-btn" },
                        onclick: copy_to_clipboard,
                        if copied() {
                            "Sigil copied to clipboard"
                        } else {
                            "Copy Sigil"
                        }
                    }

                    // Expiration countdown
                    if let Some(secs) = expiry_seconds() {
                        div { class: "invite-expiry",
                            span { class: "expiry-label", "sigil expires in: " }
                            span { class: "expiry-countdown", "{format_countdown(secs)}" }
                        }
                    }

                    // Realm name if available
                    if let Some(ref ticket) = invite_ticket() {
                        if let Some(ref name) = ticket.realm_name {
                            div { class: "invite-realm-name",
                                span { "realm: " }
                                span { class: "sacred-term", "{name}" }
                            }
                        }
                    }

                    // Generate new button
                    button {
                        class: "btn-badge invite-new-btn",
                        onclick: move |_| {
                            invite_ticket.set(None);
                            invite_string.set(None);
                            expiry_seconds.set(None);
                        },
                        "generate new sigil"
                    }
                }
            }
        }
    }
}

/// Join Realm Modal component.
///
/// Modal dialog for joining a realm via invite ticket.
/// Uses sacred language:
/// - "Enter invite sigil" as placeholder
/// - "Join the Field" as button text
#[component]
pub fn JoinRealmModal(
    /// Whether the modal is visible
    show: bool,
    /// Callback when modal should close
    on_close: EventHandler<()>,
    /// Callback when join is successful (passes the invite ticket string)
    on_join: EventHandler<String>,
) -> Element {
    let mut invite_input: Signal<String> = use_signal(String::new);
    let mut loading: Signal<bool> = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut success: Signal<bool> = use_signal(|| false);

    // Core join logic - shared between button click and Enter key
    let mut do_join = move || {
        let ticket_str = invite_input.read().clone();
        if ticket_str.trim().is_empty() {
            error.set(Some("Please enter an invite sigil".to_string()));
            return;
        }

        spawn(async move {
            loading.set(true);
            error.set(None);

            // Validate the ticket format first
            match InviteTicket::decode(&ticket_str) {
                Ok(ticket) => {
                    // Check if expired
                    if ticket.is_expired() {
                        error.set(Some("This sigil has expired".to_string()));
                        loading.set(false);
                        return;
                    }

                    // Try to join the realm
                    let data_dir = crate::context::get_data_dir();
                    match syncengine_core::SyncEngine::new(&data_dir).await {
                        Ok(mut engine) => {
                            match engine.join_via_invite(&ticket).await {
                                Ok(_realm_id) => {
                                    success.set(true);

                                    // Wait a moment to show success, then close
                                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                                    on_join.call(ticket_str);
                                }
                                Err(e) => {
                                    error.set(Some(format!("Failed to join realm: {}", e)));
                                }
                            }
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to connect to engine: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    error.set(Some(format!("Invalid sigil format: {}", e)));
                }
            }

            loading.set(false);
        });
    };

    // Handle join button click
    let handle_join_click = move |_evt: Event<MouseData>| {
        do_join();
    };

    // Handle enter key
    let on_keydown = move |evt: KeyboardEvent| {
        if evt.key() == Key::Enter && !loading() {
            do_join();
        } else if evt.key() == Key::Escape {
            on_close.call(());
        }
    };

    // Reset state when modal opens/closes
    use_effect(move || {
        if !show {
            invite_input.set(String::new());
            error.set(None);
            success.set(false);
        }
    });

    if !show {
        return rsx! {};
    }

    rsx! {
        div { class: "modal-overlay",
            onclick: move |_| on_close.call(()),

            div {
                class: "modal-content join-realm-modal",
                onclick: move |evt| evt.stop_propagation(),

                // Header
                header { class: "modal-header",
                    h2 { class: "section-header", "Join a Realm" }
                    button {
                        class: "modal-close-btn",
                        onclick: move |_| on_close.call(()),
                        "\u{00D7}"
                    }
                }

                // Body
                div { class: "modal-body",
                    // Error display
                    if let Some(err) = error() {
                        div { class: "join-error",
                            span { "{err}" }
                        }
                    }

                    // Success message
                    if success() {
                        div { class: "join-success",
                            span { "Successfully joined the realm!" }
                        }
                    }

                    // Input field
                    if !success() {
                        div { class: "form-field",
                            label { class: "input-label", "invite sigil" }
                            textarea {
                                class: "input-field join-input",
                                placeholder: "Enter invite sigil...",
                                value: "{invite_input}",
                                oninput: move |e| invite_input.set(e.value()),
                                onkeydown: on_keydown,
                                disabled: loading(),
                                rows: "4"
                            }
                            p { class: "input-hint",
                                "paste the sigil you received from another steward"
                            }
                        }
                    }
                }

                // Footer
                footer { class: "modal-footer",
                    if !success() {
                        button {
                            class: "btn-small btn-cancel",
                            onclick: move |_| on_close.call(()),
                            disabled: loading(),
                            "Release"
                        }
                        button {
                            class: "btn-primary",
                            onclick: handle_join_click,
                            disabled: loading() || invite_input.read().trim().is_empty(),
                            if loading() {
                                "joining..."
                            } else {
                                "Join the Field"
                            }
                        }
                    } else {
                        button {
                            class: "btn-primary",
                            onclick: move |_| on_close.call(()),
                            "Enter the Field"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_countdown_seconds() {
        assert_eq!(format_countdown(45), "45s");
        assert_eq!(format_countdown(1), "1s");
    }

    #[test]
    fn test_format_countdown_minutes() {
        assert_eq!(format_countdown(90), "1m 30s");
        assert_eq!(format_countdown(60), "1m 0s");
        assert_eq!(format_countdown(125), "2m 5s");
    }

    #[test]
    fn test_format_countdown_hours() {
        assert_eq!(format_countdown(3600), "1h 0m");
        assert_eq!(format_countdown(3660), "1h 1m");
        assert_eq!(format_countdown(7200), "2h 0m");
    }

    #[test]
    fn test_format_countdown_expired() {
        assert_eq!(format_countdown(0), "expired");
        assert_eq!(format_countdown(-1), "expired");
        assert_eq!(format_countdown(-100), "expired");
    }

    #[test]
    fn test_base64_encode_simple() {
        // Test basic encoding
        let result = base64_encode("Hello");
        assert!(!result.is_empty());
        // "Hello" in base64 is "SGVsbG8="
        assert_eq!(result, "SGVsbG8=");
    }

    #[test]
    fn test_base64_encode_padding() {
        // Test different padding scenarios
        assert_eq!(base64_encode("a"), "YQ==");     // 1 byte -> 2 padding
        assert_eq!(base64_encode("ab"), "YWI=");    // 2 bytes -> 1 padding
        assert_eq!(base64_encode("abc"), "YWJj");   // 3 bytes -> no padding
    }
}
