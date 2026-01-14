//! QR Signature - QR code generator for node identity.

use dioxus::prelude::*;
use qrcode::QrCode;
use qrcode::render::svg;

/// Props for the QR signature component.
#[derive(Props, Clone, PartialEq)]
pub struct QRSignatureProps {
    /// Data to encode in QR code
    pub data: String,
    /// QR code size in pixels
    pub size: u32,
}

/// QR code generator component.
#[component]
pub fn QRSignature(props: QRSignatureProps) -> Element {
    // Generate QR code SVG
    let qr_svg = use_memo(move || {
        match QrCode::new(props.data.as_bytes()) {
            Ok(code) => {
                let svg_string = code
                    .render()
                    .min_dimensions(props.size, props.size)
                    .dark_color(svg::Color("#00d4aa")) // Cyan
                    .light_color(svg::Color("transparent")) // Transparent background
                    .build();
                svg_string
            }
            Err(e) => {
                tracing::error!("Failed to generate QR code: {:?}", e);
                String::new()
            }
        }
    });

    rsx! {
        if !qr_svg().is_empty() {
            div {
                dangerous_inner_html: "{qr_svg()}",
            }
        } else {
            div { class: "qr-error", "Failed to generate QR code" }
        }
    }
}
