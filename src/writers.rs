//! Writers for different log output destinations.

use crate::config::{FileConfig, LogLevel, RotationConfig, RotationFrequency};
use crate::error::{LoggerError, Result};
use crate::formatters::{Formatter, LogRecord};
use chrono::{DateTime, Datelike, Local, Weekday};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Trait for log writers.
pub trait Writer: Send + Sync {
    /// Writes a formatted log record.
    fn write(&mut self, record: &LogRecord, formatted: &str) -> Result<()>;

    /// Flushes any buffered output.
    fn flush(&mut self) -> Result<()>;

    /// Returns whether this writer should be used for the given log level.
    fn should_write(&self, level: LogLevel) -> bool {
        true // Default: write all levels
    }
}

/// Console writer that outputs to stdout/stderr.
pub struct ConsoleWriter {
    /// Whether to use stderr for error/warning levels
    use_stderr: bool,
    /// Formatter for this writer
    formatter: Box<dyn Formatter>,
}

impl std::fmt::Debug for ConsoleWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsoleWriter")
            .field("use_stderr", &self.use_stderr)
            .field("formatter", &"<dyn Formatter>")
            .finish()
    }
}

impl ConsoleWriter {
    /// Creates a new console writer.
    pub fn new(use_stderr: bool, formatter: Box<dyn Formatter>) -> Self {
        Self {
            use_stderr,
            formatter,
        }
    }
}

impl Writer for ConsoleWriter {
    fn write(&mut self, record: &LogRecord, _formatted: &str) -> Result<()> {
        // Use our own formatter instead of the pre-formatted string
        let output = self.formatter.format(record);

        if self.use_stderr && (record.level == LogLevel::Error || record.level == LogLevel::Warning)
        {
            eprintln!("{}", output);
        } else {
            println!("{}", output);
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        use std::io::{stderr, stdout, Write};
        let _ = stdout().flush();
        let _ = stderr().flush();
        Ok(())
    }
}

/// File writer with rotation support.
pub struct FileWriter {
    /// Configuration for file output
    config: FileConfig,
    /// Current file writer
    writer: Option<BufWriter<File>>,
    /// Current file path
    current_path: PathBuf,
    /// Current file size
    current_size: u64,
    /// Last rotation check time
    last_rotation_check: SystemTime,
    /// Formatter for this writer
    formatter: Box<dyn Formatter>,
}

impl std::fmt::Debug for FileWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileWriter")
            .field("config", &self.config)
            .field("current_path", &self.current_path)
            .field("current_size", &self.current_size)
            .field("formatter", &"<dyn Formatter>")
            .finish()
    }
}

impl FileWriter {
    /// Creates a new file writer.
    pub fn new(config: FileConfig, formatter: Box<dyn Formatter>) -> Result<Self> {
        let current_path = config.path.clone();

        // Ensure parent directory exists
        if let Some(parent) = current_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut writer = Self {
            config,
            writer: None,
            current_path,
            current_size: 0,
            last_rotation_check: SystemTime::now(),
            formatter,
        };

        writer.open_file()?;
        Ok(writer)
    }

