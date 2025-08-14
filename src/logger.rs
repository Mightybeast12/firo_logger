//! Core logger implementation with async support and dual singleton/instance pattern.

use crate::config::{LogLevel, LoggerConfig};
use crate::error::{LoggerError, Result};
use crate::formatters::{create_formatter, get_thread_info, CallerInfo, LogRecord};
use crate::writers::{ConsoleWriter, FileWriter, MultiWriter, Writer};
#[cfg(feature = "async")]
use crossbeam_channel::{unbounded, Receiver, Sender};
use once_cell::sync::OnceCell;
use parking_lot::{Mutex, RwLock};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Arguments;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

/// Global logger instance.
static GLOBAL_LOGGER: OnceCell<Arc<LoggerInstance>> = OnceCell::new();

// Thread-local logger storage for scoped logging.
thread_local! {
    static THREAD_LOCAL_LOGGER: RefCell<Option<Arc<LoggerInstance>>> = const { RefCell::new(None) };
}

/// Sets a scoped logger for the current thread.
pub fn with_scoped_logger<F, R>(logger: Arc<LoggerInstance>, f: F) -> R
where
    F: FnOnce() -> R,
{
    THREAD_LOCAL_LOGGER.with(|tl| {
        let previous = tl.replace(Some(logger));
        let result = f();
        tl.replace(previous);
        result
    })
}

/// Gets the current logger (thread-local first, then global, then auto-initialize).
pub fn current_logger() -> Result<Arc<LoggerInstance>> {
    // First check for thread-local logger
    THREAD_LOCAL_LOGGER.with(|tl| {
        if let Some(logger) = tl.borrow().as_ref() {
            return Ok(Arc::clone(logger));
        }

        // Fall back to global logger
        if let Some(logger) = GLOBAL_LOGGER.get() {
            return Ok(Arc::clone(logger));
        }

        // If no logger is available, try to initialize a default one
        // This provides backward compatibility for tests and simple usage
        if let Ok(()) = init_default() {
            // If initialization succeeded, get the global logger
            return Ok(Arc::clone(GLOBAL_LOGGER.get().unwrap()));
        }

        // If initialization failed (already initialized by another thread), try again
        if let Some(logger) = GLOBAL_LOGGER.get() {
            return Ok(Arc::clone(logger));
        }

        Err(LoggerError::NotInitialized)
    })
}

/// Async log message for the background thread.
#[cfg(feature = "async")]
#[derive(Debug)]
struct AsyncLogMessage {
    record: LogRecord,
    #[allow(dead_code)]
    caller: Option<CallerInfo>,
    #[allow(dead_code)]
    module: Option<String>,
}

/// The main logger instance structure.
/// Each instance is independent and can have its own configuration.
pub struct LoggerInstance {
    /// Logger configuration
    config: RwLock<LoggerConfig>,
    /// Writer for output
    writer: Mutex<Box<dyn Writer>>,
    /// Async channel sender (if async is enabled)
    #[cfg(feature = "async")]
    async_sender: Option<Sender<AsyncLogMessage>>,
    /// Background thread handle (if async is enabled)
    #[cfg(feature = "async")]
    _async_handle: Option<JoinHandle<()>>,
    /// Statistics
    stats: Mutex<LoggerStats>,
}

/// Logger statistics.
#[derive(Debug, Default, Clone)]
pub struct LoggerStats {
    /// Total number of log messages
    pub total_messages: u64,
    /// Messages by level
    pub messages_by_level: HashMap<LogLevel, u64>,
    /// Logger start time
    pub start_time: Option<SystemTime>,
    /// Number of errors during logging
    pub error_count: u64,
}

