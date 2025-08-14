//! Convenient macros for logging.

/// Logs an error message.
///
/// # Examples
///
/// ```
/// use firo_logger::log_error;
///
/// log_error!("This is an error message");
/// log_error!("Error processing user {}: {}", user_id, error);
/// ```
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::__log_with_location(
            $crate::config::LogLevel::Error,
            format_args!($($arg)*),
            file!(),
            line!(),
            Some(module_path!())
        )
    };
}

/// Logs a warning message.
///
/// # Examples
///
/// ```
/// use firo_logger::log_warning;
///
/// log_warning!("This is a warning message");
/// log_warning!("Warning: {} attempts remaining", attempts);
/// ```
#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {
        $crate::logger::__log_with_location(
            $crate::config::LogLevel::Warning,
            format_args!($($arg)*),
            file!(),
            line!(),
            Some(module_path!())
        )
    };
}

/// Logs an info message.
///
/// # Examples
///
/// ```
/// use firo_logger::log_info;
///
/// log_info!("Application started");
/// log_info!("Processing {} items", item_count);
/// ```
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::__log_with_location(
            $crate::config::LogLevel::Info,
            format_args!($($arg)*),
            file!(),
            line!(),
            Some(module_path!())
        )
    };
}

/// Logs a success message.
///
/// # Examples
///
/// ```
/// use firo_logger::log_success;
///
/// log_success!("Operation completed successfully");
/// log_success!("Successfully processed {} records", count);
/// ```
#[macro_export]
macro_rules! log_success {
    ($($arg:tt)*) => {
        $crate::logger::__log_with_location(
            $crate::config::LogLevel::Success,
            format_args!($($arg)*),
            file!(),
            line!(),
            Some(module_path!())
        )
    };
}

/// Logs a debug message.
///
/// # Examples
///
/// ```
/// use firo_logger::log_debug;
///
/// log_debug!("Debug information");
/// log_debug!("Variable value: {:?}", some_variable);
/// ```
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::__log_with_location(
            $crate::config::LogLevel::Debug,
            format_args!($($arg)*),
            file!(),
            line!(),
            Some(module_path!())
        )
    };
}

/// Logs a message with a specific level.
///
/// # Examples
///
/// ```
/// use firo_logger::{log, LogLevel};
///
/// log!(LogLevel::Error, "This is an error");
/// log!(LogLevel::Info, "User {} logged in", username);
/// ```
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        $crate::logger::__log_with_location(
            $level,
            format_args!($($arg)*),
            file!(),
            line!(),
            Some(module_path!())
        )
    };
}

/// Logs a message with metadata.
///
/// This macro allows you to attach key-value metadata to a log message.
///
/// # Examples
///
/// ```
/// use firo_logger::{log_with_metadata, LogLevel};
///
/// log_with_metadata!(
///     LogLevel::Info,
///     "User action performed",
///     "user_id" => "12345",
///     "action" => "login",
///     "ip" => "192.168.1.1"
/// );
/// ```
#[macro_export]
macro_rules! log_with_metadata {
    ($level:expr, $message:expr, $($key:expr => $value:expr),+ $(,)?) => {
        {
            use $crate::formatters::LogRecord;
            use $crate::formatters::CallerInfo;

            let caller = CallerInfo {
                file: file!(),
                line: line!(),
                module: Some(module_path!()),
            };

            let mut record = LogRecord::new($level, format_args!("{}", $message));
            record = record.with_module(module_path!());
            record = record.with_caller(caller);

            $(
                record = record.with_metadata($key, $value);
            )+

            if let Ok(logger) = $crate::logger::logger() {
                let config = logger.config();
                let formatter = $crate::formatters::create_formatter(
                    config.format,
                    config.console.colors,
                    &config.datetime_format,
                    config.include_caller,
                    config.include_thread,
                    true,
                );
                let formatted = formatter.format(&record);

                let mut writer = logger.writer.lock();
                let _ = writer.write(&record, &formatted);
            }
        }
    };
}

/// Conditional logging macro that only logs if a condition is true.
///
/// # Examples
///
/// ```
/// use firo_logger::{log_if, LogLevel};
///
/// let debug_mode = true;
/// log_if!(debug_mode, LogLevel::Debug, "Debug mode is enabled");
///
/// let error_occurred = check_for_errors();
/// log_if!(error_occurred, LogLevel::Error, "An error occurred during processing");
/// ```
#[macro_export]
macro_rules! log_if {
    ($condition:expr, $level:expr, $($arg:tt)*) => {
        if $condition {
            $crate::log!($level, $($arg)*);
        }
    };
}

