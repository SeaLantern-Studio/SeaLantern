//! Error types for Java environment detection and execution.

use std::fmt;
use std::io;

/// Errors that can occur when working with Java installations.
#[derive(Debug)]
pub enum JavaError {
    /// The provided Java path is invalid (does not exist or cannot be used).
    InvalidJavaPath(String),

    /// A required Java executable or file was not found.
    NotFound(String),

    /// An I/O error occurred (e.g., reading a file or spawning a process).
    IoError(io::Error),

    /// An error during command execution (e.g., `java -version` failed).
    ExecuteError(String),

    /// A runtime error, such as unexpected output format.
    RuntimeError(String),

    /// Execution of a Java process failed (non-zero exit code).
    ExecutionFailed(String),

    /// A generic error with a custom message.
    Other(String),
}

impl fmt::Display for JavaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JavaError::InvalidJavaPath(msg) => write!(f, "Invalid Java path: {}", msg),
            JavaError::NotFound(msg) => write!(f, "Not found: {}", msg),
            JavaError::IoError(err) => write!(f, "IO error: {}", err),
            JavaError::ExecuteError(msg) => write!(f, "Execute error: {}", msg),
            JavaError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            JavaError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            JavaError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for JavaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JavaError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for JavaError {
    fn from(err: io::Error) -> Self {
        JavaError::IoError(err)
    }
}
