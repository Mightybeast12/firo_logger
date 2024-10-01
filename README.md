# firo_logger

`firo_logger` is a simple, customizable logger for Rust applications that supports colored console output and file logging.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
firo_logger = "0.1.0"

'''rust
use firo_logger::Logger;

fn main() {
    Logger::log("This is a log message");
    Logger::error("This is an error message");
    Logger::warning("This is a warning message");
    Logger::info("This is an info message");
    Logger::debug("This is a debug message");
    Logger::success("This is a success message");
}

2024-10-01 10:32:45 [INFO]:    Test logg
2024-10-01 10:32:45 [ERROR]:   Test error
2024-10-01 10:32:45 [WARNING]: Test warn