    /// Opens or reopens the log file.
    fn open_file(&mut self) -> Result<()> {
        let file = if self.config.append {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.current_path)?
        } else {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&self.current_path)?
        };

        // Get current file size
        self.current_size = file.metadata()?.len();

        self.writer = if self.config.buffer_size > 0 {
            Some(BufWriter::with_capacity(self.config.buffer_size, file))
        } else {
            Some(BufWriter::new(file))
        };

        Ok(())
    }

    /// Checks if rotation is needed and performs it if necessary.
    fn check_and_rotate(&mut self) -> Result<()> {
        // Extract rotation info to avoid borrowing conflicts
        let rotation_info = match &self.config.rotation {
            RotationConfig::None => None,
            RotationConfig::Size { max_size, .. } => Some((*max_size, None)),
            RotationConfig::Time { frequency, .. } => Some((0, Some(*frequency))),
        };

        let should_rotate = match rotation_info {
            None => false,
            Some((max_size, None)) => self.current_size >= max_size,
            Some((_, Some(frequency))) => self.should_rotate_by_time(&frequency)?,
        };

        if should_rotate {
            self.rotate_file()?;
        }

        Ok(())
    }

    /// Determines if rotation should occur based on time.
    fn should_rotate_by_time(&mut self, frequency: &RotationFrequency) -> Result<bool> {
        let now = SystemTime::now();

        // Only check rotation at most once per minute to avoid excessive checks
        if now.duration_since(self.last_rotation_check)?.as_secs() < 60 {
            return Ok(false);
        }

        self.last_rotation_check = now;

        let current_time = Local::now();
        let file_modified = self.get_file_modified_time()?;

        match frequency {
            RotationFrequency::Daily => Ok(current_time.date_naive() != file_modified.date_naive()),
            RotationFrequency::Weekly => {
                let current_week = current_time.iso_week();
                let file_week = file_modified.iso_week();
                Ok(current_week != file_week)
            }
            RotationFrequency::Monthly => Ok(current_time.month() != file_modified.month()
                || current_time.year() != file_modified.year()),
        }
    }

    /// Gets the modification time of the current log file.
    fn get_file_modified_time(&self) -> Result<DateTime<Local>> {
        let metadata = std::fs::metadata(&self.current_path)?;
        let modified = metadata.modified()?;
        let datetime = DateTime::<Local>::from(modified);
        Ok(datetime)
    }

    /// Performs file rotation.
    fn rotate_file(&mut self) -> Result<()> {
        // Flush and close current file
        if let Some(ref mut writer) = self.writer {
            writer.flush()?;
        }
        self.writer = None;

        let keep_files = match &self.config.rotation {
            RotationConfig::Size { keep_files, .. } => *keep_files,
            RotationConfig::Time { keep_files, .. } => *keep_files,
            RotationConfig::None => return Ok(()), // Should not happen
        };

        // Generate rotation suffix
        let suffix = match &self.config.rotation {
            RotationConfig::Size { .. } => {
                format!(
                    "{}",
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
                )
            }
            RotationConfig::Time { frequency, .. } => {
                let now = Local::now();
                match frequency {
                    RotationFrequency::Daily => now.format("%Y-%m-%d").to_string(),
                    RotationFrequency::Weekly => now.format("%Y-W%U").to_string(),
                    RotationFrequency::Monthly => now.format("%Y-%m").to_string(),
                }
            }
            RotationConfig::None => return Ok(()), // Should not happen, but handle it
        };

        // Move current file to backup
        let backup_path = self.current_path.with_extension(format!(
            "{}.{}",
            self.current_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("log"),
            suffix
        ));

        if self.current_path.exists() {
            std::fs::rename(&self.current_path, &backup_path)?;
        }

        // Clean up old backup files
        self.cleanup_old_backups(keep_files)?;

        // Reset file size and reopen
        self.current_size = 0;
        self.open_file()?;

        Ok(())
    }

    /// Cleans up old backup files.
    fn cleanup_old_backups(&self, keep_files: usize) -> Result<()> {
        if keep_files == 0 {
            return Ok(());
        }

        let parent_dir = self
            .current_path
            .parent()
            .ok_or_else(|| LoggerError::Config("Invalid log file path".to_string()))?;

        let file_stem = self
            .current_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| LoggerError::Config("Invalid log file name".to_string()))?;

        let extension = self
            .current_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("log");

        // Find all backup files
        let mut backup_files = Vec::new();

        if let Ok(entries) = std::fs::read_dir(parent_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let pattern = format!("{}.{}", file_stem, extension);
                    if name.starts_with(&pattern)
                        && name != self.current_path.file_name().unwrap().to_str().unwrap()
                    {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                backup_files.push((path, modified));
                            }
                        }
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backup_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove excess files
        for (path, _) in backup_files.into_iter().skip(keep_files) {
            let _ = std::fs::remove_file(path);
        }

        Ok(())
    }
}

