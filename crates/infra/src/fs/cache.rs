use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use cap_std::ambient_authority;
use cap_std::fs::Dir;
use cap_std::time::SystemClock;

use super::atomic::write_atomic_in;
use super::{DataLimit, FsError, SafeRelativePath};

/// A directory-backed byte cache scoped to an opened directory capability.
#[derive(Clone, Debug)]
pub struct FileCache {
    root: Arc<Dir>,
    root_path: PathBuf,
}

impl FileCache {
    /// Opens a cache rooted at the supplied directory.
    ///
    /// The directory handle, rather than a joined path, is used for every
    /// cache operation so concurrent symlink replacement cannot escape root.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, FsError> {
        let root_path = root.into();
        let result = (|| {
            std::fs::create_dir_all(&root_path)
                .map_err(|error| FsError::io("create cache root", &root_path, error))?;
            let root = Dir::open_ambient_dir(&root_path, ambient_authority())
                .map_err(|error| FsError::io("open cache root", &root_path, error))?;
            Ok(Self {
                root: Arc::new(root),
                root_path: root_path.clone(),
            })
        })();
        if let Err(error) = &result {
            crate::observability::cache_operation_failed("open cache", &root_path, error);
        }
        result
    }

    /// Returns the cache root selected when this cache was opened.
    pub fn root(&self) -> &Path {
        &self.root_path
    }

    /// Stores bytes under a safe relative key.
    pub async fn put(&self, key: impl AsRef<Path>, bytes: &[u8]) -> Result<(), FsError> {
        let root = Arc::clone(&self.root);
        let key = SafeRelativePath::parse(key)?;
        let bytes = bytes.to_vec();
        let path = self.root_path.join(key.as_path());
        let result =
            run_blocking("write cache entry", move || write_atomic_in(&root, &key, &bytes)).await;
        trace_cache_result("write cache entry", &path, &result);
        result
    }

    /// Reads cached bytes subject to a caller-defined data limit.
    pub async fn get(
        &self,
        key: impl AsRef<Path>,
        limit: DataLimit,
    ) -> Result<Option<Vec<u8>>, FsError> {
        let root = Arc::clone(&self.root);
        let key = SafeRelativePath::parse(key)?;
        let path = self.root_path.join(key.as_path());
        let result =
            run_blocking("read cache entry", move || read_optional(&root, &key, limit)).await;
        trace_cache_result("read cache entry", &path, &result);
        result
    }

    /// Reads an entry only when its modification time is within max_age.
    pub async fn get_fresh(
        &self,
        key: impl AsRef<Path>,
        limit: DataLimit,
        max_age: Duration,
    ) -> Result<Option<Vec<u8>>, FsError> {
        let root = Arc::clone(&self.root);
        let key = SafeRelativePath::parse(key)?;
        let path = self.root_path.join(key.as_path());
        let result = run_blocking("read fresh cache entry", move || {
            let metadata = match root.symlink_metadata(key.as_path()) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => {
                    return Err(FsError::io("read cache entry metadata", key.as_path(), error))
                }
            };
            if metadata.file_type().is_symlink() {
                return Err(FsError::InvalidPath {
                    path: key.as_path().to_path_buf(),
                    reason: "cache entry must not be a symbolic link",
                });
            }
            if metadata
                .modified()
                .ok()
                .and_then(|time| SystemClock::new(ambient_authority()).elapsed(time).ok())
                .is_some_and(|age| age > max_age)
            {
                return Ok(None);
            }
            read_optional(&root, &key, limit)
        })
        .await;
        trace_cache_result("read fresh cache entry", &path, &result);
        result
    }

    /// Deletes every cached entry but keeps the cache root available.
    pub async fn clear(&self) -> Result<(), FsError> {
        let root = Arc::clone(&self.root);
        let path = self.root_path.clone();
        let result = run_blocking("clear cache", move || {
            for entry in root
                .read_dir(".")
                .map_err(|error| FsError::io("read cache directory", ".", error))?
            {
                let entry =
                    entry.map_err(|error| FsError::io("iterate cache directory", ".", error))?;
                let name = entry.file_name();
                let metadata = root
                    .symlink_metadata(&name)
                    .map_err(|error| FsError::io("read cache entry metadata", &name, error))?;
                if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
                    root.remove_dir_all(&name)
                        .map_err(|error| FsError::io("remove cached directory", &name, error))?;
                } else {
                    root.remove_file(&name)
                        .map_err(|error| FsError::io("remove cache entry", &name, error))?;
                }
            }
            Ok(())
        })
        .await;
        trace_cache_result("clear cache", &path, &result);
        result
    }
}

fn read_optional(
    root: &Dir,
    key: &SafeRelativePath,
    limit: DataLimit,
) -> Result<Option<Vec<u8>>, FsError> {
    let metadata = match root.symlink_metadata(key.as_path()) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(FsError::io("read cache entry metadata", key.as_path(), error)),
    };
    if metadata.file_type().is_symlink() {
        return Err(FsError::InvalidPath {
            path: key.as_path().to_path_buf(),
            reason: "cache entry must not be a symbolic link",
        });
    }
    if !metadata.file_type().is_file() {
        return Err(FsError::InvalidPath {
            path: key.as_path().to_path_buf(),
            reason: "cache entry is not a regular file",
        });
    }

    let file = root
        .open(key.as_path())
        .map_err(|error| FsError::io("open cache entry", key.as_path(), error))?;
    let mut reader = file.take((limit.max_bytes() as u64).saturating_add(1));
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| FsError::io("read cache entry", key.as_path(), error))?;
    if bytes.len() > limit.max_bytes() {
        return Err(FsError::DataLimitExceeded {
            path: key.as_path().to_path_buf(),
            limit: limit.max_bytes(),
            observed_at_least: bytes.len(),
        });
    }
    Ok(Some(bytes))
}

async fn run_blocking<T: Send + 'static>(
    operation: &'static str,
    task: impl FnOnce() -> Result<T, FsError> + Send + 'static,
) -> Result<T, FsError> {
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|error| FsError::task(operation, error.to_string()))?
}

fn trace_cache_result<T>(operation: &str, path: &Path, result: &Result<T, FsError>) {
    if let Err(error) = result {
        crate::observability::cache_operation_failed(operation, path, error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn round_trips_cached_bytes() {
        let root = crate::fs::test_dir("cache");
        let cache = FileCache::new(&root).unwrap();
        cache.put("entries/item.bin", b"cached").await.unwrap();

        assert_eq!(
            cache
                .get("entries/item.bin", DataLimit::new(32))
                .await
                .unwrap(),
            Some(b"cached".to_vec())
        );
        cache.clear().await.unwrap();
        assert_eq!(
            cache
                .get("entries/item.bin", DataLimit::new(32))
                .await
                .unwrap(),
            None
        );
        drop(cache);
        std::fs::remove_dir_all(root).unwrap();
    }
}
