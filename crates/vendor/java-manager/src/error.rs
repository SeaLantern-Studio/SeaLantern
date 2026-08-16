//! Error types for Java environment detection and execution.

use std::fmt;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

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

    /// 版本探测超出执行时限。
    ProcessTimeout {
        executable: PathBuf,
        timeout: Duration,
    },

    /// 版本探测输出超过配置上限。
    OutputLimitExceeded { executable: PathBuf, limit: usize },

    /// 重定向路径越出根目录或包含不允许的路径结构。
    InvalidRedirectPath {
        stream: &'static str,
        path: PathBuf,
        reason: String,
    },

    /// 重定向文件创建或打开失败，并保留目标上下文。
    RedirectIoError {
        stream: &'static str,
        path: PathBuf,
        source: io::Error,
    },
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
            JavaError::ProcessTimeout { executable, timeout } => write!(
                f,
                "Java version probe timed out after {} ms: {}",
                timeout.as_millis(),
                executable.display()
            ),
            JavaError::OutputLimitExceeded { executable, limit } => write!(
                f,
                "Java version probe output exceeded {} bytes: {}",
                limit,
                executable.display()
            ),
            JavaError::InvalidRedirectPath { stream, path, reason } => {
                write!(f, "Invalid {} redirect path '{}': {}", stream, path.display(), reason)
            }
            JavaError::RedirectIoError { stream, path, source } => {
                write!(f, "Failed to open {} redirect '{}': {}", stream, path.display(), source)
            }
        }
    }
}

impl std::error::Error for JavaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            JavaError::IoError(err) => Some(err),
            JavaError::RedirectIoError { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<io::Error> for JavaError {
    fn from(err: io::Error) -> Self {
        JavaError::IoError(err)
    }
}
