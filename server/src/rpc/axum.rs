//! Axum HTTP 到 RPC 契约的传输适配器。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::{
    extract::{rejection::JsonRejection, State},
    http::{header::HeaderName, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::Deserialize;

use crate::observability;

use super::{
    dispatch,
    methods::server::{ConsoleCommandRequest, SendConsoleCommand},
    service::ConsoleCommandService,
    RpcAccess, RpcContext, RpcError, RpcErrorCode, RpcMethod, RpcRequest, RpcRequestId, RpcResult,
    RpcTransport,
};

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");
static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// 由 HTTP 宿主实现的认证与授权端口。
///
/// 适配器不自行信任任意 HTTP 请求。宿主必须根据已经验证的身份材料返回权限集合，或返回
/// 一个可安全公开的 [`RpcError`]。请求头的原始内容不得进入 RPC 日志或错误响应。
pub trait HttpRpcAccessResolver: Send + Sync + 'static {
    /// 解析当前 HTTP 请求所获授的 RPC 权限。
    fn resolve(&self, headers: &HeaderMap) -> RpcResult<RpcAccess>;
}

/// 组装现有服务器控制台 RPC 的 Axum 路由。
///
/// 该函数只注册已实现的稳定 RPC 方法。调用方可将返回的路由嵌套进更大的 Axum 应用，认证
/// 实现则通过 [`HttpRpcAccessResolver`] 注入，避免 HTTP 传输默认获得写入服务器控制台的权限。
pub fn router<S, A>(console_service: S, access_resolver: A) -> Router
where
    S: ConsoleCommandService + 'static,
    A: HttpRpcAccessResolver,
{
    let path = SendConsoleCommand::<S>::NAME.http_path();
    let state = AxumRpcState {
        console_command: Arc::new(SendConsoleCommand::new(console_service)),
        access_resolver: Arc::new(access_resolver),
    };

    Router::new()
        .route(&path, post(send_console_command::<S, A>))
        .with_state(state)
}

struct AxumRpcState<S, A> {
    console_command: Arc<SendConsoleCommand<S>>,
    access_resolver: Arc<A>,
}

