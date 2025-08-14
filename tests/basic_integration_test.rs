//! Basic integration tests for firo_logger.
//!
//! Due to the global singleton nature of the logger, these tests focus on
//! functionality that works regardless of the initial configuration.

use firo_logger::{
    init_default, log_debug, log_error, log_info, log_success, log_warning, LogLevel, LoggerConfig,
    OutputFormat,
};
use std::thread;
use std::time::Duration;

#[test]
fn test_logger_initialization_and_basic_logging() {
    // Initialize logger with default settings (may already be initialized)
    let _ = init_default(); // Ignore AlreadyInitialized error

    // Test all log levels - these should succeed regardless of the current level setting
    assert!(log_error!("Integration test error message").is_ok());
    assert!(log_warning!("Integration test warning message").is_ok());
    assert!(log_info!("Integration test info message").is_ok());
    assert!(log_success!("Integration test success message").is_ok());
    assert!(log_debug!("Integration test debug message").is_ok());

    // Allow time for async operations
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_configuration_builder() {
    // Test that configuration builder works correctly
    let config = LoggerConfig::builder()
        .level(LogLevel::Debug)
        .console(true)
        .colors(false)
        .format(OutputFormat::Json)
        .include_caller(true)
        .include_thread(true)
        .metadata("test", "value")
        .build();

    assert_eq!(config.level, LogLevel::Debug);
    assert!(config.console_enabled);
    assert!(!config.console.colors);
    assert_eq!(config.format, OutputFormat::Json);
    assert!(config.include_caller);
    assert!(config.include_thread);
    assert_eq!(config.metadata.get("test"), Some(&"value".to_string()));
}

#[test]
fn test_environment_configuration() {
    // Set environment variables
    std::env::set_var("FIRO_LOG_LEVEL", "DEBUG");
    std::env::set_var("FIRO_LOG_FORMAT", "json");
    std::env::set_var("NO_COLOR", "1");

    let config = LoggerConfig::from_env();

    assert_eq!(config.level, LogLevel::Debug);
    assert_eq!(config.format, OutputFormat::Json);
    assert!(!config.console.colors);

    // Clean up environment
    std::env::remove_var("FIRO_LOG_LEVEL");
    std::env::remove_var("FIRO_LOG_FORMAT");
    std::env::remove_var("NO_COLOR");
}

#[test]
fn test_configuration_validation() {
    // Test invalid configuration
    let config = LoggerConfig {
        console_enabled: false,
        file_enabled: false,
        ..Default::default()
    };

    let result = config.validate();
    assert!(
        result.is_err(),
        "Expected validation to fail with no outputs"
    );

    // Test valid configuration
    let valid_config = LoggerConfig::builder().console(true).build();
    assert!(valid_config.validate().is_ok());
}

#[test]
fn test_log_record_creation() {
    use firo_logger::formatters::{CallerInfo, LogRecord};
    use std::collections::HashMap;

    let record = LogRecord::new(LogLevel::Info, format_args!("Test message"));
    assert_eq!(record.level, LogLevel::Info);
    assert_eq!(record.message, "Test message");
    assert!(record.timestamp.timestamp() > 0);

    // Test with metadata
    let mut metadata = HashMap::new();
    metadata.insert("key1".to_string(), "value1".to_string());

    let record_with_metadata = record
        .with_metadata("key2", "value2")
        .with_metadata_map(metadata);

    assert_eq!(
        record_with_metadata.metadata.get("key1"),
        Some(&"value1".to_string())
    );
    assert_eq!(
        record_with_metadata.metadata.get("key2"),
        Some(&"value2".to_string())
    );

    // Test with caller info
    let caller = CallerInfo {
        file: "test.rs",
        line: 42,
        module: Some("test_module"),
    };

    let record_with_caller = record_with_metadata.with_caller(caller);
    assert!(record_with_caller.caller.is_some());
    assert_eq!(record_with_caller.caller.unwrap().file, "test.rs");
}

#[test]
fn test_concurrent_logging() {
    // Test that concurrent logging doesn't panic or cause errors
    let num_threads = 5;
    let messages_per_thread = 50;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            thread::spawn(move || {
                for i in 0..messages_per_thread {
                    // These should succeed without error
                    assert!(log_info!("Thread {} message {}", thread_id, i).is_ok());
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Give async logger time to process
    thread::sleep(Duration::from_millis(200));
}

#[test]
fn test_macro_usage() {
    // Test various macro usage patterns
    let user = "alice";
    let count = 42;

    assert!(log_info!("User {} processed {} items", user, count).is_ok());
    assert!(log_error!("Error code: {}", 500).is_ok());
    assert!(log_success!("Operation completed successfully").is_ok());
    assert!(log_warning!("Resource usage at {}%", 85).is_ok());
    assert!(log_debug!("Debug info: {:?}", vec![1, 2, 3]).is_ok());
}
