use crate::adapters::http::command_registry::CommandRegistry;
use axum::http::StatusCode;
use sea_lantern_runtime::HeadlessHttpConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ApiErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_kind: Option<String>,
}

/// HTTP 日志流里的单条日志事件
#[derive(Clone, Serialize)]
pub struct LogEvent {
    /// 产生日志的服务器 ID
    pub server_id: String,
    /// 推送给订阅方的日志内容
    pub line: String,
}

static LOG_BROADCAST: once_cell::sync::Lazy<tokio::sync::broadcast::Sender<LogEvent>> =
    once_cell::sync::Lazy::new(|| {
        let (tx, _rx) = tokio::sync::broadcast::channel(1024);
        tx
    });

/// 读取日志广播发送器
pub fn get_log_sender() -> tokio::sync::broadcast::Sender<LogEvent> {
    LOG_BROADCAST.clone()
}

pub(super) fn subscribe_log_events() -> tokio::sync::broadcast::Receiver<LogEvent> {
    LOG_BROADCAST.subscribe()
}

/// HTTP 路由共用状态
#[derive(Clone)]
pub(crate) struct AppState {
    /// 命令名到处理函数的注册表
    pub(super) command_registry: Arc<CommandRegistry>,
    pub(super) config: Arc<HeadlessHttpConfig>,
}

/// `/api/{command}` 的请求体
#[derive(Serialize, Deserialize)]
pub(super) struct ApiRequest {
    /// 透传给命令处理器的参数
    #[serde(default)]
    pub params: Value,
}

/// 统一的 HTTP 响应包裹结构
#[derive(Serialize, Deserialize)]
pub(crate) struct ApiResponse {
    /// 请求是否成功
    pub success: bool,
    /// 成功时的数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// 失败时的错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// 失败时的结构化错误详情
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_detail: Option<ApiErrorDetail>,
}

impl ApiResponse {
    /// 构建成功响应
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            error_detail: None,
        }
    }

    /// 构建失败响应
    pub fn error(message: String) -> Self {
        let detail = infer_error_detail(&message);
        Self {
            success: false,
            data: None,
            error: Some(message.clone()),
            error_detail: Some(ApiErrorDetail {
                code: detail.code.to_string(),
                message,
                args: None,
                error_kind: Some(detail.error_kind.to_string()),
            }),
        }
    }

    pub fn error_with_detail(message: String, detail: ApiErrorDetail) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            error_detail: Some(detail),
        }
    }
}

struct InferredErrorDetail {
    code: &'static str,
    error_kind: &'static str,
}

fn infer_error_detail(message: &str) -> InferredErrorDetail {
    let lower = message.to_ascii_lowercase();

    if lower.contains("unauthorized") {
        return InferredErrorDetail {
            code: "common.message_unauthorized",
            error_kind: "unauthorized",
        };
    }

    if message.contains("未找到服务器") || lower.contains("not found") {
        return InferredErrorDetail {
            code: "common.message_server_not_found",
            error_kind: "not_found",
        };
    }

    if lower.contains("invalid parameters")
        || lower.contains("missing path")
        || lower.contains("bad request")
        || lower.contains("failed to parse multipart payload")
        || lower.contains("invalid upload filename")
        || lower.contains("uploaded file reference not found")
    {
        return InferredErrorDetail {
            code: "common.message_unknown_error",
            error_kind: "invalid_request",
        };
    }

    InferredErrorDetail {
        code: "common.message_unknown_error",
        error_kind: "runtime",
    }
}

pub(super) struct UploadFailure {
    pub(super) status: StatusCode,
    pub(super) message: String,
}

impl UploadFailure {
    pub(super) fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }
}