impl<S, A> Clone for AxumRpcState<S, A> {
    fn clone(&self) -> Self {
        Self {
            console_command: Arc::clone(&self.console_command),
            access_resolver: Arc::clone(&self.access_resolver),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SendConsoleCommandPayload {
    instance_id: String,
    command: String,
}

async fn send_console_command<S, A>(
    State(state): State<AxumRpcState<S, A>>,
    headers: HeaderMap,
    payload: Result<Json<SendConsoleCommandPayload>, JsonRejection>,
) -> Response
where
    S: ConsoleCommandService + 'static,
    A: HttpRpcAccessResolver,
{
    let request_id = match request_id(&headers) {
        Ok(request_id) => request_id,
        Err((request_id, error)) => return reject(request_id, "invalid_request_id", error),
    };

    let access = match state.access_resolver.resolve(&headers) {
        Ok(access) => access,
        Err(error) => return reject(request_id, "authorization_rejected", error),
    };

    let payload = match payload {
        Ok(Json(payload)) => payload,
        Err(_) => {
            return reject(
                request_id,
                "invalid_json",
                RpcError::invalid_argument("request", "must be a valid JSON object"),
            );
        }
    };

    let params = match ConsoleCommandRequest::new(payload.instance_id, payload.command) {
        Ok(params) => params,
        Err(error) => return reject(request_id, "invalid_params", error),
    };

    let context = RpcContext::new(request_id, RpcTransport::Http).with_access(access);
    match dispatch(state.console_command.as_ref(), RpcRequest::new(context, params)).await {
        Ok(response) => {
            let request_id = response.request_id().clone();
            respond(StatusCode::OK, &request_id, response)
        }
        Err(error) => rpc_error_response(error),
    }
}

fn request_id(headers: &HeaderMap) -> Result<RpcRequestId, (RpcRequestId, RpcError)> {
    let generated = generated_request_id();
    let Some(value) = headers.get(&REQUEST_ID_HEADER) else {
        return Ok(generated);
    };

    let value = match value.to_str() {
        Ok(value) => value,
        Err(_) => {
            return Err((
                generated,
                RpcError::invalid_argument("x-request-id", "must be valid ASCII"),
            ));
        }
    };

    RpcRequestId::new(value).map_err(|error| (generated, error))
}

fn generated_request_id() -> RpcRequestId {
    let sequence = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
    RpcRequestId::new(format!("http-{sequence}")).expect("generated request ID must be valid")
}

fn reject(request_id: RpcRequestId, reason: &'static str, error: RpcError) -> Response {
    let error = error.with_request_id(request_id);
    observability::rpc_http_request_rejected(
        error
            .request_id()
            .expect("rejection must have a request ID")
            .as_str(),
        reason,
        error.code().as_str(),
    );
    rpc_error_response(error)
}

fn rpc_error_response(error: RpcError) -> Response {
    let status = status_for(error.code());
    let request_id = error
        .request_id()
        .expect("all Axum RPC errors must have a request ID")
        .clone();
    respond(status, &request_id, error)
}

fn respond<T>(status: StatusCode, request_id: &RpcRequestId, body: T) -> Response
where
    T: serde::Serialize,
{
    let mut response = (status, Json(body)).into_response();
    response.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(request_id.as_str())
            .expect("validated request ID must be a header value"),
    );
    response
}

const fn status_for(code: RpcErrorCode) -> StatusCode {
    match code {
        RpcErrorCode::InvalidArgument => StatusCode::BAD_REQUEST,
        RpcErrorCode::NotFound => StatusCode::NOT_FOUND,
        RpcErrorCode::Conflict => StatusCode::CONFLICT,
        RpcErrorCode::PermissionDenied => StatusCode::FORBIDDEN,
        RpcErrorCode::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        RpcErrorCode::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
        RpcErrorCode::Cancelled => StatusCode::REQUEST_TIMEOUT,
        RpcErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use serde_json::Value;
    use tower::ServiceExt;

    use super::*;
    use crate::rpc::{
        methods::server::PERMISSION_SERVER_CONSOLE_SEND, service::ConsoleCommandServiceError,
    };

    #[derive(Default)]
    struct RecordingConsoleService {
        commands: Mutex<Vec<(String, String)>>,
    }

    impl ConsoleCommandService for Arc<RecordingConsoleService> {
        fn send_console_command(
            &self,
            instance_id: &str,
            command: &str,
        ) -> Result<(), ConsoleCommandServiceError> {
            self.commands
                .lock()
                .expect("recording service lock")
                .push((instance_id.into(), command.into()));
            Ok(())
        }
    }

    struct AllowConsoleSend;

    impl HttpRpcAccessResolver for AllowConsoleSend {
        fn resolve(&self, _headers: &HeaderMap) -> RpcResult<RpcAccess> {
            Ok(RpcAccess::allow([PERMISSION_SERVER_CONSOLE_SEND]))
        }
    }

    struct DenyAll;

    impl HttpRpcAccessResolver for DenyAll {
        fn resolve(&self, _headers: &HeaderMap) -> RpcResult<RpcAccess> {
            Ok(RpcAccess::deny_all())
        }
    }

    fn request(body: &'static str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/api/rpc/server/console/send")
            .header("content-type", "application/json")
            .header("x-request-id", "http-test-42")
            .body(Body::from(body))
            .expect("build HTTP request")
    }

    #[tokio::test]
    async fn dispatches_a_valid_http_request_through_the_rpc_method() {
        let service = Arc::new(RecordingConsoleService::default());
        let response = router(service.clone(), AllowConsoleSend)
            .oneshot(request(r#"{"instanceId":"alpha","command":"say hello"}"#))
            .await
            .expect("route should respond");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[&REQUEST_ID_HEADER], "http-test-42");
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read response body");
        let body: Value = serde_json::from_slice(&body).expect("parse response JSON");
        assert_eq!(body["requestId"], "http-test-42");
        assert!(body["data"].is_null());
        assert_eq!(
            *service.commands.lock().expect("recording service lock"),
            vec![("alpha".into(), "say hello".into())]
        );
    }

    #[tokio::test]
    async fn rejects_an_unprivileged_request_without_calling_the_service() {
        let service = Arc::new(RecordingConsoleService::default());
        let response = router(service.clone(), DenyAll)
            .oneshot(request(r#"{"instanceId":"alpha","command":"stop"}"#))
            .await
            .expect("route should respond");

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read response body");
        let body: Value = serde_json::from_slice(&body).expect("parse response JSON");
        assert_eq!(body["code"], "permission_denied");
        assert_eq!(body["requestId"], "http-test-42");
        assert!(service
            .commands
            .lock()
            .expect("recording service lock")
            .is_empty());
    }

    #[tokio::test]
    async fn maps_invalid_json_to_the_rpc_error_envelope() {
        let service = Arc::new(RecordingConsoleService::default());
        let response = router(service.clone(), AllowConsoleSend)
            .oneshot(request("not json"))
            .await
            .expect("route should respond");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read response body");
        let body: Value = serde_json::from_slice(&body).expect("parse response JSON");
        assert_eq!(body["code"], "invalid_argument");
        assert_eq!(body["requestId"], "http-test-42");
        assert!(service
            .commands
            .lock()
            .expect("recording service lock")
            .is_empty());
    }

    #[test]
    fn maps_rpc_errors_to_stable_http_statuses() {
        assert_eq!(status_for(RpcErrorCode::Unavailable), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(status_for(RpcErrorCode::DeadlineExceeded), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(status_for(RpcErrorCode::Cancelled), StatusCode::REQUEST_TIMEOUT);
    }
}
