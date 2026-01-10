//! Category Pills Component
//!
//! Horizontal selection of category filter pills.
//! Uses moss green borders with cyan text for selected state.

use dioxus::prelude::*;

/// Properties for the CategoryPills component
#[derive(Clone, PartialEq, Props)]
pub struct CategoryPillsProps {
    /// List of available categories
    pub categories: Vec<String>,
    /// Currently selected category
    pub selected: String,
    /// Handler called when a category is selected
    pub on_select: EventHandler<String>,
}

/// Displays a horizontal row of selectable category pills
///
/// # Design Notes
///
/// - Pills have moss green borders
/// - Selected pill has filled background
/// - Hover state shows brighter border
/// - Uses monospace font for terminal aesthetic
///
/// # Example
///
/// ```rust,ignore
/// let mut selected = use_signal(|| "general".to_string());
///
/// rsx! {
///     CategoryPills {
///         categories: vec![
///             "general".to_string(),
///             "intention".to_string(),
///             "offering".to_string(),
///             "collective".to_string(),
///         ],
///         selected: selected(),
///         on_select: move |cat| selected.set(cat)
///     }
/// }
/// ```
#[component]
pub fn CategoryPills(props: CategoryPillsProps) -> Element {
    let selected = props.selected.clone();

    rsx! {
        div {
            class: "category-pills",
            role: "radiogroup",
            "aria-label": "Category selection",
            for cat in props.categories.iter() {
                {
                    let cat_clone = cat.clone();
                    let is_selected = selected == *cat;
                    let on_select = props.on_select;
                    rsx! {
                        button {
                            class: if is_selected { "pill selected" } else { "pill" },
                            role: "radio",
                            "aria-checked": if is_selected { "true" } else { "false" },
                            onclick: move |_| {
                                on_select.call(cat_clone.clone());
                            },
                            "{cat}"
                        }
                    }
                }
            }
        }
    }
}

/// A single category pill (for custom layouts)
///
/// # Example
///
/// ```rust,ignore
/// rsx! {
///     CategoryPill {
///         label: "offering".to_string(),
///         selected: current == "offering",
///         on_click: move |_| set_category("offering")
///     }
/// }
/// ```
#[derive(Clone, PartialEq, Props)]
pub struct CategoryPillProps {
    /// The category label
    pub label: String,
    /// Whether this pill is selected
    #[props(default = false)]
    pub selected: bool,
    /// Handler called when clicked
    pub on_click: EventHandler<()>,
}

#[component]
pub fn CategoryPill(props: CategoryPillProps) -> Element {
    let is_selected = props.selected;

    rsx! {
        button {
            class: if is_selected { "pill selected" } else { "pill" },
            onclick: move |_| props.on_click.call(()),
            "{props.label}"
        }
    }
}

/// Default categories for the SyncEngine application
pub fn default_categories() -> Vec<String> {
    vec![
        "general".to_string(),
        "intention".to_string(),
        "offering".to_string(),
        "collective".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_categories_has_four() {
        let cats = default_categories();
        assert_eq!(cats.len(), 4);
        assert!(cats.contains(&"general".to_string()));
        assert!(cats.contains(&"intention".to_string()));
        assert!(cats.contains(&"offering".to_string()));
        assert!(cats.contains(&"collective".to_string()));
    }

    #[test]
    fn category_pills_module_exists() {
        // Basic compile-time test to ensure the module is properly structured.
        // Component rendering tests require a Dioxus runtime and should be
        // done in integration tests or with the dioxus testing utilities.
        assert!(true);
    }
}
