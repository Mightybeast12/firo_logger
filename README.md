firo_logger is a simple, customizable logger for Rust applications that supports colored console output and file logging.

Installation
Add this to your Cargo.toml:
 
[dependencies]
firo_logger = "0.1.0"
 
Usage
Import the logger and use it like this:

rust
use firo_logger::Logger;

fn main() {
    Logger::log("This is a log message");
    Logger::error("This is an error message");
    Logger::warning("This is a warning message");
    Logger::info("This is an info message");
    Logger::debug("This is a debug message");
    Logger::success("This is a success message");
}
Example Code:
use firo_logger::Logger;

fn main() {
    Logger::info("Test logg"); 
    Logger::error("Test error"); 
    Logger::warning("Test warn"); 
}
Example Output:
2024-10-01 10:32:45 [INFO]:    Test logg
2024-10-01 10:32:45 [ERROR]:   Test error
2024-10-01 10:32:45 [WARNING]: Test warn
The logger automatically adds timestamps and log levels to each message.
Console output is color-coded based on the log level (e.g., red for errors, yellow for warnings, green for success).
Features
Color-coded output: Errors, warnings, successes, and more are color-coded in the terminal.
File logging: Log messages are also written to a file named after your script (e.g., my_project.log).
Timestamped logs: Each log message is automatically prepended with the current timestamp.