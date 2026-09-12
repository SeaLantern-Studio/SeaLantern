//! 资源管理错误类型。

use std::path::PathBuf;

use sealantern_infra::fs::FsError;

use super::models::InstanceExtensionError;

/// 实例资源管理过程中可能出现的错误。
#[derive(Debug, thiserror::Error)]
pub enum ResourceManagerError {
    /// 底层文件系统基础设施操作失败（原子写、锁、哈希等）。
    #[error("文件系统操作失败: {0}")]
    Infra(#[from] FsError),

    /// 普通 I/O 操作失败（复制、重命名、删除等）。
    #[error("资源文件操作失败: {0}")]
    Io(#[from] std::io::Error),

    /// 实例目录不存在。
    #[error("实例目录不存在: {0}")]
    InstanceDirNotFound(PathBuf),

    /// 清单读写或解析失败。
    #[error("资源清单操作失败 ({path}): {message}")]
    Manifest {
        /// 清单文件路径。
        path: PathBuf,
        /// 失败原因。
        message: String,
    },

    /// 非法的资源文件名。
    #[error("非法的资源文件名: {0}")]
    InvalidFileName(String),

    /// 不是可管理的资源文件类型。
    #[error("不支持的资源文件类型: {0}")]
    UnsupportedExtension(String),

    /// 目标资源已存在。
    #[error("资源已存在: {0}")]
    AlreadyExists(String),

    /// 目标资源不存在。
    #[error("资源不存在: {0}")]
    NotFound(String),

    /// 扩展描述构造失败。
    #[error("扩展描述非法: {0}")]
    InvalidExtension(#[from] InstanceExtensionError),
}
