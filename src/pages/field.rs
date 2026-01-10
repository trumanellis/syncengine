//! The Field - Main application view.
//!
//! Where intentions manifest and synchronicities form.

use dioxus::prelude::*;
use syncengine_core::{RealmId, RealmInfo, Task};

use crate::components::{FieldState, FieldStatus, RealmSelector, TaskList};
use crate::context::{use_engine, use_engine_ready};

/// Main application view component.
#[component]
pub fn Field() -> Element {
    // Get shared engine from context (initialized in App)
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // Local UI state
    let mut realms: Signal<Vec<RealmInfo>> = use_signal(Vec::new);
    let mut selected_realm: Signal<Option<RealmId>> = use_signal(|| None);
    let mut tasks: Signal<Vec<Task>> = use_signal(Vec::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut field_state: Signal<FieldState> = use_signal(|| FieldState::Listening);

    // Load realms when engine becomes ready
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let guard = shared.read().await;
                if let Some(ref eng) = *guard {
                    match eng.list_realms().await {
                        Ok(realm_list) => {
                            realms.set(realm_list);
                            field_state.set(FieldState::Resonating);
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to load realms: {}", e)));
                            field_state.set(FieldState::Dormant);
                        }
                    }
                }
            });
        }
    });

    // Load tasks when selected realm changes
    use_effect(move || {
        let selected = selected_realm();
        if let Some(realm_id) = selected {
            if engine_ready() {
                spawn(async move {
                    let shared = engine();
                    let guard = shared.read().await;
                    if let Some(ref eng) = *guard {
                        match eng.list_tasks(&realm_id) {
                            Ok(task_list) => tasks.set(task_list),
                            Err(_) => tasks.set(vec![]),
                        }
                    }
                });
            }
        } else {
            tasks.set(vec![]);
        }
    });

    // Handler for selecting a realm
    let select_realm = move |realm_id: RealmId| {
        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;
            if let Some(ref mut eng) = *guard {
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

    // Handler for creating a new realm
    let create_realm = move |name: String| {
        if name.trim().is_empty() {
            return;
        }

        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;
            if let Some(ref mut eng) = *guard {
                match eng.create_realm(&name).await {
                    Ok(realm_id) => {
                        let realm_list = eng.list_realms().await.unwrap_or_default();
                        realms.set(realm_list);
                        selected_realm.set(Some(realm_id));
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to create realm: {}", e)));
                    }
                }
            }
        });
    };

    // Handler for adding a new task
    let add_task = move |title: String| {
        if title.trim().is_empty() {
            return;
        }

        let realm_id = match selected_realm() {
            Some(id) => id,
            None => return,
        };

        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;
            if let Some(ref mut eng) = *guard {
                match eng.add_task(&realm_id, &title).await {
                    Ok(_) => {
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
            let shared = engine();
            let mut guard = shared.write().await;
            if let Some(ref mut eng) = *guard {
                match eng.toggle_task(&realm_id, &task_id).await {
                    Ok(_) => {
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
            let shared = engine();
            let mut guard = shared.write().await;
            if let Some(ref mut eng) = *guard {
                match eng.delete_task(&realm_id, &task_id).await {
                    Ok(_) => {
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
            if !engine_ready() {
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
