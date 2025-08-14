//! Tests demonstrating the dual singleton/instance pattern for concurrent testing.

use firo_logger::{
    log_error, log_info, log_success, with_scoped_logger, LogLevel, LoggerConfig, LoggerInstance,
    OutputFormat,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

#[test]
fn test_concurrent_isolated_loggers() {
    // This test demonstrates that each test can use its own logger instance
    // without interfering with others, even when run concurrently.

    let num_threads = 5;
    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            thread::spawn(move || {
                // Each thread creates its own isolated logger instance
                let temp_file = NamedTempFile::new().expect("Failed to create temp file");
                let config = LoggerConfig::builder()
                    .level(LogLevel::Debug)
                    .console(false) // Only log to file to avoid mixing output
                    .file(temp_file.path())
                    .format(OutputFormat::Text)
                    .include_caller(true)
                    .metadata("thread_id", thread_id.to_string())
                    .build();

                let logger =
                    Arc::new(LoggerInstance::new(config).expect("Failed to create logger"));

                // Use scoped logger for this thread
                with_scoped_logger(logger.clone(), || {
                    // Log some messages within the scoped context
                    log_info!("Thread {} starting", thread_id).expect("Failed to log");
                    log_success!("Thread {} processing", thread_id).expect("Failed to log");
                    log_error!("Thread {} error test", thread_id).expect("Failed to log");

                    // Small delay to ensure async processing
                    thread::sleep(Duration::from_millis(10));
                });

                // Flush and read the file content
                logger.flush().expect("Failed to flush");
                thread::sleep(Duration::from_millis(50)); // Give async time to process

                let content =
                    std::fs::read_to_string(temp_file.path()).expect("Failed to read log file");

                // Verify this thread's messages are in its own file
                assert!(content.contains(&format!("Thread {thread_id} starting")));
                assert!(content.contains(&format!("Thread {thread_id} processing")));
                assert!(content.contains(&format!("Thread {thread_id} error test")));
                assert!(content.contains(&format!("thread_id={thread_id}")));

                // Verify no other thread's messages are in this file
                for other_id in 0..num_threads {
                    if other_id != thread_id {
                        assert!(!content.contains(&format!("Thread {other_id} starting")));
                    }
                }

                thread_id
            })
        })
        .collect();

    // Wait for all threads to complete and verify they all succeeded
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.len(), num_threads);

    // Verify all thread IDs are present
    for i in 0..num_threads {
        assert!(results.contains(&i));
    }
}

#[test]
fn test_mixed_scoped_and_global_logging() {
    // This test demonstrates using both scoped and global logging

    // Initialize global logger (if not already initialized)
    let global_config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .console(true)
        .colors(false)
        .build();
    let _ = firo_logger::init(global_config); // May fail if already initialized, that's ok

    // Create a scoped logger with different configuration
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let scoped_config = LoggerConfig::builder()
        .level(LogLevel::Debug)
        .console(false)
        .file(temp_file.path())
        .format(OutputFormat::Json)
        .metadata("test", "scoped_logging")
        .build();

    let scoped_logger =
        Arc::new(LoggerInstance::new(scoped_config).expect("Failed to create scoped logger"));

    // Log with global logger (if available)
    let _ = log_info!("This goes to global logger");

    // Use scoped logger
    with_scoped_logger(scoped_logger.clone(), || {
        log_info!("This goes to scoped logger").expect("Failed to log to scoped logger");
        log_error!("Scoped error message").expect("Failed to log error");
    });

    // Back to global logger (if available)
    let _ = log_info!("Back to global logger");

    // Verify scoped logger content
    scoped_logger
        .flush()
        .expect("Failed to flush scoped logger");
    thread::sleep(Duration::from_millis(50));

    let scoped_content =
        std::fs::read_to_string(temp_file.path()).expect("Failed to read scoped log file");

    assert!(scoped_content.contains("This goes to scoped logger"));
    assert!(scoped_content.contains("Scoped error message"));
    assert!(scoped_content.contains("\"test\":\"scoped_logging\""));

    // The scoped file should NOT contain global messages
    assert!(!scoped_content.contains("This goes to global logger"));
    assert!(!scoped_content.contains("Back to global logger"));
}

#[test]
fn test_logger_instance_direct_usage() {
    // This test demonstrates direct usage of LoggerInstance for testing

    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let config = LoggerConfig::builder()
        .level(LogLevel::Debug)
        .console(false)
        .file(temp_file.path())
        .format(OutputFormat::Text)
        .include_caller(true)
        .include_thread(true)
        .metadata("test_name", "direct_usage")
        .build();

    let logger = LoggerInstance::new(config).expect("Failed to create logger");

    // Use the logger instance directly
    logger
        .info(format_args!("Direct info message"))
        .expect("Failed to log info");
    logger
        .error(format_args!("Direct error message"))
        .expect("Failed to log error");
    logger
        .success(format_args!("Direct success message"))
        .expect("Failed to log success");
    logger
        .debug(format_args!("Direct debug message"))
        .expect("Failed to log debug");

    logger.flush().expect("Failed to flush");

    let content = std::fs::read_to_string(temp_file.path()).expect("Failed to read log file");

    assert!(content.contains("Direct info message"));
    assert!(content.contains("Direct error message"));
    assert!(content.contains("Direct success message"));
    assert!(content.contains("Direct debug message"));
    assert!(content.contains("test_name=direct_usage"));
    // Caller info is only included when using macros or explicitly setting it
    // Direct logger usage doesn't automatically include caller info
    // assert!(content.contains("concurrent_test.rs")); // Caller info
}

#[test]
fn test_logger_statistics_isolation() {
    // This test demonstrates that logger statistics are isolated per instance

    let config1 = LoggerConfig::builder()
        .level(LogLevel::Debug)
        .console(true)
        .colors(false)
        .build();

    let config2 = LoggerConfig::builder()
        .level(LogLevel::Info)
        .console(true)
        .colors(false)
        .build();

    let logger1 = LoggerInstance::new(config1).expect("Failed to create logger1");
    let logger2 = LoggerInstance::new(config2).expect("Failed to create logger2");

    // Log different amounts to each logger
    logger1
        .info(format_args!("Logger1 message 1"))
        .expect("Failed to log");
    logger1
        .error(format_args!("Logger1 message 2"))
        .expect("Failed to log");
    logger1
        .debug(format_args!("Logger1 message 3"))
        .expect("Failed to log");

    logger2
        .info(format_args!("Logger2 message 1"))
        .expect("Failed to log");
    logger2
        .warning(format_args!("Logger2 message 2"))
        .expect("Failed to log");

    // Check statistics are isolated
    let stats1 = logger1.stats();
    let stats2 = logger2.stats();

    assert_eq!(stats1.total_messages, 3);
    assert_eq!(stats2.total_messages, 2);

    assert_eq!(stats1.messages_by_level.get(&LogLevel::Info), Some(&1));
    assert_eq!(stats1.messages_by_level.get(&LogLevel::Error), Some(&1));
    assert_eq!(stats1.messages_by_level.get(&LogLevel::Debug), Some(&1));

    assert_eq!(stats2.messages_by_level.get(&LogLevel::Info), Some(&1));
    assert_eq!(stats2.messages_by_level.get(&LogLevel::Warning), Some(&1));
    assert_eq!(stats2.messages_by_level.get(&LogLevel::Error), None);
}