impl LoggerInstance {
    /// Creates a new logger with the given configuration.
    pub fn new(config: LoggerConfig) -> Result<Self> {
        config.validate()?;

        // Create writers based on configuration
        let mut multi_writer = MultiWriter::new();

        // Add console writer if enabled
        if config.console_enabled {
            let formatter = create_formatter(
                config.format,
                config.console.colors,
                &config.datetime_format,
                config.include_caller,
                config.include_thread,
                true, // Always include module for console
            );
            let console_writer = ConsoleWriter::new(config.console.use_stderr, formatter);
            multi_writer = multi_writer.add_writer(Box::new(console_writer));
        }

        // Add file writer if enabled
        if config.file_enabled {
            let formatter = create_formatter(
                config.format,
                false, // File output should not have colors
                &config.datetime_format,
                config.include_caller,
                config.include_thread,
                true, // Always include module for file
            );
            let file_writer = FileWriter::new(config.file.clone(), formatter)?;
            multi_writer = multi_writer.add_writer(Box::new(file_writer));
        }

        #[cfg(feature = "async")]
        let (async_sender, async_handle) = if config.async_enabled {
            let (sender, receiver) = unbounded();
            // Create a separate multi_writer for the async thread
            let mut async_multi_writer = MultiWriter::new();

            // Add console writer if enabled
            if config.console_enabled {
                let formatter = create_formatter(
                    config.format,
                    config.console.colors,
                    &config.datetime_format,
                    config.include_caller,
                    config.include_thread,
                    true,
                );
                let console_writer = ConsoleWriter::new(config.console.use_stderr, formatter);
                async_multi_writer = async_multi_writer.add_writer(Box::new(console_writer));
            }

            // Add file writer if enabled
            if config.file_enabled {
                let formatter = create_formatter(
                    config.format,
                    false,
                    &config.datetime_format,
                    config.include_caller,
                    config.include_thread,
                    true,
                );
                let file_writer = FileWriter::new(config.file.clone(), formatter)?;
                async_multi_writer = async_multi_writer.add_writer(Box::new(file_writer));
            }

            let writer_clone = Box::new(async_multi_writer);
            let handle = Self::start_async_thread(receiver, writer_clone)?;
            (Some(sender), Some(handle))
        } else {
            (None, None)
        };

        let stats = LoggerStats {
            start_time: Some(SystemTime::now()),
            ..Default::default()
        };

        Ok(LoggerInstance {
            config: RwLock::new(config),
            writer: Mutex::new(Box::new(multi_writer)),
            #[cfg(feature = "async")]
            async_sender,
            #[cfg(feature = "async")]
            _async_handle: async_handle,
            stats: Mutex::new(stats),
        })
    }

    /// Starts the async logging thread.
    #[cfg(feature = "async")]
    fn start_async_thread(
        receiver: Receiver<AsyncLogMessage>,
        mut writer: Box<dyn Writer>,
    ) -> Result<JoinHandle<()>> {
        let handle = thread::Builder::new()
            .name("firo-logger-async".to_string())
            .spawn(move || {
                let mut last_flush = SystemTime::now();
                const FLUSH_INTERVAL: Duration = Duration::from_millis(100);

                loop {
                    // Process messages with timeout to allow periodic flushing
                    match receiver.recv_timeout(FLUSH_INTERVAL) {
                        Ok(msg) => {
                            let formatted = {
                                let config = LoggerConfig::default(); // TODO: Pass config properly
                                let formatter = create_formatter(
                                    config.format,
                                    false,
                                    &config.datetime_format,
                                    config.include_caller,
                                    config.include_thread,
                                    true,
                                );
                                formatter.format(&msg.record)
                            };

                            if writer.write(&msg.record, &formatted).is_err() {
                                // Log errors are silently ignored in async mode
                                // to prevent infinite loops
                            }
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                            // Timeout - flush if needed
                            if last_flush.elapsed().unwrap_or(Duration::ZERO) >= FLUSH_INTERVAL {
                                let _ = writer.flush();
                                last_flush = SystemTime::now();
                            }
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                            // Channel closed - flush and exit
                            let _ = writer.flush();
                            break;
                        }
                    }
                }
            })?;

        Ok(handle)
    }

