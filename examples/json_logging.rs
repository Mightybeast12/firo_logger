//! JSON logging example for firo_logger.

use firo_logger::{
    init, log_debug, log_error, log_info, log_success, log_warning, LogLevel, LoggerConfig,
    OutputFormat,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure JSON logging
    let config = LoggerConfig::builder()
        .level(LogLevel::Debug)
        .format(OutputFormat::Json)
        .console(true)
        .colors(false) // JSON doesn't need colors
        .include_caller(true)
        .include_thread(true)
        .datetime_format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .metadata("service", "json-demo")
        .metadata("version", "1.0.0")
        .metadata("environment", "development")
        .build();

    init(config)?;

    println!("=== JSON Logging Demo ===");
    println!("Each log entry will be a JSON object with structured data.\n");

    // Basic logging
    log_info!("Application started with JSON logging enabled")?;
    log_success!("JSON formatter initialized successfully")?;

    // Logging with different levels
    log_debug!("Debug information: processing user request")?;
    log_warning!("Warning: API rate limit approaching threshold")?;
    log_error!("Error: Failed to connect to external service")?;

    // Simulate some application events
    simulate_user_events()?;
    simulate_api_requests()?;
    simulate_database_operations()?;

    log_info!("JSON logging demo completed")?;

    println!("\n=== Pretty JSON Example ===");
    println!("Now let's try with pretty-printed JSON:");

    // Configure pretty JSON logging
    let _pretty_config = LoggerConfig::builder()
        .level(LogLevel::Info)
        .format(OutputFormat::Json) // Note: Pretty printing would need to be added to JsonFormatter
        .console(true)
        .include_caller(false) // Less clutter for pretty printing
        .include_thread(false)
        .metadata("format", "pretty")
        .build();

    // Note: This would be a second initialization in a real app, but for demo purposes
    log_info!(
        "This would be pretty-printed JSON if we extended JsonFormatter with a pretty option"
    )?;

    Ok(())
}

fn simulate_user_events() -> Result<(), Box<dyn std::error::Error>> {
    log_info!("Simulating user events")?;

    // Simulate user login
    log_success!("User authentication successful")?;

    // Simulate user actions
    log_info!("User performed search operation")?;
    log_debug!("Search query processed with 42 results")?;

    // Simulate user logout
    log_info!("User session terminated gracefully")?;

    Ok(())
}

fn simulate_api_requests() -> Result<(), Box<dyn std::error::Error>> {
    log_info!("Simulating API requests")?;

    // Simulate successful API calls
    log_success!("GET /api/users returned 200 OK")?;
    log_success!("POST /api/orders returned 201 Created")?;

    // Simulate API errors
    log_warning!("GET /api/deprecated-endpoint returned 404 Not Found")?;
    log_error!("POST /api/payment failed with 500 Internal Server Error")?;

    // Simulate rate limiting
    log_warning!("Rate limit warning: 90% of quota used")?;

    Ok(())
}

fn simulate_database_operations() -> Result<(), Box<dyn std::error::Error>> {
    log_info!("Simulating database operations")?;

    // Simulate database queries
    log_debug!("Executing SELECT query on users table")?;
    log_success!("Database query completed successfully")?;

    // Simulate slow query
    log_warning!("Slow query detected: took 2.5 seconds to complete")?;

    // Simulate database error
    log_error!("Database connection timeout after 30 seconds")?;

    // Simulate recovery
    log_success!("Database connection restored automatically")?;

    Ok(())
}
