use crate::adapters::http::command_registry::CommandRegistry;
use crate::services::events::{
    AppEventEnvelope, EventConsumer, EventConsumerKind, EventConsumerMetadata, ServerEventEnvelope,
    ServerEventSubscription,
};
use crate::services::web_auth::WebAuthService;
use axum::http::StatusCode;
use runtime::HeadlessHttpConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::sync::Once;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ApiErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<serde_json::Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_kind: Option<String>,
}

impl ApiErrorDetail {
    pub(crate) fn invalid_request(message: impl Into<String>) -> Self {
        Self::new("common.message_unknown_error", "invalid_request", message)
    }

    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self::new("common.message_server_not_found", "not_found", message)
    }

    pub(crate) fn runtime(message: impl Into<String>) -> Self {
        Self::new("common.message_unknown_error", "runtime", message)
    }

    fn new(code: &str, error_kind: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            args: None,
            error_kind: Some(error_kind.to_string()),
        }
    }
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

#[derive(Clone, Serialize)]
#[serde(tag = "event_scope", content = "event", rename_all = "snake_case")]
pub enum RuntimeEventEnvelope {
    Server(ServerEventEnvelope),
    App(AppEventEnvelope),
}

static RUNTIME_EVENT_BROADCAST: once_cell::sync::Lazy<
    tokio::sync::broadcast::Sender<RuntimeEventEnvelope>,
> = once_cell::sync::Lazy::new(|| {
    let (tx, _rx) = tokio::sync::broadcast::channel(1024);
    tx
});

static RUNTIME_EVENT_BRIDGE_INIT: Once = Once::new();

/// 读取日志广播发送器
pub fn get_log_sender() -> tokio::sync::broadcast::Sender<LogEvent> {
    LOG_BROADCAST.clone()
}

pub(super) fn subscribe_log_events() -> tokio::sync::broadcast::Receiver<LogEvent> {
    LOG_BROADCAST.subscribe()
}

pub(super) fn subscribe_runtime_events() -> tokio::sync::broadcast::Receiver<RuntimeEventEnvelope> {
    RUNTIME_EVENT_BROADCAST.subscribe()
}

pub(super) fn ensure_runtime_event_bridge() {
    RUNTIME_EVENT_BRIDGE_INIT.call_once(|| {
        let server_sender = RUNTIME_EVENT_BROADCAST.clone();
        let app_sender = RUNTIME_EVENT_BROADCAST.clone();
        crate::services::global::event_manager().register_named_consumer_with_metadata(
            "http.sse.runtime_events",
            EventConsumer::both(
                Arc::new(move |event| {
                    let _ = server_sender.send(RuntimeEventEnvelope::Server(event.clone()));
                    Ok(())
                }),
                Arc::new(move |event| {
                    let _ = app_sender.send(RuntimeEventEnvelope::App(event.clone()));
                    Ok(())
                }),
            )
            .with_server_filter(ServerEventSubscription {
                classes: vec!["output".to_string(), "command".to_string(), "lifecycle".to_string()],
                event_kinds: Vec::new(),
                server_ids: Vec::new(),
            }),
            EventConsumerMetadata::new(
                EventConsumerKind::TransportAdapter,
                "http.sse",
                "Expose runtime server/app events over HTTP SSE.",
            ),
        );
    });
}

/// HTTP 路由共用状态
#[derive(Clone)]
pub(crate) struct AppState {
    /// 命令名到处理函数的注册表
    pub(super) command_registry: Arc<CommandRegistry>,
    pub(super) config: Arc<HeadlessHttpConfig>,
    pub(super) web_auth: Arc<WebAuthService>,
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
    #[allow(dead_code)]
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

#[allow(dead_code)]
struct InferredErrorDetail {
    code: &'static str,
    error_kind: &'static str,
}

#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::{ApiErrorDetail, ApiResponse, RuntimeEventEnvelope};
    use crate::services::events::{
        AppEventEnvelope, AppEventKind, AppEventPayload, EventScope, ServerEventEnvelope,
        ServerEventKind, ServerEventPayload,
    };
    use serde_json::Value;

    #[test]
    fn runtime_event_envelope_serializes_with_explicit_scope() {
        let server_event = RuntimeEventEnvelope::Server(ServerEventEnvelope {
            event_id: "server-event-1".to_string(),
            occurred_at: 1,
            scope: EventScope::Server,
            server_id: "alpha".to_string(),
            source: "runtime_stdout".to_string(),
            kind: ServerEventKind::OutputRawLine,
            payload: ServerEventPayload::RawLine {
                line: "hello".to_string(),
                stream: "stdout".to_string(),
            },
        });

        let app_event = RuntimeEventEnvelope::App(AppEventEnvelope {
            event_id: "app-event-1".to_string(),
            occurred_at: 2,
            scope: EventScope::App,
            action: "create_server".to_string(),
            source: "frontend_user".to_string(),
            kind: AppEventKind::OperationRequested,
            payload: AppEventPayload::Operation {
                action: "create_server".to_string(),
                detail: Some("requested".to_string()),
                error: None,
            },
        });

        let server_json = serde_json::to_value(server_event).expect("serialize server event");
        let app_json = serde_json::to_value(app_event).expect("serialize app event");

        assert_eq!(
            server_json
                .get("event_scope")
                .and_then(|value| value.as_str()),
            Some("server")
        );
        assert_eq!(app_json.get("event_scope").and_then(|value| value.as_str()), Some("app"));
        assert!(server_json.get("event").is_some());
        assert!(app_json.get("event").is_some());
    }

    #[test]
    fn api_response_error_keeps_legacy_inference_for_untyped_errors() {
        let response = ApiResponse::error("Unauthorized".to_string());

        assert!(!response.success);
        assert_eq!(response.error.as_deref(), Some("Unauthorized"));
        let detail = response.error_detail.expect("error detail");
        assert_eq!(detail.code, "common.message_unauthorized");
        assert_eq!(detail.error_kind.as_deref(), Some("unauthorized"));
    }

    #[test]
    fn api_error_detail_invalid_request_constructor_sets_expected_fields() {
        let detail = ApiErrorDetail::invalid_request("Missing path");
        let value = serde_json::to_value(&detail).expect("serialize detail");

        assert_eq!(detail.code, "common.message_unknown_error");
        assert_eq!(detail.message, "Missing path");
        assert_eq!(detail.error_kind.as_deref(), Some("invalid_request"));
        assert_eq!(value.get("args"), None);
    }

    #[test]
    fn api_response_error_with_detail_preserves_explicit_classification() {
        let response = ApiResponse::error_with_detail(
            "Missing path".to_string(),
            ApiErrorDetail::invalid_request("Missing path"),
        );
        let payload = serde_json::to_value(response).expect("serialize response");

        assert_eq!(payload.get("success"), Some(&Value::Bool(false)));
        assert_eq!(payload.get("error"), Some(&Value::String("Missing path".to_string())));
        assert_eq!(
            payload
                .get("error_detail")
                .and_then(|detail| detail.get("error_kind"))
                .and_then(|kind| kind.as_str()),
            Some("invalid_request")
        );
    }
}
