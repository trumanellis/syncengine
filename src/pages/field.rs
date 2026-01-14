//! Synchronicity Engine - Main application view with unified realm-task layout.
//!
//! Where intentions manifest and synchronicities form.

use dioxus::prelude::*;
use std::collections::HashMap;
use syncengine_core::{NetworkDebugInfo, RealmId, RealmInfo, SyncEvent, Task};
use crate::components::IntentionData;

use crate::app::Route;
use crate::components::{
    InvitePanel, JoinRealmModal, NetworkResonance, NetworkState, UnifiedFieldView,
};
use crate::context::{use_engine, use_engine_ready};

/// Main application view component with unified realm-task interface.
#[component]
pub fn Field() -> Element {
    // Get shared engine from context (initialized in App)
    let engine = use_engine();
    let engine_ready = use_engine_ready();

    // Local UI state
    let mut realms: Signal<Vec<RealmInfo>> = use_signal(Vec::new);
    let mut tasks_by_realm: Signal<HashMap<RealmId, Vec<Task>>> =
        use_signal(HashMap::new);
    let mut generation: Signal<usize> = use_signal(|| 0); // Force re-renders on HashMap changes
    let mut data_loaded: Signal<bool> = use_signal(|| false); // Track if initial data loaded
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut network_state: Signal<NetworkState> = use_signal(NetworkState::default);
    let mut network_debug: Signal<Option<NetworkDebugInfo>> = use_signal(|| None);

    // Track currently opened realm for network status
    let mut opened_realm: Signal<Option<RealmId>> = use_signal(|| None);

    // Invite UI state
    let mut show_join_modal: Signal<bool> = use_signal(|| false);
    let mut show_invite_panel: Signal<bool> = use_signal(|| false);

    // Load all realms and their tasks when engine becomes ready
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let mut guard = shared.write().await;
                if let Some(ref mut eng) = *guard {
                    match eng.list_realms().await {
                        Ok(realm_list) => {
                            // Open ALL realms first (required for loading tasks)
                            for realm in &realm_list {
                                let _ = eng.open_realm(&realm.id).await;
                            }

                            // Load tasks for each realm (AFTER opening all realms)
                            let mut tasks_map = HashMap::new();
                            for realm in &realm_list {
                                match eng.list_tasks(&realm.id) {
                                    Ok(task_list) => {
                                        tracing::info!("Loaded {} tasks for realm {}", task_list.len(), realm.name);
                                        tasks_map.insert(realm.id.clone(), task_list);
                                    }
                                    Err(e) => {
                                        tracing::warn!("Failed to load tasks for realm {}: {:?}", realm.name, e);
                                    }
                                }
                            }
                            tracing::info!("Total realms loaded: {}, total task entries: {}", realm_list.len(), tasks_map.len());

                            // Set network status for first realm
                            if let Some(first_realm) = realm_list.first() {
                                let _ = eng.process_pending_sync();

                                // Update network status for first realm
                                let status = eng.sync_status(&first_realm.id);
                                network_state.set(NetworkState::from_status(status));
                                network_debug.set(Some(eng.network_debug_info(&first_realm.id)));
                                opened_realm.set(Some(first_realm.id.clone()));
                            }

                            // Log before setting signals
                            tracing::info!("Setting signals - realms: {}, tasks_by_realm entries: {}", realm_list.len(), tasks_map.len());

                            realms.set(realm_list);
                            tasks_by_realm.set(tasks_map);
                            // Increment generation to force re-render
                            let current = *generation.peek();
                            generation.set(current + 1);
                            // Mark data as loaded
                            data_loaded.set(true);

                            tracing::info!("Signals set - generation now: {}, data_loaded: true", current + 1);
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to load realms: {}", e)));
                        }
                    }
                }
            });
        }
    });

    // Listen for sync events and refresh tasks when changes arrive
    use_effect(move || {
        if engine_ready() {
            spawn(async move {
                let shared = engine();
                let guard = shared.read().await;
                if let Some(ref eng) = *guard {
                    let mut events = eng.subscribe_events();
                    drop(guard); // Release lock before waiting

                    loop {
                        match events.recv().await {
                            Ok(SyncEvent::RealmChanged { realm_id, .. }) => {
                                // Refresh tasks for this realm
                                let shared = engine();
                                let mut guard = shared.write().await;
                                if let Some(ref mut eng) = *guard {
                                    // Process any pending sync messages first
                                    let _ = eng.process_pending_sync();

                                    // Update tasks for this realm
                                    if let Ok(task_list) = eng.list_tasks(&realm_id) {
                                        let mut map = tasks_by_realm.read().clone();
                                        map.insert(realm_id, task_list);
                                        tasks_by_realm.set(map);
                                        let current = *generation.peek();
                                        generation.set(current + 1);
                                    }
                                }
                            }
                            Ok(SyncEvent::StatusChanged { realm_id, status }) => {
                                // Update network state if this is the opened realm
                                if opened_realm() == Some(realm_id) {
                                    network_state.set(NetworkState::from_status(status));
                                }
                            }
                            Ok(
                                SyncEvent::PeerConnected { realm_id, .. }
                                | SyncEvent::PeerDisconnected { realm_id, .. },
                            ) => {
                                // Refresh sync status and debug info for opened realm
                                if opened_realm() == Some(realm_id.clone()) {
                                    let shared = engine();
                                    let guard = shared.read().await;
                                    if let Some(ref eng) = *guard {
                                        let status = eng.sync_status(&realm_id);
                                        network_state.set(NetworkState::from_status(status));
                                        network_debug.set(Some(eng.network_debug_info(&realm_id)));
                                    }
                                }
                            }
                            Ok(_) => {}      // Ignore other events
                            Err(_) => break, // Channel closed
                        }
                    }
                }
            });
        }
    });

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
                        // Refresh realm list
                        let realm_list = eng.list_realms().await.unwrap_or_default();
                        realms.set(realm_list);

                        // Initialize empty task list for new realm
                        let mut map = tasks_by_realm.read().clone();
                        map.insert(realm_id.clone(), vec![]);
                        tasks_by_realm.set(map);
                        let current = *generation.peek();
                        generation.set(current + 1);

                        // Open the new realm
                        let _ = eng.open_realm(&realm_id).await;
                        opened_realm.set(Some(realm_id.clone()));

                        // Update network status
                        let status = eng.sync_status(&realm_id);
                        network_state.set(NetworkState::from_status(status));
                        network_debug.set(Some(eng.network_debug_info(&realm_id)));
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to create realm: {}", e)));
                    }
                }
            }
        });
    };

    // Handler for adding a task to a specific realm
    let add_task = move |(realm_id, data): (RealmId, IntentionData)| {
        if data.title.trim().is_empty() {
            return;
        }

        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;
            if let Some(ref mut eng) = *guard {
                // Use add_quest if there's rich metadata, otherwise use simple add_task
                let result = if data.subtitle.is_some() || !data.description.is_empty() || data.category.is_some() || data.image_blob_id.is_some() {
                    eng.add_quest(
                        &realm_id,
                        &data.title,
                        data.subtitle,
                        &data.description,
                        data.category,
                        data.image_blob_id,
                    ).await
                } else {
                    eng.add_task(&realm_id, &data.title).await
                };

                match result {
                    Ok(_) => {
                        // Refresh tasks for this realm
                        if let Ok(task_list) = eng.list_tasks(&realm_id) {
                            let mut map = tasks_by_realm.read().clone();
                            map.insert(realm_id, task_list);
                            tasks_by_realm.set(map);
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to add intention: {}", e)));
                    }
                }
            }
        });
    };

    // Handler for toggling a task
    let toggle_task = move |(realm_id, task_id): (RealmId, syncengine_core::TaskId)| {
        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;
            if let Some(ref mut eng) = *guard {
                match eng.toggle_task(&realm_id, &task_id).await {
                    Ok(_) => {
                        if let Ok(task_list) = eng.list_tasks(&realm_id) {
                            let mut map = tasks_by_realm.read().clone();
                            map.insert(realm_id, task_list);
                            tasks_by_realm.set(map);
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to toggle intention: {}", e)));
                    }
                }
            }
        });
    };

    // Handler for deleting a task
    let delete_task = move |(realm_id, task_id): (RealmId, syncengine_core::TaskId)| {
        spawn(async move {
            let shared = engine();
            let mut guard = shared.write().await;
            if let Some(ref mut eng) = *guard {
                match eng.delete_task(&realm_id, &task_id).await {
                    Ok(_) => {
                        if let Ok(task_list) = eng.list_tasks(&realm_id) {
                            let mut map = tasks_by_realm.read().clone();
                            map.insert(realm_id, task_list);
                            tasks_by_realm.set(map);
                        }
                    }
                    Err(e) => {
                        error.set(Some(format!("Failed to dissolve intention: {}", e)));
                    }
                }
            }
        });
    };

    // Handler for showing invite panel for a specific realm
    let show_invite_for_realm = move |realm_id: RealmId| {
        opened_realm.set(Some(realm_id));
        show_invite_panel.set(true);
    };

    // Handler called after JoinRealmModal successfully joins
    let on_realm_joined = move |_invite_string: String| {
        spawn(async move {
            let shared = engine();
            let guard = shared.read().await;
            if let Some(ref eng) = *guard {
                // Refresh realm list to show the newly joined realm
                let realm_list = eng.list_realms().await.unwrap_or_default();

                // Find the most recently added realm (highest created_at)
                let newest_realm = realm_list.iter().max_by_key(|r| r.created_at).cloned();

                // Load tasks for new realm
                let mut tasks_map = tasks_by_realm.read().clone();
                if let Some(ref realm) = newest_realm {
                    if let Ok(task_list) = eng.list_tasks(&realm.id) {
                        tasks_map.insert(realm.id.clone(), task_list);
                    }
                }

                realms.set(realm_list);
                tasks_by_realm.set(tasks_map);
                generation.set(generation() + 1);
                show_join_modal.set(false);
            }
        });
    };

    // Render
    rsx! {
        div { class: "app-shell",
            // Header
            header { class: "app-header",
                h1 { class: "app-title", "Synchronicity Engine" }

                // Header actions
                div { class: "header-actions",
                    button {
                        class: "header-btn join-btn",
                        onclick: move |_| show_join_modal.set(true),
                        "Join Realm"
                    }

                    // Profile navigation button
                    Link {
                        to: Route::Profile {},
                        button {
                            class: "profile-nav-button",
                            title: "Profile",
                            "👤"
                        }
                    }
                }

                NetworkResonance { state: network_state(), debug_info: network_debug() }
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
            if !engine_ready() || !data_loaded() {
                div { class: "loading-state",
                    p { class: "loading-message", "synchronicities are forming..." }
                }
            }

            // Main content - Unified Field View
            else {
                div { class: "field-content-unified",
                    // Main unified view
                    main { class: "unified-main",
                        {
                            let gen = generation(); // Read generation to subscribe to changes
                            let realms_list = realms();
                            let tasks_map = tasks_by_realm();

                            rsx! {
                                UnifiedFieldView {
                                    key: "{gen}",
                                    realms: realms_list,
                                    tasks_by_realm: tasks_map,
                                    generation: gen,
                                    on_add_task: add_task,
                                    on_toggle_task: toggle_task,
                                    on_delete_task: delete_task,
                                    on_create_realm: create_realm,
                                    on_show_invite: show_invite_for_realm,
                                }
                            }
                        }
                    }

                    // Invite panel (right sidebar, shown when toggled)
                    if show_invite_panel() {
                        if let Some(realm_id) = opened_realm() {
                            aside { class: "invite-sidebar",
                                InvitePanel {
                                    realm_id: realm_id,
                                    on_close: move |_| show_invite_panel.set(false),
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

            // Join Realm Modal (overlay)
            JoinRealmModal {
                show: show_join_modal(),
                on_close: move |_| show_join_modal.set(false),
                on_join: on_realm_joined,
            }
        }
    }
}