    /// Logs a message with the given level.
    pub fn log_with_caller(
        &self,
        level: LogLevel,
        args: Arguments,
        caller: Option<CallerInfo>,
        module: Option<&str>,
    ) -> Result<()> {
        // Clone caller early to avoid borrow issues
        let caller_clone = caller.clone();

        let config = self.config.read();

        // Check if this message should be logged based on level and module filters
        let effective_level = if let Some(module_name) = module {
            config.effective_level(module_name)
        } else {
            config.level
        };

        if level > effective_level {
            return Ok(());
        }

        // Create log record
        let mut record = LogRecord::new(level, args);

        // Add module information
        if let Some(module_name) = module {
            record = record.with_module(module_name);
        }

        // Add caller information
        if let Some(caller_info) = &caller_clone {
            record = record.with_caller(caller_info.clone());
        }

        // Add thread information if enabled
        if config.include_thread {
            record = record.with_thread(get_thread_info());
        }

        // Add global metadata
        record = record.with_metadata_map(config.metadata.clone());

        // Update statistics
        {
            let mut stats = self.stats.lock();
            stats.total_messages += 1;
            *stats.messages_by_level.entry(level).or_insert(0) += 1;
        }

        drop(config); // Release read lock early

        // Handle async vs sync logging
        #[cfg(feature = "async")]
        if let Some(ref sender) = self.async_sender {
            let async_msg = AsyncLogMessage {
                record,
                caller: caller_clone,
                module: module.map(|s| s.to_string()),
            };

            sender.send(async_msg).map_err(|_| {
                let mut stats = self.stats.lock();
                stats.error_count += 1;
                LoggerError::Channel("Failed to send message to async thread".to_string())
            })?;
        } else {
            // Synchronous logging
            let config = self.config.read();
            let formatter = create_formatter(
                config.format,
                config.console.colors,
                &config.datetime_format,
                config.include_caller,
                config.include_thread,
                true,
            );
            let formatted = formatter.format(&record);
            drop(config);

            let mut writer = self.writer.lock();
            writer.write(&record, &formatted).inspect_err(|_e| {
                let mut stats = self.stats.lock();
                stats.error_count += 1;
            })?;
        }

        #[cfg(not(feature = "async"))]
        {
            // Synchronous logging only
            let config = self.config.read();
            let formatter = create_formatter(
                config.format,
                config.console.colors,
                &config.datetime_format,
                config.include_caller,
                config.include_thread,
                true,
            );
            let formatted = formatter.format(&record);
            drop(config);

            let mut writer = self.writer.lock();
            writer.write(&record, &formatted).map_err(|e| {
                let mut stats = self.stats.lock();
                stats.error_count += 1;
                e
            })?;
        }

        Ok(())
    }

    /// Logs a message without caller information.
    pub fn log(&self, level: LogLevel, args: Arguments) -> Result<()> {
        self.log_with_caller(level, args, None, None)
    }

    /// Logs an error message.
    pub fn error(&self, args: Arguments) -> Result<()> {
        self.log(LogLevel::Error, args)
    }

    /// Logs a warning message.
    pub fn warning(&self, args: Arguments) -> Result<()> {
        self.log(LogLevel::Warning, args)
    }

    /// Logs an info message.
    pub fn info(&self, args: Arguments) -> Result<()> {
        self.log(LogLevel::Info, args)
    }

    /// Logs a success message.
    pub fn success(&self, args: Arguments) -> Result<()> {
        self.log(LogLevel::Success, args)
    }

    /// Logs a debug message.
    pub fn debug(&self, args: Arguments) -> Result<()> {
        self.log(LogLevel::Debug, args)
    }

