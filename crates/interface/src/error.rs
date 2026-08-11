//! 接口契约错误类型。
//!
//! 定义跨宿主（tauri / server）统一消费的契约错误：薄分类、不携带底层细节，
//! 由 `application` 层的主错误（`application::error`）转换而来。
//! 底层失败详情由应用层记录到受控日志，不跨传输面泄漏。

/// 服务器定时任务操作失败的契约错误类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CronTaskServiceError {
    /// 指定的任务不存在。
    TaskNotFound,
    /// 任务配置或标识不合法。
    InvalidInput,
    /// JSON 持久化读写失败。
    StorageFailed,
    /// 任务对应的服务器动作执行失败。
    ExecutionFailed,
    /// 未分类的内部操作失败。
    OperationFailed,
    /// 该能力尚未实现。
    Unsupported,
}

impl std::fmt::Display for CronTaskServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::TaskNotFound => "cron task not found",
            Self::InvalidInput => "invalid cron task input",
            Self::StorageFailed => "cron task storage failed",
            Self::ExecutionFailed => "cron task execution failed",
            Self::OperationFailed => "cron task operation failed",
            Self::Unsupported => "operation not supported",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for CronTaskServiceError {}

/// 实例管理操作失败的契约错误类别。
///
/// 分类风格与 `server` 侧 `ConsoleCommandServiceError` 保持一致：
/// 不携带主机路径、实例内容等敏感细节，底层失败详情由应用层写入受控日志。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceServiceError {
    /// 指定的实例不存在。
    InstanceNotFound,
    /// 目标实例标识已存在（创建冲突）。
    AlreadyExists,
    /// 客户端提供的输入不合法（如空 ID、格式错误）。
    InvalidInput,
    /// 实例当前状态不允许该操作（如未运行时停止、已运行时重复启动）。
    InvalidState,
    /// 底层 IO / 供给 / 进程操作失败。
    OperationFailed,
    /// 该能力尚未实现（占位）。
    Unsupported,
}

impl std::fmt::Display for InstanceServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InstanceNotFound => "server instance not found",
            Self::AlreadyExists => "server instance already exists",
            Self::InvalidInput => "invalid input",
            Self::InvalidState => "server instance is in an invalid state",
            Self::OperationFailed => "server instance operation failed",
            Self::Unsupported => "operation not supported",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for InstanceServiceError {}

/// 服务器进程管理失败的契约错误类别。
///
/// 分类风格与其他契约错误一致：不携带主机路径、进程细节等敏感信息，
/// 底层失败详情由应用层写入受控日志。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerServiceError {
    /// 指定的实例不存在。
    InstanceNotFound,
    /// 服务器进程当前状态不允许该操作（如未运行时停止、已运行时重复启动）。
    InvalidState,
    /// 客户端提供的输入不合法。
    InvalidInput,
    /// 底层进程 / IO 操作失败。
    OperationFailed,
    /// 该能力尚未实现（占位）。
    Unsupported,
}

impl std::fmt::Display for ServerServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InstanceNotFound => "server instance not found",
            Self::InvalidState => "server is in an invalid state for this operation",
            Self::InvalidInput => "invalid input",
            Self::OperationFailed => "server operation failed",
            Self::Unsupported => "operation not supported",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ServerServiceError {}

/// 设置信息服务失败的契约错误类别。
///
/// 分类风格与 [`InstanceServiceError`] 一致：不携带敏感细节，
/// 底层失败详情由应用层写入受控日志。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SettingsServiceError {
    /// 设置分组或设置项不存在。
    NotFound,
    /// 客户端提供的设置或导入内容不合法。
    InvalidInput,
    /// 底层配置加载、锁定或持久化失败。
    StorageFailed,
    /// 设置服务暂时不可用或尚未完成装配。
    Unavailable,
    /// 未分类的设置操作失败。
    OperationFailed,
    /// 该能力尚未实现（占位）。
    Unsupported,
}

/// 下载任务管理失败的契约错误类别。
///
/// 分类风格与其他契约错误一致：不携带 URL、路径等敏感信息，底层失败详情
/// 由应用层写入受控日志。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadServiceError {
    /// 指定的下载任务不存在。
    TaskNotFound,
    /// 客户端提供的输入不合法（如空 URL）。
    InvalidInput,
    /// 底层网络 / IO 操作失败。
    OperationFailed,
    /// 该能力尚未实现（占位）。
    Unsupported,
}

impl std::fmt::Display for SettingsServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::NotFound => "settings not found",
            Self::InvalidInput => "invalid settings input",
            Self::StorageFailed => "settings storage failed",
            Self::Unavailable => "settings service unavailable",
            Self::OperationFailed => "settings operation failed",
            Self::Unsupported => "operation not supported",
        };
        formatter.write_str(message)
    }
}

impl std::fmt::Display for DownloadServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::TaskNotFound => "download task not found",
            Self::InvalidInput => "invalid input",
            Self::OperationFailed => "download operation failed",
            Self::Unsupported => "operation not supported",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SettingsServiceError {}

impl std::error::Error for DownloadServiceError {}