/// Logs an error and returns early with the error.
///
/// This is useful for error handling where you want to log the error
/// and return it at the same time.
///
/// # Examples
///
/// ```
/// use firo_logger::log_error_and_return;
///
/// fn process_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
///     let file = std::fs::File::open(path)
///         .map_err(|e| log_error_and_return!("Failed to open file {}: {}", path, e))?;
///     // ... rest of processing
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! log_error_and_return {
    ($($arg:tt)*) => {
        {
            let _ = $crate::log_error!($($arg)*);
            return Err($crate::error::LoggerError::Custom(format!($($arg)*)).into());
        }
    };
}

/// Times a block of code and logs the execution time.
///
/// # Examples
///
/// ```
/// use firo_logger::{time_block, LogLevel};
///
/// time_block!(LogLevel::Info, "Database query", {
///     // Your code here
///     std::thread::sleep(std::time::Duration::from_millis(100));
/// });
/// ```
#[macro_export]
macro_rules! time_block {
    ($level:expr, $name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        let _ = $crate::log!($level, "{} completed in {:?}", $name, duration);
        result
    }};
}

/// Logs entry and exit of a function.
///
/// This macro is useful for tracing function calls in debug mode.
///
/// # Examples
///
/// ```
/// use firo_logger::trace_function;
///
/// fn my_function(param: i32) -> i32 {
///     trace_function!("my_function", param);
///     // Function implementation
///     param * 2
/// }
/// ```
#[macro_export]
macro_rules! trace_function {
    ($func_name:expr) => {
        let _ = $crate::log_debug!("Entering {}", $func_name);
        let _guard = $crate::__FunctionTraceGuard::new($func_name);
    };
    ($func_name:expr, $($arg:expr),+ $(,)?) => {
        let _ = $crate::log_debug!("Entering {} with args: {:?}", $func_name, ($($arg,)+));
        let _guard = $crate::__FunctionTraceGuard::new($func_name);
    };
}

/// Helper struct for function tracing.
#[doc(hidden)]
pub struct __FunctionTraceGuard {
    func_name: &'static str,
}

impl __FunctionTraceGuard {
    #[doc(hidden)]
    pub fn new(func_name: &'static str) -> Self {
        Self { func_name }
    }
}

impl Drop for __FunctionTraceGuard {
    fn drop(&mut self) {
        let _ = crate::log_debug!("Exiting {}", self.func_name);
    }
}

/// Logs once per program execution.
///
/// This is useful for logging messages that should only appear once,
/// even if the code path is executed multiple times.
///
/// # Examples
///
/// ```
/// use firo_logger::{log_once, LogLevel};
///
/// for i in 0..10 {
///     log_once!(LogLevel::Warning, "This warning will only appear once");
/// }
/// ```
#[macro_export]
macro_rules! log_once {
    ($level:expr, $($arg:tt)*) => {
        {
            use std::sync::Once;
            static ONCE: Once = Once::new();
            ONCE.call_once(|| {
                let _ = $crate::log!($level, $($arg)*);
            });
        }
    };
}

/// Logs at most N times per program execution.
///
/// This is useful for limiting spam from code paths that are executed frequently.
///
/// # Examples
///
/// ```
/// use firo_logger::{log_at_most, LogLevel};
///
/// for i in 0..100 {
///     log_at_most!(3, LogLevel::Warning, "This will only log 3 times");
/// }
/// ```
#[macro_export]
macro_rules! log_at_most {
    ($max_times:expr, $level:expr, $($arg:tt)*) => {
        {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static COUNTER: AtomicUsize = AtomicUsize::new(0);

            let count = COUNTER.fetch_add(1, Ordering::Relaxed);
            if count < $max_times {
                let _ = $crate::log!($level, $($arg)*);
            }
        }
    };
}

