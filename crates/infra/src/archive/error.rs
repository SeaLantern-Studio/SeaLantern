use std::fmt;
use std::path::PathBuf;

use zip::result::ZipError;

/// Errors returned by ZIP archive operations.
#[derive(Debug)]
pub enum ArchiveError {
    /// An operating system file operation failed.
    Io {
        operation: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    /// The ZIP format could not be read or written.
    Zip {
        operation: &'static str,
        path: PathBuf,
        source: ZipError,
    },
    /// The requested ZIP source is not a regular directory.
    InvalidSource { path: PathBuf, reason: &'static str },
    /// The extraction destination is not a new, ordinary directory.
    InvalidDestination { path: PathBuf, reason: &'static str },
    /// An output archive or extraction destination already exists.
    DestinationExists { path: PathBuf },
    /// A source entry cannot be represented safely in a portable ZIP archive.
    UnsupportedSourceEntry { path: PathBuf, kind: &'static str },
    /// An archive entry name would escape or otherwise violate the extraction root.
    UnsafeEntry {
        archive: PathBuf,
        entry: String,
        reason: String,
    },
    /// An archive entry uses a type intentionally unsupported by this API.
    UnsupportedEntry {
        archive: PathBuf,
        entry: String,
        kind: &'static str,
    },
    /// Archive metadata or streamed content exceeded a configured resource limit.
    LimitExceeded {
        archive: PathBuf,
        limit: &'static str,
        observed: u64,
        maximum: u64,
    },
    /// A symbolic-link payload is not a portable, safe relative path.
    InvalidSymbolicLinkTarget { reason: &'static str },
    /// A symbolic-link payload is not safe for a specific archive entry.
    InvalidSymbolicLinkTargetEntry {
        archive: PathBuf,
        entry: String,
        reason: &'static str,
    },
    /// A symbolic-link payload could not be read from the archive.
    SymbolicLinkTargetRead {
        archive: PathBuf,
        entry: String,
        source: std::io::Error,
    },
}

impl ArchiveError {
    pub(crate) fn io(
        operation: &'static str,
        path: impl Into<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::Io { operation, path: path.into(), source }
    }

    pub(crate) fn zip(operation: &'static str, path: impl Into<PathBuf>, source: ZipError) -> Self {
        Self::Zip { operation, path: path.into(), source }
    }

    pub(crate) fn entry(&self) -> Option<&str> {
        match self {
            Self::UnsafeEntry { entry, .. }
            | Self::UnsupportedEntry { entry, .. }
            | Self::InvalidSymbolicLinkTargetEntry { entry, .. }
            | Self::SymbolicLinkTargetRead { entry, .. } => Some(entry),
            _ => None,
        }
    }
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, path, source } => {
                write!(formatter, "failed to {operation} '{}': {source}", path.display())
            }
            Self::Zip { operation, path, source } => write!(
                formatter,
                "failed to {operation} ZIP archive '{}': {source}",
                path.display()
            ),
            Self::InvalidSource { path, reason } => {
                write!(formatter, "invalid ZIP source '{}': {reason}", path.display())
            }
            Self::InvalidDestination { path, reason } => {
                write!(formatter, "invalid ZIP destination '{}': {reason}", path.display())
            }
            Self::DestinationExists { path } => {
                write!(formatter, "ZIP destination already exists: '{}'", path.display())
            }
            Self::UnsupportedSourceEntry { path, kind } => write!(
                formatter,
                "cannot add {kind} source entry '{}' to a portable ZIP archive",
                path.display()
            ),
            Self::UnsafeEntry { archive, entry, reason } => {
                write!(formatter, "unsafe ZIP entry '{entry}' in '{}': {reason}", archive.display())
            }
            Self::UnsupportedEntry { archive, entry, kind } => write!(
                formatter,
                "unsupported {kind} ZIP entry '{entry}' in '{}'",
                archive.display()
            ),
            Self::LimitExceeded { archive, limit, observed, maximum } => write!(
                formatter,
                "ZIP archive '{}' exceeds the {limit} limit: {observed} > {maximum}",
                archive.display()
            ),
            Self::InvalidSymbolicLinkTarget { reason } => {
                write!(formatter, "invalid ZIP symbolic-link target: {reason}")
            }
            Self::InvalidSymbolicLinkTargetEntry { archive, entry, reason } => {
                write!(
                    formatter,
                    "invalid symbolic-link target for ZIP entry '{entry}' in '{}': {reason}",
                    archive.display()
                )
            }
            Self::SymbolicLinkTargetRead { archive, entry, source } => {
                write!(
                    formatter,
                    "failed to read symbolic-link target for ZIP entry '{entry}' in '{}': {source}",
                    archive.display()
                )
            }
        }
    }
}

impl std::error::Error for ArchiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Zip { source, .. } => Some(source),
            Self::SymbolicLinkTargetRead { source, .. } => Some(source),
            _ => None,
        }
    }
}
