//! Formatters for different log output formats.

use crate::config::{Colors, LogLevel, OutputFormat};
use chrono::{DateTime, Local};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt::Arguments;

/// Information about the caller of a log statement.
#[derive(Debug, Clone)]
pub struct CallerInfo {
    /// File path where the log was called
    pub file: &'static str,
    /// Line number where the log was called
    pub line: u32,
    /// Module path where the log was called
    pub module: Option<&'static str>,
}

/// Information about the current thread.
#[derive(Debug, Clone)]
pub struct ThreadInfo {
    /// Thread ID
    pub id: String,
    /// Thread name (if available)
    pub name: Option<String>,
}

/// A complete log record with all metadata.
#[derive(Debug, Clone)]
pub struct LogRecord {
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Timestamp when the log was created
    pub timestamp: DateTime<Local>,
    /// Module where the log originated
    pub module: Option<String>,
    /// Caller information
    pub caller: Option<CallerInfo>,
    /// Thread information
    pub thread: Option<ThreadInfo>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl LogRecord {
    /// Creates a new log record.
    pub fn new(level: LogLevel, args: Arguments) -> Self {
        Self {
            level,
            message: format!("{}", args),
            timestamp: Local::now(),
            module: None,
            caller: None,
            thread: None,
            metadata: HashMap::new(),
        }
    }

    /// Sets the module information.
    pub fn with_module<S: Into<String>>(mut self, module: S) -> Self {
        self.module = Some(module.into());
        self
    }

    /// Sets the caller information.
    pub fn with_caller(mut self, caller: CallerInfo) -> Self {
        self.caller = Some(caller);
        self
    }

    /// Sets the thread information.
    pub fn with_thread(mut self, thread: ThreadInfo) -> Self {
        self.thread = Some(thread);
        self
    }

    /// Adds custom metadata.
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Adds multiple metadata entries.
    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }
}

/// Trait for formatting log records.
pub trait Formatter: Send + Sync {
    /// Formats a log record into a string.
    fn format(&self, record: &LogRecord) -> String;

    /// Returns whether this formatter supports colors.
    fn supports_colors(&self) -> bool {
        false
    }
}

/// Text formatter with optional colors.
#[derive(Debug, Clone)]
pub struct TextFormatter {
    /// Whether to include colors in output
    pub colors: bool,
    /// DateTime format string
    pub datetime_format: String,
    /// Whether to include caller information
    pub include_caller: bool,
    /// Whether to include thread information
    pub include_thread: bool,
    /// Whether to include module information
    pub include_module: bool,
}

impl Default for TextFormatter {
    fn default() -> Self {
        Self {
            colors: true,
            datetime_format: "%Y-%m-%d %H:%M:%S".to_string(),
            include_caller: false,
            include_thread: false,
            include_module: false,
        }
    }
}

impl TextFormatter {
    /// Creates a new text formatter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to use colors.
    pub fn with_colors(mut self, colors: bool) -> Self {
        self.colors = colors;
        self
    }

    /// Sets the datetime format.
    pub fn with_datetime_format<S: Into<String>>(mut self, format: S) -> Self {
        self.datetime_format = format.into();
        self
    }

    /// Sets whether to include caller information.
    pub fn with_caller(mut self, include: bool) -> Self {
        self.include_caller = include;
        self
    }

    /// Sets whether to include thread information.
    pub fn with_thread(mut self, include: bool) -> Self {
        self.include_thread = include;
        self
    }

    /// Sets whether to include module information.
    pub fn with_module(mut self, include: bool) -> Self {
        self.include_module = include;
        self
    }
}

impl Formatter for TextFormatter {
    fn format(&self, record: &LogRecord) -> String {
        let timestamp = record.timestamp.format(&self.datetime_format);

        let level_str = if self.colors {
            let color = Colors::for_level(record.level);
            format!("{}{:>7}{}", color, record.level.as_str(), Colors::RESET)
        } else {
            format!("{:>7}", record.level.as_str())
        };

        let mut parts = vec![format!("{}", timestamp), format!("[{}]:", level_str)];

        // Add thread information if requested
        if self.include_thread {
            if let Some(ref thread) = record.thread {
                let thread_info = if let Some(ref name) = thread.name {
                    format!("[{}:{}]", name, thread.id)
                } else {
                    format!("[{}]", thread.id)
                };
                parts.push(thread_info);
            }
        }

        // Add module information if requested
        if self.include_module {
            if let Some(ref module) = record.module {
                parts.push(format!("[{}]", module));
            }
        }

        // Add caller information if requested
        if self.include_caller {
            if let Some(ref caller) = record.caller {
                let caller_info = format!("{}:{}", caller.file, caller.line);
                parts.push(format!("[{}]", caller_info));
            }
        }

        // Add the message
        parts.push(record.message.clone());

        // Add metadata if any
        if !record.metadata.is_empty() {
            let metadata_parts: Vec<String> = record
                .metadata
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            parts.push(format!("[{}]", metadata_parts.join(", ")));
        }

        parts.join(" ")
    }

