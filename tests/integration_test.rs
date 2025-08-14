//! Integration tests for firo_logger using the new dual singleton/instance pattern.
//!
//! These tests use isolated logger instances to avoid conflicts.

use firo_logger::{
    log_debug, log_error, log_info, log_success, log_warning, with_scoped_logger, LogLevel,
    LoggerConfig, LoggerInstance, OutputFormat,
};
use std::fs;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::{tempdir, NamedTempFile};

#[test]
fn test_basic_logging() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let config = LoggerConfig::builder()
        .level(LogLevel::Debug)
        .console(false)
        .file(temp_file.path())
        .colors(false)
        .build();

    let logger = Arc::new(LoggerInstance::new(config).expect("Failed to create logger"));

    with_scoped_logger(logger.clone(), || {
        // Test all log levels
        log_error!("Test error message").expect("Failed to log error");
        log_warning!("Test warning message").expect("Failed to log warning");
        log_info!("Test info message").expect("Failed to log info");
        log_success!("Test success message").expect("Failed to log success");
        log_debug!("Test debug message").expect("Failed to log debug");
    });

    // Allow time for async operations
    logger.flush().expect("Failed to flush");
    thread::sleep(Duration::from_millis(100));

    let content = fs::read_to_string(temp_file.path()).expect("Failed to read log file");

    assert!(content.contains("Test error message"));
    assert!(content.contains("Test warning message"));
    assert!(content.contains("Test info message"));
    assert!(content.contains("Test success message"));
    assert!(content.contains("Test debug message"));
}

#[test]
fn test_level_filtering() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let config = LoggerConfig::builder()
        .level(LogLevel::Warning) // Only warning and above
        .console(false)
        .file(temp_file.path())
        .build();

    let logger = Arc::new(LoggerInstance::new(config).expect("Failed to create logger"));

    with_scoped_logger(logger.clone(), || {
        log_error!("Should appear").expect("Failed to log error");
        log_warning!("Should appear").expect("Failed to log warning");
        log_info!("Should not appear").expect("Failed to log info");
        log_debug!("Should not appear").expect("Failed to log debug");
    });

    logger.flush().expect("Failed to flush");
    thread::sleep(Duration::from_millis(100));

    let content = fs::read_to_string(temp_file.path()).expect("Failed to read log file");

    assert!(content.contains("Should appear"));
    assert!(!content.contains("Should not appear"));
}

#[test]
fn test_json_formatting() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .format(OutputFormat::Json)
        .console(false)
        .file(temp_file.path())
        .include_caller(true)
        .metadata("test", "json_formatting")
        .build();

    let logger = Arc::new(LoggerInstance::new(config).expect("Failed to create logger"));

    with_scoped_logger(logger.clone(), || {
        log_info!("JSON test message").expect("Failed to log info");
    });

    logger.flush().expect("Failed to flush");
    thread::sleep(Duration::from_millis(100));

    let content = fs::read_to_string(temp_file.path()).expect("Failed to read log file");

    // Verify JSON structure
    assert!(content.contains("\"level\":\"INFO\""));
    assert!(content.contains("\"message\":\"JSON test message\""));
    assert!(content.contains("\"timestamp\""));
    assert!(content.contains("\"metadata\""));
    assert!(content.contains("\"test\":\"json_formatting\""));
}

#[test]
fn test_metadata() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .console(false)
        .file(temp_file.path())
        .metadata("service", "test-service")
        .metadata("version", "1.0.0")
        .build();

    let logger = Arc::new(LoggerInstance::new(config).expect("Failed to create logger"));

    with_scoped_logger(logger.clone(), || {
        log_info!("Metadata test").expect("Failed to log info");
    });

    logger.flush().expect("Failed to flush");
    thread::sleep(Duration::from_millis(100));

    let content = fs::read_to_string(temp_file.path()).expect("Failed to read log file");

    assert!(content.contains("service=test-service"));
    assert!(content.contains("version=1.0.0"));
}

#[test]
fn test_caller_information() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .console(false)
        .file(temp_file.path())
        .include_caller(true)
        .build();

    let logger = Arc::new(LoggerInstance::new(config).expect("Failed to create logger"));

    with_scoped_logger(logger.clone(), || {
        log_info!("Caller test").expect("Failed to log info"); // This line number matters
    });

    logger.flush().expect("Failed to flush");
    thread::sleep(Duration::from_millis(100));

    let content = fs::read_to_string(temp_file.path()).expect("Failed to read log file");

    assert!(content.contains("integration_test.rs"));
    assert!(content.contains("Caller test"));
}

