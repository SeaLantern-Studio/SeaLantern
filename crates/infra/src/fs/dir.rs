//! 目录创建辅助。
//!
//! 对 `create_dir_all` 的薄封装，供写入路径在落盘前确保父目录存在。函数是
//! 幂等的（目录已存在视为成功），但不处理权限、符号链接或并发创建冲突——
//! 这些错误原样向上抛出。

use std::path::Path;

use super::FsError;

/// 创建目录及其所有缺失的父目录。
pub async fn ensure_dir(path: impl AsRef<Path>) -> Result<(), FsError> {
    let path = path.as_ref();
    tokio::fs::create_dir_all(path)
        .await
        .map_err(|error| FsError::io("create directory", path, error))?;
    Ok(())
}

/// 当路径存在父目录时创建其父目录。
pub async fn ensure_parent(path: impl AsRef<Path>) -> Result<(), FsError> {
    if let Some(parent) = path.as_ref().parent() {
        ensure_dir(parent).await?;
    }
    Ok(())
}
