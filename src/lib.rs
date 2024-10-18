use chrono::Local;
use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::fmt::Arguments;

struct Colours;

impl Colours {
    pub const RED: &'static str = "\x1b[31m";
    pub const GREEN: &'static str = "\x1b[32m";
    pub const YELLOW: &'static str = "\x1b[33m";
    pub const BLUE: &'static str = "\x1b[34m";
    pub const CYAN: &'static str = "\x1b[36m";
    pub const WHITE: &'static str = "\x1b[37m";
}

#[derive(Debug, PartialEq)]
pub enum LogLevel {
    Error,
    Warning,
    Debug,
    Success,
    Info,
    Log,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warning => "WARNING",
            LogLevel::Debug => "DEBUG",
            LogLevel::Success => "SUCCESS",
            LogLevel::Info => "INFO",
            LogLevel::Log => "LOG",
        }
    }
}

#[derive(Debug)]
pub struct Logger {}

impl Logger {
    fn format_message(level: LogLevel, message: &str) -> (String, String) {
        let colour_code = match level {
            LogLevel::Error => Colours::RED,
            LogLevel::Warning => Colours::YELLOW,
            LogLevel::Debug => Colours::BLUE,
            LogLevel::Success => Colours::GREEN,
            LogLevel::Info => Colours::CYAN,
            LogLevel::Log => Colours::WHITE,
        };

        let current_datetime = Local::now();
        let date = current_datetime.format("%Y-%m-%d %H:%M:%S").to_string();

        let console_fmt = format!("{date}{colour_code} [{}]: \x1b[0m {message} ", level.as_str());
        let log_file_fmt = format!("{date} [{}]: {message} ", level.as_str());
        (console_fmt, log_file_fmt)
    }

    fn file_log(message: &str) -> io::Result<()> {
        let mut script_name = env::args()
            .next()
            .map(|arg| {
                arg.split('/')
                    .last()
                    .unwrap_or(arg.as_str())
                    .split('\\')
                    .last()
                    .unwrap_or(arg.as_str())
                    .to_owned()
            })
            .unwrap_or("unknown".to_owned());

        if script_name.ends_with(".exe") {
            script_name = script_name.replace(".exe", "");
        }
        let log_file_name = format!("{}.log", script_name);

        let mut file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file_name)
        {
            Ok(file) => file,
            Err(err) => {
                println!("Error opening log file: {}", err);
                return Err(err);
            }
        };

        file.write_all(message.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    // Updated to take `fmt::Arguments`
    fn log_msg(level: LogLevel, message: Arguments) {
        let formatted_message = format!("{}", message); // Convert `Arguments` to a String
        let (console_fmt, log_file_fmt) = Self::format_message(level, &formatted_message);
        println!("{}", console_fmt);
        let _ = Self::file_log(&log_file_fmt);
    }

    // Public logging functions, accepting formatted arguments
    pub fn log(args: Arguments) {
        Self::log_msg(LogLevel::Log, args);
    }

    pub fn error(args: Arguments) {
        Self::log_msg(LogLevel::Error, args);
    }

    pub fn warning(args: Arguments) {
        Self::log_msg(LogLevel::Warning, args);
    }

    pub fn debug(args: Arguments) {
        Self::log_msg(LogLevel::Debug, args);
    }

    pub fn info(args: Arguments) {
        Self::log_msg(LogLevel::Info, args);
    }

    pub fn success(args: Arguments) {
        Self::log_msg(LogLevel::Success, args);
    }
}

// Helper macros to allow formatted logging
macro_rules! log_error {
    ($($arg:tt)*) => {
        Logger::error(format_args!($($arg)*))
    };
}

macro_rules! log_warning {
    ($($arg:tt)*) => {
        Logger::warning(format_args!($($arg)*))
    };
}

macro_rules! log_info {
    ($($arg:tt)*) => {
        Logger::info(format_args!($($arg)*))
    };
}

macro_rules! log_debug {
    ($($arg:tt)*) => {
        Logger::debug(format_args!($($arg)*))
    };
}

macro_rules! log_success {
    ($($arg:tt)*) => {
        Logger::success(format_args!($($arg)*))
    };
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_message_error() {
        let (console, file) = Logger::format_message(LogLevel::Error, "This is an error message");
        assert!(console.contains("[ERROR]"));
        assert!(file.contains("[ERROR]"));
    }

    #[test]
    fn test_format_message_info() {
        let (console, file) = Logger::format_message(LogLevel::Info, "Information log");
        assert!(console.contains("[INFO]"));
        assert!(file.contains("[INFO]"));
    }

    #[test]
    fn test_format_message_debug() {
        let (console, file) = Logger::format_message(LogLevel::Debug, "Debugging message");
        assert!(console.contains("[DEBUG]"));
        assert!(file.contains("[DEBUG]"));
    }

    #[test]
    fn test_log_success() {
        let (console, _) = Logger::format_message(LogLevel::Success, "Successful operation");
        assert!(console.contains(Colours::GREEN)); // Ensure it uses the correct color
        assert!(console.contains("[SUCCESS]"));
    }
    
    #[test]
    fn test_log_warning() {
        let (console, _) = Logger::format_message(LogLevel::Warning, "Warning message");
        assert!(console.contains(Colours::YELLOW)); // Ensure correct color is applied
        assert!(console.contains("[WARNING]"));
    }
}
