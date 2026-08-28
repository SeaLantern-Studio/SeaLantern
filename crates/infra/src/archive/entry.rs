//! 归档条目路径校验与目标目录内的目录创建。
//!
//! 这些辅助函数与具体归档格式无关：ZIP 与 tar.gz 都需要把归档内声明的
//! 条目名收敛为安全相对路径，并在基于目录句柄（`cap_std::fs::Dir`）的
//! 解压根目录下按需创建父目录。
//!
//! 所有路径操作都通过目录句柄完成，不拼接绝对路径，避免校验与创建之间
//! 出现符号链接替换竞争。`destination` 参数仅用于错误消息展示。

use std::io;
use std::path::{Path, PathBuf};

use cap_std::fs::Dir;

use crate::fs::SafeRelativePath;

use super::ArchiveError;

/// 将归档条目名解析为安全的相对路径。
///
/// 拒绝绝对路径、含 `..` 遍历组件、含反斜杠等不可移植的条目名。
pub(super) fn safe_entry_path(
    archive_path: &Path,
    entry_name: &str,
) -> Result<SafeRelativePath, ArchiveError> {
    SafeRelativePath::parse(entry_name).map_err(|error| ArchiveError::UnsafeEntry {
        archive: archive_path.to_path_buf(),
        entry: entry_name.to_string(),
        reason: error.to_string(),
    })
}

/// 在解压根目录下按需逐层创建条目的父目录。
///
/// 归档不保证父目录条目先于子条目出现，因此写入文件前必须补齐路径。
/// 逐层创建而非一次性递归，确保每一层都经过目录句柄解析。
pub(super) fn ensure_parent_dirs(
    root: &Dir,
    path: &Path,
    destination: &Path,
) -> Result<(), ArchiveError> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    let mut current = PathBuf::new();
    for component in parent.components() {
        current.push(component);
        match root.open_dir(&current) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                root.create_dir(&current).map_err(|error| {
                    ArchiveError::io(
                        "create archive entry parent directory",
                        destination.join(&current),
                        error,
                    )
                })?;
                root.open_dir(&current).map_err(|error| {
                    ArchiveError::io(
                        "open archive entry parent directory",
                        destination.join(&current),
                        error,
                    )
                })?;
            }
            Err(error) => {
                return Err(ArchiveError::io(
                    "open archive entry parent directory",
                    destination.join(&current),
                    error,
                ));
            }
        }
    }
    Ok(())
}

/// 在解压根目录下创建显式的目录条目。
///
/// 目录条目可能出现在其内部文件之后（隐式父目录已被创建），此时视为成功。
pub(super) fn ensure_directory(
    root: &Dir,
    path: &Path,
    destination: &Path,
) -> Result<(), ArchiveError> {
    ensure_parent_dirs(root, path, destination)?;
    match root.create_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            root.open_dir(path).map(|_| ()).map_err(|error| {
                ArchiveError::io("open archive entry directory", destination.join(path), error)
            })
        }
        Err(error) => Err(ArchiveError::io(
            "create archive entry directory",
            destination.join(path),
            error,
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn rejects_traversal_entry_names() {
        let archive = Path::new("archive.zip");
        assert!(matches!(
            safe_entry_path(archive, "../outside.txt"),
            Err(ArchiveError::UnsafeEntry { .. })
        ));
        assert!(matches!(
            safe_entry_path(archive, "/absolute"),
            Err(ArchiveError::UnsafeEntry { .. })
        ));
    }

    #[test]
    fn accepts_nested_entry_names() {
        let parsed = safe_entry_path(Path::new("archive.zip"), "config/server.properties").unwrap();
        assert_eq!(parsed.as_path(), Path::new("config/server.properties"));
    }
}
