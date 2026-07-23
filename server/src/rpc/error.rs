//! RPC 的稳定错误契约。

use serde::Serialize;

use super::RpcRequestId;

/// RPC 方法返回的统一结果类型。
pub type RpcResult<T> = Result<T, RpcError>;

/// 供 Tauri、HTTP 和其他传输一致映射的错误类别。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RpcErrorCode {
    /// 调用参数不符合方法契约。
    InvalidArgument,
    /// 请求的资源不存在。
    NotFound,
    /// 当前状态不允许执行该操作。
    Conflict,
    /// 调用方未获授执行该操作的权限。
    PermissionDenied,
    /// 依赖暂时不可用，可以由调用方重试。
    Unavailable,
    /// 调用超过适配器或应用服务声明的截止时间。
    DeadlineExceeded,
    /// 未分类的服务端失败，不暴露内部错误文本。
    Internal,
}

impl RpcErrorCode {
    /// 返回稳定的机器可读错误代码。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidArgument => "invalid_argument",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::PermissionDenied => "permission_denied",
            Self::Unavailable => "unavailable",
            Self::DeadlineExceeded => "deadline_exceeded",
            Self::Internal => "internal",
        }
    }

    /// 指示传输适配器是否可以向调用方建议重试。
    pub const fn is_retryable(self) -> bool {
        matches!(self, Self::Unavailable | Self::DeadlineExceeded)
    }
}

/// 可安全返回给 RPC 调用方的错误。
///
/// 不保存底层错误对象或未脱敏参数。方法实现应先将底层错误记录到受控日志，再映射为本
/// 类型，防止路径、凭据或供应商错误文本越过传输边界。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RpcError {
    code: RpcErrorCode,
    message: String,
    retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<RpcRequestId>,
}

impl RpcError {
    /// 构建参数无效错误。
    pub fn invalid_argument(field: &'static str, reason: &'static str) -> Self {
        Self::new(RpcErrorCode::InvalidArgument, format!("Invalid '{field}': {reason}."))
    }

    /// 构建资源不存在错误。
    pub fn not_found(resource: &'static str) -> Self {
        Self::new(RpcErrorCode::NotFound, format!("The requested {resource} was not found."))
    }

    /// 构建状态冲突错误。
    pub fn conflict(operation: &'static str) -> Self {
        Self::new(RpcErrorCode::Conflict, format!("The current state does not allow {operation}."))
    }

    /// 构建拒绝访问错误。
    pub fn permission_denied() -> Self {
        Self::new(
            RpcErrorCode::PermissionDenied,
            "The caller is not permitted to perform this operation.".to_string(),
        )
    }

    /// 构建可重试的依赖不可用错误。
    pub fn unavailable(dependency: &'static str) -> Self {
        Self::new(
            RpcErrorCode::Unavailable,
            format!("The {dependency} is temporarily unavailable. Please retry."),
        )
    }

    /// 构建可重试的截止时间错误。
    pub fn deadline_exceeded(operation: &'static str) -> Self {
        Self::new(
            RpcErrorCode::DeadlineExceeded,
            format!("The {operation} did not complete before its deadline. Please retry."),
        )
    }

    /// 构建不泄露内部实现细节的服务端错误。
    pub fn internal(operation: &'static str) -> Self {
        Self::new(
            RpcErrorCode::Internal,
            format!("The server could not complete {operation}. Inspect server logs with the request ID."),
        )
    }

    fn new(code: RpcErrorCode, message: String) -> Self {
        Self {
            code,
            message,
            retryable: code.is_retryable(),
            request_id: None,
        }
    }

    /// 返回稳定的错误代码。
    pub const fn code(&self) -> RpcErrorCode {
        self.code
    }

    /// 返回可安全展示给调用方的错误文本。
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 返回调用方是否可重试该操作。
    pub const fn is_retryable(&self) -> bool {
        self.retryable
    }

    /// 返回调度器附加的请求关联标识。
    pub fn request_id(&self) -> Option<&RpcRequestId> {
        self.request_id.as_ref()
    }

    pub(crate) fn with_request_id(mut self, request_id: RpcRequestId) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RpcError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_error_does_not_expose_a_cause() {
        let error = RpcError::internal("the requested operation");

        assert_eq!(error.code(), RpcErrorCode::Internal);
        assert!(!error.message().contains("database password"));
        assert!(!error.is_retryable());
    }

    #[test]
    fn retryable_codes_are_marked_for_adapters() {
        assert!(RpcError::unavailable("test dependency").is_retryable());
        assert!(RpcError::deadline_exceeded("test operation").is_retryable());
        assert!(!RpcError::permission_denied().is_retryable());
    }
}
