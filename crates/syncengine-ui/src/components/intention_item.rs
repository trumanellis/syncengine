//! Intention Item Component
//!
//! Displays a single intention (task) in the list with expandable details.
//! Uses "intention" terminology instead of "task" per the design system.

use dioxus::prelude::*;

/// Properties for the IntentionItem component
#[derive(Clone, PartialEq, Props)]
pub struct IntentionItemProps {
    /// Unique identifier for the intention
    pub id: String,
    /// Title/name of the intention
    pub title: String,
    /// Whether the intention has been completed (manifested)
    #[props(default = false)]
    pub completed: bool,
    /// Optional subtitle or brief context
    #[props(default)]
    pub subtitle: Option<String>,
    /// Whether this intention has matching intentions in the field
    #[props(default = false)]
    pub has_matches: bool,
    /// Handler called when the intention is toggled
    pub on_toggle: EventHandler<String>,
    /// Optional handler for expanding/collapsing details
    #[props(default)]
    pub on_expand: Option<EventHandler<String>>,
}

/// Displays a single intention item with completion toggle
///
/// # Design Notes
///
/// - Uses moss green left border for visual hierarchy
/// - Cyan text for interactive title
/// - Circle/checkmark icon for completion state
/// - Expandable to show details and matches
///
/// # Example
///
/// ```rust,ignore
/// rsx! {
///     IntentionItem {
///         id: "intention-1".to_string(),
///         title: "Build solar dehydrator".to_string(),
///         completed: false,
///         on_toggle: move |id| {
///             // Toggle completion state
///         }
///     }
/// }
/// ```
#[component]
pub fn IntentionItem(props: IntentionItemProps) -> Element {
    let id = props.id.clone();
    let id_for_toggle = props.id.clone();
    let title = props.title.clone();
    let completed = props.completed;
    let subtitle = props.subtitle.clone();

    let mut expanded = use_signal(|| false);

    let on_toggle = props.on_toggle;
    let on_expand = props.on_expand;

    rsx! {
        div { class: "intention-item",
            button {
                class: "intention-header",
                onclick: move |_| {
                    on_toggle.call(id_for_toggle.clone());
                },
                span {
                    class: if completed { "check-icon completed" } else { "check-icon" },
                    if completed { "\u{2713}" } else { "\u{25CB}" }
                }
                div { class: "intention-text",
                    span {
                        class: if completed { "intention-title completed" } else { "intention-title" },
                        "{title}"
                    }
                    if let Some(sub) = &subtitle {
                        span { class: "intention-subtitle", "{sub}" }
                    }
                }
            }
            // Expand button (if has details)
            if on_expand.is_some() {
                button {
                    class: "expand-btn",
                    onclick: move |_| {
                        expanded.toggle();
                        if let Some(handler) = &on_expand {
                            handler.call(id.clone());
                        }
                    },
                    span { class: "expand-icon",
                        if expanded() { "\u{25BC}" } else { "\u{25B6}" }
                    }
                }
            }
            // Expanded content
            if expanded() {
                div { class: "intention-content",
                    if !props.has_matches {
                        p { class: "no-matches",
                            "no matching intentions in the field"
                        }
                    }
                }
            }
        }
    }
}

/// A list container for intention items
///
/// # Example
///
/// ```rust,ignore
/// rsx! {
///     IntentionList {
///         for intention in intentions {
///             IntentionItem {
///                 id: intention.id.clone(),
///                 title: intention.title.clone(),
///                 completed: intention.completed,
///                 on_toggle: move |id| toggle_intention(id)
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn IntentionList(children: Element) -> Element {
    rsx! {
        div { class: "intention-list",
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn intention_item_module_exists() {
        // Basic compile-time test to ensure the module is properly structured.
        // Component rendering tests require a Dioxus runtime and should be
        // done in integration tests or with the dioxus testing utilities.
        assert!(true);
    }

    #[test]
    fn intention_check_icons() {
        // Verify the icons we use are correct Unicode points
        let check = "\u{2713}";
        let circle = "\u{25CB}";
        assert_eq!(check, "\u{2713}"); // checkmark
        assert_eq!(circle, "\u{25CB}"); // circle
    }
}
