//! 设置服务错误类型。

use std::fmt;

use sealantern_extra::config::SettingsError as ExtraSettingsError;
use sealantern_infra::net::{NetError, NetworkCommitError};
use sealantern_infra::platform::SystemProxyReadError;
use sealantern_interface::error::SettingsServiceError;

/// 设置服务主错误类型。
///
/// 携带底层失败细节（source），供应用层日志排查；向 [`SettingsServiceError`]
/// 转换时收敛为分类，不向宿主泄漏敏感信息。
#[derive(Debug)]
pub enum SettingsError {
    /// 设置内容不符合业务约束或导入内容不合法。
    InvalidInput {
        /// extra 配置层提供的原始分类与诊断信息。
        source: ExtraSettingsError,
    },
    /// 配置加载、锁定、读取或持久化失败。
    StorageFailed {
        /// extra 配置层提供的原始分类与诊断信息。
        source: ExtraSettingsError,
    },
    /// 未分类的配置操作失败。
    OperationFailed {
        /// 底层来源错误。
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// 持久化代理设置与进程网络运行时同步失败。
    NetworkSyncFailed {
        /// 不包含代理凭据的底层失败。
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { source } => {
                write!(formatter, "invalid settings input: {source}")
            }
            Self::StorageFailed { source } => {
                write!(formatter, "settings storage failed: {source}")
            }
            Self::OperationFailed { source } => {
                write!(formatter, "settings operation failed: {source}")
            }
            Self::NetworkSyncFailed { source } => {
                write!(formatter, "settings network synchronization failed: {source}")
            }
        }
    }
}

impl std::error::Error for SettingsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidInput { source } => Some(source),
            Self::StorageFailed { source } => Some(source),
            Self::OperationFailed { source } | Self::NetworkSyncFailed { source } => {
                Some(source.as_ref())
            }
        }
    }
}

impl From<NetError> for SettingsError {
    fn from(source: NetError) -> Self {
        Self::NetworkSyncFailed { source: Box::new(source) }
    }
}

impl From<NetworkCommitError> for SettingsError {
    fn from(source: NetworkCommitError) -> Self {
        Self::NetworkSyncFailed { source: Box::new(source) }
    }
}

impl From<SystemProxyReadError> for SettingsError {
    fn from(source: SystemProxyReadError) -> Self {
        Self::NetworkSyncFailed { source: Box::new(source) }
    }
}

impl From<ExtraSettingsError> for SettingsError {
    fn from(source: ExtraSettingsError) -> Self {
        match source {
            ExtraSettingsError::InvalidInput { .. } => Self::InvalidInput { source },
            ExtraSettingsError::Storage { .. } => Self::StorageFailed { source },
        }
    }
}

/// 应用层主错误 → 接口契约错误的收敛转换。
///
/// 细节被抹平为分类，敏感字段不跨传输面。
impl From<SettingsError> for SettingsServiceError {
    fn from(error: SettingsError) -> Self {
        match error {
            SettingsError::InvalidInput { .. } => Self::InvalidInput,
            SettingsError::StorageFailed { .. } => Self::StorageFailed,
            SettingsError::OperationFailed { .. } => Self::OperationFailed,
            SettingsError::NetworkSyncFailed { .. } => Self::Unavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use sealantern_extra::config::SettingsError as ExtraSettingsError;
    use sealantern_infra::fs::FsError;

    use super::*;

    #[test]
    fn extra_settings_errors_map_to_stable_contract_categories() {
        let invalid = SettingsError::from(ExtraSettingsError::invalid_input(
            "default_port",
            "must be greater than zero",
        ));
        assert_eq!(SettingsServiceError::from(invalid), SettingsServiceError::InvalidInput);

        let storage = SettingsError::from(ExtraSettingsError::Storage {
            source: FsError::AlreadyLocked(PathBuf::from("settings.json")),
        });
        assert_eq!(SettingsServiceError::from(storage), SettingsServiceError::StorageFailed);
    }
}
