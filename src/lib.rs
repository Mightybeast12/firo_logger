//! # firo_logger
//!
//! A high-performance, feature-rich logger for Rust applications with colored output,
//! structured logging, file rotation, async logging, and advanced configuration.
//!
//! ## Features
//!
//! - **Colored console output** with customizable colors
//! - **Structured logging** with JSON format support
//! - **File logging** with rotation (size-based and time-based)
//! - **Async logging** for high-performance applications
//! - **Level filtering** with module-specific filters
//! - **Thread-safe** with minimal overhead
//! - **Caller information** (file, line, module)
//! - **Custom metadata** support
//! - **Environment configuration** support
//! - **Builder pattern** for easy configuration
//!
//! ## Quick Start
//!
//! ```rust
//! use firo_logger::{init_default, log_info, log_error, log_success};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the logger with default settings
//!     init_default()?;
//!
//!     // Log some messages
//!     log_info!("Application started").unwrap();
//!     log_success!("Configuration loaded successfully").unwrap();
//!     log_error!("Failed to connect to database: {}", "Connection timeout").unwrap();
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration
//!
//! ```rust
//! use firo_logger::{LoggerConfig, LogLevel, OutputFormat, init, log_info};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = LoggerConfig::builder()
//!         .level(LogLevel::Debug)
//!         .format(OutputFormat::Json)
//!         .console(true)
//!         .colors(true)
//!         .file("app.log")
//!         .rotate_by_size(10 * 1024 * 1024, 5) // 10MB, keep 5 files
//!         .async_logging(1000)
//!         .include_caller(true)
//!         .include_thread(true)
//!         .metadata("app", "my-app")
//!         .metadata("version", "1.0.0")
//!         .build();
//!
//!     init(config)?;
//!
//!     log_info!("Logger initialized with custom configuration").unwrap();
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Environment Configuration
//!
//! The logger can be configured using environment variables:
//!
//! - `FIRO_LOG_LEVEL`: Set log level (ERROR, WARNING, INFO, SUCCESS, DEBUG)
//! - `FIRO_LOG_FILE`: Set log file path
//! - `FIRO_LOG_FORMAT`: Set output format (text, json, plain)
//! - `NO_COLOR`: Disable colored output
//! - `FORCE_COLOR`: Force colored output even when not in a terminal
//!
//! ```rust
//! use firo_logger::{init_from_env, log_info};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     init_from_env()?;
//!     log_info!("Logger configured from environment").unwrap();
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Usage
//!
//! ### Structured Logging with Metadata
//!
//! ```rust
//! use firo_logger::{log_with_metadata, LogLevel};
//!
//! log_with_metadata!(
//!     LogLevel::Info,
//!     "User login",
//!     "user_id" => "12345",
//!     "ip_address" => "192.168.1.100",
//!     "user_agent" => "Mozilla/5.0...",
//! );
//! ```
//!
//! ### Conditional and Rate-Limited Logging
//!
//! ```rust
//! use firo_logger::{log_if, log_rate_limited, LogLevel};
//! use std::time::Duration;
//!
//! let debug_mode = std::env::var("DEBUG").is_ok();
//! log_if!(debug_mode, LogLevel::Debug, "Debug mode is enabled");
//!
//! // Rate-limited logging (max once per second)
//! for i in 0..1000 {
//!     log_rate_limited!(Duration::from_secs(1), LogLevel::Info, "Processing item {}", i);
//! }
//! ```
//!
//! ### Function Tracing
//!
//! ```rust
//! use firo_logger::trace_function;
//!
//! fn process_data(data: &[u8]) -> Result<(), std::io::Error> {
//!     trace_function!("process_data", data.len());
//!     // Function implementation...
//!     Ok(())
//! }
//! ```
//!
//! ### Performance Timing
//!
//! ```rust
//! use firo_logger::{time_block, LogLevel};
//!
//! let result = time_block!(LogLevel::Info, "Database query", {
//!     // Your expensive operation here
//!     std::thread::sleep(std::time::Duration::from_millis(100));
//!     "query result"
//! });
//! ```

// Core modules
pub mod config;
pub mod error;
pub mod formatters;
pub mod logger;
pub mod macros;
pub mod writers;

// Re-export commonly used types and functions
pub use config::{
    Colors, ConsoleConfig, FileConfig, LogLevel, LoggerConfig, LoggerConfigBuilder, OutputFormat,
    RotationConfig, RotationFrequency,
};
pub use error::{LoggerError, Result};
pub use formatters::{CallerInfo, Formatter, LogRecord, ThreadInfo};
pub use logger::{
    config, current_logger, flush, init, init_default, init_from_env, is_initialized, log_debug,
    log_error, log_info, log_success, log_warning, log_with_caller, logger, stats,
    with_scoped_logger, LoggerInstance, LoggerStats,
};
pub use macros::__FunctionTraceGuard;

// Re-export macros - they are automatically available when the crate is used
// due to the #[macro_export] attribute, but we can also make them available
// through the crate root for documentation purposes.