    fn supports_colors(&self) -> bool {
        self.colors
    }
}

/// JSON formatter for structured logging.
#[derive(Debug, Clone)]
pub struct JsonFormatter {
    /// Whether to pretty-print JSON
    pub pretty: bool,
    /// DateTime format string
    pub datetime_format: String,
    /// Whether to include caller information
    pub include_caller: bool,
    /// Whether to include thread information
    pub include_thread: bool,
    /// Whether to include module information
    pub include_module: bool,
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self {
            pretty: false,
            datetime_format: "%Y-%m-%dT%H:%M:%S%.3fZ".to_string(),
            include_caller: true,
            include_thread: true,
            include_module: true,
        }
    }
}

impl JsonFormatter {
    /// Creates a new JSON formatter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to pretty-print JSON.
    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    /// Sets the datetime format.
    pub fn with_datetime_format<S: Into<String>>(mut self, format: S) -> Self {
        self.datetime_format = format.into();
        self
    }

    /// Sets whether to include caller information.
    pub fn with_caller(mut self, include: bool) -> Self {
        self.include_caller = include;
        self
    }

    /// Sets whether to include thread information.
    pub fn with_thread(mut self, include: bool) -> Self {
        self.include_thread = include;
        self
    }

    /// Sets whether to include module information.
    pub fn with_module(mut self, include: bool) -> Self {
        self.include_module = include;
        self
    }
}

impl Formatter for JsonFormatter {
    fn format(&self, record: &LogRecord) -> String {
        let mut json_obj = json!({
            "timestamp": record.timestamp.format(&self.datetime_format).to_string(),
            "level": record.level.as_str(),
            "message": record.message,
        });

        // Add module information if requested and available
        if self.include_module {
            if let Some(ref module) = record.module {
                json_obj["module"] = json!(module);
            }
        }

        // Add caller information if requested and available
        if self.include_caller {
            if let Some(ref caller) = record.caller {
                json_obj["caller"] = json!({
                    "file": caller.file,
                    "line": caller.line,
                    "module": caller.module,
                });
            }
        }

        // Add thread information if requested and available
        if self.include_thread {
            if let Some(ref thread) = record.thread {
                json_obj["thread"] = json!({
                    "id": thread.id,
                    "name": thread.name,
                });
            }
        }

        // Add custom metadata
        if !record.metadata.is_empty() {
            json_obj["metadata"] = json!(record.metadata);
        }

        if self.pretty {
            serde_json::to_string_pretty(&json_obj).unwrap_or_else(|_| "{}".to_string())
        } else {
            serde_json::to_string(&json_obj).unwrap_or_else(|_| "{}".to_string())
        }
    }
}

/// Plain text formatter without any colors or special formatting.
#[derive(Debug, Clone)]
pub struct PlainFormatter {
    /// DateTime format string
    pub datetime_format: String,
    /// Whether to include caller information
    pub include_caller: bool,
    /// Whether to include thread information
    pub include_thread: bool,
    /// Whether to include module information
    pub include_module: bool,
}

impl Default for PlainFormatter {
    fn default() -> Self {
        Self {
            datetime_format: "%Y-%m-%d %H:%M:%S".to_string(),
            include_caller: false,
            include_thread: false,
            include_module: false,
        }
    }
}

impl PlainFormatter {
    /// Creates a new plain formatter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the datetime format.
    pub fn with_datetime_format<S: Into<String>>(mut self, format: S) -> Self {
        self.datetime_format = format.into();
        self
    }

    /// Sets whether to include caller information.
    pub fn with_caller(mut self, include: bool) -> Self {
        self.include_caller = include;
        self
    }

    /// Sets whether to include thread information.
    pub fn with_thread(mut self, include: bool) -> Self {
        self.include_thread = include;
        self
    }

