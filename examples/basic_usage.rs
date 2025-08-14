//! Basic usage example for firo_logger.

use firo_logger::{
    init, log_debug, log_error, log_info, log_success, log_warning, LogLevel, LoggerConfig,
    OutputFormat,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger with custom configuration
    let config = LoggerConfig::builder()
        .level(LogLevel::Debug)
        .format(OutputFormat::Text)
        .console(true)
        .colors(true)
        .include_caller(true)
        .include_thread(false)
        .datetime_format("%H:%M:%S%.3f")
        .metadata("app", "basic_example")
        .metadata("version", "1.0.0")
        .build();

    init(config)?;

    // Test all log levels
    log_error!("This is an error message with code: {}", 500)?;
    log_warning!("This is a warning: {} items remaining", 3)?;
    log_info!("Application started successfully")?;
    log_success!("Database connection established")?;
    log_debug!("Debug information: user_id={}", 12345)?;

    // Test formatted logging
    let user_name = "Alice";
    let items_processed = 42;
    log_info!("User {} processed {} items", user_name, items_processed)?;

    // Test conditional logging
    let debug_mode = true;
    if debug_mode {
        log_debug!("Debug mode is enabled - showing detailed information")?;
    }

    println!("\n=== Testing with file output ===");

    // Test with file output
    let file_config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .console(true)
        .file("example.log")
        .colors(false)
        .include_caller(true)
        .datetime_format("%Y-%m-%d %H:%M:%S")
        .build();

    // Note: In a real application, you would typically only initialize once
    // Here we're just demonstrating different configurations
    println!("Logging to file: example.log");
    log_info!("This message will also be written to example.log")?;

    Ok(())
}