/// Legacy compatibility with the old simple API
pub mod legacy {
    //! Legacy compatibility module for the old firo_logger API.
    //!
    //! This module provides compatibility with the old logger API while
    //! internally using the new improved implementation.

    use crate::{init_default, is_initialized};
    use std::fmt::Arguments;

    /// Legacy Logger struct for compatibility.
    #[deprecated(note = "Use the new firo_logger API instead")]
    pub struct Logger;

    #[allow(deprecated)]
    impl Logger {
        /// Logs a message (legacy compatibility).
        pub fn log(args: Arguments) {
            if !is_initialized() {
                let _ = init_default();
            }
            let _ = crate::log_info!("{}", args);
        }

        /// Logs an error message (legacy compatibility).
        pub fn error(args: Arguments) {
            if !is_initialized() {
                let _ = init_default();
            }
            let _ = crate::log_error!("{}", args);
        }

        /// Logs a warning message (legacy compatibility).
        pub fn warning(args: Arguments) {
            if !is_initialized() {
                let _ = init_default();
            }
            let _ = crate::log_warning!("{}", args);
        }

        /// Logs a debug message (legacy compatibility).
        pub fn debug(args: Arguments) {
            if !is_initialized() {
                let _ = init_default();
            }
            let _ = crate::log_debug!("{}", args);
        }

        /// Logs an info message (legacy compatibility).
        pub fn info(args: Arguments) {
            if !is_initialized() {
                let _ = init_default();
            }
            let _ = crate::log_info!("{}", args);
        }

        /// Logs a success message (legacy compatibility).
        pub fn success(args: Arguments) {
            if !is_initialized() {
                let _ = init_default();
            }
            let _ = crate::log_success!("{}", args);
        }
    }

    /// Legacy Colors struct for compatibility.
    #[deprecated(note = "Use firo_logger::Colors instead")]
    pub struct Colours;

    #[allow(deprecated)]
    impl Colours {
        pub const RED: &'static str = "\x1b[31m";
        pub const GREEN: &'static str = "\x1b[32m";
        pub const YELLOW: &'static str = "\x1b[33m";
        pub const BLUE: &'static str = "\x1b[34m";
        pub const CYAN: &'static str = "\x1b[36m";
        pub const WHITE: &'static str = "\x1b[37m";
    }

    /// Legacy LogLevel enum for compatibility.
    #[deprecated(note = "Use firo_logger::LogLevel instead")]
    #[derive(Debug, PartialEq)]
    pub enum LogLevel {
        Error,
        Warning,
        Debug,
        Success,
        Info,
        Log,
    }

    #[allow(deprecated)]
    impl LogLevel {
        #[allow(dead_code)]
        fn as_str(&self) -> &'static str {
            match self {
                LogLevel::Error => "ERROR",
                LogLevel::Warning => "WARNING",
                LogLevel::Debug => "DEBUG",
                LogLevel::Success => "SUCCESS",
                LogLevel::Info => "INFO",
                LogLevel::Log => "LOG",
            }
        }
    }
}

// Integration with the standard `log` crate (optional feature)
#[cfg(feature = "log")]
pub mod log_integration {
    //! Integration with the standard `log` crate.
    //!
    //! This module provides a bridge to use firo_logger as a backend
    //! for the standard `log` crate.

    use crate::{init_default, is_initialized, LogLevel};
    use log::{Level, Metadata, Record};

    /// A log implementation that forwards to firo_logger.
    pub struct FiroLoggerAdapter;

    impl log::Log for FiroLoggerAdapter {
        fn enabled(&self, metadata: &Metadata) -> bool {
            // Enable all log levels - firo_logger will handle filtering
            true
        }

        fn log(&self, record: &Record) {
            if !is_initialized() {
                let _ = init_default();
            }

            let level = match record.level() {
                Level::Error => LogLevel::Error,
                Level::Warn => LogLevel::Warning,
                Level::Info => LogLevel::Info,
                Level::Debug => LogLevel::Debug,
                Level::Trace => LogLevel::Debug,
            };

            let module = record.module_path();
            let file = record.file().unwrap_or("<unknown>");
            let line = record.line().unwrap_or(0);

            let caller = crate::CallerInfo { file, line, module };

            let _ = crate::log_with_caller(level, *record.args(), Some(caller), module);
        }

        fn flush(&self) {
            let _ = crate::flush();
        }
    }

    /// Initialize firo_logger as the global logger for the `log` crate.
    pub fn init_with_log() -> Result<(), crate::LoggerError> {
        init_default()?;
        log::set_boxed_logger(Box::new(FiroLoggerAdapter))
            .map_err(|_| crate::LoggerError::AlreadyInitialized)?;
        log::set_max_level(log::LevelFilter::Trace);
        Ok(())
    }
}

/// Utility functions and helpers.
pub mod utils {
    //! Utility functions for common logging patterns.

    use crate::LogLevel;
    use std::fmt::Arguments;
    use std::time::{Duration, Instant};

    /// Logs execution time of a closure.
    pub fn log_execution_time<F, R>(level: LogLevel, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        let _ = crate::log!(level, "{} completed in {:?}", name, duration);
        result
    }