#[test]
fn test_size_based_rotation() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let log_path = temp_dir.path().join("rotation_test.log");

    let config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .console(false)
        .file(&log_path)
        .rotate_by_size(100, 2) // Very small size to trigger rotation
        .build();

    let logger = Arc::new(LoggerInstance::new(config).expect("Failed to create logger"));

    with_scoped_logger(logger.clone(), || {
        // Write enough data to trigger rotation
        for i in 0..10 {
            log_info!("Rotation test message number {} with extra padding", i)
                .expect("Failed to log");
        }
    });

    logger.flush().expect("Failed to flush");
    thread::sleep(Duration::from_millis(200));

    // Check if rotation occurred by looking for backup files
    let entries: Vec<_> = fs::read_dir(temp_dir.path())
        .expect("Failed to read temp dir")
        .collect();

    assert!(
        entries.len() > 1,
        "Expected rotation to create backup files"
    );
}

#[test]
fn test_async_logging() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .console(false)
        .file(temp_file.path())
        .async_logging(1000) // Enable async with buffer
        .build();

    let logger = Arc::new(LoggerInstance::new(config).expect("Failed to create logger"));

    with_scoped_logger(logger.clone(), || {
        // Log many messages quickly
        for i in 0..100 {
            log_info!("Async test message {}", i).expect("Failed to log");
        }
    });

    // Give async thread time to process
    thread::sleep(Duration::from_millis(500));

    let content = fs::read_to_string(temp_file.path()).expect("Failed to read log file");

    // Verify all messages were logged
    assert!(content.contains("Async test message 0"));
    assert!(content.contains("Async test message 99"));

    // Count lines to verify all messages
    let line_count = content.lines().count();
    assert_eq!(line_count, 100, "Expected 100 log lines");
}

#[test]
fn test_module_filtering() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let mut config = LoggerConfig::builder()
        .level(LogLevel::Warning) // Default level
        .console(false)
        .file(temp_file.path())
        .build();

    // Allow debug logs for this specific module
    config
        .module_filters
        .insert("integration_test".to_string(), LogLevel::Debug);

    let logger = Arc::new(LoggerInstance::new(config).expect("Failed to create logger"));

    with_scoped_logger(logger.clone(), || {
        log_debug!("This debug message should appear").expect("Failed to log debug");
        log_info!("This info message should appear").expect("Failed to log info");
    });

    logger.flush().expect("Failed to flush");
    thread::sleep(Duration::from_millis(100));

    let content = fs::read_to_string(temp_file.path()).expect("Failed to read log file");

    assert!(content.contains("This debug message should appear"));
    assert!(content.contains("This info message should appear"));
}

#[test]
fn test_statistics() {
    let config = LoggerConfig::builder()
        .level(LogLevel::Debug)
        .console(true) // Enable console for statistics test
        .colors(false) // Disable colors to avoid interference
        .build();

    let logger = Arc::new(LoggerInstance::new(config).expect("Failed to create logger"));

    with_scoped_logger(logger.clone(), || {
        // Log different levels
        log_error!("Error 1").expect("Failed to log");
        log_error!("Error 2").expect("Failed to log");
        log_warning!("Warning 1").expect("Failed to log");
        log_info!("Info 1").expect("Failed to log");
    });

    thread::sleep(Duration::from_millis(100));

    let logger_stats = logger.stats();

    assert_eq!(logger_stats.total_messages, 4);
    assert_eq!(
        logger_stats.messages_by_level.get(&LogLevel::Error),
        Some(&2)
    );
    assert_eq!(
        logger_stats.messages_by_level.get(&LogLevel::Warning),
        Some(&1)
    );
    assert_eq!(
        logger_stats.messages_by_level.get(&LogLevel::Info),
        Some(&1)
    );
    assert!(logger_stats.start_time.is_some());
}

#[test]
fn test_default_initialization() {
    // Test that default initialization works for global logger
    let _ = firo_logger::init_default(); // May already be initialized
    log_info!("Default init test").expect("Failed to log with default init");
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
    let mut config = LoggerConfig::default();
    config.console_enabled = false;
    config.file_enabled = false;

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
fn test_concurrent_logging() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .console(false)
        .file(temp_file.path())
        .async_logging(5000) // Large buffer for concurrent access
        .build();

    let logger = Arc::new(LoggerInstance::new(config).expect("Failed to create logger"));

    let num_threads = 10;
    let messages_per_thread = 100;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let logger_clone = Arc::clone(&logger);
            thread::spawn(move || {
                with_scoped_logger(logger_clone, || {
                    for i in 0..messages_per_thread {
                        log_info!("Thread {} message {}", thread_id, i).expect("Failed to log");
                    }
                });
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Give async logger time to process all messages
    thread::sleep(Duration::from_millis(1000));

    let content = fs::read_to_string(temp_file.path()).expect("Failed to read log file");
    let line_count = content.lines().count();

    assert_eq!(
        line_count,
        num_threads * messages_per_thread,
        "Expected {} log lines from concurrent threads",
        num_threads * messages_per_thread
    );
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
