use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::observability;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

use super::FsError;

const STALE_LOCK_AFTER: Duration = Duration::from_secs(15 * 60);

/// 一个跨进程的锁，通过原子创建的同级文件来表示。
///
/// 锁在 drop 时释放。锁文件还记录持有者 PID；获取锁时会回收已退出
/// 进程留下的锁，无法确认持有者时则使用保守的过期时间兜底。
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
            let path = lock_path(resource)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| FsError::io("create lock directory", parent, error))?;
            }

            let mut stale_recovery_attempted = false;
            let mut file = loop {
                match std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&path)
                {
                    Ok(file) => break file,
                    Err(error)
                        if error.kind() == std::io::ErrorKind::AlreadyExists
                            && !stale_recovery_attempted
                            && recover_stale_lock(&path)? =>
                    {
                        stale_recovery_attempted = true;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                        return Err(FsError::AlreadyLocked(path));
                    }
                    Err(error) => return Err(FsError::io("create lock file", &path, error)),
                }
            };

            if let Err(error) =
                writeln!(file, "pid={}\ncreated_at_ms={}", std::process::id(), timestamp_ms())
            {
                drop(file);
                let _ = std::fs::remove_file(&path);
                return Err(FsError::io("write lock metadata", &path, error));
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

fn lock_path(resource: &Path) -> Result<PathBuf, FsError> {
    let file_name = resource.file_name().ok_or_else(|| FsError::InvalidPath {
        path: resource.to_path_buf(),
        reason: "lock resource has no file name",
    })?;
    Ok(resource.with_file_name(format!("{}.lock", file_name.to_string_lossy())))
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// 仅在持有者已经退出，或旧格式没有可验证的 PID 且已超过兜底期限时回收锁。
fn recover_stale_lock(path: &Path) -> Result<bool, FsError> {
    if !lock_is_stale(path) {
        return Ok(false);
    }

    match std::fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(FsError::io("remove stale lock file", path, error)),
    }
}

fn lock_is_stale(path: &Path) -> bool {
    let content = std::fs::read_to_string(path).ok();
    if let Some(pid) = content
        .as_deref()
        .and_then(|content| metadata_value(content, "pid"))
        .and_then(|value| value.parse::<u32>().ok())
    {
        if pid == std::process::id() {
            return false;
        }
        return !process_is_alive(pid);
    }

    let created_at_is_stale = content
        .as_deref()
        .and_then(|content| metadata_value(content, "created_at_ms"))
        .and_then(|value| value.parse::<u128>().ok())
        .is_some_and(timestamp_is_stale);
    created_at_is_stale || file_age_is_stale(path)
}

fn metadata_value<'a>(content: &'a str, key: &str) -> Option<&'a str> {
    content
        .lines()
        .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
}

fn timestamp_is_stale(created_at_ms: u128) -> bool {
    timestamp_ms().saturating_sub(created_at_ms) >= STALE_LOCK_AFTER.as_millis()
}

fn file_age_is_stale(path: &Path) -> bool {
    std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .and_then(|modified| modified.elapsed().map_err(std::io::Error::other))
        .is_ok_and(|age| age >= STALE_LOCK_AFTER)
}

fn process_is_alive(pid: u32) -> bool {
    let pid = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::new(),
    );
    system.process(pid).is_some()
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if !self.released
            && let Err(error) = std::fs::remove_file(&self.path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            observability::lock_release_failed(&self.path, &error);
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

    #[test]
    fn recovers_lock_owned_by_dead_process() {
        let root = crate::fs::test_dir("lock-stale");
        let resource = root.join("state.json");
        let path = lock_path(&resource).unwrap();
        std::fs::write(&path, "pid=4294967295\ncreated_at_ms=0\n")
            .expect("stale lock fixture should be written");

        let lock = FileLock::try_acquire(&resource).expect("stale lock should be reclaimed");
        assert!(lock.path().exists());
        drop(lock);
        std::fs::remove_dir_all(root).unwrap();
    }
}
