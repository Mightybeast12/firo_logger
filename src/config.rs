//! Configuration system for firo_logger.

use crate::error::{LoggerError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Log levels supported by the logger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    /// Error level - highest priority
    Error = 0,
    /// Warning level
    Warning = 1,
    /// Info level
    Info = 2,
    /// Debug level
    /// Success level
    Success = 3,
    /// Debug level - lowest priority
    Debug = 4,
}

impl LogLevel {
    /// Returns the string representation of the log level.
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warning => "WARNING",
            LogLevel::Info => "INFO",
            LogLevel::Success => "SUCCESS",
            LogLevel::Debug => "DEBUG",
        }
    }

    /// Returns all log levels in order of priority.
    pub fn all() -> Vec<LogLevel> {
        vec![
            LogLevel::Error,
            LogLevel::Warning,
            LogLevel::Info,
            LogLevel::Success,
            LogLevel::Debug,
        ]
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for LogLevel {
    type Err = LoggerError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "ERROR" => Ok(LogLevel::Error),
            "WARNING" | "WARN" => Ok(LogLevel::Warning),
            "INFO" => Ok(LogLevel::Info),
            "SUCCESS" => Ok(LogLevel::Success),
            "DEBUG" => Ok(LogLevel::Debug),
            _ => Err(LoggerError::Config(format!("Invalid log level: {s}"))),
        }
    }
}

/// Output format for log messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Plain text format with colors
    Text,
    /// JSON structured format
    Json,
    /// Plain text without colors
    Plain,
}

/// Log rotation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationConfig {
    /// No rotation
    None,
    /// Rotate based on file size
    Size {
        /// Maximum file size in bytes
        max_size: u64,
        /// Number of backup files to keep
        keep_files: usize,
    },
    /// Rotate based on time
    Time {
        /// Rotation frequency
        frequency: RotationFrequency,
        /// Number of backup files to keep
        keep_files: usize,
    },
}

/// Rotation frequency for time-based rotation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RotationFrequency {
    /// Rotate daily
    Daily,
    /// Rotate weekly
    Weekly,
    /// Rotate monthly
    Monthly,
}

/// Configuration for file output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    /// Path to the log file
    pub path: PathBuf,
    /// Whether to append to existing file or overwrite
    pub append: bool,
    /// Rotation configuration
    pub rotation: RotationConfig,
    /// Buffer size for file writes (0 = unbuffered)
    pub buffer_size: usize,
    /// Auto-flush interval in milliseconds (0 = flush immediately)
    pub flush_interval: u64,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("app.log"),
            append: true,
            rotation: RotationConfig::None,
            buffer_size: 8192,    // 8KB buffer
            flush_interval: 1000, // 1 second
        }
    }
}

/// Configuration for console output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleConfig {
    /// Whether to enable colored output
    pub colors: bool,
    /// Whether to use stderr for error/warning levels
    pub use_stderr: bool,
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            colors: true,
            use_stderr: true,
        }
    }
}

/// Main logger configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    /// Minimum log level to output
    pub level: LogLevel,
    /// Output format
    pub format: OutputFormat,
    /// Whether to enable console output
    pub console_enabled: bool,
    /// Console configuration
    pub console: ConsoleConfig,
    /// Whether to enable file output
    pub file_enabled: bool,
    /// File configuration
    pub file: FileConfig,
    /// Whether to enable async logging
    pub async_enabled: bool,
    /// Channel buffer size for async logging
    pub async_buffer_size: usize,
    /// Date/time format string
    pub datetime_format: String,
    /// Module-based log level filters
    pub module_filters: HashMap<String, LogLevel>,
    /// Include caller information (file, line, module)
    pub include_caller: bool,
    /// Include thread information
    pub include_thread: bool,
    /// Custom metadata fields
    pub metadata: HashMap<String, String>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: OutputFormat::Text,
            console_enabled: true,
            console: ConsoleConfig::default(),
            file_enabled: false,
            file: FileConfig::default(),
            async_enabled: false,
            async_buffer_size: 1000,
            datetime_format: "%Y-%m-%d %H:%M:%S".to_string(),
            module_filters: HashMap::new(),
            include_caller: false,
            include_thread: false,
            metadata: HashMap::new(),
        }
    }
}