impl Writer for FileWriter {
    fn write(&mut self, record: &LogRecord, formatted: &str) -> Result<()> {
        // Check for rotation before writing
        self.check_and_rotate()?;

        if let Some(ref mut writer) = self.writer {
            writeln!(writer, "{}", formatted)?;
            self.current_size += formatted.len() as u64 + 1; // +1 for newline

            // Auto-flush if interval is 0 or if enough time has passed
            if self.config.flush_interval == 0 {
                writer.flush()?;
            }
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        if let Some(ref mut writer) = self.writer {
            writer.flush()?;
        }
        Ok(())
    }
}

/// Multi-writer that forwards to multiple writers.
pub struct MultiWriter {
    /// List of writers
    writers: Vec<Box<dyn Writer>>,
}

impl std::fmt::Debug for MultiWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiWriter")
            .field("writers", &format!("{} writers", self.writers.len()))
            .finish()
    }
}

impl MultiWriter {
    /// Creates a new multi-writer.
    pub fn new() -> Self {
        Self {
            writers: Vec::new(),
        }
    }

    /// Adds a writer.
    pub fn add_writer(mut self, writer: Box<dyn Writer>) -> Self {
        self.writers.push(writer);
        self
    }

    /// Adds multiple writers.
    pub fn add_writers(mut self, writers: Vec<Box<dyn Writer>>) -> Self {
        self.writers.extend(writers);
        self
    }
}

impl Default for MultiWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer for MultiWriter {
    fn write(&mut self, record: &LogRecord, formatted: &str) -> Result<()> {
        let mut errors = Vec::new();

        for writer in &mut self.writers {
            if writer.should_write(record.level) {
                if let Err(e) = writer.write(record, formatted) {
                    errors.push(e);
                }
            }
        }

        // Return the first error if any occurred
        if let Some(error) = errors.into_iter().next() {
            Err(error)
        } else {
            Ok(())
        }
    }

    fn flush(&mut self) -> Result<()> {
        let mut errors = Vec::new();

        for writer in &mut self.writers {
            if let Err(e) = writer.flush() {
                errors.push(e);
            }
        }

        // Return the first error if any occurred
        if let Some(error) = errors.into_iter().next() {
            Err(error)
        } else {
            Ok(())
        }
    }

    fn should_write(&self, level: LogLevel) -> bool {
        self.writers.iter().any(|w| w.should_write(level))
    }
}

/// Level-filtered writer that only writes logs above a certain level.
pub struct LevelFilterWriter {
    /// Minimum log level to write
    min_level: LogLevel,
    /// Inner writer
    inner: Box<dyn Writer>,
}

impl std::fmt::Debug for LevelFilterWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LevelFilterWriter")
            .field("min_level", &self.min_level)
            .field("inner", &"<dyn Writer>")
            .finish()
    }
}

impl LevelFilterWriter {
    /// Creates a new level-filtered writer.
    pub fn new(min_level: LogLevel, inner: Box<dyn Writer>) -> Self {
        Self { min_level, inner }
    }
}

impl Writer for LevelFilterWriter {
    fn write(&mut self, record: &LogRecord, formatted: &str) -> Result<()> {
        if record.level <= self.min_level {
            self.inner.write(record, formatted)
        } else {
            Ok(())
        }
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }

    fn should_write(&self, level: LogLevel) -> bool {
        level <= self.min_level && self.inner.should_write(level)
    }
}

/// Buffered writer that flushes periodically.
pub struct BufferedWriter {
    /// Inner writer
    inner: Arc<Mutex<Box<dyn Writer>>>,
    /// Flush interval in milliseconds
    flush_interval: u64,
    /// Last flush time
    last_flush: Arc<Mutex<SystemTime>>,
}

impl std::fmt::Debug for BufferedWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferedWriter")
            .field("flush_interval", &self.flush_interval)
            .field("inner", &"<dyn Writer>")
            .finish()
    }
}

