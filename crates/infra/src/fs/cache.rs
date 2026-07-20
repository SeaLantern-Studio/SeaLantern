use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{
    clear_directory, describe, ensure_dir, read_limited, write_atomic, DataLimit, FsError,
    SafeRelativePath,
};

/// A directory-backed byte cache with safe relative keys.
#[derive(Clone, Debug)]
pub struct FileCache {
    root: PathBuf,
}

impl FileCache {
    /// Creates a cache rooted at the supplied directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the cache root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Stores bytes under a safe relative key.
    pub async fn put(&self, key: impl AsRef<Path>, bytes: &[u8]) -> Result<(), FsError> {
        ensure_dir(&self.root).await?;
        let destination = self.destination(key)?;
        write_atomic(destination, bytes).await
    }

    /// Reads cached bytes subject to a caller-defined data limit.
    pub async fn get(
        &self,
        key: impl AsRef<Path>,
        limit: DataLimit,
    ) -> Result<Option<Vec<u8>>, FsError> {
        let destination = self.destination(key)?;
        self.read_optional(&destination, limit).await
    }

    /// Reads an entry only when its modification time is within max_age.
    pub async fn get_fresh(
        &self,
        key: impl AsRef<Path>,
        limit: DataLimit,
        max_age: Duration,
    ) -> Result<Option<Vec<u8>>, FsError> {
        let destination = self.destination(key)?;
        let metadata = match describe(&destination).await {
            Ok(metadata) => metadata,
            Err(FsError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None)
            }
            Err(error) => return Err(error),
        };
        if metadata
            .modified
            .and_then(|time| time.elapsed().ok())
            .is_some_and(|age| age > max_age)
        {
            return Ok(None);
        }
        self.read_optional(&destination, limit).await
    }

    /// Deletes every cached entry but keeps the cache root available.
    pub async fn clear(&self) -> Result<(), FsError> {
        ensure_dir(&self.root).await?;
        clear_directory(&self.root).await
    }

    fn destination(&self, key: impl AsRef<Path>) -> Result<PathBuf, FsError> {
        SafeRelativePath::parse(key).and_then(|key| key.resolve_under(&self.root))
    }

    async fn read_optional(
        &self,
        destination: &Path,
        limit: DataLimit,
    ) -> Result<Option<Vec<u8>>, FsError> {
        match read_limited(destination, limit).await {
            Ok(bytes) => Ok(Some(bytes)),
            Err(FsError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn round_trips_cached_bytes() {
        let root = crate::fs::test_dir("cache");
        let cache = FileCache::new(&root);
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
        std::fs::remove_dir_all(root).unwrap();
    }
}