/// 系统资源信息服务失败的契约错误类别。
///
/// 分类风格与 [`InstanceServiceError`] 一致：不携带主机路径、进程细节等敏感
/// 信息，底层失败详情由应用层写入受控日志。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemServiceError {
    /// 指定的进程不存在或无权访问。
    ProcessNotFound,
    /// 指定的路径不存在或不可访问。
    PathNotFound,
    /// 底层系统采集 / IO 操作失败。
    OperationFailed,
    /// 该能力尚未实现（占位）。
    Unsupported,
}

impl std::fmt::Display for SystemServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::ProcessNotFound => "process not found",
            Self::PathNotFound => "path not found",
            Self::OperationFailed => "system operation failed",
            Self::Unsupported => "operation not supported",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for SystemServiceError {}

/// 应用更新检查失败的契约错误类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateCheckServiceError {
    /// 更新源检查或响应处理失败。
    CheckFailed,
    /// 当前宿主不支持更新检查。
    Unsupported,
}

impl std::fmt::Display for UpdateCheckServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::CheckFailed => "update check failed",
            Self::Unsupported => "update check not supported",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for UpdateCheckServiceError {}

/// 服务端检查或供给计划失败的契约错误类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvisioningServiceError {
    /// 客户端提供的输入不合法（如路径为空、请求字段冲突）。
    InvalidInput,
    /// 服务器目录检查失败。
    InspectionFailed,
    /// 未分类的内部供给计划操作失败。
    OperationFailed,
}

impl std::fmt::Display for ProvisioningServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "invalid provisioning input",
            Self::InspectionFailed => "server inspection failed",
            Self::OperationFailed => "provisioning operation failed",
        })
    }
}

impl std::error::Error for ProvisioningServiceError {}

/// Java 检测与校验失败的契约错误类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JavaServiceError {
    /// 客户端提供的 Java 路径不合法或不可访问。
    InvalidInput,
    /// 底层 Java 检测或校验操作失败。
    OperationFailed,
}
impl std::fmt::Display for JavaServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidInput => "invalid Java installation",
            Self::OperationFailed => "Java operation failed",
        })
    }
}
impl std::error::Error for JavaServiceError {}

/// 服务器核心下载目录失败的契约错误类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerCatalogServiceError {
    /// 指定的服务器核心类型或版本不存在。
    NotFound,
    /// 底层下载目录查询操作失败。
    OperationFailed,
}
impl std::fmt::Display for ServerCatalogServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NotFound => "server catalog entry not found",
            Self::OperationFailed => "server catalog operation failed",
        })
    }
}
impl std::error::Error for ServerCatalogServiceError {}

/// 应用更新下载与安装失败的契约错误类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateInstallServiceError {
    /// 客户端提供的安装输入不合法（如版本号为空）。
    InvalidInput,
    /// 底层下载、校验或安装过程失败。
    OperationFailed,
    /// 当前宿主不支持更新安装。
    Unsupported,
}
impl std::fmt::Display for UpdateInstallServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidInput => "invalid update installation input",
            Self::OperationFailed => "update installation operation failed",
            Self::Unsupported => "update installation is unsupported",
        })
    }
}
impl std::error::Error for UpdateInstallServiceError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OnlineTunnelServiceError {
    /// 客户端提供的隧道请求不合法（如票据为空）。
    InvalidInput,
    /// 已有隧道正在启动、运行或停止，无法响应新操作。
    Busy,
    /// 当前没有运行中的隧道。
    NotRunning,
    /// 底层隧道操作失败。
    OperationFailed,
}

impl std::fmt::Display for OnlineTunnelServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "invalid online tunnel input",
            Self::Busy => "online tunnel is busy",
            Self::NotRunning => "online tunnel is not running",
            Self::OperationFailed => "online tunnel operation failed",
        })
    }
}

impl std::error::Error for OnlineTunnelServiceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_service_errors_serialize_as_snake_case() {
        let cases = [
            (serde_json::to_string(&CronTaskServiceError::TaskNotFound), "\"task_not_found\""),
            (serde_json::to_string(&InstanceServiceError::InvalidInput), "\"invalid_input\""),
            (serde_json::to_string(&ServerServiceError::InvalidState), "\"invalid_state\""),
            (
                serde_json::to_string(&SettingsServiceError::StorageFailed),
                "\"storage_failed\"",
            ),
            (serde_json::to_string(&DownloadServiceError::TaskNotFound), "\"task_not_found\""),
            (
                serde_json::to_string(&SystemServiceError::ProcessNotFound),
                "\"process_not_found\"",
            ),
            (serde_json::to_string(&UpdateCheckServiceError::CheckFailed), "\"check_failed\""),
            (
                serde_json::to_string(&ProvisioningServiceError::InspectionFailed),
                "\"inspection_failed\"",
            ),
            (serde_json::to_string(&JavaServiceError::InvalidInput), "\"invalid_input\""),
            (
                serde_json::to_string(&ServerCatalogServiceError::OperationFailed),
                "\"operation_failed\"",
            ),
            (
                serde_json::to_string(&UpdateInstallServiceError::OperationFailed),
                "\"operation_failed\"",
            ),
            (serde_json::to_string(&OnlineTunnelServiceError::NotRunning), "\"not_running\""),
        ];

        for (serialized, expected) in cases {
            assert_eq!(serialized.expect("error enum must serialize"), expected);
        }
    }
}
