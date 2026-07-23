//! RPC 请求元数据和传输无关的请求封装。

use serde::Serialize;

use super::{RpcError, RpcResult};

const MAX_REQUEST_ID_LENGTH: usize = 128;

/// 由受信任传输适配器生成或校验后的请求关联标识。
///
/// 此标识仅用于关联日志和返回给调用方排障，不能用作身份凭据或授权依据。
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct RpcRequestId(String);

impl RpcRequestId {
    /// 构建一个适合记录到结构化日志中的请求标识。
    ///
    /// 仅允许 ASCII 字母、数字、`-` 和 `_`，防止调用方通过关联标识注入控制字符或
    /// 破坏日志字段格式。
    pub fn new(value: impl Into<String>) -> RpcResult<Self> {
        let value = value.into();
        if value.is_empty() || value.len() > MAX_REQUEST_ID_LENGTH {
            return Err(RpcError::invalid_argument(
                "request_id",
                "must contain between 1 and 128 characters",
            ));
        }

        if !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(RpcError::invalid_argument(
                "request_id",
                "may contain only ASCII letters, digits, hyphens, and underscores",
            ));
        }

        Ok(Self(value))
    }

    /// 返回用于传输响应和结构化日志的标识文本。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RpcRequestId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// 进入 RPC 层的传输类型。
///
/// 该字段只用于审计和可观测性。鉴权必须在适配器或后续显式授权端口中完成，不能仅凭
/// 此枚举授予权限。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RpcTransport {
    /// Tauri `invoke` 请求。
    TauriInvoke,
    /// Axum HTTP 请求。
    Http,
    /// 插件桥接请求。
    Plugin,
    /// 可信进程内调用。
    Internal,
}

impl RpcTransport {
    /// 返回稳定的可观测性字段值。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TauriInvoke => "tauri_invoke",
            Self::Http => "http",
            Self::Plugin => "plugin",
            Self::Internal => "internal",
        }
    }
}

/// RPC 方法执行时可安全使用的请求元数据。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RpcContext {
    request_id: RpcRequestId,
    transport: RpcTransport,
}

impl RpcContext {
    /// 由传输适配器构建请求上下文。
    pub fn new(request_id: RpcRequestId, transport: RpcTransport) -> Self {
        Self { request_id, transport }
    }

    /// 返回当前调用的关联标识。
    pub fn request_id(&self) -> &RpcRequestId {
        &self.request_id
    }

    /// 返回当前调用的传输类型。
    pub const fn transport(&self) -> RpcTransport {
        self.transport
    }
}

/// 传输适配器传给 RPC 方法的已解析请求。
///
/// 参数由该类型拥有，调用方法时会被消费，从而避免为跨异步边界保留请求数据而额外克隆。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RpcRequest<T> {
    context: RpcContext,
    params: T,
}

impl<T> RpcRequest<T> {
    /// 使用已校验的上下文和已解析参数构建 RPC 请求。
    pub fn new(context: RpcContext, params: T) -> Self {
        Self { context, params }
    }

    /// 取出请求上下文和参数供调度器执行。
    pub fn into_parts(self) -> (RpcContext, T) {
        (self.context, self.params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_log_safe_request_id() {
        let request_id = RpcRequestId::new("http_42-request").expect("request id should be valid");

        assert_eq!(request_id.as_str(), "http_42-request");
    }

    #[test]
    fn rejects_control_characters_in_request_id() {
        let error = RpcRequestId::new("request\n42").expect_err("newline must be rejected");

        assert_eq!(error.code(), super::super::RpcErrorCode::InvalidArgument);
    }

    #[test]
    fn rejects_request_ids_that_are_too_long() {
        let error = RpcRequestId::new("a".repeat(MAX_REQUEST_ID_LENGTH + 1))
            .expect_err("oversized request id must be rejected");

        assert_eq!(error.code(), super::super::RpcErrorCode::InvalidArgument);
    }
}
