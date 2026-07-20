use std::fmt;
use std::path::PathBuf;

/// Errors returned by file system infrastructure operations.
#[derive(Debug)]
pub enum FsError {
    /// An underlying file system operation failed.
    Io {
        /// The operation being attempted.
        operation: &'static str,
        /// The path involved in the operation.
        path: PathBuf,
        /// The underlying operating system error.
        source: std::io::Error,
    },
    /// A user-supplied path was not a safe relative path.
    InvalidPath { path: PathBuf, reason: &'static str },
    /// A read exceeded its configured maximum size.
    DataLimitExceeded {
        path: PathBuf,
        limit: usize,
        observed_at_least: usize,
    },
    /// Another process or task owns the requested lock.
    AlreadyLocked(PathBuf),
    /// Serialization or deserialization failed.
    Serialization {
        format: &'static str,
        operation: &'static str,
        path: PathBuf,
        message: String,
    },
    /// Text could not be decoded as UTF-8.
    Encoding {
        path: PathBuf,
        encoding: &'static str,
        message: String,
    },
    /// A blocking file system task could not complete.
    Task {
        operation: &'static str,
        message: String,
    },
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, path, source } => {
                write!(f, "failed to {operation} '{}': {source}", path.display())
            }
            Self::InvalidPath { path, reason } => {
                write!(f, "unsafe path '{}': {reason}", path.display())
            }
            Self::DataLimitExceeded { path, limit, observed_at_least } => {
                write!(
                    f,
                    "file '{}' exceeds the {limit}-byte read limit (observed at least {observed_at_least} bytes)",
                    path.display()
                )
            }
            Self::AlreadyLocked(path) => {
                write!(f, "file lock is already held: '{}'", path.display())
            }
            Self::Serialization { format, operation, path, message } => {
                write!(f, "failed to {operation} {format} file '{}': {message}", path.display())
            }
            Self::Encoding { path, encoding, message } => {
                write!(f, "failed to decode '{}' as {encoding}: {message}", path.display())
            }
            Self::Task { operation, message } => {
                write!(f, "file system task failed while attempting to {operation}: {message}")
            }
        }
    }
}

impl std::error::Error for FsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl FsError {
    /// Wraps an I/O error with the operation and affected path.
    pub(crate) fn io(
        operation: &'static str,
        path: impl Into<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::Io { operation, path: path.into(), source }
    }

    /// Builds a detailed structured-data error.
    pub(crate) fn serialization(
        format: &'static str,
        operation: &'static str,
        path: impl Into<PathBuf>,
        message: impl Into<String>,
    ) -> Self {
        Self::Serialization {
            format,
            operation,
            path: path.into(),
            message: message.into(),
        }
    }

    /// Builds an error for a failed blocking operation.
    pub(crate) fn task(operation: &'static str, message: impl Into<String>) -> Self {
        Self::Task { operation, message: message.into() }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn io_error_includes_operation_and_path() {
        let error = FsError::io(
            "read file",
            "cache/state.json",
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
        );

        assert_eq!(error.to_string(), "failed to read file 'cache/state.json': missing");
        assert!(error.source().is_some());
    }
}
