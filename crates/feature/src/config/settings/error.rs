//! SeaLantern 设置管理错误。

use std::fmt;

use sealantern_infra::fs::FsError;

use crate::models::SettingsValidationError;

/// 设置加载、校验和持久化操作失败。
#[derive(Debug)]
pub enum SettingsError {
    /// 设置内容不符合业务约束或导入内容不是有效 JSON。
    InvalidInput {
        /// 失败字段或输入类别。
        field: &'static str,
        /// 不包含敏感数据的失败原因。
        message: String,
    },
    /// 配置文件读取、锁定或持久化失败。
    Storage {
        /// 底层文件系统错误。
        source: FsError,
    },
}

impl SettingsError {
    /// 创建输入错误。
    pub fn invalid_input(field: &'static str, message: impl Into<String>) -> Self {
        Self::InvalidInput { field, message: message.into() }
    }
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { field, message } => {
                write!(formatter, "invalid setting '{field}': {message}")
            }
            Self::Storage { source } => write!(formatter, "settings storage failed: {source}"),
        }
    }
}

impl std::error::Error for SettingsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidInput { .. } => None,
            Self::Storage { source } => Some(source),
        }
    }
}

impl From<FsError> for SettingsError {
    fn from(source: FsError) -> Self {
        Self::Storage { source }
    }
}

impl From<SettingsValidationError> for SettingsError {
    fn from(error: SettingsValidationError) -> Self {
        Self::invalid_input(error.field(), error.message())
    }
}
