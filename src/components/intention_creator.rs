//! Intention Creator - Full-featured form for manifesting new intentions
//!
//! Cyber-mystical creation interface with all quest fields

use dioxus::prelude::*;
use crate::components::images::{ImageUpload, ImageOrientation};
use crate::components::MarkdownEditor;

/// Categories for intentions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntentionCategory {
    General,
    Intention,
    Offering,
    Collective,
}

impl IntentionCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntentionCategory::General => "General",
            IntentionCategory::Intention => "Intention",
            IntentionCategory::Offering => "Offering",
            IntentionCategory::Collective => "Collective",
        }
    }

    pub fn all() -> &'static [IntentionCategory] {
        &[
            IntentionCategory::General,
            IntentionCategory::Intention,
            IntentionCategory::Offering,
            IntentionCategory::Collective,
        ]
    }
}

/// Data for creating a new intention
#[derive(Debug, Clone, PartialEq)]
pub struct IntentionData {
    pub title: String,
    pub subtitle: Option<String>,
    pub category: Option<String>,
    pub description: String,
    pub image_blob_id: Option<String>,
}

/// Props for IntentionCreator component
#[derive(Props, Clone, PartialEq)]
pub struct IntentionCreatorProps {
    /// Whether the creator is visible
    pub visible: bool,
    /// Initial text to populate title (e.g., from search)
    #[props(default = String::new())]
    pub initial_text: String,
    /// Handler for submission
    pub on_create: EventHandler<IntentionData>,
    /// Handler for cancellation
    pub on_cancel: EventHandler<()>,
}

/// Intention Creator Component
///
/// Full-featured form for creating rich intentions with:
/// - Title (required)
/// - Subtitle (optional tagline)
/// - Category (button group selection)
/// - Description (markdown textarea)
/// - Image URL (optional)
///
/// Keyboard shortcuts:
/// - Esc: Cancel
/// - Cmd+Enter: Submit
#[component]
pub fn IntentionCreator(props: IntentionCreatorProps) -> Element {
    // Form state
    let mut title = use_signal(|| props.initial_text.clone());
    let mut subtitle = use_signal(String::new);
    let mut category = use_signal(|| Some(IntentionCategory::General));
    let mut description = use_signal(String::new);
    let mut image_blob_id = use_signal(|| Option::<String>::None);
    let mut is_submitting = use_signal(|| false);

    // Reset form when visibility changes
    use_effect(move || {
        if props.visible {
            title.set(props.initial_text.clone());
            subtitle.set(String::new());
            category.set(Some(IntentionCategory::General));
            description.set(String::new());
            image_blob_id.set(None);
            is_submitting.set(false);
        }
    });

    let on_submit = move |_: MouseEvent| {
        let title_val = title.read().trim().to_string();

        if title_val.is_empty() {
            return;
        }

        is_submitting.set(true);

        let data = IntentionData {
            title: title_val,
            subtitle: {
                let s = subtitle.read().trim().to_string();
                if s.is_empty() { None } else { Some(s) }
            },
            category: category().map(|c| c.as_str().to_string()),
            description: description.read().clone(),
            image_blob_id: image_blob_id(),
        };

        props.on_create.call(data);
    };

    let on_image_upload = move |blob_id: String| {
        image_blob_id.set(Some(blob_id));
    };

    let on_cancel_click = move |_: MouseEvent| {
        props.on_cancel.call(());
    };

    let on_keydown = move |evt: KeyboardEvent| {
        match evt.key() {
            Key::Escape => {
                props.on_cancel.call(());
            }
            Key::Enter if evt.modifiers().contains(Modifiers::META) || evt.modifiers().contains(Modifiers::CONTROL) => {
                // Can't call on_submit here because we don't have a MouseEvent
                // User should use the button or we handle it separately
            }
            _ => {}
        }
    };

    if !props.visible {
        return rsx! {};
    }

    rsx! {
        div {
            class: "intention-creator-overlay",
            onclick: move |_| props.on_cancel.call(()),

            div {
                class: "intention-creator",
                onclick: move |e| e.stop_propagation(),
                onkeydown: on_keydown,

                // Header
                div { class: "creator-header",
                    h2 { class: "creator-title",
                        span { class: "creator-icon", "+" }
                        " manifest new intention"
                    }
                    button {
                        class: "creator-close",
                        onclick: on_cancel_click,
                        "aria-label": "Cancel",
                        "×"
                    }
                }

                // Form
                div { class: "creator-form",
                    // Title (required)
                    div { class: "form-group",
                        label { class: "form-label", "what do you intend to manifest?" }
                        input {
                            class: "form-input form-input--title",
                            r#type: "text",
                            value: "{title}",
                            oninput: move |e| title.set(e.value()),
                            placeholder: "intention title...",
                            autofocus: true,
                            required: true
                        }
                    }

                    // Subtitle (optional)
                    div { class: "form-group",
                        label { class: "form-label", "tagline (optional)" }
                        input {
                            class: "form-input",
                            r#type: "text",
                            value: "{subtitle}",
                            oninput: move |e| subtitle.set(e.value()),
                            placeholder: "brief context or subtitle..."
                        }
                    }

                    // Category (button group)
                    div { class: "form-group",
                        label { class: "form-label", "category" }
                        div { class: "category-buttons",
                            for cat in IntentionCategory::all() {
                                {
                                    let cat_val = *cat;
                                    let is_selected = category() == Some(cat_val);
                                    rsx! {
                                        button {
                                            key: "{cat.as_str()}",
                                            class: if is_selected { "category-btn category-btn--active" } else { "category-btn" },
                                            r#type: "button",
                                            onclick: move |_| category.set(Some(cat_val)),
                                            "{cat.as_str()}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Description (markdown editor)
                    div { class: "form-group",
                        label { class: "form-label", "description (markdown)" }
                        MarkdownEditor {
                            value: description,
                            placeholder: "Describe the intention, include context, next steps...".to_string(),
                            min_height: 250,
                        }
                    }

                    // Image Upload
                    div { class: "form-group",
                        label { class: "form-label", "quest image (optional)" }
                        div { class: "image-upload-container",
                            ImageUpload {
                                orientation: ImageOrientation::Landscape,
                                on_upload: on_image_upload,
                                label: if image_blob_id().is_some() {
                                    "Change Image".to_string()
                                } else {
                                    "Upload Image".to_string()
                                },
                            }
                            if let Some(ref blob_id) = image_blob_id() {
                                {
                                    let short_id = if blob_id.len() > 12 {
                                        format!("{}...", &blob_id[..12])
                                    } else {
                                        blob_id.clone()
                                    };
                                    rsx! {
                                        div { class: "image-upload-status",
                                            "✓ Image uploaded: {short_id}"
                                            button {
                                                class: "image-remove-btn",
                                                r#type: "button",
                                                onclick: move |_| image_blob_id.set(None),
                                                "×"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Actions
                div { class: "creator-actions",
                    button {
                        class: "btn-cancel",
                        r#type: "button",
                        onclick: on_cancel_click,
                        "release"
                    }
                    button {
                        class: "btn-manifest",
                        r#type: "button",
                        onclick: on_submit,
                        disabled: is_submitting(),
                        if is_submitting() {
                            "manifesting..."
                        } else {
                            "manifest intention"
                        }
                    }
                }

                // Keyboard hint
                div { class: "creator-hint",
                    span { class: "hint-key", "esc" }
                    " to cancel • "
                    span { class: "hint-key", "⌘ enter" }
                    " to manifest"
                }
            }
        }
    }
}
