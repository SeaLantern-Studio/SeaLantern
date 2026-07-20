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
        std::fs::create_dir_all(&root_path)?;
        let root = Dir::open_ambient_dir(&root_path, ambient_authority())?;
        Ok(Self { root: Arc::new(root), root_path })
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
        run_blocking(move || write_atomic_in(&root, &key, &bytes)).await
    }

    /// Reads cached bytes subject to a caller-defined data limit.
    pub async fn get(
        &self,
        key: impl AsRef<Path>,
        limit: DataLimit,
    ) -> Result<Option<Vec<u8>>, FsError> {
        let root = Arc::clone(&self.root);
        let key = SafeRelativePath::parse(key)?;
        run_blocking(move || read_optional(&root, &key, limit)).await
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
        run_blocking(move || {
            let metadata = match root.symlink_metadata(key.as_path()) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(error) => return Err(error.into()),
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
        .await
    }

    /// Deletes every cached entry but keeps the cache root available.
    pub async fn clear(&self) -> Result<(), FsError> {
        let root = Arc::clone(&self.root);
        run_blocking(move || {
            for entry in root.read_dir(".")? {
                let entry = entry?;
                let name = entry.file_name();
                let metadata = root.symlink_metadata(&name)?;
                if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
                    root.remove_dir_all(&name)?;
                } else {
                    root.remove_file(&name)?;
                }
            }
            Ok(())
        })
        .await
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
        Err(error) => return Err(error.into()),
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

    let file = root.open(key.as_path())?;
    let mut reader = file.take((limit.max_bytes() as u64).saturating_add(1));
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    if bytes.len() > limit.max_bytes() {
        return Err(FsError::DataLimitExceeded {
            path: key.as_path().to_path_buf(),
            limit: limit.max_bytes(),
        });
    }
    Ok(Some(bytes))
}

async fn run_blocking<T: Send + 'static>(
    task: impl FnOnce() -> Result<T, FsError> + Send + 'static,
) -> Result<T, FsError> {
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|error| FsError::Task(error.to_string()))?
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
