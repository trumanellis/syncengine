//! Field Status Indicator Component
//!
//! Displays the connection status using sacred terminology:
//! - "field listening" - actively waiting for connections
//! - "field resonating" - connected and syncing
//! - "field dormant" - offline/inactive

use dioxus::prelude::*;

/// Represents the state of the synchronization field
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FieldState {
    /// Actively listening for connections (connecting state)
    Listening,
    /// Connected and actively syncing with peers
    Resonating,
    /// Offline or inactive
    #[default]
    Dormant,
}

impl FieldState {
    /// Returns the display label for this state
    pub fn label(&self) -> &'static str {
        match self {
            FieldState::Listening => "field listening",
            FieldState::Resonating => "field resonating",
            FieldState::Dormant => "field dormant",
        }
    }

    /// Returns whether this state represents an active connection
    pub fn is_active(&self) -> bool {
        matches!(self, FieldState::Listening | FieldState::Resonating)
    }
}

/// Properties for the FieldStatus component
#[derive(Clone, PartialEq, Props)]
pub struct FieldStatusProps {
    /// The current field state to display
    pub status: FieldState,
}

/// Displays the field connection status with a pulsing indicator
///
/// # Example
///
/// ```rust,ignore
/// rsx! {
///     FieldStatus { status: FieldState::Resonating }
/// }
/// ```
#[component]
pub fn FieldStatus(props: FieldStatusProps) -> Element {
    let label = props.status.label();
    let is_active = props.status.is_active();

    rsx! {
        div { class: "field-status",
            span {
                class: if is_active { "status-dot active" } else { "status-dot" },
                // ARIA for screen readers
                role: "img",
                "aria-label": if is_active { "Active" } else { "Inactive" },
            }
            span { class: "status-label", "{label}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_state_labels() {
        assert_eq!(FieldState::Listening.label(), "field listening");
        assert_eq!(FieldState::Resonating.label(), "field resonating");
        assert_eq!(FieldState::Dormant.label(), "field dormant");
    }

    #[test]
    fn field_state_active() {
        assert!(FieldState::Listening.is_active());
        assert!(FieldState::Resonating.is_active());
        assert!(!FieldState::Dormant.is_active());
    }

    #[test]
    fn field_state_default_is_dormant() {
        assert_eq!(FieldState::default(), FieldState::Dormant);
    }
}
