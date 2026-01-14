//! Quest Card Component
//!
//! Golden ratio landscape card for rich task/quest display.

use dioxus::prelude::*;
use syncengine_core::Task;

use super::{CardGallery, CardHeader, CardOrientation, GoldenCard, MarkdownRenderer};
use super::card_gallery::GalleryItem;
use crate::components::images::AsyncImage;

/// Quest card with image, description, and peer gallery
///
/// # Examples
///
/// ```rust
/// rsx! {
///     QuestCard {
///         quest: task,
///         expanded: false,
///         on_expand: move |expanded| {
///             // Handle expand/collapse
///         },
///     }
/// }
/// ```
#[component]
pub fn QuestCard(
    /// Task/quest data
    quest: Task,
    /// Optional click handler for the entire card
    #[props(default = None)]
    on_click: Option<EventHandler<String>>,
) -> Element {
    let handle_card_click = move |_| {
        if let Some(handler) = &on_click {
            handler.call(quest.id.to_string());
        }
    };

    rsx! {
        GoldenCard {
            orientation: CardOrientation::Landscape,
            interactive: on_click.is_some(),

            // Top: Quest image area (38.2%)
            div {
                class: "card-image-area",
                onclick: handle_card_click,

                if let Some(blob_id) = &quest.image_blob_id {
                    AsyncImage {
                        blob_id: blob_id.clone(),
                        alt: quest.title.clone(),
                        class: Some("card-image__quest".to_string()),
                    }
                } else {
                    // Default quest image
                    img {
                        class: "card-image__default card-image__quest",
                        src: asset!("/assets/quest-default.png"),
                        alt: "Quest",
                    }
                }

                // Completion badge overlay
                if quest.completed {
                    div { class: "card-image__badge card-image__badge--completed",
                        "✓ Completed"
                    }
                }
            }

            // Bottom: Content area (61.8%)
            div { class: "card-content",
                // Header: Title, subtitle, link
                CardHeader {
                    title: quest.title.clone(),
                    subtitle: quest.subtitle.clone(),
                    link: quest.quest_link.as_ref().map(|l| format!("quest/{}", l)),
                }

                // Gallery: Involved peers
                if !quest.involved_peers.is_empty() {
                    CardGallery {
                        title: "Peers".to_string(),
                        items: quest.involved_peers.iter().map(|peer_id| {
                            GalleryItem {
                                id: peer_id.clone(),
                                image_url: None, // TODO: Load peer avatar
                                label: None,
                            }
                        }).collect::<Vec<_>>(),
                    }
                }

                // Description (markdown, always expanded)
                div { class: "card-markdown-section",
                    if !quest.description.is_empty() {
                        {
                            let desc_signal = use_memo(move || quest.description.clone());
                            rsx! {
                                MarkdownRenderer {
                                    content: desc_signal,
                                    collapsible: false,
                                    collapsed: false,
                                }
                            }
                        }
                    } else {
                        div { class: "card-empty-state",
                            "No description..."
                        }
                    }
                }

                // Metadata footer
                div { class: "card-footer",
                    if let Some(category) = &quest.category {
                        span { class: "card-category",
                            "{category}"
                        }
                    }
                    if let Some(creator) = &quest.created_by {
                        span { class: "card-creator",
                            "Created by: {creator}"
                        }
                    }
                }
            }
        }
    }
}
