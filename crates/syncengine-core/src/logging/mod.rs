//! Multi-instance logging system with JSONL storage.
//!
//! This module provides a race-condition-free logging system for multiple
//! concurrent instances of Synchronicity Engine. Each instance writes to
//! its own JSONL file, and reports can be generated on demand.
//!
//! ## Architecture
//!
//! ```text
//! logs/
//! ├── raw/                              # Machine-readable (one file per instance per day)
//! │   ├── 2026-01-21_love.jsonl
//! │   ├── 2026-01-21_joy.jsonl
//! │   └── 2026-01-21_peace.jsonl
//! ├── sessions/                         # Session metadata
//! │   └── 2026-01-21T14-13-48.json
//! └── LOGS.md                           # Generated on demand (regenerable)
//! ```
//!
//! ## Usage
//!
//! ### Setting up logging for an instance
//!
//! ```ignore
//! use syncengine_core::logging::{JsonlLayer, LoggingBuilder};
//! use tracing_subscriber::prelude::*;
//!
//! // Create the JSONL layer
//! let jsonl_layer = JsonlLayer::new("./logs", "love")?;
//!
//! // Combine with other layers (e.g., console output)
//! let subscriber = tracing_subscriber::registry()
//!     .with(jsonl_layer)
//!     .with(tracing_subscriber::fmt::layer());
//!
//! tracing::subscriber::set_global_default(subscriber)?;
//! ```
//!
//! ### Generating reports
//!
//! ```ignore
//! use syncengine_core::logging::report::{generate_report, ReportOptions};
//!
//! let options = ReportOptions::default();
//! let markdown = generate_report("./logs", &options)?;
//! std::fs::write("LOGS.md", markdown)?;
//! ```
//!
//! ### Querying logs with jq
//!
//! ```bash
//! # Find all errors
//! jq 'select(.level == "error")' logs/raw/*.jsonl
//!
//! # Get logs for specific instance
//! jq 'select(.instance == "love")' logs/raw/*.jsonl
//!
//! # Timeline (sorted)
//! cat logs/raw/*.jsonl | jq -s 'sort_by(.ts)'
//! ```

pub mod entry;
pub mod layer;
pub mod report;
pub mod writer;

// Re-exports for convenience
pub use entry::{JsonLogEntry, SessionMetadata};
pub use layer::{JsonlLayer, LoggingBuilder};
pub use report::{generate_report, generate_timeline, write_report, LogStats, ReportOptions};
pub use writer::{read_all_entries, read_entries_for_date, write_session_metadata, InstanceLogWriter};
