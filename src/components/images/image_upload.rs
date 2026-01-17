//! Image Upload Component
//!
//! File picker with automatic golden ratio cropping.

use dioxus::prelude::*;
use image::{DynamicImage, GenericImageView, ImageFormat};
use rfd::FileDialog;
use crate::context::use_engine;

/// Image upload button with golden ratio cropping
///
/// # Examples
///
/// ```rust
/// rsx! {
///     ImageUpload {
///         orientation: ImageOrientation::Portrait,
///         on_upload: move |blob_id| {
///             // Handle successful upload
///             println!("Uploaded: {}", blob_id);
///         },
///     }
/// }
/// ```
#[derive(Clone, Copy, PartialEq)]
pub enum ImageOrientation {
    /// Portrait: 1:1.618 (height > width)
    Portrait,
    /// Landscape: 1.618:1 (width > height)
    Landscape,
}

#[component]
pub fn ImageUpload(
    /// Desired image orientation (crops to golden ratio)
    #[props(default = ImageOrientation::Portrait)]
    orientation: ImageOrientation,
    /// Callback with blob hash on successful upload
    on_upload: EventHandler<String>,
    /// Optional button label
    #[props(default = "Upload Image".to_string())]
    label: String,
    /// Show only icon (no text label)
    #[props(default = false)]
    icon_only: bool,
) -> Element {
    let engine = use_engine();
    let mut uploading = use_signal(|| false);
    let mut error = use_signal(|| Option::<String>::None);

    let handle_upload = move |_| {
        uploading.set(true);
        error.set(None);

        spawn(async move {
            // Open file picker (blocking, but in spawn so UI stays responsive)
            let file_path = tokio::task::spawn_blocking(move || {
                FileDialog::new()
                    .add_filter("images", &["png", "jpg", "jpeg", "webp"])
                    .set_title("Select Image")
                    .pick_file()
            })
            .await;

            match file_path {
                Ok(Some(path)) => {
                    // Load image
                    match image::open(&path) {
                        Ok(img) => {
                            // Crop to golden ratio
                            let cropped = crop_to_golden_ratio(img, orientation);

                            // Encode as PNG (lossless)
                            let mut buffer = Vec::new();
                            match cropped.write_to(
                                &mut std::io::Cursor::new(&mut buffer),
                                ImageFormat::Png,
                            ) {
                                Ok(_) => {
                                    // Upload to blob storage
                                    let shared = engine();
                                    let guard = shared.read().await;
                                    if let Some(ref eng) = *guard {
                                        match eng.upload_image(buffer).await {
                                            Ok(blob_id) => {
                                                uploading.set(false);
                                                on_upload.call(blob_id);
                                            }
                                            Err(e) => {
                                                error.set(Some(format!("Upload failed: {:?}", e)));
                                                uploading.set(false);
                                            }
                                        }
                                    } else {
                                        error.set(Some("Engine not initialized".to_string()));
                                        uploading.set(false);
                                    }
                                }
                                Err(e) => {
                                    error.set(Some(format!("Failed to encode: {:?}", e)));
                                    uploading.set(false);
                                }
                            }
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to load image: {:?}", e)));
                            uploading.set(false);
                        }
                    }
                }
                Ok(None) => {
                    // User cancelled
                    uploading.set(false);
                }
                Err(e) => {
                    error.set(Some(format!("File picker error: {:?}", e)));
                    uploading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "image-upload",
            button {
                class: if icon_only { "image-upload-btn--icon" } else { "image-upload-btn" },
                onclick: handle_upload,
                disabled: uploading(),
                title: if icon_only { "Change image" } else { "" },
                if uploading() {
                    if icon_only {
                        "⏳"
                    } else {
                        "Uploading..."
                    }
                } else {
                    if icon_only {
                        "📷"
                    } else {
                        "{label}"
                    }
                }
            }

            if let Some(err) = error() {
                div { class: "image-upload__error",
                    "⚠️ {err}"
                }
            }
        }
    }
}

/// Crop image to golden ratio (1:1.618)
///
/// Centers the crop and maintains the golden proportion based on orientation.
fn crop_to_golden_ratio(img: DynamicImage, orientation: ImageOrientation) -> DynamicImage {
    let (width, height) = img.dimensions();
    let phi = 1.618;

    let (target_w, target_h) = match orientation {
        ImageOrientation::Portrait => {
            // Portrait: 1:1.618 (taller)
            let new_h = (width as f64 * phi) as u32;
            if new_h <= height {
                (width, new_h)
            } else {
                let new_w = (height as f64 / phi) as u32;
                (new_w, height)
            }
        }
        ImageOrientation::Landscape => {
            // Landscape: 1.618:1 (wider)
            let new_w = (height as f64 * phi) as u32;
            if new_w <= width {
                (new_w, height)
            } else {
                let new_h = (width as f64 / phi) as u32;
                (width, new_h)
            }
        }
    };

    // Center crop
    let x = (width.saturating_sub(target_w)) / 2;
    let y = (height.saturating_sub(target_h)) / 2;

    img.crop_imm(x, y, target_w, target_h)
}
