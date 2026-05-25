//! 密钥文件管理：加载、生成并持久化 iroh `SecretKey`。

use std::path::Path;

use crate::Result;
use crate::error::PersistError;
use crate::types::SecretKey;

const KEY_LEN: usize = 32;

/// 从文件加载密钥；若文件不存在则原子生成新密钥并保存。
///
/// 通过先加载、失败时原子创建的顺序避免 TOCTOU 竞态。
pub fn load_or_generate_key(path: &Path) -> Result<SecretKey> {
    match load_key(path) {
        Ok(key) => Ok(key),
        Err(e) if is_not_found(&e) => generate_new_key(path),
        Err(e) => Err(e),
    }
}

/// 强制重新生成新密钥并保存。
///
/// 使用原子创建（`create_new`）写入；若文件已被并发创建则 fallback 到加载。
pub fn generate_new_key(path: &Path) -> Result<SecretKey> {
    let bytes: [u8; KEY_LEN] = rand::random();
    let key = SecretKey::from_bytes(&bytes);
    match save_key_exclusive(path, &key) {
        Ok(()) => Ok(key),
        Err(e) if is_already_exists(&e) => load_key(path),
        Err(e) => Err(e),
    }
}

fn load_key(path: &Path) -> Result<SecretKey> {
    let bytes = std::fs::read(path).map_err(|e| PersistError::PathIo {
        op: "read key file",
        path: path.to_path_buf(),
        source: e,
    })?;
    if bytes.len() != KEY_LEN {
        return Err(PersistError::InvalidKeyLength {
            expected: KEY_LEN,
            actual: bytes.len(),
        }
        .into());
    }
    let arr: [u8; KEY_LEN] =
        bytes
            .try_into()
            .map_err(|v: Vec<u8>| PersistError::InvalidKeyLength {
                expected: KEY_LEN,
                actual: v.len(),
            })?;
    Ok(SecretKey::from_bytes(&arr))
}

/// 原子创建并写入密钥文件（`create_new` 模式，文件已存在则失败）。
fn save_key_exclusive(path: &Path, key: &SecretKey) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| PersistError::PathIo {
            op: "create key directory",
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| PersistError::PathIo {
                op: "create key file",
                path: path.to_path_buf(),
                source: e,
            })?;
        file.write_all(&key.to_bytes())
            .map_err(|e| PersistError::PathIo {
                op: "write key file",
                path: path.to_path_buf(),
                source: e,
            })?;
    }

    #[cfg(not(unix))]
    {
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| PersistError::PathIo {
                op: "create key file",
                path: path.to_path_buf(),
                source: e,
            })?;
        file.write_all(&key.to_bytes())
            .map_err(|e| PersistError::PathIo {
                op: "write key file",
                path: path.to_path_buf(),
                source: e,
            })?;
    }

    Ok(())
}

/// 检查错误链中是否包含 `io::ErrorKind::NotFound`。
fn is_not_found(err: &crate::error::SculkError) -> bool {
    matches!(
        err,
        crate::error::SculkError::Persist(PersistError::PathIo { source, .. })
        if source.kind() == std::io::ErrorKind::NotFound
    )
}

/// 检查错误链中是否包含 `io::ErrorKind::AlreadyExists`。
fn is_already_exists(err: &crate::error::SculkError) -> bool {
    matches!(
        err,
        crate::error::SculkError::Persist(PersistError::PathIo { source, .. })
        if source.kind() == std::io::ErrorKind::AlreadyExists
    )
}
