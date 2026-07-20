use std::path::{Path, PathBuf};

use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::observability;

use super::{ensure_parent, FsError};

/// Writes a complete file through a sibling temporary file.
///
/// On Unix, replacing an existing destination uses an atomic rename. Windows
/// does not allow that rename when the destination exists, so replacement is
/// durable but has a short non-atomic remove-and-rename interval.
pub async fn write_atomic(path: impl AsRef<Path>, contents: &[u8]) -> Result<(), FsError> {
    let path = path.as_ref();
    let result = write_atomic_inner(path, contents).await;
    if let Err(error) = &result {
        observability::atomic_write_failed(path, error);
    }
    result
}

async fn write_atomic_inner(path: &Path, contents: &[u8]) -> Result<(), FsError> {
    let file_name = path.file_name().ok_or_else(|| FsError::InvalidPath {
        path: path.to_path_buf(),
        reason: "destination has no file name",
    })?;
    ensure_parent(path).await?;

    let temporary = temporary_path(path, file_name);
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .await?;

    let mut write_result = async {
        file.write_all(contents).await?;
        file.flush().await?;
        file.sync_all().await?;
        Ok::<(), FsError>(())
    }
    .await;
    drop(file);

    if write_result.is_ok() {
        write_result = replace_file(&temporary, path).await;
    }

    if write_result.is_err() {
        let _ = tokio::fs::remove_file(&temporary).await;
    }
    write_result
}

fn temporary_path(destination: &Path, file_name: &std::ffi::OsStr) -> PathBuf {
    destination.with_file_name(format!(".{}.{}.tmp", file_name.to_string_lossy(), Uuid::new_v4()))
}

#[cfg(not(windows))]
async fn replace_file(temporary: &Path, destination: &Path) -> Result<(), FsError> {
    tokio::fs::rename(temporary, destination).await?;
    Ok(())
}

#[cfg(windows)]
async fn replace_file(temporary: &Path, destination: &Path) -> Result<(), FsError> {
    match tokio::fs::remove_file(destination).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    tokio::fs::rename(temporary, destination).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn replaces_existing_file() {
        let root = crate::fs::test_dir("atomic");
        let target = root.join("settings.json");
        tokio::fs::write(&target, b"old").await.unwrap();

        write_atomic(&target, b"new").await.unwrap();

        assert_eq!(tokio::fs::read(&target).await.unwrap(), b"new");
        std::fs::remove_dir_all(root).unwrap();
    }
}
