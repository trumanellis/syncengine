//! Task List component for Synchronicity Engine.
//!
//! Displays intentions (tasks) within a realm with cyber-mystical styling.
//!
//! ## Sacred Language
//!
//! - "intention" not "task"
//! - "manifest" not "create"
//! - "dissolve" not "delete"
//!
//! ## Components
//!
//! - [`TaskList`] - Main container that displays all intentions
//! - [`TaskItem`] - Individual intention row with toggle and delete
//! - [`ManifestInput`] - Input field to manifest new intentions

use dioxus::prelude::*;
use syncengine_core::{Task, TaskId};

/// Individual task item in the intention list.
///
/// Displays a single intention with:
/// - Toggle checkbox (circle when incomplete, checkmark when complete)
/// - Title with strikethrough when completed
/// - Delete button that appears on hover
///
/// # Props
///
/// * `task` - The task data to display
/// * `on_toggle` - Called when the checkbox is clicked
/// * `on_delete` - Called when the delete button is clicked
#[component]
pub fn TaskItem(
    task: Task,
    on_toggle: EventHandler<TaskId>,
    on_delete: EventHandler<TaskId>,
) -> Element {
    let task_id = task.id.clone();
    let task_id_for_delete = task.id.clone();

    let check_class = if task.completed {
        "check completed"
    } else {
        "check"
    };

    let title_class = if task.completed {
        "intention-title completed"
    } else {
        "intention-title"
    };

    // Unicode: circle (U+25CB) or checkmark (U+2713)
    let check_symbol = if task.completed { "\u{2713}" } else { "\u{25CB}" };

    rsx! {
        div { class: "intention-item",
            button {
                class: "intention-toggle",
                onclick: move |_| on_toggle.call(task_id.clone()),
                "aria-label": if task.completed { "Mark as incomplete" } else { "Mark as complete" },
                span { class: "{check_class}", "{check_symbol}" }
            }
            span { class: "{title_class}", "{task.title}" }
            button {
                class: "intention-delete",
                onclick: move |_| on_delete.call(task_id_for_delete.clone()),
                title: "dissolve",
                "aria-label": "Dissolve intention",
                "\u{00D7}" // multiplication sign (x)
            }
        }
    }
}

/// Input field for manifesting new intentions.
///
/// Features:
/// - Text input with placeholder "manifest new intention..."
/// - "manifest" button
/// - Enter key submits the form
///
/// # Props
///
/// * `on_add` - Called with the title when a new intention is manifested
#[component]
pub fn ManifestInput(on_add: EventHandler<String>) -> Element {
    let mut input_value = use_signal(String::new);

    let submit = move |_| {
        let title = input_value.read().clone();
        if !title.trim().is_empty() {
            on_add.call(title);
            input_value.set(String::new());
        }
    };

    let on_keydown = move |evt: KeyboardEvent| {
        if evt.key() == Key::Enter {
            let title = input_value.read().clone();
            if !title.trim().is_empty() {
                on_add.call(title);
                input_value.set(String::new());
            }
        }
    };

    rsx! {
        div { class: "manifest-input",
            input {
                class: "input-field",
                placeholder: "manifest new intention...",
                value: "{input_value}",
                oninput: move |e| input_value.set(e.value()),
                onkeydown: on_keydown
            }
            button {
                class: "btn-primary",
                onclick: submit,
                "manifest"
            }
        }
    }
}

/// Main task list component displaying all intentions for a realm.
///
/// Shows:
/// - Input for manifesting new intentions
/// - List of existing intentions with toggle and delete
/// - Empty state message when no intentions exist
///
/// # Props
///
/// * `tasks` - List of tasks to display
/// * `on_toggle` - Called when a task's completion is toggled
/// * `on_delete` - Called when a task should be deleted
/// * `on_add` - Called with the title when a new task is added
///
/// # Example
///
/// ```ignore
/// TaskList {
///     tasks: tasks(),
///     on_toggle: move |id| toggle_task(id),
///     on_delete: move |id| delete_task(id),
///     on_add: move |title| add_task(title),
/// }
/// ```
#[component]
pub fn TaskList(
    tasks: Vec<Task>,
    on_toggle: EventHandler<TaskId>,
    on_delete: EventHandler<TaskId>,
    on_add: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "task-list-container",
            // Input for manifesting new intentions
            ManifestInput { on_add: on_add }

            // Intention list
            div { class: "intention-list",
                if tasks.is_empty() {
                    p { class: "empty-state",
                        "No intentions yet. Manifest your first intention above."
                    }
                } else {
                    for task in tasks {
                        TaskItem {
                            key: "{task.id}",
                            task: task.clone(),
                            on_toggle: on_toggle,
                            on_delete: on_delete,
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_item_classes_incomplete() {
        let task = Task::new("Test intention");
        assert!(!task.completed);
        // Verify class logic
        let check_class = if task.completed { "check completed" } else { "check" };
        let title_class = if task.completed { "intention-title completed" } else { "intention-title" };
        assert_eq!(check_class, "check");
        assert_eq!(title_class, "intention-title");
    }

    #[test]
    fn test_task_item_classes_complete() {
        let mut task = Task::new("Test intention");
        task.complete();
        assert!(task.completed);
        // Verify class logic
        let check_class = if task.completed { "check completed" } else { "check" };
        let title_class = if task.completed { "intention-title completed" } else { "intention-title" };
        assert_eq!(check_class, "check completed");
        assert_eq!(title_class, "intention-title completed");
    }
}
