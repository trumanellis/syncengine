//! Button Components
//!
//! Various button styles following the design system:
//! - Primary: Main actions with moss green border
//! - Badge: Small tag-like buttons
//! - Enter: Special "Enter the Field" button
//! - Sacred: Gold-accented for important actions

use dioxus::prelude::*;

/// Button style variants
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ButtonVariant {
    /// Primary action button - moss green border, hover glow
    #[default]
    Primary,
    /// Small badge/tag style button
    Badge,
    /// "Enter the Field" style - larger, more prominent
    Enter,
    /// Sacred/important action - gold accents
    Sacred,
    /// Subtle/secondary action
    Ghost,
}

impl ButtonVariant {
    /// Returns the CSS class for this variant
    pub fn class(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "btn-primary",
            ButtonVariant::Badge => "btn-badge",
            ButtonVariant::Enter => "btn-enter",
            ButtonVariant::Sacred => "btn-sacred",
            ButtonVariant::Ghost => "btn-ghost",
        }
    }
}

/// Properties for the Button component
#[derive(Clone, PartialEq, Props)]
pub struct ButtonProps {
    /// Visual style variant
    #[props(default)]
    pub variant: ButtonVariant,
    /// Button content (text, icons, etc.)
    pub children: Element,
    /// Click handler
    #[props(default)]
    pub onclick: Option<EventHandler<()>>,
    /// Whether the button is disabled
    #[props(default = false)]
    pub disabled: bool,
    /// Optional type attribute (button, submit, reset)
    #[props(default = "button".to_string())]
    pub button_type: String,
    /// Optional additional CSS classes
    #[props(default)]
    pub class: Option<String>,
}

/// Styled button component following the design system
///
/// # Design Notes
///
/// - Transparent backgrounds with moss green borders
/// - Glow effect on hover using box-shadow
/// - Subtle lift animation on hover
/// - Uses JetBrains Mono font
///
/// # Example
///
/// ```rust,ignore
/// rsx! {
///     Button {
///         variant: ButtonVariant::Primary,
///         onclick: move |_| do_something(),
///         "Manifest"
///     }
///
///     Button {
///         variant: ButtonVariant::Enter,
///         onclick: move |_| navigate_to_field(),
///         "Enter the Field"
///     }
/// }
/// ```
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let base_class = props.variant.class();
    let extra_class = props.class.as_deref().unwrap_or("");
    let full_class = if extra_class.is_empty() {
        base_class.to_string()
    } else {
        format!("{} {}", base_class, extra_class)
    };

    rsx! {
        button {
            class: "{full_class}",
            r#type: "{props.button_type}",
            disabled: props.disabled,
            onclick: move |_| {
                if let Some(handler) = &props.onclick {
                    handler.call(());
                }
            },
            {props.children}
        }
    }
}

/// Icon button for compact actions (close, expand, etc.)
#[derive(Clone, PartialEq, Props)]
pub struct IconButtonProps {
    /// The icon content (character or element)
    pub children: Element,
    /// Click handler
    pub onclick: EventHandler<()>,
    /// Accessible label for screen readers
    pub aria_label: String,
    /// Optional additional CSS classes
    #[props(default)]
    pub class: Option<String>,
}

#[component]
pub fn IconButton(props: IconButtonProps) -> Element {
    let extra_class = props.class.as_deref().unwrap_or("");
    let full_class = if extra_class.is_empty() {
        "icon-btn".to_string()
    } else {
        format!("icon-btn {}", extra_class)
    };

    rsx! {
        button {
            class: "{full_class}",
            "aria-label": "{props.aria_label}",
            onclick: move |_| props.onclick.call(()),
            {props.children}
        }
    }
}

/// Close button with X icon
#[component]
pub fn CloseButton(onclick: EventHandler<()>) -> Element {
    rsx! {
        IconButton {
            onclick: onclick,
            aria_label: "Close".to_string(),
            class: "close-btn".to_string(),
            "\u{00D7}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_variant_classes() {
        assert_eq!(ButtonVariant::Primary.class(), "btn-primary");
        assert_eq!(ButtonVariant::Badge.class(), "btn-badge");
        assert_eq!(ButtonVariant::Enter.class(), "btn-enter");
        assert_eq!(ButtonVariant::Sacred.class(), "btn-sacred");
        assert_eq!(ButtonVariant::Ghost.class(), "btn-ghost");
    }

    #[test]
    fn button_variant_default() {
        assert_eq!(ButtonVariant::default(), ButtonVariant::Primary);
    }
}
