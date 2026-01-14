//! Unified Field View - Realms as Section Headers with Nested Tasks
//!
//! This component provides a streamlined single-view interface where realms
//! appear as expandable section headers, with their tasks listed directly below.
//!
//! ## Design Principles
//! - Realms are section titles (gold, serif, italic)
//! - Tasks nest visually under their realm
//! - Each realm section is independently expandable
//! - Maintains cyber-mystical terminal aesthetic

use dioxus::prelude::*;
use syncengine_core::{RealmId, RealmInfo, Task, TaskId};
use crate::components::cards::QuestCard;
use crate::components::{IntentionCreator, IntentionData};

/// Props for the UnifiedFieldView component
#[derive(Props, Clone, PartialEq)]
pub struct UnifiedFieldViewProps {
    /// All available realms
    pub realms: Vec<RealmInfo>,
    /// Tasks organized by realm_id
    pub tasks_by_realm: std::collections::HashMap<RealmId, Vec<Task>>,
    /// Generation counter to force re-renders when HashMap changes
    #[props(default = 0)]
    pub generation: usize,
    /// Handler for adding a task to a specific realm
    pub on_add_task: EventHandler<(RealmId, IntentionData)>,
    /// Handler for toggling a task in a specific realm
    pub on_toggle_task: EventHandler<(RealmId, TaskId)>,
    /// Handler for deleting a task from a specific realm
    pub on_delete_task: EventHandler<(RealmId, TaskId)>,
    /// Handler for creating a new realm
    pub on_create_realm: EventHandler<String>,
    /// Handler for showing invite panel for a realm
    pub on_show_invite: EventHandler<RealmId>,
}

/// Main unified field view component
///
/// Displays all realms as collapsible sections with their tasks nested below.
/// No separate sidebar - everything in one scrollable view.
#[component]
pub fn UnifiedFieldView(props: UnifiedFieldViewProps) -> Element {
    // Debug logging
    tracing::info!(
        "UnifiedFieldView rendering - realms: {}, tasks_by_realm entries: {}, generation: {}",
        props.realms.len(),
        props.tasks_by_realm.len(),
        props.generation
    );

    // Create realm form state
    let mut show_create_form = use_signal(|| false);
    let mut new_realm_name = use_signal(String::new);

    let on_create_submit = move |_| {
        let name = new_realm_name.read().trim().to_string();
        if !name.is_empty() {
            props.on_create_realm.call(name);
            new_realm_name.set(String::new());
            show_create_form.set(false);
        }
    };

    let on_create_keydown = move |evt: KeyboardEvent| match evt.key() {
        Key::Enter => {
            let name = new_realm_name.read().trim().to_string();
            if !name.is_empty() {
                props.on_create_realm.call(name);
                new_realm_name.set(String::new());
                show_create_form.set(false);
            }
        }
        Key::Escape => {
            show_create_form.set(false);
            new_realm_name.set(String::new());
        }
        _ => {}
    };

    rsx! {
        div { class: "unified-field-view",
            // Realm sections
            div { class: "realm-sections",
                // Render each realm as a section
                for realm in props.realms.iter() {
                    {
                        let realm_id = realm.id.clone();
                        let realm_id_add = realm_id.clone();
                        let realm_id_toggle = realm_id.clone();
                        let realm_id_delete = realm_id.clone();
                        let tasks = props.tasks_by_realm.get(&realm_id).cloned().unwrap_or_default();

                        tracing::info!("Rendering realm {} with {} tasks", realm.name, tasks.len());

                        rsx! {
                            RealmSection {
                                key: "{realm_id}",
                                realm: realm.clone(),
                                tasks: tasks,
                                on_add_task: move |title| props.on_add_task.call((realm_id_add.clone(), title)),
                                on_toggle_task: move |task_id| props.on_toggle_task.call((realm_id_toggle.clone(), task_id)),
                                on_delete_task: move |task_id| props.on_delete_task.call((realm_id_delete.clone(), task_id)),
                                on_show_invite: move |id| props.on_show_invite.call(id),
                            }
                        }
                    }
                }

                // Empty state when no realms
                if props.realms.is_empty() {
                    div { class: "empty-realms-state",
                        p { class: "body-text",
                            "No "
                            span { class: "sacred-term", "realms" }
                            " yet. Manifest your first realm below."
                        }
                    }
                }
            }

            // Create realm section at bottom
            div { class: "create-realm-section",
                if show_create_form() {
                    div { class: "new-realm-form",
                        input {
                            class: "input-field",
                            placeholder: "realm name...",
                            value: "{new_realm_name}",
                            oninput: move |e| new_realm_name.set(e.value()),
                            onkeydown: on_create_keydown,
                            autofocus: true
                        }
                        div { class: "form-actions",
                            button {
                                class: "btn-small",
                                onclick: on_create_submit,
                                "manifest"
                            }
                            button {
                                class: "btn-small btn-cancel",
                                onclick: move |_| {
                                    show_create_form.set(false);
                                    new_realm_name.set(String::new());
                                },
                                "release"
                            }
                        }
                    }
                } else {
                    button {
                        class: "btn-badge create-realm-btn",
                        onclick: move |_| show_create_form.set(true),
                        "+ manifest new realm"
                    }
                }
            }
        }
    }
}

