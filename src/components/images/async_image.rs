//! Async Image Loader
//!
//! Loads images from blob storage and displays with loading state.

use dioxus::prelude::*;
use crate::context::use_engine;

/// Asynchronously load and display image from blob storage
///
/// # Examples
///
/// ```rust
/// rsx! {
///     AsyncImage {
///         blob_id: "abc123def...".to_string(),
///         alt: "Profile avatar".to_string(),
///     }
/// }
/// ```
#[component]
pub fn AsyncImage(
    /// Blob content hash (BLAKE3 hex string)
    blob_id: String,
    /// Alt text for accessibility
    alt: String,
    /// Optional CSS class
    #[props(default = None)]
    class: Option<String>,
) -> Element {
    let engine = use_engine();
    let mut image_data = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| Option::<String>::None);

    // Load image on mount or when blob_id changes
    use_effect(move || {
        let blob_id = blob_id.clone();
        spawn(async move {
            loading.set(true);
            error.set(None);

            let shared = engine();
            let guard = shared.read().await;

            if let Some(ref eng) = *guard {
                match eng.load_image(&blob_id) {
                    Ok(Some(data)) => {
                        // Convert to base64 data URI
                        use base64::Engine;
                        let base64 = base64::engine::general_purpose::STANDARD.encode(&data);
                        let data_uri = format!("data:image/png;base64,{}", base64);
                        image_data.set(Some(data_uri));
                        loading.set(false);
                    }
                    Ok(None) => {
                        error.set(Some("Image not found".to_string()));
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(format!("Error loading image: {:?}", e)));
                        loading.set(false);
                    }
                }
            } else {
                error.set(Some("Engine not initialized".to_string()));
                loading.set(false);
            }
        });
    });

    let css_class = class.unwrap_or_else(|| "card-image__img".to_string());

    rsx! {
        if loading() {
            div {
                class: "card-image__loading",
                div { class: "loading-spinner" }
                "Loading..."
            }
        } else if let Some(err) = error() {
            div {
                class: "card-image__error",
                "⚠️ {err}"
            }
        } else if let Some(uri) = image_data() {
            img {
                class: "{css_class}",
                src: "{uri}",
                alt: "{alt}",
            }
        }
    }
}
