use std::path::{Path, PathBuf};

use crate::observability;

use super::FsError;

/// A cross-process lock represented by an atomically created sibling file.
///
/// The lock is released on drop. An abnormal process exit can leave a stale
/// lock file, so callers that need recovery should define an explicit stale
/// lock policy at their application boundary.
#[derive(Debug)]
pub struct FileLock {
    path: PathBuf,
    released: bool,
}

impl FileLock {
    /// Acquires a lock for a resource by creating a sibling .lock file.
    pub fn try_acquire(resource: impl AsRef<Path>) -> Result<Self, FsError> {
        let resource = resource.as_ref();
        let result = (|| {
            let file_name = resource.file_name().ok_or_else(|| FsError::InvalidPath {
                path: resource.to_path_buf(),
                reason: "lock resource has no file name",
            })?;
            let path = resource.with_file_name(format!("{}.lock", file_name.to_string_lossy()));
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| FsError::io("create lock directory", parent, error))?;
            }

            let mut file = match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    return Err(FsError::AlreadyLocked(path));
                }
                Err(error) => return Err(FsError::io("create lock file", &path, error)),
                Ok(file) => file,
            };
            {
                use std::io::Write;
                if let Err(error) = writeln!(file, "pid={}", std::process::id()) {
                    drop(file);
                    let _ = std::fs::remove_file(&path);
                    return Err(FsError::io("write lock metadata", &path, error));
                }
            }
            Ok(Self { path, released: false })
        })();
        if let Err(error) = &result {
            observability::lock_acquire_failed(resource, error);
        }
        result
    }

    /// Returns the lock-file path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Releases the lock before the guard is dropped.
    pub fn release(mut self) -> Result<(), FsError> {
        let result = std::fs::remove_file(&self.path)
            .map_err(|error| FsError::io("release lock", &self.path, error));
        if result.is_ok() {
            self.released = true;
        } else if let Err(error) = &result {
            observability::lock_release_failed(&self.path, error);
        }
        result
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if !self.released {
            if let Err(error) = std::fs::remove_file(&self.path) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    observability::lock_release_failed(&self.path, &error);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prevents_concurrent_acquisition() {
        let root = crate::fs::test_dir("lock");
        let resource = root.join("state.json");
        let lock = FileLock::try_acquire(&resource).unwrap();

        assert!(matches!(FileLock::try_acquire(&resource), Err(FsError::AlreadyLocked(_))));
        lock.release().unwrap();
        let replacement = FileLock::try_acquire(&resource).unwrap();
        assert!(replacement.path().exists());
        drop(replacement);
        std::fs::remove_dir_all(root).unwrap();
    }
}
