//! 导入源目录相关的共享校验与比较工具。
//!
//! 同时被 Tauri 命令与 Axum handler 复用，避免两份实现漂移导致行为不一致。

use std::path::{Path, PathBuf};

/// 导入源目录校验失败的原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceDirectoryError {
    /// 目录不存在或不可访问。
    Unavailable(PathBuf),
    /// 路径存在但不是目录。
    NotDirectory(PathBuf),
}

impl std::fmt::Display for SourceDirectoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(path) => {
                write!(f, "the selected directory does not exist: {}", path.display())
            }
            Self::NotDirectory(path) => {
                write!(f, "the selected path is not a directory: {}", path.display())
            }
        }
    }
}

impl std::error::Error for SourceDirectoryError {}

/// 校验导入源目录可读：必须存在且为目录。
pub fn validate_source_directory(path: &Path) -> Result<(), SourceDirectoryError> {
    if !path.exists() {
        return Err(SourceDirectoryError::Unavailable(path.to_path_buf()));
    }
    if !path.is_dir() {
        return Err(SourceDirectoryError::NotDirectory(path.to_path_buf()));
    }
    Ok(())
}

/// 判断两个路径是否指向同一目录。
///
/// 优先对两路径做规范化（`canonicalize`，解析符号链接并采用文件系统真实大小写）
/// 后严格比较；任一规范化失败（如权限不足或路径已不存在）则退化为严格相等 `==`。
///
/// 不使用大小写不敏感比较：在区分大小写的文件系统与非 ASCII 路径上，大小写不敏感
/// 比较会产生误判（将不同目录判为相同，或漏判真实相同的目录）。
pub fn source_directories_equal(first: &Path, second: &Path) -> bool {
    match (first.canonicalize(), second.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => first == second,
    }
}
