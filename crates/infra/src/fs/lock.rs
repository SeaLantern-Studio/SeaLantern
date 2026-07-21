use std::path::{Path, PathBuf};

use crate::observability;

use super::FsError;

/// 一个跨进程的锁，通过原子创建的同级文件来表示。
///
/// 锁在 drop 时释放。异常进程退出可能留下过时的
/// 锁文件，因此需要恢复的调用者应在应用程序边界
/// 定义明确的过期锁策略。
#[derive(Debug)]
pub struct FileLock {
    path: PathBuf,
    released: bool,
}

impl FileLock {
    /// 通过创建同级的 .lock 文件来获取资源的锁。
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

    /// 返回锁文件的路径。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 在守卫被释放之前手动释放锁。
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
