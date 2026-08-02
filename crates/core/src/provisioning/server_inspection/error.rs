use std::fmt;
use std::io;
use std::path::PathBuf;

use zip::result::ZipError;

#[derive(Debug)]
pub enum ServerInspectionError {
    InvalidOptions {
        detail: &'static str,
    },
    Metadata {
        path: PathBuf,
        source: io::Error,
    },
    UnsupportedSubject {
        path: PathBuf,
    },
    Open {
        path: PathBuf,
        source: io::Error,
    },
    Archive {
        path: PathBuf,
        source: ZipError,
    },
    ArchiveEntry {
        path: PathBuf,
        entry: String,
        source: ZipError,
    },
    ArchiveEntryRead {
        path: PathBuf,
        entry: String,
        source: io::Error,
    },
    MetadataRead {
        path: PathBuf,
        source: io::Error,
    },
    TooManyArchiveEntries {
        path: PathBuf,
        count: usize,
        limit: usize,
    },
    TooManyDirectoryEntries {
        path: PathBuf,
        count: usize,
        limit: usize,
    },
    FingerprintRead {
        path: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for ServerInspectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOptions { detail } => {
                write!(formatter, "invalid server inspection options: {detail}")
            }
            Self::Metadata { path, source } => {
                write!(formatter, "could not inspect server artifact {}: {source}", path.display())
            }
            Self::UnsupportedSubject { path } => write!(
                formatter,
                "server artifact must be a regular file or directory: {}",
                path.display()
            ),
            Self::Open { path, source } => {
                write!(formatter, "could not open server artifact {}: {source}", path.display())
            }
            Self::Archive { path, source } => {
                write!(formatter, "could not parse JAR archive {}: {source}", path.display())
            }
            Self::ArchiveEntry { path, entry, source } => write!(
                formatter,
                "could not open metadata entry {entry} in {}: {source}",
                path.display()
            ),
            Self::ArchiveEntryRead { path, entry, source } => write!(
                formatter,
                "could not read metadata entry {entry} in {}: {source}",
                path.display()
            ),
            Self::MetadataRead { path, source } => {
                write!(formatter, "could not read metadata file {}: {source}", path.display())
            }
            Self::TooManyArchiveEntries { path, count, limit } => write!(
                formatter,
                "JAR archive {} contains {count} entries, exceeding the inspection limit of {limit}",
                path.display()
            ),
            Self::TooManyDirectoryEntries { path, count, limit } => write!(
                formatter,
                "directory {} contains {count} immediate entries, exceeding the inspection limit of {limit}",
                path.display()
            ),
            Self::FingerprintRead { path, source } => write!(
                formatter,
                "could not calculate fingerprint for {}: {source}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for ServerInspectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidOptions { .. }
            | Self::UnsupportedSubject { .. }
            | Self::TooManyArchiveEntries { .. }
            | Self::TooManyDirectoryEntries { .. } => None,
            Self::Metadata { source, .. }
            | Self::Open { source, .. }
            | Self::ArchiveEntryRead { source, .. }
            | Self::MetadataRead { source, .. }
            | Self::FingerprintRead { source, .. } => Some(source),
            Self::Archive { source, .. } | Self::ArchiveEntry { source, .. } => Some(source),
        }
    }
}