    /// Sets whether to include module information.
    pub fn with_module(mut self, include: bool) -> Self {
        self.include_module = include;
        self
    }
}

impl Formatter for PlainFormatter {
    fn format(&self, record: &LogRecord) -> String {
        let timestamp = record.timestamp.format(&self.datetime_format);

        let mut parts = vec![
            format!("{}", timestamp),
            format!("[{}]:", record.level.as_str()),
        ];

        // Add thread information if requested
        if self.include_thread {
            if let Some(ref thread) = record.thread {
                let thread_info = if let Some(ref name) = thread.name {
                    format!("[{}:{}]", name, thread.id)
                } else {
                    format!("[{}]", thread.id)
                };
                parts.push(thread_info);
            }
        }

        // Add module information if requested
        if self.include_module {
            if let Some(ref module) = record.module {
                parts.push(format!("[{}]", module));
            }
        }

        // Add caller information if requested
        if self.include_caller {
            if let Some(ref caller) = record.caller {
                let caller_info = format!("{}:{}", caller.file, caller.line);
                parts.push(format!("[{}]", caller_info));
            }
        }

        // Add the message
        parts.push(record.message.clone());

        // Add metadata if any
        if !record.metadata.is_empty() {
            let metadata_parts: Vec<String> = record
                .metadata
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            parts.push(format!("[{}]", metadata_parts.join(", ")));
        }

        parts.join(" ")
    }
}

/// Creates a formatter based on the output format.
pub fn create_formatter(
    format: OutputFormat,
    colors: bool,
    datetime_format: &str,
    include_caller: bool,
    include_thread: bool,
    include_module: bool,
) -> Box<dyn Formatter> {
    match format {
        OutputFormat::Text => Box::new(
            TextFormatter::new()
                .with_colors(colors)
                .with_datetime_format(datetime_format)
                .with_caller(include_caller)
                .with_thread(include_thread)
                .with_module(include_module),
        ),
        OutputFormat::Json => Box::new(
            JsonFormatter::new()
                .with_datetime_format(datetime_format)
                .with_caller(include_caller)
                .with_thread(include_thread)
                .with_module(include_module),
        ),
        OutputFormat::Plain => Box::new(
            PlainFormatter::new()
                .with_datetime_format(datetime_format)
                .with_caller(include_caller)
                .with_thread(include_thread)
                .with_module(include_module),
        ),
    }
}

/// Helper function to get current thread information.
pub fn get_thread_info() -> ThreadInfo {
    let current_thread = std::thread::current();
    ThreadInfo {
        id: format!("{:?}", current_thread.id()),
        name: current_thread.name().map(|s| s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_formatter() {
        let formatter = TextFormatter::new().with_colors(false);
        let record = LogRecord::new(LogLevel::Info, format_args!("Test message"));

        let output = formatter.format(&record);
        assert!(output.contains("[   INFO]:"));
        assert!(output.contains("Test message"));
    }

    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter::new();
        let record = LogRecord::new(LogLevel::Error, format_args!("Error message"));

        let output = formatter.format(&record);
        let parsed: Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["level"], "ERROR");
        assert_eq!(parsed["message"], "Error message");
        assert!(parsed["timestamp"].is_string());
    }

    #[test]
    fn test_plain_formatter() {
        let formatter = PlainFormatter::new();
        let record = LogRecord::new(LogLevel::Warning, format_args!("Warning message"));

        let output = formatter.format(&record);
        assert!(output.contains("[WARNING]:"));
        assert!(output.contains("Warning message"));
        assert!(!output.contains("\x1b")); // No ANSI codes
    }

    #[test]
    fn test_formatter_with_metadata() {
        let formatter = TextFormatter::new().with_colors(false);
        let record = LogRecord::new(LogLevel::Debug, format_args!("Debug message"))
            .with_metadata("user_id", "123")
            .with_metadata("request_id", "abc-def");

        let output = formatter.format(&record);
        assert!(output.contains("Debug message"));
        assert!(output.contains("user_id=123"));
        assert!(output.contains("request_id=abc-def"));
    }

    #[test]
    fn test_formatter_with_caller() {
        let formatter = TextFormatter::new().with_colors(false).with_caller(true);

        let caller = CallerInfo {
            file: "test.rs",
            line: 42,
            module: Some("test_module"),
        };

        let record =
            LogRecord::new(LogLevel::Info, format_args!("Test message")).with_caller(caller);

        let output = formatter.format(&record);
        assert!(output.contains("test.rs:42"));
    }
}