impl BufferedWriter {
    /// Creates a new buffered writer.
    pub fn new(inner: Box<dyn Writer>, flush_interval: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner)),
            flush_interval,
            last_flush: Arc::new(Mutex::new(SystemTime::now())),
        }
    }

    /// Checks if flush is needed and performs it.
    fn check_and_flush(&self) -> Result<()> {
        if self.flush_interval == 0 {
            return Ok(());
        }

        let now = SystemTime::now();
        let mut last_flush = self
            .last_flush
            .lock()
            .map_err(|_| LoggerError::Custom("Failed to acquire flush lock".to_string()))?;

        if now.duration_since(*last_flush)?.as_millis() >= self.flush_interval as u128 {
            let mut writer = self
                .inner
                .lock()
                .map_err(|_| LoggerError::Custom("Failed to acquire writer lock".to_string()))?;
            writer.flush()?;
            *last_flush = now;
        }

        Ok(())
    }
}

impl Writer for BufferedWriter {
    fn write(&mut self, record: &LogRecord, formatted: &str) -> Result<()> {
        {
            let mut writer = self
                .inner
                .lock()
                .map_err(|_| LoggerError::Custom("Failed to acquire writer lock".to_string()))?;
            writer.write(record, formatted)?;
        }

        // Check if we need to flush
        self.check_and_flush()?;

        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        let mut writer = self
            .inner
            .lock()
            .map_err(|_| LoggerError::Custom("Failed to acquire writer lock".to_string()))?;
        writer.flush()
    }

    fn should_write(&self, level: LogLevel) -> bool {
        if let Ok(writer) = self.inner.lock() {
            writer.should_write(level)
        } else {
            true // Default to allowing writes if lock fails
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OutputFormat;
    use crate::formatters::{create_formatter, TextFormatter};
    use tempfile::NamedTempFile;

    #[test]
    fn test_console_writer() {
        let formatter = Box::new(TextFormatter::new().with_colors(false));
        let mut writer = ConsoleWriter::new(true, formatter);
        let record = LogRecord::new(LogLevel::Info, format_args!("Test message"));

        assert!(writer.write(&record, "Test formatted message").is_ok());
        assert!(writer.flush().is_ok());
    }

    #[test]
    fn test_file_writer() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let config = FileConfig {
            path: temp_file.path().to_path_buf(),
            append: true,
            rotation: RotationConfig::None,
            buffer_size: 0,
            flush_interval: 0,
        };

        let formatter = create_formatter(
            OutputFormat::Text,
            false,
            "%Y-%m-%d %H:%M:%S",
            false,
            false,
            false,
        );

        let mut writer = FileWriter::new(config, formatter)?;
        let record = LogRecord::new(LogLevel::Info, format_args!("Test message"));

        writer.write(&record, "Test formatted message")?;
        writer.flush()?;

        // Check file contents
        let contents = std::fs::read_to_string(temp_file.path())?;
        assert!(contents.contains("Test formatted message"));

        Ok(())
    }

    #[test]
    fn test_multi_writer() {
        let formatter1 = Box::new(TextFormatter::new().with_colors(false));
        let formatter2 = Box::new(TextFormatter::new().with_colors(false));

        let writer1 = Box::new(ConsoleWriter::new(false, formatter1));
        let writer2 = Box::new(ConsoleWriter::new(true, formatter2));

        let mut multi_writer = MultiWriter::new().add_writer(writer1).add_writer(writer2);

        let record = LogRecord::new(LogLevel::Error, format_args!("Error message"));
        assert!(multi_writer
            .write(&record, "Formatted error message")
            .is_ok());
        assert!(multi_writer.flush().is_ok());
    }

    #[test]
    fn test_level_filter_writer() {
        let formatter = Box::new(TextFormatter::new().with_colors(false));
        let console_writer = Box::new(ConsoleWriter::new(false, formatter));
        let mut filtered_writer = LevelFilterWriter::new(LogLevel::Warning, console_writer);

        // Should write error (higher priority)
        let error_record = LogRecord::new(LogLevel::Error, format_args!("Error"));
        assert!(filtered_writer
            .write(&error_record, "Error message")
            .is_ok());

        // Should write warning (equal priority)
        let warning_record = LogRecord::new(LogLevel::Warning, format_args!("Warning"));
        assert!(filtered_writer
            .write(&warning_record, "Warning message")
            .is_ok());

        // Should not write info (lower priority)
        assert!(!filtered_writer.should_write(LogLevel::Info));
    }
}
