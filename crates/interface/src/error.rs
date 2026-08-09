//! 接口契约错误类型。
//!
//! 定义跨宿主（tauri / server）统一消费的契约错误：薄分类、不携带底层细节，
//! 由 `application` 层的主错误（`application::error`）转换而来。
//! 底层失败详情由应用层记录到受控日志，不跨传输面泄漏。

/// 服务器定时任务操作失败的契约错误类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
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
pub enum SettingsServiceError {
    /// 设置分组或设置项不存在。
    NotFound,
    /// 底层配置加载/保存操作失败。
    OperationFailed,
    /// 该能力尚未实现（占位）。
    Unsupported,
}

/// 下载任务管理失败的契约错误类别。
///
/// 分类风格与其他契约错误一致：不携带 URL、路径等敏感信息，底层失败详情
/// 由应用层写入受控日志。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
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
