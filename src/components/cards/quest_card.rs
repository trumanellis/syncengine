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
    /// Whether description is expanded
    #[props(default = false)]
    expanded: bool,
    /// Callback when expand state changes
    #[props(default = None)]
    on_expand: Option<EventHandler<bool>>,
    /// Optional click handler for the entire card
    #[props(default = None)]
    on_click: Option<EventHandler<String>>,
) -> Element {
    let toggle_expand = move |_| {
        if let Some(handler) = &on_expand {
            handler.call(!expanded);
        }
    };

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
                    div { class: "card-image__default card-image__quest",
                        // Default quest placeholder
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            view_box: "0 0 100 100",
                            class: "default-quest-icon",
                            // Mountain/quest icon
                            path {
                                d: "M 10 80 L 30 40 L 50 60 L 70 20 L 90 80 Z",
                                fill: "currentColor",
                                opacity: "0.3",
                            }
                            circle {
                                cx: "70",
                                cy: "20",
                                r: "5",
                                fill: "#d4af37",
                            }
                        }
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

                // Description (markdown, collapsible)
                div { class: "card-markdown-section",
                    if !quest.description.is_empty() {
                        {
                            let desc_signal = use_memo(move || quest.description.clone());
                            rsx! {
                                MarkdownRenderer {
                                    content: desc_signal,
                                    collapsible: true,
                                    collapsed: !expanded,
                                }

                                // Expand/collapse button
                                button {
                                    class: "expand-toggle",
                                    onclick: toggle_expand,
                                    if expanded {
                                        "Collapse"
                                    } else {
                                        "Expand"
                                    }
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
