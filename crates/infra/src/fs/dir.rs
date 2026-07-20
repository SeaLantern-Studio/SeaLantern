use std::path::Path;

use super::FsError;

/// Creates a directory and all missing ancestors.
pub async fn ensure_dir(path: impl AsRef<Path>) -> Result<(), FsError> {
    let path = path.as_ref();
    tokio::fs::create_dir_all(path)
        .await
        .map_err(|error| FsError::io("create directory", path, error))?;
    Ok(())
}

/// Creates the parent directory of a path when it has one.
pub async fn ensure_parent(path: impl AsRef<Path>) -> Result<(), FsError> {
    if let Some(parent) = path.as_ref().parent() {
        ensure_dir(parent).await?;
    }
    Ok(())
}
