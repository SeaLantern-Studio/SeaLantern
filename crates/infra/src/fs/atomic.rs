use std::io::Write;
use std::path::Path;

use atomicwrites::{AllowOverwrite, AtomicFile};
use cap_std::fs::{Dir, OpenOptions};
use uuid::Uuid;

use crate::observability;

use super::{ensure_parent, FsError, SafeRelativePath};

/// Atomically replaces a complete file through a sibling temporary file.
///
/// The replacement is delegated to platform-specific operations. It provides
/// atomic visibility, but does not promise crash durability of the parent
/// directory entry on every supported file system.
pub async fn write_atomic(path: impl AsRef<Path>, contents: &[u8]) -> Result<(), FsError> {
    let path = path.as_ref();
    let result = write_atomic_inner(path, contents).await;
    if let Err(error) = &result {
        observability::atomic_write_failed(path, error);
    }
    result
}

async fn write_atomic_inner(path: &Path, contents: &[u8]) -> Result<(), FsError> {
    ensure_parent(path).await?;
    let destination = path.to_path_buf();
    let error_path = destination.clone();
    let contents = contents.to_vec();
    tokio::task::spawn_blocking(move || {
        AtomicFile::new(&destination, AllowOverwrite)
            .write(|file| file.write_all(&contents))
            .map_err(std::io::Error::from)
            .map_err(|error| FsError::io("atomically replace file", &error_path, error))
    })
    .await
    .map_err(|error| FsError::task("atomically replace file", error.to_string()))?
}

/// Writes bytes atomically within a capability-based directory root.
pub(crate) fn write_atomic_in(
    root: &Dir,
    path: &SafeRelativePath,
    contents: &[u8],
) -> Result<(), FsError> {
    let parent = path.as_path().parent().unwrap_or_else(|| Path::new(""));
    if !parent.as_os_str().is_empty() {
        root.create_dir_all(parent)
            .map_err(|error| FsError::io("create cache directory", path.as_path(), error))?;
    }

    let file_name = path
        .as_path()
        .file_name()
        .ok_or_else(|| FsError::InvalidPath {
            path: path.as_path().to_path_buf(),
            reason: "destination has no file name",
        })?;
    let temporary = parent.join(format!(".{}.{}.tmp", file_name.to_string_lossy(), Uuid::new_v4()));

    let write_result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = root
            .open_with(&temporary, &options)
            .map_err(|error| FsError::io("create cache temporary file", &temporary, error))?;
        file.write_all(contents)
            .map_err(|error| FsError::io("write cache temporary file", &temporary, error))?;
        file.sync_all()
            .map_err(|error| FsError::io("sync cache temporary file", &temporary, error))?;
        root.rename(&temporary, root, path.as_path())
            .map_err(|error| {
                FsError::io("atomically replace cache entry", path.as_path(), error)
            })?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = root.remove_file(&temporary);
    }
    write_result
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
