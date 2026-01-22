//! Custom tracing Layer that writes to JSONL files.
//!
//! This layer integrates with the `tracing` crate to capture all log events
//! and write them to instance-specific JSONL files.

use std::fmt::Write as FmtWrite;
use std::sync::Arc;

use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Record};
use tracing::{Event, Id, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

use super::entry::JsonLogEntry;
use super::writer::InstanceLogWriter;

/// A tracing Layer that writes events to JSONL files.
///
/// Each instance of the application creates its own JsonlLayer with
/// its instance name, ensuring log files don't conflict.
pub struct JsonlLayer {
    writer: Arc<InstanceLogWriter>,
}

impl JsonlLayer {
    /// Create a new JSONL layer for an instance.
    ///
    /// # Arguments
    /// * `logs_dir` - Directory for log files (e.g., "./logs")
    /// * `instance` - Instance name (e.g., "love", "joy")
    pub fn new(
        logs_dir: impl AsRef<std::path::Path>,
        instance: impl Into<String>,
    ) -> std::io::Result<Self> {
        let writer = InstanceLogWriter::new(logs_dir, instance)?;
        Ok(Self {
            writer: Arc::new(writer),
        })
    }

    /// Get the path to the log file.
    pub fn log_path(&self) -> &std::path::Path {
        self.writer.path()
    }

    /// Get the instance name.
    pub fn instance(&self) -> &str {
        self.writer.instance()
    }
}

impl<S> Layer<S> for JsonlLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let metadata = event.metadata();

        // Build the log entry
        let level = metadata.level().as_str().to_lowercase();
        let target = metadata.target();

        // Extract the message and fields
        let mut visitor = JsonVisitor::new();
        event.record(&mut visitor);

        let message = visitor.message.unwrap_or_default();

        let mut entry = JsonLogEntry::new(&level, self.writer.instance(), target, message);

        // Add fields if any
        if !visitor.fields.is_empty() {
            entry = entry.with_fields(serde_json::Value::Object(visitor.fields));
        }

        // Add span context if available
        if let Some(scope) = ctx.event_scope(event) {
            let spans: Vec<String> = scope.from_root().map(|span| span.name().to_string()).collect();
            if !spans.is_empty() {
                entry = entry.with_span(spans.join(" > "));
            }
        }

        // Write the entry (ignore errors to avoid panics in logging)
        let _ = self.writer.write(&entry);
    }

    fn on_new_span(&self, _attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
        // We could log span creation if needed, but for now we just track events
    }

    fn on_record(&self, _span: &Id, _values: &Record<'_>, _ctx: Context<'_, S>) {
        // Recording values to spans - not needed for basic logging
    }

    fn on_close(&self, _id: Id, _ctx: Context<'_, S>) {
        // Span closed - not needed for basic logging
    }
}

/// Visitor that extracts fields from tracing events.
struct JsonVisitor {
    message: Option<String>,
    fields: serde_json::Map<String, serde_json::Value>,
}

impl JsonVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: serde_json::Map::new(),
        }
    }
}

impl Visit for JsonVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let name = field.name();
        let mut buf = String::new();
        let _ = write!(&mut buf, "{:?}", value);

        if name == "message" {
            self.message = Some(buf);
        } else {
            self.fields
                .insert(name.to_string(), serde_json::Value::String(buf));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        let name = field.name();
        if name == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields
                .insert(name.to_string(), serde_json::Value::String(value.to_string()));
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), serde_json::Value::Number(value.into()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), serde_json::Value::Number(value.into()));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), serde_json::Value::Bool(value));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        if let Some(n) = serde_json::Number::from_f64(value) {
            self.fields
                .insert(field.name().to_string(), serde_json::Value::Number(n));
        }
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.fields.insert(
            field.name().to_string(),
            serde_json::Value::String(value.to_string()),
        );
    }
}

/// Builder for creating a tracing subscriber with JSONL logging.
pub struct LoggingBuilder {
    logs_dir: std::path::PathBuf,
    instance: String,
    console_output: bool,
    env_filter: Option<String>,
}

impl LoggingBuilder {
    /// Create a new logging builder.
    pub fn new(logs_dir: impl Into<std::path::PathBuf>, instance: impl Into<String>) -> Self {
        Self {
            logs_dir: logs_dir.into(),
            instance: instance.into(),
            console_output: true,
            env_filter: None,
        }
    }

    /// Disable console output (only write to JSONL).
    pub fn no_console(mut self) -> Self {
        self.console_output = false;
        self
    }

    /// Set the environment filter (e.g., "syncengine=info,syncengine_core=debug").
    pub fn with_filter(mut self, filter: impl Into<String>) -> Self {
        self.env_filter = Some(filter.into());
        self
    }

    /// Build and return the JSONL layer (for manual composition).
    pub fn build_layer(&self) -> std::io::Result<JsonlLayer> {
        JsonlLayer::new(&self.logs_dir, &self.instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tracing_subscriber::prelude::*;

    #[test]
    fn test_jsonl_layer_captures_events() {
        let temp = TempDir::new().unwrap();
        let logs_dir = temp.path().join("logs");

        let layer = JsonlLayer::new(&logs_dir, "test").unwrap();
        let log_path = layer.log_path().to_path_buf();

        // Set up subscriber with our layer
        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("Test message");
            tracing::warn!(count = 42, "Warning with field");
        });

        // Read back the log file
        let content = std::fs::read_to_string(&log_path).unwrap();
        let lines: Vec<_> = content.lines().collect();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("Test message"));
        assert!(lines[0].contains("\"level\":\"info\""));
        assert!(lines[1].contains("Warning with field"));
        assert!(lines[1].contains("\"count\""));
    }
}
