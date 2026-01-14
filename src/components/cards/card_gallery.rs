//! Card Gallery Component
//!
//! Displays a grid of thumbnails (quests, peers, etc.)

use dioxus::prelude::*;

/// Gallery item data
#[derive(Clone, PartialEq)]
pub struct GalleryItem {
    pub id: String,
    pub image_url: Option<String>,
    pub label: Option<String>,
}

/// Thumbnail grid gallery component
///
/// # Examples
///
/// ```rust
/// let items = vec![
///     GalleryItem {
///         id: "quest1".to_string(),
///         image_url: Some("data:...".to_string()),
///         label: Some("Build App".to_string()),
///     },
/// ];
///
/// rsx! {
///     CardGallery {
///         title: "Top Quests",
///         items: items,
///         on_click: move |id| { /* Navigate to quest */ },
///     }
/// }
/// ```
#[component]
pub fn CardGallery(
    /// Gallery section title
    title: String,
    /// Items to display
    items: Vec<GalleryItem>,
    /// Click handler (receives item ID)
    #[props(default)]
    on_click: Option<EventHandler<String>>,
) -> Element {
    if items.is_empty() {
        return VNode::empty();
    }

    rsx! {
        div { class: "card-gallery-section",
            h3 { class: "card-gallery__title",
                "{title}"
            }

            div { class: "card-gallery",
                for item in items.iter() {
                    {
                        let item_id = item.id.clone();
                        rsx! {
                            div {
                                key: "{item.id}",
                                class: "card-gallery__item",
                                onclick: move |_| {
                                    if let Some(handler) = &on_click {
                                        handler.call(item_id.clone());
                                    }
                                },

                        // Image or placeholder
                        if let Some(url) = &item.image_url {
                            img {
                                src: "{url}",
                                alt: item.label.clone().unwrap_or_else(|| "Gallery item".to_string()),
                                class: "card-gallery__img",
                            }
                        } else {
                            div { class: "card-gallery__placeholder",
                                "?"
                            }
                        }

                                // Optional label overlay
                                if let Some(label) = &item.label {
                                    div { class: "card-gallery__label",
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
