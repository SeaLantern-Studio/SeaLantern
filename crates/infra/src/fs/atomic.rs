use std::io::Write;
use std::path::Path;

use atomicwrites::{AllowOverwrite, AtomicFile};
use cap_std::fs::{Dir, OpenOptions};
use uuid::Uuid;

use crate::observability;

use super::{FsError, SafeRelativePath, ensure_parent};

/// 通过同级临时文件原子性地替换整个文件。
///
/// 替换操作委托给平台特定的实现。它提供
/// 原子可见性，但不保证在所有支持的文件系统上父目录项的崩溃持久性。
pub async fn write_atomic(path: impl AsRef<Path>, contents: &[u8]) -> Result<(), FsError> {
    let path = path.as_ref();
    let result = write_atomic_inner(path, contents).await;
    if let Err(error) = &result {
        observability::atomic_write_failed(path, error);
    }
    result
}

/// 通过同级临时文件同步原子性地替换整个文件。
///
/// 此入口适用于不能等待异步 future 的受限运行时回调。调用方应仅写入小而有界的数据，
/// 避免在交互线程上执行长时间磁盘操作。
pub fn write_atomic_blocking(path: impl AsRef<Path>, contents: &[u8]) -> Result<(), FsError> {
    let path = path.as_ref();
    let result = write_atomic_blocking_inner(path, contents);
    if let Err(error) = &result {
        observability::atomic_write_failed(path, error);
    }
    result
}

async fn write_atomic_inner(path: &Path, contents: &[u8]) -> Result<(), FsError> {
    ensure_parent(path).await?;
    let destination = path.to_path_buf();
    let contents = contents.to_vec();
    tokio::task::spawn_blocking(move || write_atomic_blocking_inner(&destination, &contents))
        .await
        .map_err(|error| FsError::task("atomically replace file", error.to_string()))?
}

fn write_atomic_blocking_inner(path: &Path, contents: &[u8]) -> Result<(), FsError> {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    if !parent.as_os_str().is_empty() {
        std::fs::create_dir_all(parent)
            .map_err(|error| FsError::io("create parent directory", parent, error))?;
    }
    AtomicFile::new(path, AllowOverwrite)
        .write(|file| file.write_all(contents))
        .map_err(std::io::Error::from)
        .map_err(|error| FsError::io("atomically replace file", path, error))
}

/// 在基于能力的目录根内原子性地写入字节。
pub(crate) fn write_atomic_in(
    root: &Dir,
    path: &SafeRelativePath,
    contents: &[u8],
) -> Result<(), FsError> {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    if !parent.as_os_str().is_empty() {
        root.create_dir_all(parent)
            .map_err(|error| FsError::io("create cache directory", path.as_path(), error))?;
    }

    let file_name = path.file_name().ok_or_else(|| FsError::InvalidPath {
        path: path.to_path_buf(),
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
        root.rename(&temporary, root, path).map_err(|error| {
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

    #[test]
    fn blocking_replaces_existing_file() {
        let root = crate::fs::test_dir("atomic-blocking");
        let target = root.join("settings.json");
        std::fs::write(&target, b"old").unwrap();

        write_atomic_blocking(&target, b"new").unwrap();

        assert_eq!(std::fs::read(&target).unwrap(), b"new");
        std::fs::remove_dir_all(root).unwrap();
    }
}
