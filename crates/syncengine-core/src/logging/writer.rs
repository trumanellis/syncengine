//! JSONL file writer for instance-specific logs.
//!
//! Each instance writes to its own JSONL file, eliminating race conditions.
//! Files are append-only, making them safe for concurrent writes.

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use super::entry::{JsonLogEntry, SessionMetadata};

/// Writer that appends log entries to a JSONL file.
///
/// Each instance gets its own file: `logs/raw/2026-01-21_love.jsonl`
pub struct InstanceLogWriter {
    /// Instance name (e.g., "love", "joy")
    instance: String,

    /// Buffered file writer (wrapped in Mutex for thread safety)
    writer: Mutex<BufWriter<File>>,

    /// Path to the JSONL file
    path: PathBuf,
}

impl InstanceLogWriter {
    /// Create a new log writer for an instance.
    ///
    /// Creates the logs directory structure if needed:
    /// ```text
    /// logs/
    /// ├── raw/           # JSONL files (one per instance per day)
    /// │   └── 2026-01-21_love.jsonl
    /// └── sessions/      # Session metadata
    ///     └── 2026-01-21T14-13-48.json
    /// ```
    pub fn new(logs_dir: impl AsRef<Path>, instance: impl Into<String>) -> std::io::Result<Self> {
        let instance = instance.into();
        let logs_dir = logs_dir.as_ref();

        // Create directory structure
        let raw_dir = logs_dir.join("raw");
        fs::create_dir_all(&raw_dir)?;

        // Generate filename with date and instance name
        let date = chrono::Local::now().format("%Y-%m-%d");
        let filename = format!("{}_{}.jsonl", date, instance);
        let path = raw_dir.join(&filename);

        // Open file in append mode
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        let writer = BufWriter::new(file);

        Ok(Self {
            instance,
            writer: Mutex::new(writer),
            path,
        })
    }

    /// Get the instance name.
    pub fn instance(&self) -> &str {
        &self.instance
    }

    /// Get the path to the JSONL file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Write a log entry to the file.
    ///
    /// Each entry is written as a single line followed by a newline.
    /// The write is atomic at the OS level for lines under ~4KB.
    pub fn write(&self, entry: &JsonLogEntry) -> std::io::Result<()> {
        let json = entry
            .to_json_line()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut writer = self.writer.lock().unwrap();
        writeln!(writer, "{}", json)?;
        writer.flush()?;

        Ok(())
    }

    /// Write a raw message (for non-structured logs).
    pub fn write_raw(
        &self,
        level: &str,
        target: &str,
        message: &str,
        fields: Option<serde_json::Value>,
    ) -> std::io::Result<()> {
        let mut entry = JsonLogEntry::new(level, &self.instance, target, message);
        if let Some(f) = fields {
            entry = entry.with_fields(f);
        }
        self.write(&entry)
    }

    /// Flush any buffered data to disk.
    pub fn flush(&self) -> std::io::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.flush()
    }
}

impl Drop for InstanceLogWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Write session metadata to the sessions directory.
pub fn write_session_metadata(
    logs_dir: impl AsRef<Path>,
    metadata: &SessionMetadata,
) -> std::io::Result<PathBuf> {
    let sessions_dir = logs_dir.as_ref().join("sessions");
    fs::create_dir_all(&sessions_dir)?;

    let filename = format!("{}.json", metadata.session_id);
    let path = sessions_dir.join(&filename);

    let json = serde_json::to_string_pretty(metadata)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    fs::write(&path, json)?;

    Ok(path)
}

/// Read all JSONL files from the raw logs directory.
pub fn read_all_entries(logs_dir: impl AsRef<Path>) -> std::io::Result<Vec<JsonLogEntry>> {
    let raw_dir = logs_dir.as_ref().join("raw");

    if !raw_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    for entry in fs::read_dir(&raw_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
            let content = fs::read_to_string(&path)?;
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                match JsonLogEntry::from_json_line(line) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => {
                        // Log parse errors but don't fail
                        eprintln!("Warning: Failed to parse log line in {:?}: {}", path, e);
                    }
                }
            }
        }
    }

    // Sort by timestamp
    entries.sort_by(|a, b| a.ts.cmp(&b.ts));

    Ok(entries)
}

/// Read entries from a specific date.
pub fn read_entries_for_date(
    logs_dir: impl AsRef<Path>,
    date: &str,
) -> std::io::Result<Vec<JsonLogEntry>> {
    let raw_dir = logs_dir.as_ref().join("raw");

    if !raw_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    for entry in fs::read_dir(&raw_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Check if filename starts with the date
        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
            if filename.starts_with(date) && filename.ends_with(".jsonl") {
                let content = fs::read_to_string(&path)?;
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(entry) = JsonLogEntry::from_json_line(line) {
                        entries.push(entry);
                    }
                }
            }
        }
    }

    entries.sort_by(|a, b| a.ts.cmp(&b.ts));

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_writer_creates_directory_structure() {
        let temp = TempDir::new().unwrap();
        let logs_dir = temp.path().join("logs");

        let writer = InstanceLogWriter::new(&logs_dir, "love").unwrap();

        assert!(logs_dir.join("raw").exists());
        assert!(writer.path().exists());
        assert!(writer.path().to_string_lossy().contains("love.jsonl"));
    }

    #[test]
    fn test_writer_appends_entries() {
        let temp = TempDir::new().unwrap();
        let logs_dir = temp.path().join("logs");

        let writer = InstanceLogWriter::new(&logs_dir, "joy").unwrap();

        writer
            .write_raw("info", "test::module", "First message", None)
            .unwrap();
        writer
            .write_raw("debug", "test::module", "Second message", None)
            .unwrap();

        // Read back
        let content = fs::read_to_string(writer.path()).unwrap();
        let lines: Vec<_> = content.lines().collect();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("First message"));
        assert!(lines[1].contains("Second message"));
    }

    #[test]
    fn test_read_all_entries() {
        let temp = TempDir::new().unwrap();
        let logs_dir = temp.path().join("logs");

        // Write from two "instances"
        let writer1 = InstanceLogWriter::new(&logs_dir, "love").unwrap();
        let writer2 = InstanceLogWriter::new(&logs_dir, "joy").unwrap();

        writer1
            .write_raw("info", "sync", "Love connected", None)
            .unwrap();
        writer2
            .write_raw("info", "sync", "Joy connected", None)
            .unwrap();

        drop(writer1);
        drop(writer2);

        // Read all
        let entries = read_all_entries(&logs_dir).unwrap();

        assert_eq!(entries.len(), 2);

        let instances: Vec<_> = entries.iter().map(|e| e.instance.as_str()).collect();
        assert!(instances.contains(&"love"));
        assert!(instances.contains(&"joy"));
    }

    #[test]
    fn test_session_metadata() {
        let temp = TempDir::new().unwrap();
        let logs_dir = temp.path().join("logs");

        let meta = SessionMetadata::new(vec!["love".into(), "joy".into()]);
        let path = write_session_metadata(&logs_dir, &meta).unwrap();

        assert!(path.exists());
        assert!(logs_dir.join("sessions").exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"love\""));
        assert!(content.contains("\"joy\""));
    }
}