/// Logs with a specific rate limit (maximum once per duration).
///
/// # Examples
///
/// ```
/// use firo_logger::{log_rate_limited, LogLevel};
/// use std::time::Duration;
///
/// // In a loop that runs frequently
/// for i in 0..1000 {
///     log_rate_limited!(
///         Duration::from_secs(1),
///         LogLevel::Info,
///         "Processing item {}", i
///     );
/// }
/// ```
#[macro_export]
macro_rules! log_rate_limited {
    ($duration:expr, $level:expr, $($arg:tt)*) => {
        {
            use std::sync::Mutex;
            use std::time::{Instant, Duration};

            static LAST_LOG: Mutex<Option<Instant>> = Mutex::new(None);

            let now = Instant::now();
            let mut last_log = LAST_LOG.lock().unwrap();

            let should_log = match *last_log {
                Some(last) => now.duration_since(last) >= $duration,
                None => true,
            };

            if should_log {
                *last_log = Some(now);
                let _ = $crate::log!($level, $($arg)*);
            }
        }
    };
}

/// Assert macro that logs the assertion failure.
///
/// # Examples
///
/// ```
/// use firo_logger::log_assert;
///
/// let x = 5;
/// log_assert!(x > 0, "x should be positive, got {}", x);
/// ```
#[macro_export]
macro_rules! log_assert {
    ($condition:expr) => {
        if !$condition {
            let _ = $crate::log_error!("Assertion failed: {}", stringify!($condition));
            panic!("Assertion failed: {}", stringify!($condition));
        }
    };
    ($condition:expr, $($arg:tt)*) => {
        if !$condition {
            let _ = $crate::log_error!("Assertion failed: {} - {}", stringify!($condition), format!($($arg)*));
            panic!("Assertion failed: {} - {}", stringify!($condition), format!($($arg)*));
        }
    };
}

/// Debug assert macro that only works in debug builds and logs failures.
#[macro_export]
macro_rules! log_debug_assert {
    ($condition:expr) => {
        #[cfg(debug_assertions)]
        $crate::log_assert!($condition);
    };
    ($condition:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::log_assert!($condition, $($arg)*);
    };
}

#[cfg(test)]
mod tests {
    use crate::config::{LogLevel, LoggerConfig};
    use crate::logger;
    use std::time::Duration;

    #[test]
    fn test_basic_logging_macros() {
        let config = LoggerConfig::builder().console(true).colors(false).build();

        logger::init(config).unwrap();

        // Test all basic logging macros
        assert!(log_error!("Test error message").is_ok());
        assert!(log_warning!("Test warning message").is_ok());
        assert!(log_info!("Test info message").is_ok());
        assert!(log_success!("Test success message").is_ok());
        assert!(log_debug!("Test debug message").is_ok());
    }

    #[test]
    fn test_log_macro_with_level() {
        let config = LoggerConfig::builder().console(true).colors(false).build();

        logger::init(config).unwrap();

        assert!(log!(LogLevel::Error, "Custom level message").is_ok());
        assert!(log!(LogLevel::Info, "User {} logged in", "alice").is_ok());
    }

    #[test]
    fn test_conditional_logging() {
        let config = LoggerConfig::builder().console(true).colors(false).build();

        logger::init(config).unwrap();

        let debug_enabled = true;
        let debug_disabled = false;

        // This should log
        log_if!(debug_enabled, LogLevel::Debug, "Debug is enabled");

        // This should not log
        log_if!(debug_disabled, LogLevel::Debug, "Debug is disabled");
    }

    #[test]
    fn test_time_block_macro() {
        let config = LoggerConfig::builder().console(true).colors(false).build();

        logger::init(config).unwrap();

        let result = time_block!(LogLevel::Info, "Test operation", {
            std::thread::sleep(Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
    }

    #[test]
    fn test_log_once_macro() {
        let config = LoggerConfig::builder().console(true).colors(false).build();

        logger::init(config).unwrap();

        // This should only log once despite being called multiple times
        for _ in 0..5 {
            log_once!(LogLevel::Warning, "This should only appear once");
        }
    }

    #[test]
    fn test_log_at_most_macro() {
        let config = LoggerConfig::builder().console(true).colors(false).build();

        logger::init(config).unwrap();

        // This should only log 3 times despite being called 10 times
        for i in 0..10 {
            log_at_most!(3, LogLevel::Info, "Message {}", i);
        }
    }

    #[test]
    fn test_trace_function_macro() {
        let config = LoggerConfig::builder()
            .console(true)
            .colors(false)
            .level(LogLevel::Debug)
            .build();

        logger::init(config).unwrap();

        fn test_function(x: i32, y: i32) -> i32 {
            trace_function!("test_function", x, y);
            x + y
        }

        let result = test_function(5, 10);
        assert_eq!(result, 15);
    }
}
