//! Engine context provider for Synchronicity Engine.
//!
//! Provides the SyncEngine instance to all components via use_context.
//!
//! ## Usage
//!
//! ```ignore
//! // In App component
//! EngineProvider { }
//!
//! // In child components
//! let engine = use_engine();
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use dioxus::prelude::*;
use syncengine_core::SyncEngine;
use tokio::sync::RwLock;

/// Shared engine type for context.
///
/// The engine is wrapped in Arc<RwLock<>> to allow:
/// - Multiple components to read concurrently
/// - Safe mutation when needed
pub type SharedEngine = Arc<RwLock<Option<SyncEngine>>>;

/// Get the data directory for the application.
/// Uses the global data dir set from command line args.
pub fn get_data_dir() -> PathBuf {
    crate::get_data_dir()
}

/// Hook to access the SyncEngine from context.
///
/// Returns a Signal containing the shared engine state.
///
/// # Example
///
/// ```ignore
/// let engine = use_engine();
///
/// // Read engine state
/// if let Some(ref eng) = *engine.read().await {
///     let realms = eng.list_realms().await?;
/// }
/// ```
pub fn use_engine() -> Signal<SharedEngine> {
    use_context::<Signal<SharedEngine>>()
}

/// Hook to check if the engine is initialized.
///
/// Returns a reactive signal that updates when engine state changes.
pub fn use_engine_ready() -> Signal<bool> {
    use_context::<Signal<bool>>()
}
