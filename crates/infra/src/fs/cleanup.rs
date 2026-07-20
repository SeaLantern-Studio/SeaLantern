use std::path::Path;

use super::FsError;

/// Removes a file or directory when it exists.
pub async fn remove_if_exists(path: impl AsRef<Path>) -> Result<bool, FsError> {
    let path = path.as_ref();
    match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {
            tokio::fs::remove_dir_all(path).await?;
            Ok(true)
        }
        Ok(_) => {
            tokio::fs::remove_file(path).await?;
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

/// Deletes all direct children of a directory without deleting the directory itself.
pub async fn clear_directory(path: impl AsRef<Path>) -> Result<(), FsError> {
    let mut entries = tokio::fs::read_dir(path).await?;
    while let Some(entry) = entries.next_entry().await? {
        remove_if_exists(entry.path()).await?;
    }
    Ok(())
}
