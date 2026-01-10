//! Field status indicator component.
//!
//! Displays connection state using sacred terminology:
//! - "field listening" (connecting)
//! - "field resonating" (connected)
//! - "field dormant" (offline)

use dioxus::prelude::*;

/// Connection state of the synchronization field.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldState {
    /// Actively listening for peers (connecting)
    Listening,
    /// Connected and synchronizing with peers
    Resonating,
    /// Offline / not connected
    #[default]
    Dormant,
}

impl FieldState {
    /// Get the display label for this state.
    pub fn label(&self) -> &'static str {
        match self {
            FieldState::Listening => "field listening",
            FieldState::Resonating => "field resonating",
            FieldState::Dormant => "field dormant",
        }
    }

    /// Check if this state represents an active connection.
    pub fn is_active(&self) -> bool {
        matches!(self, FieldState::Listening | FieldState::Resonating)
    }
}

/// Status indicator showing the synchronization field state.
#[component]
pub fn FieldStatus(status: FieldState) -> Element {
    let dot_class = if status.is_active() {
        "status-dot active"
    } else {
        "status-dot"
    };

    rsx! {
        div { class: "field-status",
            span { class: "{dot_class}" }
            span { class: "status-label", "{status.label()}" }
        }
    }
}