/// Props for a single realm section
#[derive(Props, Clone, PartialEq)]
struct RealmSectionProps {
    /// The realm to display
    realm: RealmInfo,
    /// Tasks belonging to this realm
    tasks: Vec<Task>,
    /// Handler for adding a task
    on_add_task: EventHandler<IntentionData>,
    /// Handler for toggling a task
    on_toggle_task: EventHandler<TaskId>,
    /// Handler for deleting a task
    on_delete_task: EventHandler<TaskId>,
    /// Handler for showing invite panel for this realm
    on_show_invite: EventHandler<RealmId>,
}

/// A single realm section with header and nested tasks
///
/// The realm name appears as a gold, italic, serif section header.
/// Tasks are listed below with a left border indicating hierarchy.
/// Each section has its own manifest input.
#[component]
fn RealmSection(props: RealmSectionProps) -> Element {
    // Expansion state for this realm section
    let mut expanded = use_signal(|| true);

    // Intention creator visibility
    let mut show_creator = use_signal(|| false);

    let on_create_intention = move |data: IntentionData| {
        props.on_add_task.call(data);
        show_creator.set(false);
    };

    let on_cancel_creator = move |_| {
        show_creator.set(false);
    };

    let expand_icon = if expanded() { "▼" } else { "▶" };
    let task_count = props.tasks.len();
    let completed_count = props.tasks.iter().filter(|t| t.completed).count();

    rsx! {
        section { class: "realm-section",
            // Realm header (clickable to expand/collapse)
            button {
                class: "realm-header",
                onclick: move |_| expanded.toggle(),
                span { class: "expand-icon", "{expand_icon}" }
                h2 { class: "realm-title section-header",
                    "{props.realm.name}"
                }
                // Realm metadata badges
                div { class: "realm-meta",
                    if props.realm.is_shared {
                        span { class: "realm-badge shared-badge", "shared" }
                    }
                    span { class: "realm-badge count-badge",
                        "{completed_count}/{task_count}"
                    }
                    // Invite button (only for non-Private realms)
                    if !props.realm.name.eq_ignore_ascii_case("Private") {
                        button {
                            class: "realm-invite-btn",
                            onclick: move |e| {
                                e.stop_propagation(); // Prevent realm collapse/expand
                                props.on_show_invite.call(props.realm.id.clone());
                            },
                            title: "Summon others to this realm",
                            "aria-label": "Show invite for this realm",
                            "+"
                        }
                    }
                }
            }

            // Expanded content: tasks + creator button
            if expanded() {
                div { class: "realm-content",
                    // Button to show intention creator
                    div { class: "realm-manifest-input",
                        button {
                            class: "btn-manifest-new",
                            onclick: move |_| show_creator.set(true),
                            span { class: "btn-icon", "+" }
                            " manifest new intention"
                        }
                    }

                    // Intention Creator Modal
                    IntentionCreator {
                        visible: show_creator(),
                        initial_text: String::new(),
                        on_create: on_create_intention,
                        on_cancel: on_cancel_creator,
                    }

                    // Quest card grid
                    div { class: "realm-quest-grid",
                        if props.tasks.is_empty() {
                            p { class: "empty-task-state",
                                "No intentions yet in this realm."
                            }
                        } else {
                            for task in props.tasks.iter() {
                                {
                                    let task_id_key = task.id.to_string();
                                    let task_id_for_toggle = task.id.clone();
                                    let task_id_for_delete = task.id.clone();
                                    let task_for_card = task.clone();

                                    rsx! {
                                        div {
                                            key: "{task_id_key}",
                                            class: "quest-card-wrapper",

                                            QuestCard {
                                                quest: task_for_card,
                                                on_click: move |_| {
                                                    // Toggle task on card click
                                                    props.on_toggle_task.call(task_id_for_toggle.clone());
                                                }
                                            }

                                            // Delete button overlay
                                            button {
                                                class: "quest-card-delete",
                                                onclick: move |e| {
                                                    e.stop_propagation();
                                                    props.on_delete_task.call(task_id_for_delete.clone());
                                                },
                                                title: "dissolve intention",
                                                "aria-label": "Dissolve intention",
                                                "\u{00D7}" // ×
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
    }
}

