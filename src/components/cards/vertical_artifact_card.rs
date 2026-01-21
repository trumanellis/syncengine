//! Vertical Artifact Card
//!
//! Portrait card with 1:1.618 golden ratio aspect ratio.
//! Image fills background, title and content overlay on top.

use dioxus::prelude::*;

use super::MarkdownRenderer;
use crate::components::images::AsyncImage;

// Embed default artifact image as base64 data URI
const ARTIFACT_DEFAULT_BYTES: &[u8] = include_bytes!("../../../assets/quest-default.png");

fn artifact_default_uri() -> String {
    use base64::Engine;
    let base64 = base64::engine::general_purpose::STANDARD.encode(ARTIFACT_DEFAULT_BYTES);
    format!("data:image/png;base64,{}", base64)
}

/// Vertical artifact card with background image and overlay content
///
/// # Examples
///
/// ```rust
/// rsx! {
///     VerticalArtifactCard {
///         title: "My Artifact".to_string(),
///         description: "Some markdown content".to_string(),
///         image_blob_id: None,
///         on_click: move |id| { /* handle click */ },
///     }
/// }
/// ```
#[component]
pub fn VerticalArtifactCard(
    /// Unique identifier
    #[props(default = String::new())]
    id: String,
    /// Card title
    title: String,
    /// Optional subtitle
    #[props(default = None)]
    subtitle: Option<String>,
    /// Markdown description
    #[props(default = String::new())]
    description: String,
    /// Optional image blob ID
    #[props(default = None)]
    image_blob_id: Option<String>,
    /// Whether the artifact is completed
    #[props(default = false)]
    completed: bool,
    /// Peer IDs involved in this artifact
    #[props(default = Vec::new())]
    involved_peers: Vec<String>,
    /// Optional click handler
    #[props(default = None)]
    on_click: Option<EventHandler<String>>,
) -> Element {
    let card_id = id.clone();
    let handle_click = move |_| {
        if let Some(handler) = &on_click {
            handler.call(card_id.clone());
        }
    };

    let interactive_class = if on_click.is_some() { "interactive" } else { "" };
    let completed_class = if completed { "artifact-card--completed" } else { "" };

    rsx! {
        div {
            class: "vertical-artifact-card {interactive_class} {completed_class}",
            onclick: handle_click,

            // Background image layer
            div { class: "artifact-card__background",
                if let Some(blob_id) = &image_blob_id {
                    AsyncImage {
                        blob_id: blob_id.clone(),
                        alt: title.clone(),
                        class: Some("artifact-card__image".to_string()),
                    }
                } else {
                    img {
                        class: "artifact-card__image",
                        src: "{artifact_default_uri()}",
                        alt: "Artifact",
                    }
                }
            }

            // Content overlay
            div { class: "artifact-card__overlay",
                // Title section at top
                div { class: "artifact-card__header",
                    h3 { class: "artifact-card__title",
                        "{title}"
                    }
                    if let Some(sub) = &subtitle {
                        span { class: "artifact-card__subtitle",
                            "{sub}"
                        }
                    }
                }

                // Description (if present)
                if !description.is_empty() {
                    div { class: "artifact-card__content",
                        {
                            let desc_signal = use_memo(move || description.clone());
                            rsx! {
                                MarkdownRenderer {
                                    content: desc_signal,
                                    collapsible: false,
                                    collapsed: false,
                                }
                            }
                        }
                    }
                }

                // Mini peer avatars at bottom
                if !involved_peers.is_empty() {
                    div { class: "artifact-card__peers",
                        for (i, peer_id) in involved_peers.iter().take(5).enumerate() {
                            div {
                                key: "{peer_id}",
                                class: "artifact-card__peer-avatar",
                                style: "z-index: {5 - i};",
                                title: "{peer_id}",
                                // First letter of peer ID as placeholder
                                "{peer_id.chars().next().unwrap_or('?').to_uppercase()}"
                            }
                        }
                        if involved_peers.len() > 5 {
                            div { class: "artifact-card__peer-more",
                                "+{involved_peers.len() - 5}"
                            }
                        }
                    }
                }

                // Completion badge
                if completed {
                    div { class: "artifact-card__badge",
                        "✓"
                    }
                }
            }
        }
    }
}
