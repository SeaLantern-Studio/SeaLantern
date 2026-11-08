//! 实例配置文档（`sl.json`）的加载与持久化错误。

use std::fmt;
use std::path::PathBuf;

use sealantern_infra::fs::FsError;

/// 实例配置文档操作失败。
#[derive(Debug)]
pub enum DocumentError {
    /// 文件读取、锁定或持久化失败（含反序列化失败——损坏的文档保持
    /// 错误上抛，交由发现层分类为「无有效配置」）。
    Storage {
        /// 底层文件系统错误。
        source: FsError,
    },
    /// 文档由更新版本的程序写入，当前版本无法解析。
    UnsupportedSchemaVersion {
        /// 文档写入的 schema 版本。
        found: u32,
        /// 当前程序支持的最高 schema 版本。
        current: u32,
    },
    /// 目标文档已存在，拒绝重复创建。
    AlreadyExists {
        /// 已存在的文档路径。
        path: PathBuf,
    },
    /// 文档内容不符合结构约束（空标识、空名称、绝对路径启动目标、
    /// 非法内存区间、启动模式与载荷不一致等）。
    Invalid {
        /// 不稳定的、便于诊断的失败原因；不含敏感数据。
        message: String,
    },
}

impl DocumentError {
    /// 创建文档约束错误。
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid { message: message.into() }
    }
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage { source } => {
                write!(formatter, "instance document storage failed: {source}")
            }
            Self::UnsupportedSchemaVersion { found, current } => write!(
                formatter,
                "instance document schema version {found} is newer than supported version {current}; please upgrade SeaLantern"
            ),
            Self::AlreadyExists { path } => {
                write!(formatter, "instance document already exists: {}", path.display())
            }
            Self::Invalid { message } => write!(formatter, "invalid instance document: {message}"),
        }
    }
}

impl std::error::Error for DocumentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage { source } => Some(source),
            Self::UnsupportedSchemaVersion { .. }
            | Self::AlreadyExists { .. }
            | Self::Invalid { .. } => None,
        }
    }
}

impl From<FsError> for DocumentError {
    fn from(source: FsError) -> Self {
        Self::Storage { source }
    }
}
