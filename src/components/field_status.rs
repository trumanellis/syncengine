//! Network Resonance Indicator
//!
//! A beautiful, informative status indicator that shows the true state
//! of peer-to-peer synchronization using sacred terminology.
//!
//! ## States
//!
//! | State | Sacred Term | Meaning |
//! |-------|-------------|---------|
//! | Idle | "field dormant" | Not connected, not syncing |
//! | Connecting | "seeking resonance..." | Establishing peer connections |
//! | Syncing(0) | "field listening" | Connected but no peers yet |
//! | Syncing(1+) | "field resonating • N souls" | Actively syncing with peers |
//! | Error | "dissonance" | Connection error |

use dioxus::prelude::*;
use syncengine_core::SyncStatus;

/// Legacy field state enum for backwards compatibility.
/// New code should use NetworkState with actual SyncStatus.
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

/// Network resonance state with full sync information.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct NetworkState {
    /// Current sync status from the engine
    pub status: SyncStatus,
    /// Whether data was recently synced (for activity indicator)
    pub recently_active: bool,
}

impl NetworkState {
    /// Create from a SyncStatus
    pub fn from_status(status: SyncStatus) -> Self {
        Self {
            status,
            recently_active: false,
        }
    }

    /// Get the sacred label for this state
    pub fn label(&self) -> String {
        match &self.status {
            SyncStatus::Idle => "field dormant".to_string(),
            SyncStatus::Connecting => "seeking resonance".to_string(),
            SyncStatus::Syncing { peer_count: 0 } => "field listening".to_string(),
            SyncStatus::Syncing { peer_count } => {
                let souls = if *peer_count == 1 { "soul" } else { "souls" };
                format!("field resonating · {} {}", peer_count, souls)
            }
            SyncStatus::Error(_) => "dissonance".to_string(),
        }
    }

    /// Get CSS class for the status dot
    pub fn dot_class(&self) -> &'static str {
        match &self.status {
            SyncStatus::Idle => "resonance-dot dormant",
            SyncStatus::Connecting => "resonance-dot seeking",
            SyncStatus::Syncing { peer_count: 0 } => "resonance-dot listening",
            SyncStatus::Syncing { .. } => "resonance-dot resonating",
            SyncStatus::Error(_) => "resonance-dot dissonance",
        }
    }

    /// Get CSS class for the label
    pub fn label_class(&self) -> &'static str {
        match &self.status {
            SyncStatus::Idle => "resonance-label dormant",
            SyncStatus::Connecting => "resonance-label seeking",
            SyncStatus::Syncing { peer_count: 0 } => "resonance-label listening",
            SyncStatus::Syncing { .. } => "resonance-label resonating",
            SyncStatus::Error(_) => "resonance-label dissonance",
        }
    }

    /// Check if actively syncing with peers
    pub fn is_resonating(&self) -> bool {
        matches!(&self.status, SyncStatus::Syncing { peer_count } if *peer_count > 0)
    }

    /// Get peer count if syncing
    pub fn peer_count(&self) -> Option<usize> {
        match &self.status {
            SyncStatus::Syncing { peer_count } => Some(*peer_count),
            _ => None,
        }
    }
}

/// Legacy status indicator (for backwards compatibility)
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

/// Modern Network Resonance Indicator
///
/// Shows the true state of peer-to-peer synchronization with:
/// - Animated status orb with state-specific colors
/// - Sacred terminology describing connection state
/// - Peer count when actively syncing
/// - Activity pulse when data flows
#[component]
pub fn NetworkResonance(state: NetworkState) -> Element {
    let dot_class = state.dot_class();
    let label_class = state.label_class();
    let label = state.label();
    let peer_count = state.peer_count();

    rsx! {
        div {
            class: "network-resonance",
            "aria-live": "polite",
            "aria-label": "Network status: {label}",

            // Resonance orb with concentric rings for peer visualization
            div { class: "resonance-orb",
                // Outer glow ring (visible when resonating)
                if state.is_resonating() {
                    div { class: "resonance-ring outer" }
                }

                // Middle ring (visible with 2+ peers)
                if peer_count.unwrap_or(0) >= 2 {
                    div { class: "resonance-ring middle" }
                }

                // Core status dot
                span { class: "{dot_class}" }
            }

            // Sacred status label
            span { class: "{label_class}", "{label}" }
        }
    }
}

/// Compact version for header use
#[component]
pub fn NetworkResonanceCompact(state: NetworkState) -> Element {
    let dot_class = state.dot_class();
    let label = state.label();

    rsx! {
        div {
            class: "network-resonance compact",
            title: "{label}",

            div { class: "resonance-orb compact",
                if state.is_resonating() {
                    div { class: "resonance-ring outer compact" }
                }
                span { class: "{dot_class}" }
            }

            span { class: "resonance-label compact", "{label}" }
        }
    }
}