    /// Flushes all writers.
    pub fn flush(&self) -> Result<()> {
        #[cfg(feature = "async")]
        if self.async_sender.is_some() {
            // For async logging, we can't directly flush the async thread
            // The thread handles flushing automatically
            return Ok(());
        }

        let mut writer = self.writer.lock();
        writer.flush()
    }

    /// Gets the current configuration.
    pub fn config(&self) -> LoggerConfig {
        self.config.read().clone()
    }

    /// Updates the logger configuration.
    pub fn update_config(&self, new_config: LoggerConfig) -> Result<()> {
        new_config.validate()?;

        // Note: This is a simplified implementation.
        // A full implementation would recreate writers and async threads
        // if the configuration changes significantly.
        *self.config.write() = new_config;
        Ok(())
    }

    /// Gets logger statistics.
    pub fn stats(&self) -> LoggerStats {
        self.stats.lock().clone()
    }

    /// Resets logger statistics.
    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock();
        *stats = LoggerStats {
            start_time: Some(SystemTime::now()),
            ..Default::default()
        };
    }
}

impl Drop for LoggerInstance {
    fn drop(&mut self) {
        // Flush any remaining logs
        let _ = self.flush();

        // Close async channel if it exists
        #[cfg(feature = "async")]
        if let Some(sender) = &self.async_sender {
            let _ = sender;
        }
    }
}

/// Initializes the global logger with the given configuration.
pub fn init(config: LoggerConfig) -> Result<()> {
    let logger = Arc::new(LoggerInstance::new(config)?);
    GLOBAL_LOGGER
        .set(logger)
        .map_err(|_| LoggerError::AlreadyInitialized)?;
    Ok(())
}

/// Initializes the global logger with default configuration.
pub fn init_default() -> Result<()> {
    init(LoggerConfig::default())
}

/// Initializes the global logger from environment variables.
pub fn init_from_env() -> Result<()> {
    init(LoggerConfig::from_env())
}

/// Gets the global logger instance.
pub fn logger() -> Result<&'static Arc<LoggerInstance>> {
    GLOBAL_LOGGER.get().ok_or(LoggerError::NotInitialized)
}

/// Checks if the logger is initialized.
pub fn is_initialized() -> bool {
    GLOBAL_LOGGER.get().is_some()
}

/// Logs an error message using the current logger (scoped or global).
pub fn log_error(args: Arguments) -> Result<()> {
    current_logger()?.error(args)
}

/// Logs a warning message using the current logger (scoped or global).
pub fn log_warning(args: Arguments) -> Result<()> {
    current_logger()?.warning(args)
}

/// Logs an info message using the current logger (scoped or global).
pub fn log_info(args: Arguments) -> Result<()> {
    current_logger()?.info(args)
}

/// Logs a success message using the current logger (scoped or global).
pub fn log_success(args: Arguments) -> Result<()> {
    current_logger()?.success(args)
}

/// Logs a debug message using the current logger (scoped or global).
pub fn log_debug(args: Arguments) -> Result<()> {
    current_logger()?.debug(args)
}

/// Logs a message with caller information using the current logger (scoped or global).
pub fn log_with_caller(
    level: LogLevel,
    args: Arguments,
    caller: Option<CallerInfo>,
    module: Option<&str>,
) -> Result<()> {
    current_logger()?.log_with_caller(level, args, caller, module)
}

/// Flushes the current logger (scoped or global).
pub fn flush() -> Result<()> {
    current_logger()?.flush()
}

/// Gets the current logger configuration (scoped or global).
pub fn config() -> Result<LoggerConfig> {
    Ok(current_logger()?.config())
}

/// Gets the current logger statistics (scoped or global).
pub fn stats() -> Result<LoggerStats> {
    Ok(current_logger()?.stats())
}

