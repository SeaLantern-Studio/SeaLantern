use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

use super::PersistenceError;

/// 按资源路径协调同一进程内的异步持久化操作。
///
/// 此协调器只处理进程内竞争。SQLite 的 WAL 和 busy timeout
/// 仍负责与其他进程的协调。
#[derive(Default)]
pub struct ProcessLockRegistry {
    locks: Mutex<HashMap<PathBuf, Weak<AsyncMutex<()>>>>,
}

impl ProcessLockRegistry {
    /// 获取资源的独占异步锁。
    pub async fn lock(
        &self,
        resource: impl AsRef<Path>,
    ) -> Result<OwnedMutexGuard<()>, PersistenceError> {
        let resource = resource_identity(resource.as_ref())?;
        let lock = {
            let mut locks = self
                .locks
                .lock()
                .map_err(|error| PersistenceError::Coordination {
                    resource: resource.clone(),
                    message: error.to_string(),
                })?;
            locks.retain(|_, lock| lock.strong_count() > 0);
            match locks.get(&resource).and_then(Weak::upgrade) {
                Some(lock) => lock,
                None => {
                    let lock = Arc::new(AsyncMutex::new(()));
                    locks.insert(resource, Arc::downgrade(&lock));
                    lock
                }
            }
        };
        Ok(lock.lock_owned().await)
    }
}

/// 返回进程级的资源协调器。
pub fn process_lock_registry() -> &'static ProcessLockRegistry {
    static REGISTRY: OnceLock<ProcessLockRegistry> = OnceLock::new();
    REGISTRY.get_or_init(ProcessLockRegistry::default)
}

fn resource_identity(path: &Path) -> Result<PathBuf, PersistenceError> {
    if path.as_os_str().is_empty() {
        return Err(PersistenceError::Coordination {
            resource: path.to_path_buf(),
            message: "resource path is empty".to_owned(),
        });
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|current_dir| current_dir.join(path))
            .map_err(|error| PersistenceError::Coordination {
                resource: path.to_path_buf(),
                message: error.to_string(),
            })?
    };
    if let Ok(canonical) = std::fs::canonicalize(&absolute) {
        return Ok(canonical);
    }
    if let (Some(parent), Some(name)) = (absolute.parent(), absolute.file_name()) {
        if let Ok(canonical_parent) = std::fs::canonicalize(parent) {
            return Ok(canonical_parent.join(name));
        }
    }
    Ok(absolute)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[tokio::test]
    async fn serializes_access_to_the_same_resource() {
        let registry = Arc::new(ProcessLockRegistry::default());
        let first = registry.lock("same-resource").await.unwrap();
        let second_registry = Arc::clone(&registry);
        let waiting = tokio::spawn(async move { second_registry.lock("same-resource").await });

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        assert!(!waiting.is_finished());
        drop(first);
        assert!(waiting.await.unwrap().is_ok());
    }
}
