use std::fmt;
use std::path::PathBuf;

/// Errors returned by file system infrastructure operations.
#[derive(Debug)]
pub enum FsError {
    /// An underlying file system operation failed.
    Io(std::io::Error),
    /// A user-supplied path was not a safe relative path.
    InvalidPath { path: PathBuf, reason: &'static str },
    /// A read exceeded its configured maximum size.
    DataLimitExceeded { path: PathBuf, limit: usize },
    /// Another process or task owns the requested lock.
    AlreadyLocked(PathBuf),
    /// Serialization or deserialization failed.
    Serialization(String),
    /// A blocking file system task could not complete.
    Task(String),
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "file system operation failed: {error}"),
            Self::InvalidPath { path, reason } => {
                write!(f, "unsafe path '{}': {reason}", path.display())
            }
            Self::DataLimitExceeded { path, limit } => {
                write!(f, "file '{}' exceeds the {limit}-byte read limit", path.display())
            }
            Self::AlreadyLocked(path) => {
                write!(f, "file lock is already held: '{}'", path.display())
            }
            Self::Serialization(error) => write!(f, "serialization failed: {error}"),
            Self::Task(error) => write!(f, "file system task failed: {error}"),
        }
    }
}

impl std::error::Error for FsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for FsError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
