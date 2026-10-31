//! 旧版实例配置迁移领域的主错误。

use std::fmt;
use std::path::PathBuf;

use sealantern_contract::error::LegacyMigrationServiceError;
use sealantern_infra::fs::FsError;

/// 旧版实例配置迁移失败的应用层主错误。
///
/// 承载底层细节供日志排查，向 [`LegacyMigrationServiceError`] 转换时收敛为
/// 分类，不向宿主泄漏文件内容或主机路径。
#[derive(Debug)]
pub enum LegacyMigrationError {
    /// 写入 `sl.json` 失败。
    DocumentWrite {
        /// 目标目录。
        directory: PathBuf,
        /// 底层文档错误。
        source: sealantern_feature::config::instance::DocumentError,
    },
    /// 设置管理器（布局 / 名单）访问失败。
    Settings {
        /// 底层设置错误。
        source: crate::error::SettingsError,
    },
    /// 底层 IO 失败（归档旧文件等）。
    Io {
        /// 底层文件系统错误。
        source: FsError,
    },
}

impl fmt::Display for LegacyMigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DocumentWrite { directory, source } => {
                write!(
                    formatter,
                    "failed to write instance document in '{}': {source}",
                    directory.display()
                )
            }
            Self::Settings { source } => write!(formatter, "settings access failed: {source}"),
            Self::Io { source } => write!(formatter, "legacy migration I/O failed: {source}"),
        }
    }
}

impl std::error::Error for LegacyMigrationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DocumentWrite { source, .. } => Some(source),
            Self::Settings { source } => Some(source),
            Self::Io { source } => Some(source),
            _ => None,
        }
    }
}

impl From<crate::error::SettingsError> for LegacyMigrationError {
    fn from(source: crate::error::SettingsError) -> Self {
        Self::Settings { source }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
impl From<LegacyMigrationError> for LegacyMigrationServiceError {
    fn from(error: LegacyMigrationError) -> Self {
        match error {
            LegacyMigrationError::DocumentWrite { .. } => Self::MigrationFailed,
            LegacyMigrationError::Settings { .. } => Self::StorageFailed,
            LegacyMigrationError::Io { .. } => Self::StorageFailed,
        }
    }
}
