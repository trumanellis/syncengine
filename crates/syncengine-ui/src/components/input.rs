//! Input Field Components
//!
//! Text inputs and textareas following the design system.
//! Features:
//! - Transparent background with subtle border
//! - Cyan text color for entered values
//! - Cyan glow on focus
//! - Italic placeholder text

use dioxus::prelude::*;

/// Properties for the Input component
#[derive(Clone, PartialEq, Props)]
pub struct InputProps {
    /// Current input value
    pub value: String,
    /// Handler called when input changes
    pub oninput: EventHandler<String>,
    /// Placeholder text (displayed in muted italic)
    #[props(default)]
    pub placeholder: Option<String>,
    /// Input label text
    #[props(default)]
    pub label: Option<String>,
    /// Hint text below the label (e.g., "(optional)")
    #[props(default)]
    pub hint: Option<String>,
    /// Input type (text, email, password, etc.)
    #[props(default = "text".to_string())]
    pub input_type: String,
    /// Whether the input is required
    #[props(default = false)]
    pub required: bool,
    /// Whether the input is disabled
    #[props(default = false)]
    pub disabled: bool,
    /// Optional ID for label association
    #[props(default)]
    pub id: Option<String>,
    /// Optional additional CSS classes
    #[props(default)]
    pub class: Option<String>,
}

/// Text input field following the design system
///
/// # Design Notes
///
/// - Transparent background
/// - Subtle void border (--void-border: #1a1a1a)
/// - Cyan text for entered values
/// - Cyan glow on focus
/// - Muted italic placeholder
///
/// # Example
///
/// ```rust,ignore
/// let mut title = use_signal(String::new);
///
/// rsx! {
///     Input {
///         value: title(),
///         oninput: move |s| title.set(s),
///         label: "title".to_string(),
///         placeholder: "Enter the intention...".to_string()
///     }
/// }
/// ```
#[component]
pub fn Input(props: InputProps) -> Element {
    let id = props
        .id
        .clone()
        .unwrap_or_else(|| format!("input-{}", rand_id()));
    let extra_class = props.class.as_deref().unwrap_or("");
    let input_class = if extra_class.is_empty() {
        "input-field".to_string()
    } else {
        format!("input-field {}", extra_class)
    };

    rsx! {
        div { class: "form-field",
            if let Some(label) = &props.label {
                label {
                    class: "input-label",
                    r#for: "{id}",
                    "{label}"
                    if let Some(hint) = &props.hint {
                        span { class: "input-hint", " ({hint})" }
                    }
                }
            }
            input {
                id: "{id}",
                class: "{input_class}",
                r#type: "{props.input_type}",
                value: "{props.value}",
                placeholder: props.placeholder.as_deref().unwrap_or(""),
                required: props.required,
                disabled: props.disabled,
                oninput: move |e| props.oninput.call(e.value()),
            }
        }
    }
}

/// Properties for the TextArea component
#[derive(Clone, PartialEq, Props)]
pub struct TextAreaProps {
    /// Current textarea value
    pub value: String,
    /// Handler called when textarea changes
    pub oninput: EventHandler<String>,
    /// Placeholder text
    #[props(default)]
    pub placeholder: Option<String>,
    /// Textarea label
    #[props(default)]
    pub label: Option<String>,
    /// Hint text below the label
    #[props(default)]
    pub hint: Option<String>,
    /// Number of visible rows
    #[props(default = 4)]
    pub rows: u32,
    /// Whether the textarea is required
    #[props(default = false)]
    pub required: bool,
    /// Whether the textarea is disabled
    #[props(default = false)]
    pub disabled: bool,
    /// Optional ID for label association
    #[props(default)]
    pub id: Option<String>,
}

/// Multi-line text input following the design system
///
/// # Example
///
/// ```rust,ignore
/// let mut description = use_signal(String::new);
///
/// rsx! {
///     TextArea {
///         value: description(),
///         oninput: move |s| description.set(s),
///         label: "description".to_string(),
///         hint: "markdown supported".to_string(),
///         placeholder: "add details, context, or notes...".to_string(),
///         rows: 5
///     }
/// }
/// ```
#[component]
pub fn TextArea(props: TextAreaProps) -> Element {
    let id = props
        .id
        .clone()
        .unwrap_or_else(|| format!("textarea-{}", rand_id()));

    rsx! {
        div { class: "form-field",
            if let Some(label) = &props.label {
                label {
                    class: "input-label",
                    r#for: "{id}",
                    "{label}"
                    if let Some(hint) = &props.hint {
                        span { class: "input-hint", " ({hint})" }
                    }
                }
            }
            textarea {
                id: "{id}",
                class: "input-field textarea",
                rows: "{props.rows}",
                placeholder: props.placeholder.as_deref().unwrap_or(""),
                required: props.required,
                disabled: props.disabled,
                value: "{props.value}",
                oninput: move |e| props.oninput.call(e.value()),
            }
        }
    }
}

/// Generate a simple random ID for form elements
fn rand_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    (duration.as_nanos() % 1_000_000) as u32
}

/// Search input with icon
#[derive(Clone, PartialEq, Props)]
pub struct SearchInputProps {
    /// Current search value
    pub value: String,
    /// Handler called when search changes
    pub oninput: EventHandler<String>,
    /// Placeholder text
    #[props(default = "search the field...".to_string())]
    pub placeholder: String,
}

#[component]
pub fn SearchInput(props: SearchInputProps) -> Element {
    rsx! {
        div { class: "search-input-wrapper",
            span { class: "search-icon", "\u{1F50D}" }
            input {
                class: "input-field search-input",
                r#type: "search",
                placeholder: "{props.placeholder}",
                value: "{props.value}",
                oninput: move |e| props.oninput.call(e.value()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rand_id_generates_number() {
        let id1 = rand_id();
        let id2 = rand_id();
        // IDs should be reasonable numbers
        assert!(id1 < 1_000_000);
        assert!(id2 < 1_000_000);
    }
}
