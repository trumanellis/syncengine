//! Realm View - Direct link to a specific realm
//!
//! This page handles `/realms/:id` routes, allowing direct linking to realms.
//! It redirects to the Field page with the realm pre-selected.

use dioxus::prelude::*;

use crate::app::Route;

/// Direct realm view component.
///
/// Handles `/realms/:id` routes by redirecting to the Field page.
/// In the future, this could pre-select the realm in the Field view.
#[component]
pub fn RealmView(id: String) -> Element {
    let navigator = use_navigator();

    // For now, redirect to the field page
    // A future enhancement would pass the realm_id to pre-select it
    use_effect(move || {
        // Navigate to field page
        // TODO: Pass realm_id as state to pre-select the realm
        navigator.push(Route::Field {});
    });

    rsx! {
        div { class: "loading-state",
            p { class: "loading-message", "synchronicities are forming..." }
        }
    }
}
