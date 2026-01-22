//! Report generator for converting JSONL logs to human-readable markdown.
//!
//! The generated LOGS.md is a view of the raw JSONL data - it can be
//! regenerated at any time from the source files.

use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::Path;

use super::entry::JsonLogEntry;
use super::writer::read_all_entries;

/// Statistics about a set of log entries.
#[derive(Debug, Default)]
pub struct LogStats {
    pub total: usize,
    pub info: usize,
    pub warn: usize,
    pub error: usize,
    pub debug: usize,
    pub trace: usize,
}

impl LogStats {
    fn from_entries(entries: &[JsonLogEntry]) -> Self {
        let mut stats = Self::default();
        stats.total = entries.len();
        for entry in entries {
            match entry.level.as_str() {
                "info" => stats.info += 1,
                "warn" => stats.warn += 1,
                "error" => stats.error += 1,
                "debug" => stats.debug += 1,
                "trace" => stats.trace += 1,
                _ => {}
            }
        }
        stats
    }
}

/// Options for report generation.
#[derive(Debug, Clone)]
pub struct ReportOptions {
    /// Include trace-level logs (very verbose)
    pub include_trace: bool,

    /// Include debug-level logs
    pub include_debug: bool,

    /// Maximum entries per instance section (0 = unlimited)
    pub max_per_instance: usize,

    /// Show full timestamps (vs relative)
    pub full_timestamps: bool,
}

impl Default for ReportOptions {
    fn default() -> Self {
        Self {
            include_trace: false,
            include_debug: true,
            max_per_instance: 0,
            full_timestamps: false,
        }
    }
}

/// Generate a markdown report from JSONL logs.
pub fn generate_report(logs_dir: impl AsRef<Path>, options: &ReportOptions) -> std::io::Result<String> {
    let logs_dir = logs_dir.as_ref();
    let entries = read_all_entries(logs_dir)?;

    if entries.is_empty() {
        return Ok("# Synchronicity Engine - Run Logs\n\nNo log entries found.\n".to_string());
    }

    // Filter by log level
    let entries: Vec<_> = entries
        .into_iter()
        .filter(|e| {
            match e.level.as_str() {
                "trace" => options.include_trace,
                "debug" => options.include_debug,
                _ => true,
            }
        })
        .collect();

    let stats = LogStats::from_entries(&entries);

    // Group by instance
    let mut by_instance: HashMap<String, Vec<&JsonLogEntry>> = HashMap::new();
    for entry in &entries {
        by_instance
            .entry(entry.instance.clone())
            .or_default()
            .push(entry);
    }

    // Build the report
    let mut report = String::new();

    // Header
    writeln!(report, "# Synchronicity Engine - Run Logs").unwrap();
    writeln!(report).unwrap();

    // Session info
    if let (Some(first), Some(last)) = (entries.first(), entries.last()) {
        writeln!(report, "**Session:** {} to {}", first.ts, last.ts).unwrap();
        writeln!(report, "**Instances:** {}", by_instance.keys().cloned().collect::<Vec<_>>().join(", ")).unwrap();
        writeln!(report).unwrap();
    }

    // Statistics table
    writeln!(report, "## Statistics").unwrap();
    writeln!(report).unwrap();
    writeln!(report, "| Level | Count |").unwrap();
    writeln!(report, "|-------|-------|").unwrap();
    writeln!(report, "| Total | {} |", stats.total).unwrap();
    writeln!(report, "| INFO  | {} |", stats.info).unwrap();
    writeln!(report, "| WARN  | {} |", stats.warn).unwrap();
    writeln!(report, "| ERROR | {} |", stats.error).unwrap();
    if options.include_debug {
        writeln!(report, "| DEBUG | {} |", stats.debug).unwrap();
    }
    if options.include_trace {
        writeln!(report, "| TRACE | {} |", stats.trace).unwrap();
    }
    writeln!(report).unwrap();

    // Errors section (highlighted)
    let errors: Vec<_> = entries.iter().filter(|e| e.level == "error").collect();
    if !errors.is_empty() {
        writeln!(report, "## Errors").unwrap();
        writeln!(report).unwrap();
        for entry in errors {
            writeln!(
                report,
                "- **[{}]** `{}` - {}",
                entry.instance, entry.target, entry.msg
            )
            .unwrap();
            if let Some(fields) = &entry.fields {
                writeln!(report, "  - Fields: `{}`", fields).unwrap();
            }
        }
        writeln!(report).unwrap();
    }

    // Warnings section
    let warnings: Vec<_> = entries.iter().filter(|e| e.level == "warn").collect();
    if !warnings.is_empty() {
        writeln!(report, "## Warnings").unwrap();
        writeln!(report).unwrap();
        for entry in warnings {
            writeln!(
                report,
                "- **[{}]** `{}` - {}",
                entry.instance, entry.target, entry.msg
            )
            .unwrap();
        }
        writeln!(report).unwrap();
    }

    // Per-instance sections
    writeln!(report, "---").unwrap();
    writeln!(report).unwrap();

    let mut instances: Vec<_> = by_instance.keys().collect();
    instances.sort();

    for instance in instances {
        let instance_entries = by_instance.get(instance).unwrap();
        let instance_stats = LogStats::from_entries(
            &instance_entries
                .iter()
                .map(|e| (*e).clone())
                .collect::<Vec<_>>(),
        );

        writeln!(report, "## Instance: `{}`", instance).unwrap();
        writeln!(report).unwrap();
        writeln!(
            report,
            "Total: {} entries ({} info, {} warn, {} error)",
            instance_stats.total, instance_stats.info, instance_stats.warn, instance_stats.error
        )
        .unwrap();
        writeln!(report).unwrap();

        writeln!(report, "<details>").unwrap();
        writeln!(report, "<summary>Click to expand logs</summary>").unwrap();
        writeln!(report).unwrap();
        writeln!(report, "```log").unwrap();

        let entries_to_show = if options.max_per_instance > 0 && instance_entries.len() > options.max_per_instance {
            &instance_entries[..options.max_per_instance]
        } else {
            instance_entries
        };

        for entry in entries_to_show {
            let ts = if options.full_timestamps {
                entry.ts.clone()
            } else {
                // Extract just time portion
                entry.ts.split('T').nth(1).unwrap_or(&entry.ts).to_string()
            };
            let level = entry.level.to_uppercase();
            writeln!(report, "{} {} {} - {}", ts, level, entry.target, entry.msg).unwrap();
        }

        if options.max_per_instance > 0 && instance_entries.len() > options.max_per_instance {
            writeln!(
                report,
                "... ({} more entries truncated)",
                instance_entries.len() - options.max_per_instance
            )
            .unwrap();
        }

        writeln!(report, "```").unwrap();
        writeln!(report).unwrap();
        writeln!(report, "</details>").unwrap();
        writeln!(report).unwrap();
    }

    // Footer
    writeln!(report, "---").unwrap();
    writeln!(report).unwrap();
    writeln!(
        report,
        "*Generated from JSONL logs. Regenerate with: `syncengine-cli logs report`*"
    )
    .unwrap();

    Ok(report)
}