/// Builder for creating logger configurations.
#[derive(Debug)]
pub struct LoggerConfigBuilder {
    config: LoggerConfig,
}

impl LoggerConfigBuilder {
    /// Creates a new configuration builder.
    pub fn new() -> Self {
        Self {
            config: LoggerConfig::default(),
        }
    }

    /// Sets the minimum log level.
    pub fn level(mut self, level: LogLevel) -> Self {
        self.config.level = level;
        self
    }

    /// Sets the output format.
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.config.format = format;
        self
    }

    /// Enables or disables console output.
    pub fn console(mut self, enabled: bool) -> Self {
        self.config.console_enabled = enabled;
        self
    }

    /// Configures console output.
    pub fn console_config(mut self, config: ConsoleConfig) -> Self {
        self.config.console = config;
        self
    }

    /// Enables or disables colored console output.
    pub fn colors(mut self, enabled: bool) -> Self {
        self.config.console.colors = enabled;
        self
    }

    /// Configures stderr usage for errors and warnings.
    pub fn use_stderr(mut self, enabled: bool) -> Self {
        self.config.console.use_stderr = enabled;
        self
    }

    /// Enables file output with the given path.
    pub fn file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.file_enabled = true;
        self.config.file.path = path.into();
        self
    }

    /// Configures file output settings.
    pub fn file_config(mut self, config: FileConfig) -> Self {
        self.config.file_enabled = true;
        self.config.file = config;
        self
    }

    /// Enables file rotation based on size.
    pub fn rotate_by_size(mut self, max_size: u64, keep_files: usize) -> Self {
        self.config.file.rotation = RotationConfig::Size {
            max_size,
            keep_files,
        };
        self
    }

    /// Enables file rotation based on time.
    pub fn rotate_by_time(mut self, frequency: RotationFrequency, keep_files: usize) -> Self {
        self.config.file.rotation = RotationConfig::Time {
            frequency,
            keep_files,
        };
        self
    }

    /// Enables async logging.
    pub fn async_logging(mut self, buffer_size: usize) -> Self {
        self.config.async_enabled = true;
        self.config.async_buffer_size = buffer_size;
        self
    }

    /// Sets the datetime format string.
    pub fn datetime_format<S: Into<String>>(mut self, format: S) -> Self {
        self.config.datetime_format = format.into();
        self
    }

    /// Adds a module-specific log level filter.
    pub fn module_filter<S: Into<String>>(mut self, module: S, level: LogLevel) -> Self {
        self.config.module_filters.insert(module.into(), level);
        self
    }

    /// Enables caller information in log messages.
    pub fn include_caller(mut self, enabled: bool) -> Self {
        self.config.include_caller = enabled;
        self
    }

    /// Enables thread information in log messages.
    pub fn include_thread(mut self, enabled: bool) -> Self {
        self.config.include_thread = enabled;
        self
    }

    /// Adds custom metadata.
    pub fn metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.config.metadata.insert(key.into(), value.into());
        self
    }

    /// Builds the configuration.
    pub fn build(self) -> LoggerConfig {
        self.config
    }
}

impl Default for LoggerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggerConfig {
    /// Creates a new configuration builder.
    pub fn builder() -> LoggerConfigBuilder {
        LoggerConfigBuilder::new()
    }

