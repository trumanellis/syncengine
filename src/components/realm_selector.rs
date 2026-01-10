//! Realm selector sidebar component.
//!
//! Displays available realms and provides interface for creating new ones.
//! Uses sacred terminology: "manifest" instead of "create", "release" instead of "cancel".

use dioxus::prelude::*;
use syncengine_core::{RealmId, RealmInfo};

/// Props for the RealmSelector component.
#[derive(Props, Clone, PartialEq)]
pub struct RealmSelectorProps {
    /// List of available realms
    pub realms: Vec<RealmInfo>,
    /// Currently selected realm ID (if any)
    pub selected: Option<RealmId>,
    /// Handler called when a realm is selected
    pub on_select: EventHandler<RealmId>,
    /// Handler called when a new realm should be created
    pub on_create: EventHandler<String>,
}

/// Realm selector sidebar showing all available realms.
///
/// # Example
///
/// ```ignore
/// RealmSelector {
///     realms: realms(),
///     selected: selected_realm(),
///     on_select: move |id| selected_realm.set(Some(id)),
///     on_create: move |name| { /* create realm */ },
/// }
/// ```
#[component]
pub fn RealmSelector(props: RealmSelectorProps) -> Element {
    // State for new realm input form
    let mut show_input = use_signal(|| false);
    let mut new_realm_name = use_signal(String::new);

    // Copy on_create for use in multiple closures (Callback implements Copy)
    let on_create_for_submit = props.on_create;
    let on_create_for_keydown = props.on_create;

    // Handler for key events on input
    let on_keydown = move |evt: KeyboardEvent| match evt.key() {
        Key::Enter => {
            let name = new_realm_name.read().trim().to_string();
            if !name.is_empty() {
                on_create_for_keydown.call(name);
                new_realm_name.set(String::new());
                show_input.set(false);
            }
        }
        Key::Escape => {
            show_input.set(false);
            new_realm_name.set(String::new());
        }
        _ => {}
    };

    rsx! {
        aside { class: "realm-sidebar",
            // Section header in gold
            h2 { class: "section-header", "Realms" }

            // Realm list
            div { class: "realm-list",
                for realm in props.realms.iter() {
                    {
                        let realm_id = realm.id.clone();
                        let realm_id_for_check = realm.id.clone();
                        let is_selected = props.selected.as_ref() == Some(&realm_id_for_check);
                        let on_select = props.on_select; // Callback is Copy

                        rsx! {
                            RealmItem {
                                key: "{realm_id}",
                                name: realm.name.clone(),
                                is_shared: realm.is_shared,
                                is_selected: is_selected,
                                on_click: move |_| on_select.call(realm_id.clone()),
                            }
                        }
                    }
                }
            }

            // New realm input section
            if show_input() {
                NewRealmInput {
                    value: new_realm_name(),
                    on_input: move |val: String| new_realm_name.set(val),
                    on_submit: move |_| {
                        let name = new_realm_name.read().trim().to_string();
                        if !name.is_empty() {
                            on_create_for_submit.call(name);
                            new_realm_name.set(String::new());
                            show_input.set(false);
                        }
                    },
                    on_cancel: move |_| {
                        show_input.set(false);
                        new_realm_name.set(String::new());
                    },
                    on_keydown: on_keydown,
                }
            } else {
                button {
                    class: "btn-badge",
                    onclick: move |_| show_input.set(true),
                    "+ manifest realm"
                }
            }
        }
    }
}

/// Props for a single realm item in the list.
#[derive(Props, Clone, PartialEq)]
struct RealmItemProps {
    /// Realm name to display
    name: String,
    /// Whether this realm is shared with peers
    is_shared: bool,
    /// Whether this realm is currently selected
    is_selected: bool,
    /// Handler for click events
    on_click: EventHandler<()>,
}

/// A single realm item in the selector list.
#[component]
fn RealmItem(props: RealmItemProps) -> Element {
    let item_class = if props.is_selected {
        "realm-item selected"
    } else {
        "realm-item"
    };

    rsx! {
        button {
            class: "{item_class}",
            onclick: move |_| props.on_click.call(()),
            span { class: "realm-name", "{props.name}" }
            if props.is_shared {
                span { class: "realm-shared-badge", "shared" }
            }
        }
    }
}

/// Props for the new realm input form.
#[derive(Props, Clone, PartialEq)]
struct NewRealmInputProps {
    /// Current input value
    value: String,
    /// Handler for input changes
    on_input: EventHandler<String>,
    /// Handler for form submission (manifest button)
    on_submit: EventHandler<()>,
    /// Handler for cancellation (release button)
    on_cancel: EventHandler<()>,
    /// Handler for keyboard events
    on_keydown: EventHandler<KeyboardEvent>,
}

/// Inline form for creating a new realm.
///
/// Uses sacred terminology:
/// - "manifest" instead of "create" or "submit"
/// - "release" instead of "cancel"
#[component]
fn NewRealmInput(props: NewRealmInputProps) -> Element {
    rsx! {
        div { class: "new-realm-input",
            input {
                class: "input-field",
                placeholder: "realm name...",
                value: "{props.value}",
                oninput: move |e| props.on_input.call(e.value()),
                onkeydown: move |e| props.on_keydown.call(e),
                autofocus: true
            }
            div { class: "new-realm-actions",
                button {
                    class: "btn-small",
                    onclick: move |_| props.on_submit.call(()),
                    "manifest"
                }
                button {
                    class: "btn-small btn-cancel",
                    onclick: move |_| props.on_cancel.call(()),
                    "release"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_realm_item_class_when_selected() {
        // Test that selected state produces correct class
        let is_selected = true;
        let item_class = if is_selected {
            "realm-item selected"
        } else {
            "realm-item"
        };
        assert_eq!(item_class, "realm-item selected");
    }

    #[test]
    fn test_realm_item_class_when_not_selected() {
        let is_selected = false;
        let item_class = if is_selected {
            "realm-item selected"
        } else {
            "realm-item"
        };
        assert_eq!(item_class, "realm-item");
    }
}
