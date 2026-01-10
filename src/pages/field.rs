//! The Field - Main application view.
//!
//! Where intentions manifest and synchronicities form.

use std::path::PathBuf;

use dioxus::prelude::*;
use syncengine_core::{RealmId, RealmInfo, SyncEngine, Task};

use crate::components::{FieldState, FieldStatus, RealmSelector, TaskList};

/// Get the data directory for the application.
fn get_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("syncengine")
}

/// Main application view component.
#[component]
pub fn Field() -> Element {
    // Engine state (initialized on mount)
    let mut engine: Signal<Option<SyncEngine>> = use_signal(|| None);
    let mut realms: Signal<Vec<RealmInfo>> = use_signal(Vec::new);
    let mut selected_realm: Signal<Option<RealmId>> = use_signal(|| None);
    let mut tasks: Signal<Vec<Task>> = use_signal(Vec::new);
    let mut loading: Signal<bool> = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut field_state: Signal<FieldState> = use_signal(|| FieldState::Listening);

    // Initialize engine on mount
    use_effect(move || {
        spawn(async move {
            let data_dir = get_data_dir();
            match SyncEngine::new(&data_dir).await {
                Ok(eng) => {
                    let realm_list = eng.list_realms().await.unwrap_or_default();
                    realms.set(realm_list);
                    engine.set(Some(eng));
                    loading.set(false);
                    field_state.set(FieldState::Resonating);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to initialize engine: {}", e)));
                    loading.set(false);
                    field_state.set(FieldState::Dormant);
                }
            }
        });
    });

    // Load tasks when selected realm changes
    use_effect(move || {
        let selected = selected_realm();
        if let Some(realm_id) = selected {
            if let Some(ref eng) = *engine.read() {
                match eng.list_tasks(&realm_id) {
                    Ok(task_list) => tasks.set(task_list),
                    Err(_) => tasks.set(vec![]),
                }
            }
        } else {
            tasks.set(vec![]);
        }
    });

    // Handler for selecting a realm
    let select_realm = move |realm_id: RealmId| {
        spawn(async move {
            // Open the realm if needed
            if let Some(ref mut eng) = *engine.write() {
                let _ = eng.open_realm(&realm_id).await;
                match eng.list_tasks(&realm_id) {
                    Ok(task_list) => {
                        tasks.set(task_list);
                        selected_realm.set(Some(realm_id));
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to load tasks: {}", e)));
                    }
                }
            }
        });
    };

    // Handler for creating a new realm (receives name from RealmSelector component)
    let create_realm = move |name: String| {
        if name.trim().is_empty() {
            return;
        }

        spawn(async move {
            if let Some(ref mut eng) = *engine.write() {
                match eng.create_realm(&name).await {
                    Ok(realm_id) => {
                        // Refresh realm list
                        let realm_list = eng.list_realms().await.unwrap_or_default();
                        realms.set(realm_list);
                        // Select the new realm
                        selected_realm.set(Some(realm_id));
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to create realm: {}", e)));
                    }
                }
            }
        });
    };

    // Handler for adding a new task (receives title from ManifestInput)
    let add_task = move |title: String| {
        if title.trim().is_empty() {
            return;
        }

        let realm_id = match selected_realm() {
            Some(id) => id,
            None => return,
        };

        spawn(async move {
            if let Some(ref mut eng) = *engine.write() {
                match eng.add_task(&realm_id, &title).await {
                    Ok(_) => {
                        // Refresh tasks
                        match eng.list_tasks(&realm_id) {
                            Ok(task_list) => {
                                tasks.set(task_list);
                            }
                            Err(e) => {
                                error.set(Some(format!("Failed to refresh tasks: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to add task: {}", e)));
                    }
                }
            }
        });
    };

    // Handler for toggling a task
    let toggle_task = move |task_id: syncengine_core::TaskId| {
        let realm_id = match selected_realm() {
            Some(id) => id,
            None => return,
        };

        spawn(async move {
            if let Some(ref mut eng) = *engine.write() {
                match eng.toggle_task(&realm_id, &task_id).await {
                    Ok(_) => {
                        // Refresh tasks
                        if let Ok(task_list) = eng.list_tasks(&realm_id) {
                            tasks.set(task_list);
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to toggle task: {}", e)));
                    }
                }
            }
        });
    };

    // Handler for deleting a task
    let delete_task = move |task_id: syncengine_core::TaskId| {
        let realm_id = match selected_realm() {
            Some(id) => id,
            None => return,
        };

        spawn(async move {
            if let Some(ref mut eng) = *engine.write() {
                match eng.delete_task(&realm_id, &task_id).await {
                    Ok(_) => {
                        // Refresh tasks
                        if let Ok(task_list) = eng.list_tasks(&realm_id) {
                            tasks.set(task_list);
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to dissolve intention: {}", e)));
                    }
                }
            }
        });
    };

    // Render
    rsx! {
        div { class: "app-shell",
            // Header
            header { class: "app-header",
                h1 { class: "app-title", "The Field" }
                FieldStatus { status: field_state() }
            }

            // Error display
            if let Some(err) = error() {
                div { class: "error-banner",
                    span { "{err}" }
                    button {
                        class: "error-dismiss",
                        onclick: move |_| error.set(None),
                        "dismiss"
                    }
                }
            }

            // Loading state
            if loading() {
                div { class: "loading-state",
                    p { class: "loading-message", "synchronicities are forming..." }
                }
            }

            // Main content
            else {
                div { class: "field-content",
                    // Realm selector sidebar component
                    RealmSelector {
                        realms: realms(),
                        selected: selected_realm(),
                        on_select: select_realm,
                        on_create: create_realm,
                    }

                    // Task list
                    main { class: "task-area",
                        if selected_realm().is_some() {
                            TaskList {
                                tasks: tasks(),
                                on_toggle: move |id| toggle_task(id),
                                on_delete: move |id| delete_task(id),
                                on_add: move |title| add_task(title),
                            }
                        } else {
                            div { class: "no-realm-selected",
                                p { class: "body-text",
                                    "Select a "
                                    span { class: "sacred-term", "realm" }
                                    " to view intentions, or manifest a new one."
                                }
                            }
                        }
                    }
                }
            }

            // Footer
            footer { class: "app-footer",
                span { class: "app-footer-message", "synchronicities are forming" }
            }
        }
    }
}