/// Convenience functions for common log levels with automatic caller detection.
/// These would typically be used through macros.
///
/// Implementation detail for macros - logs with caller information.
#[doc(hidden)]
pub fn __log_with_location(
    level: LogLevel,
    args: Arguments,
    file: &'static str,
    line: u32,
    module: Option<&'static str>,
) -> Result<()> {
    let caller = CallerInfo { file, line, module };
    log_with_caller(level, args, Some(caller), module)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_logger_creation() {
        let config = LoggerConfig::default();
        let logger = LoggerInstance::new(config).unwrap();

        assert!(logger.info(format_args!("Test message")).is_ok());
        assert!(logger.flush().is_ok());
    }

    #[test]
    fn test_global_logger_init() {
        // Use a different logger instance for testing
        let config = LoggerConfig::builder()
            .level(LogLevel::Debug)
            .console(true)
            .colors(false)
            .build();

        assert!(init(config).is_ok());
        assert!(is_initialized());

        assert!(log_info(format_args!("Global logger test")).is_ok());
        assert!(flush().is_ok());
    }

    #[test]
    fn test_level_filtering() -> Result<()> {
        let config = LoggerConfig::builder()
            .level(LogLevel::Warning)
            .console(true)
            .colors(false)
            .build();

        let logger = LoggerInstance::new(config)?;

        // Should log error and warning
        assert!(logger.error(format_args!("Error message")).is_ok());
        assert!(logger.warning(format_args!("Warning message")).is_ok());

        // Should not log info, success, or debug (but won't error)
        assert!(logger.info(format_args!("Info message")).is_ok());
        assert!(logger.success(format_args!("Success message")).is_ok());
        assert!(logger.debug(format_args!("Debug message")).is_ok());

        Ok(())
    }

    #[test]
    fn test_file_logging() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let config = LoggerConfig::builder()
            .console(false)
            .file(temp_file.path())
            .build();

        let logger = LoggerInstance::new(config)?;
        logger.info(format_args!("File test message"))?;
        logger.flush()?;

        let content = std::fs::read_to_string(temp_file.path())?;
        assert!(content.contains("File test message"));

        Ok(())
    }

    #[test]
    fn test_async_logging() -> Result<()> {
        let config = LoggerConfig::builder()
            .console(true)
            .colors(false)
            .async_logging(100)
            .build();

        let logger = LoggerInstance::new(config)?;

        // Log several messages
        for i in 0..10 {
            logger.info(format_args!("Async message {i}"))?;
        }

        // Give async thread time to process
        std::thread::sleep(Duration::from_millis(50));

        Ok(())
    }

    #[test]
    fn test_logger_stats() -> Result<()> {
        let config = LoggerConfig::builder().console(true).colors(false).build();

        let logger = LoggerInstance::new(config)?;

        logger.error(format_args!("Error"))?;
        logger.warning(format_args!("Warning"))?;
        logger.info(format_args!("Info"))?;

        let stats = logger.stats();
        assert_eq!(stats.total_messages, 3);
        assert_eq!(stats.messages_by_level.get(&LogLevel::Error), Some(&1));
        assert_eq!(stats.messages_by_level.get(&LogLevel::Warning), Some(&1));
        assert_eq!(stats.messages_by_level.get(&LogLevel::Info), Some(&1));

        Ok(())
    }

    #[test]
    fn test_module_filtering() -> Result<()> {
        let mut config = LoggerConfig::builder()
            .level(LogLevel::Warning)
            .console(true)
            .colors(false)
            .build();

        // Allow debug logs for specific module
        config
            .module_filters
            .insert("test_module".to_string(), LogLevel::Debug);

        let logger = LoggerInstance::new(config)?;

        // Should log debug for specific module
        assert!(logger
            .log_with_caller(
                LogLevel::Debug,
                format_args!("Debug in test_module"),
                None,
                Some("test_module")
            )
            .is_ok());

        // Should not log debug for other modules
        assert!(logger
            .log_with_caller(
                LogLevel::Debug,
                format_args!("Debug in other_module"),
                None,
                Some("other_module")
            )
            .is_ok());

        Ok(())
    }
}
