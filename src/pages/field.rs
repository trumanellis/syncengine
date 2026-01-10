//! The Field - Main application view.
//!
//! Where intentions manifest and synchronicities form.

use std::path::PathBuf;

use dioxus::prelude::*;
use syncengine_core::{RealmId, RealmInfo, SyncEngine, Task};

use crate::components::{FieldState, FieldStatus};

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
    let mut new_task_title: Signal<String> = use_signal(String::new);
    let mut new_realm_name: Signal<String> = use_signal(String::new);
    let mut loading: Signal<bool> = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut show_new_realm_input: Signal<bool> = use_signal(|| false);
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

    // Handler for creating a new realm
    let create_realm = move |_| {
        let name = new_realm_name.read().clone();
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
                        new_realm_name.set(String::new());
                        show_new_realm_input.set(false);
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

    // Handler for adding a new task
    let add_task = move |_| {
        let title = new_task_title.read().clone();
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
                                new_task_title.set(String::new());
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

    // Handler for key press on task input
    let on_task_keydown = move |evt: KeyboardEvent| {
        if evt.key() == Key::Enter {
            let title = new_task_title.read().clone();
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
                            if let Ok(task_list) = eng.list_tasks(&realm_id) {
                                tasks.set(task_list);
                                new_task_title.set(String::new());
                            }
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to add task: {}", e)));
                        }
                    }
                }
            });
        }
    };

    // Handler for key press on realm input
    let on_realm_keydown = move |evt: KeyboardEvent| {
        if evt.key() == Key::Enter {
            let name = new_realm_name.read().clone();
            if name.trim().is_empty() {
                return;
            }

            spawn(async move {
                if let Some(ref mut eng) = *engine.write() {
                    match eng.create_realm(&name).await {
                        Ok(realm_id) => {
                            let realm_list = eng.list_realms().await.unwrap_or_default();
                            realms.set(realm_list);
                            new_realm_name.set(String::new());
                            show_new_realm_input.set(false);
                            selected_realm.set(Some(realm_id));
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to create realm: {}", e)));
                        }
                    }
                }
            });
        } else if evt.key() == Key::Escape {
            show_new_realm_input.set(false);
            new_realm_name.set(String::new());
        }
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
                    // Realm selector sidebar
                    aside { class: "realm-sidebar",
                        h2 { class: "section-header", "Realms" }

                        div { class: "realm-list",
                            for realm in realms() {
                                {
                                    let realm_id = realm.id.clone();
                                    let realm_id_for_check = realm.id.clone();
                                    let is_selected = selected_realm() == Some(realm_id_for_check);
                                    rsx! {
                                        button {
                                            class: if is_selected { "realm-item selected" } else { "realm-item" },
                                            onclick: move |_| select_realm(realm_id.clone()),
                                            span { class: "realm-name", "{realm.name}" }
                                            if realm.is_shared {
                                                span { class: "realm-shared-badge", "shared" }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // New realm input
                        if show_new_realm_input() {
                            div { class: "new-realm-input",
                                input {
                                    class: "input-field",
                                    placeholder: "realm name...",
                                    value: "{new_realm_name}",
                                    oninput: move |e| new_realm_name.set(e.value()),
                                    onkeydown: on_realm_keydown,
                                    autofocus: true
                                }
                                div { class: "new-realm-actions",
                                    button {
                                        class: "btn-small",
                                        onclick: create_realm,
                                        "manifest"
                                    }
                                    button {
                                        class: "btn-small btn-cancel",
                                        onclick: move |_| {
                                            show_new_realm_input.set(false);
                                            new_realm_name.set(String::new());
                                        },
                                        "release"
                                    }
                                }
                            }
                        } else {
                            button {
                                class: "btn-badge",
                                onclick: move |_| show_new_realm_input.set(true),
                                "+ manifest realm"
                            }
                        }
                    }

                    // Task list
                    main { class: "task-area",
                        if selected_realm().is_some() {
                            // Task input
                            div { class: "manifest-input",
                                input {
                                    class: "input-field",
                                    placeholder: "manifest new intention...",
                                    value: "{new_task_title}",
                                    oninput: move |e| new_task_title.set(e.value()),
                                    onkeydown: on_task_keydown
                                }
                                button {
                                    class: "btn-primary",
                                    onclick: add_task,
                                    "manifest"
                                }
                            }

                            // Task list
                            div { class: "intention-list",
                                if tasks().is_empty() {
                                    p { class: "empty-state",
                                        "No intentions yet. Manifest your first intention above."
                                    }
                                } else {
                                    for task in tasks() {
                                        {
                                            let task_id = task.id.clone();
                                            let task_id_for_delete = task.id.clone();
                                            rsx! {
                                                div { class: "intention-item",
                                                    button {
                                                        class: "intention-toggle",
                                                        onclick: move |_| toggle_task(task_id.clone()),
                                                        span {
                                                            class: if task.completed { "check completed" } else { "check" },
                                                            if task.completed { "\u{2713}" } else { "\u{25CB}" }
                                                        }
                                                    }
                                                    span {
                                                        class: if task.completed { "intention-title completed" } else { "intention-title" },
                                                        "{task.title}"
                                                    }
                                                    button {
                                                        class: "intention-delete",
                                                        onclick: move |_| delete_task(task_id_for_delete.clone()),
                                                        title: "dissolve",
                                                        "\u{00D7}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
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
