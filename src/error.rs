//! Error types for the firo_logger crate.

use std::io;
use std::time::SystemTimeError;
use thiserror::Error;

/// The main error type for firo_logger operations.
#[derive(Error, Debug)]
pub enum LoggerError {
    /// IO error occurred during file operations
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization error (for JSON logging)
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Logger already initialized
    #[error("Logger has already been initialized")]
    AlreadyInitialized,

    /// Logger not initialized
    #[error("Logger has not been initialized")]
    NotInitialized,

    /// Channel error (for async logging)
    #[error("Channel error: {0}")]
    Channel(String),

    /// Time-related error
    #[error("Time error: {0}")]
    Time(#[from] SystemTimeError),

    /// Custom error
    #[error("Logger error: {0}")]
    Custom(String),
}

/// A specialized Result type for firo_logger operations.
pub type Result<T> = std::result::Result<T, LoggerError>;
