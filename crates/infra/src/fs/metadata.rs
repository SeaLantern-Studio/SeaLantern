use std::path::Path;
use std::time::SystemTime;

use super::FsError;

/// Basic metadata collected without following symbolic links.
#[derive(Clone, Debug)]
pub struct FileMetadata {
    /// Size in bytes for files and platform-reported directory metadata.
    pub size: u64,
    /// Last modification time when supported by the file system.
    pub modified: Option<SystemTime>,
    /// Whether the target is a regular file.
    pub is_file: bool,
    /// Whether the target is a directory.
    pub is_dir: bool,
    /// Whether the target itself is a symbolic link.
    pub is_symlink: bool,
}

/// Describes a path without following symbolic links.
pub async fn describe(path: impl AsRef<Path>) -> Result<FileMetadata, FsError> {
    let metadata = tokio::fs::symlink_metadata(path).await?;
    let file_type = metadata.file_type();
    Ok(FileMetadata {
        size: metadata.len(),
        modified: metadata.modified().ok(),
        is_file: file_type.is_file(),
        is_dir: file_type.is_dir(),
        is_symlink: file_type.is_symlink(),
    })
}

/// Returns a regular file's size.
pub async fn file_size(path: impl AsRef<Path>) -> Result<u64, FsError> {
    let path = path.as_ref();
    let metadata = describe(path).await?;
    if !metadata.is_file {
        return Err(FsError::InvalidPath {
            path: path.to_path_buf(),
            reason: "path is not a regular file",
        });
    }
    Ok(metadata.size)
}
