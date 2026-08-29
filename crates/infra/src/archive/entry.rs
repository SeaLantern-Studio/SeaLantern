//! 归档条目路径校验与目标目录内的目录创建。
//!
//! 这些辅助函数与具体归档格式无关：ZIP 与 tar.gz 都需要把归档内声明的
//! 条目名收敛为安全相对路径，并在基于目录句柄（`cap_std::fs::Dir`）的
//! 解压根目录下按需创建父目录。
//!
//! 所有路径操作都通过目录句柄完成，不拼接绝对路径，避免校验与创建之间
//! 出现符号链接替换竞争。`destination` 参数仅用于错误消息展示。

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use cap_std::fs::Dir;

use crate::fs::SafeRelativePath;

use super::ArchiveError;

/// 已登记的条目输出路径，用于在写入之前发现路径冲突。
///
/// 单靠 [`SafeRelativePath`] 只能保证每个条目不逃逸解压根目录，无法发现条目
/// 之间的相互冲突。以下三类都会导致条目覆盖、写入失败或语义歧义：
///
/// - 两个条目解析到同一输出路径
/// - 某个条目的祖先路径已作为普通文件出现，该文件会阻断此条目的父目录
/// - 同一路径既作普通文件又作目录使用
///
/// 目录句柄的逐层解析最终也会让这些归档写入失败，但那时已经流式写出了部分
/// 内容，且错误退化为笼统的 I/O 失败。提前登记可以给出明确原因，并让 tar.gz
/// 这类无法预检的格式尽早中止。
#[derive(Debug, Default)]
pub(super) struct EntryPathRegistry {
    /// 已出现的显式条目路径，用于重复检测。
    seen: HashSet<PathBuf>,
    /// 已作为普通文件出现的路径。
    files: HashSet<PathBuf>,
    /// 已作为目录使用的路径，含由子条目隐式引入的祖先目录。
    directories: HashSet<PathBuf>,
}

impl EntryPathRegistry {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            seen: HashSet::with_capacity(capacity),
            files: HashSet::new(),
            directories: HashSet::new(),
        }
    }

    /// 登记条目路径，与既有条目冲突时返回 [`ArchiveError::UnsafeEntry`]。
    ///
    /// 祖先目录在登记时一并记入，因此「同一路径既作文件又作目录」的判定是
    /// 常数时间的集合查询，无需遍历已登记条目。
    pub(super) fn register(
        &mut self,
        archive_path: &Path,
        relative: &SafeRelativePath,
        entry_name: &str,
        is_directory: bool,
    ) -> Result<(), ArchiveError> {
        let path = relative.as_path();
        if !self.seen.insert(path.to_path_buf()) {
            return Err(conflict(
                archive_path,
                entry_name,
                "archive contains duplicate output paths",
            ));
        }
        if ancestors(path).any(|ancestor| self.files.contains(ancestor)) {
            return Err(conflict(
                archive_path,
                entry_name,
                "a file entry blocks a parent directory of this entry",
            ));
        }

        if is_directory {
            if self.files.contains(path) {
                return Err(conflict(
                    archive_path,
                    entry_name,
                    "path is already used by a file entry",
                ));
            }
            self.directories.insert(path.to_path_buf());
        } else {
            if self.directories.contains(path) {
                return Err(conflict(
                    archive_path,
                    entry_name,
                    "path is already used as a directory",
                ));
            }
            self.files.insert(path.to_path_buf());
        }

        self.directories
            .extend(ancestors(path).map(Path::to_path_buf));
        Ok(())
    }
}

/// 遍历路径的各级祖先，跳过路径自身与空的根。
fn ancestors(path: &Path) -> impl Iterator<Item = &Path> {
    path.ancestors()
        .skip(1)
        .filter(|ancestor| !ancestor.as_os_str().is_empty())
}

fn conflict(archive_path: &Path, entry_name: &str, reason: &'static str) -> ArchiveError {
    ArchiveError::UnsafeEntry {
        archive: archive_path.to_path_buf(),
        entry: entry_name.to_string(),
        reason: reason.to_string(),
    }
}

/// 校验条目名长度未超过上限。
///
/// 接受原始字节而非字符串：条目名在转为 `String` 时就会分配一次，超长路径
/// 必须在那之前拦下，否则限制形同虚设。
///
/// 条目名与条目内容不同，会被存入去重集合并持有到解压结束，因此需要独立于
/// [`ExtractionLimits::max_entry_bytes`] 的上限约束其累积内存占用。
pub(super) fn check_entry_path_length(
    archive_path: &Path,
    entry_name: &[u8],
    maximum: usize,
) -> Result<(), ArchiveError> {
    if entry_name.len() <= maximum {
        return Ok(());
    }
    Err(ArchiveError::LimitExceeded {
        archive: archive_path.to_path_buf(),
        limit: "entry path bytes",
        observed: entry_name.len() as u64,
        maximum: maximum as u64,
    })
}

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

    #[test]
    fn rejects_entry_names_above_the_length_limit() {
        let archive = Path::new("archive.zip");
        assert!(check_entry_path_length(archive, b"config/server.properties", 24).is_ok());
        assert!(matches!(
            check_entry_path_length(archive, b"config/server.properties", 23),
            Err(ArchiveError::LimitExceeded {
                limit: "entry path bytes",
                observed: 24,
                maximum: 23,
                ..
            })
        ));
    }

    /// 按顺序登记若干 `(条目名, 是否目录)`，返回首个失败。
    fn register_all(entries: &[(&str, bool)]) -> Result<(), ArchiveError> {
        let archive = Path::new("archive.zip");
        let mut registry = EntryPathRegistry::default();
        for (name, is_directory) in entries {
            let relative = safe_entry_path(archive, name)?;
            registry.register(archive, &relative, name, *is_directory)?;
        }
        Ok(())
    }

    #[test]
    fn accepts_files_alongside_their_explicit_parent_directory() {
        assert!(
            register_all(&[
                ("config", true),
                ("config/server.properties", false),
                ("config/nested", true),
                ("config/nested/level.dat", false),
            ])
            .is_ok()
        );
    }

    #[test]
    fn accepts_a_directory_declared_after_its_children() {
        // 目录条目晚于其内部文件出现是合法的归档顺序，隐式登记的祖先目录
        // 不应与随后的显式目录条目冲突。
        assert!(register_all(&[("config/server.properties", false), ("config", true)]).is_ok());
    }

    #[test]
    fn rejects_duplicate_output_paths() {
        assert!(matches!(
            register_all(&[("server.properties", false), ("server.properties", false)]),
            Err(ArchiveError::UnsafeEntry { .. })
        ));
    }

    #[test]
    fn rejects_a_file_that_blocks_a_later_parent_directory() {
        // `config` 先作为普通文件出现，之后的 `config/server.properties`
        // 无法在其下创建父目录。
        assert!(matches!(
            register_all(&[("config", false), ("config/server.properties", false)]),
            Err(ArchiveError::UnsafeEntry { .. })
        ));
    }

    #[test]
    fn rejects_a_path_used_as_both_file_and_directory() {
        assert!(matches!(
            register_all(&[("config", true), ("config", false)]),
            Err(ArchiveError::UnsafeEntry { .. })
        ));
        assert!(matches!(
            register_all(&[("config", false), ("config", true)]),
            Err(ArchiveError::UnsafeEntry { .. })
        ));
    }
}