/// Write the report to LOGS.md in the project directory.
pub fn write_report(
    logs_dir: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
    options: &ReportOptions,
) -> std::io::Result<()> {
    let report = generate_report(logs_dir, options)?;
    fs::write(output_path, report)
}

/// Generate a timeline view (all entries sorted by time).
pub fn generate_timeline(logs_dir: impl AsRef<Path>, limit: Option<usize>) -> std::io::Result<String> {
    let entries = read_all_entries(logs_dir)?;

    let mut output = String::new();
    writeln!(output, "# Timeline View").unwrap();
    writeln!(output).unwrap();

    let entries_to_show = if let Some(n) = limit {
        entries.into_iter().rev().take(n).collect::<Vec<_>>()
    } else {
        entries
    };

    for entry in entries_to_show {
        let level_icon = match entry.level.as_str() {
            "error" => "!",
            "warn" => "~",
            "info" => ">",
            "debug" => ".",
            "trace" => "-",
            _ => " ",
        };
        writeln!(
            output,
            "{} [{}] {} {}",
            level_icon, entry.instance, entry.ts, entry.msg
        )
        .unwrap();
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::writer::InstanceLogWriter;
    use tempfile::TempDir;

    #[test]
    fn test_generate_report() {
        let temp = TempDir::new().unwrap();
        let logs_dir = temp.path().join("logs");

        // Write some test entries
        let writer1 = InstanceLogWriter::new(&logs_dir, "love").unwrap();
        let writer2 = InstanceLogWriter::new(&logs_dir, "joy").unwrap();

        writer1
            .write_raw("info", "sync", "Love connected", None)
            .unwrap();
        writer1
            .write_raw("warn", "sync", "Slow connection", None)
            .unwrap();
        writer2
            .write_raw("info", "sync", "Joy connected", None)
            .unwrap();
        writer2
            .write_raw("error", "sync", "Connection lost", None)
            .unwrap();

        drop(writer1);
        drop(writer2);

        // Generate report
        let options = ReportOptions::default();
        let report = generate_report(&logs_dir, &options).unwrap();

        // Verify structure
        assert!(report.contains("# Synchronicity Engine"));
        assert!(report.contains("## Statistics"));
        assert!(report.contains("## Errors"));
        assert!(report.contains("Connection lost"));
        assert!(report.contains("## Warnings"));
        assert!(report.contains("Slow connection"));
        assert!(report.contains("## Instance: `joy`"));
        assert!(report.contains("## Instance: `love`"));
    }

    #[test]
    fn test_generate_timeline() {
        let temp = TempDir::new().unwrap();
        let logs_dir = temp.path().join("logs");

        let writer = InstanceLogWriter::new(&logs_dir, "test").unwrap();
        writer.write_raw("info", "mod", "First", None).unwrap();
        writer.write_raw("info", "mod", "Second", None).unwrap();
        writer.write_raw("info", "mod", "Third", None).unwrap();
        drop(writer);

        let timeline = generate_timeline(&logs_dir, Some(2)).unwrap();

        // With limit=2 and reversed, should have Third and Second
        assert!(timeline.contains("Third"));
        assert!(timeline.contains("Second"));
    }
}
