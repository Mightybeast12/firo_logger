# firo_logger

`firo_logger` is a simple, customizable logger for Rust applications that supports colored console output and file logging.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
firo_logger = "*.*.*"
```

```rust
use firo_logger::Logger;

fn main() {
    log_error!("Error occurred: {}", "File not found");
    log_success!("Operation completed successfully: {}", "data.txt");
    log_info!("User {} logged in", "Alice");

}
```
## Output
```
2024-10-01 10:32:45 [INFO]:    Test logg
2024-10-01 10:32:45 [ERROR]:   Test error
2024-10-01 10:32:45 [WARNING]: Test warn
```