    /// Creates a scoped logger that adds a prefix to all log messages.
    pub struct ScopedLogger {
        prefix: String,
    }

    impl ScopedLogger {
        /// Creates a new scoped logger with the given prefix.
        pub fn new<S: Into<String>>(prefix: S) -> Self {
            Self {
                prefix: prefix.into(),
            }
        }

        /// Logs a message with the scoped prefix.
        pub fn log(&self, level: LogLevel, args: Arguments) {
            let _ = crate::log!(level, "[{}] {}", self.prefix, args);
        }

        /// Logs an error message.
        pub fn error(&self, args: Arguments) {
            self.log(LogLevel::Error, args);
        }

        /// Logs a warning message.
        pub fn warning(&self, args: Arguments) {
            self.log(LogLevel::Warning, args);
        }

        /// Logs an info message.
        pub fn info(&self, args: Arguments) {
            self.log(LogLevel::Info, args);
        }

        /// Logs a success message.
        pub fn success(&self, args: Arguments) {
            self.log(LogLevel::Success, args);
        }

        /// Logs a debug message.
        pub fn debug(&self, args: Arguments) {
            self.log(LogLevel::Debug, args);
        }
    }

    /// Helper for rate limiting log messages.
    pub struct RateLimiter {
        last_log: std::sync::Mutex<Option<Instant>>,
        interval: Duration,
    }

    impl RateLimiter {
        /// Creates a new rate limiter with the specified interval.
        pub fn new(interval: Duration) -> Self {
            Self {
                last_log: std::sync::Mutex::new(None),
                interval,
            }
        }

        /// Attempts to log a message, respecting the rate limit.
        pub fn log(&self, level: LogLevel, args: Arguments) -> bool {
            let now = Instant::now();
            let mut last_log = self.last_log.lock().unwrap();

            let should_log = match *last_log {
                Some(last) => now.duration_since(last) >= self.interval,
                None => true,
            };

            if should_log {
                *last_log = Some(now);
                let _ = crate::log!(level, "{}", args);
                true
            } else {
                false
            }
        }
    }
}

// Tests for the main API
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_test_logger() {
        INIT.call_once(|| {
            let config = LoggerConfig::builder()
                .console(true)
                .colors(false)
                .level(LogLevel::Debug)
                .build();
            let _ = init(config);
        });
    }

    #[test]
    fn test_basic_api() {
        init_test_logger();

        assert!(log_error!("Test error").is_ok());
        assert!(log_warning!("Test warning").is_ok());
        assert!(log_info!("Test info").is_ok());
        assert!(log_success!("Test success").is_ok());
        assert!(log_debug!("Test debug").is_ok());
    }

    #[test]
    fn test_formatted_logging() {
        init_test_logger();

        let user = "alice";
        let count = 42;

        assert!(log_info!("User {} processed {} items", user, count).is_ok());
        assert!(log_error!("Error code: {}", 500).is_ok());
    }

    #[test]
    #[allow(deprecated)]
    fn test_legacy_static_functions() {
        // Test static functions (legacy API)
        legacy::Logger::info(format_args!("Legacy info"));
        legacy::Logger::error(format_args!("Legacy error"));
        legacy::Logger::log(format_args!("Legacy log"));
        legacy::Logger::warning(format_args!("Legacy warning"));
    }

    #[test]
    fn test_config_builder() {
        let config = LoggerConfig::builder()
            .level(LogLevel::Debug)
            .console(true)
            .colors(false)
            .file("test.log")
            .format(OutputFormat::Json)
            .include_caller(true)
            .include_thread(true)
            .metadata("test", "value")
            .build();

        assert_eq!(config.level, LogLevel::Debug);
        assert!(config.console_enabled);
        assert!(!config.console.colors);
        assert!(config.file_enabled);
        assert_eq!(config.format, OutputFormat::Json);
        assert!(config.include_caller);
        assert!(config.include_thread);
        assert_eq!(config.metadata.get("test"), Some(&"value".to_string()));
    }

    #[test]
    fn test_level_filtering() {
        let config = LoggerConfig::builder()
            .level(LogLevel::Warning)
            .console(true)
            .colors(false)
            .build();

        let logger = LoggerInstance::new(config).unwrap();

        // These should succeed (but may not actually log due to level filtering)
        assert!(logger.error(format_args!("Error")).is_ok());
        assert!(logger.warning(format_args!("Warning")).is_ok());
        assert!(logger.info(format_args!("Info")).is_ok());
        assert!(logger.debug(format_args!("Debug")).is_ok());
    }

    #[test]
    fn test_utils() {
        use utils::*;

        let result = log_execution_time(LogLevel::Info, "test operation", || {
            std::thread::sleep(std::time::Duration::from_millis(1));
            42
        });

        assert_eq!(result, 42);

        let scoped = ScopedLogger::new("TEST");
        scoped.info(format_args!("Scoped message"));

        let rate_limiter = RateLimiter::new(std::time::Duration::from_millis(100));
        assert!(rate_limiter.log(LogLevel::Info, format_args!("Rate limited")));
    }
}