    /// Loads configuration from environment variables.
    pub fn from_env() -> Self {
        let mut config = LoggerConfig::default();

        // Load log level from environment
        if let Ok(level_str) = std::env::var("FIRO_LOG_LEVEL") {
            if let Ok(level) = level_str.parse::<LogLevel>() {
                config.level = level;
            }
        }

        // Load file path from environment
        if let Ok(file_path) = std::env::var("FIRO_LOG_FILE") {
            config.file_enabled = true;
            config.file.path = PathBuf::from(file_path);
        }

        // Load format from environment
        if let Ok(format_str) = std::env::var("FIRO_LOG_FORMAT") {
            match format_str.to_lowercase().as_str() {
                "json" => config.format = OutputFormat::Json,
                "plain" => config.format = OutputFormat::Plain,
                _ => config.format = OutputFormat::Text,
            }
        }

        // Disable colors if NO_COLOR is set or not in a terminal
        if std::env::var("NO_COLOR").is_ok() || !atty::is(atty::Stream::Stdout) {
            config.console.colors = false;
        } else if std::env::var("FORCE_COLOR").is_ok() {
            config.console.colors = true;
        }

        config
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        if !self.console_enabled && !self.file_enabled {
            return Err(LoggerError::Config(
                "At least one output (console or file) must be enabled".to_string(),
            ));
        }

        if self.file_enabled && self.file.path.as_os_str().is_empty() {
            return Err(LoggerError::Config(
                "File path cannot be empty when file logging is enabled".to_string(),
            ));
        }

        if self.async_enabled && self.async_buffer_size == 0 {
            return Err(LoggerError::Config(
                "Async buffer size must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Gets the effective log level for a specific module.
    pub fn effective_level(&self, module: &str) -> LogLevel {
        // Check for exact module match first
        if let Some(&level) = self.module_filters.get(module) {
            return level;
        }

        // Check for parent module matches
        let parts = module.split("::");
        let mut current_path = String::new();

        for part in parts {
            if !current_path.is_empty() {
                current_path.push_str("::");
            }
            current_path.push_str(part);

            if let Some(&level) = self.module_filters.get(&current_path) {
                return level;
            }
        }

        // Return default level
        self.level
    }
}

/// ANSI color codes for console output.
pub struct Colors;

impl Colors {
    pub const RED: &'static str = "\x1b[31m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const YELLOW: &'static str = "\x1b[33m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const MAGENTA: &'static str = "\x1b[35m";
    pub const CYAN: &'static str = "\x1b[36m";
    pub const WHITE: &'static str = "\x1b[37m";
    pub const RESET: &'static str = "\x1b[0m";
    pub const BOLD: &'static str = "\x1b[1m";
    pub const DIM: &'static str = "\x1b[2m";

    /// Gets the color for a log level.
    pub fn for_level(level: LogLevel) -> &'static str {
        match level {
            LogLevel::Error => Self::RED,
            LogLevel::Warning => Self::YELLOW,
            LogLevel::Info => Self::CYAN,
            LogLevel::Success => Self::GREEN,
            LogLevel::Debug => Self::BLUE,
        }
    }
}

/// Helper to detect if colors should be used in terminal.
#[allow(dead_code)]
fn should_use_colors() -> bool {
    std::env::var("NO_COLOR").is_err() && atty::is(atty::Stream::Stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Success);
        assert!(LogLevel::Success < LogLevel::Debug);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!("ERROR".parse::<LogLevel>().unwrap(), LogLevel::Error);
        assert_eq!("WARN".parse::<LogLevel>().unwrap(), LogLevel::Warning);
        assert_eq!("info".parse::<LogLevel>().unwrap(), LogLevel::Info);
        assert!("INVALID".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = LoggerConfig::builder()
            .level(LogLevel::Debug)
            .colors(false)
            .file("test.log")
            .async_logging(500)
            .build();

        assert_eq!(config.level, LogLevel::Debug);
        assert!(!config.console.colors);
        assert!(config.file_enabled);
        assert_eq!(config.file.path, PathBuf::from("test.log"));
        assert!(config.async_enabled);
        assert_eq!(config.async_buffer_size, 500);
    }

    #[test]
    fn test_module_filter() {
        let mut config = LoggerConfig {
            level: LogLevel::Info,
            ..Default::default()
        };
        config
            .module_filters
            .insert("my_crate::module".to_string(), LogLevel::Debug);

        assert_eq!(config.effective_level("my_crate::module"), LogLevel::Debug);
        assert_eq!(
            config.effective_level("my_crate::module::submodule"),
            LogLevel::Debug
        );
        assert_eq!(config.effective_level("other_crate"), LogLevel::Info);
    }
}
