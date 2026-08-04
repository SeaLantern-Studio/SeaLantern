/// 实例管理操作失败的错误类别。
///
/// 采用分类式错误：不携带主机路径、实例内容等敏感细节，底层失败详情应写入受控日志。
/// 分类风格与 `server` 侧 `ConsoleCommandServiceError` 保持一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceServiceError {
    /// 指定的实例不存在。
    InstanceNotFound,
    /// 目标实例标识已存在（创建冲突）。
    AlreadyExists,
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
            Self::InvalidState => "server instance is in an invalid state",
            Self::OperationFailed => "server instance operation failed",
            Self::Unsupported => "operation not supported",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for InstanceServiceError {}
