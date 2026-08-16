//! Axum HTTP 到 RPC 契约的传输适配器。

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::{
    Json,
    http::{HeaderMap, HeaderValue, StatusCode, header::HeaderName},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::observability;

use super::{
    RpcAccess, RpcContext, RpcError, RpcErrorCode, RpcMethod, RpcRequest, RpcRequestId, RpcResult,
    RpcTransport, dispatch,
};

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");
static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// 所有 RPC 方法共享的 Axum HTTP 路径前缀。
///
/// 方法标识 `server.console.send` 通过 [`RpcAxumMethod::http_path`] 默认实现映射为
/// `/api/rpc/server/console/send`。
const HTTP_RPC_PREFIX: &str = "/api/rpc";

/// 由 HTTP 宿主实现的认证与授权端口。
///
/// 适配器不自行信任任意 HTTP 请求。宿主必须根据已经验证的身份材料返回权限集合，或返回
/// 一个可安全公开的 [`RpcError`]。请求头的原始内容不得进入 RPC 日志或错误响应。
pub trait HttpRpcAccessResolver: Send + Sync + 'static {
    /// 解析当前 HTTP 请求所获授的 RPC 权限。
    fn resolve(&self, headers: &HeaderMap) -> RpcResult<RpcAccess>;

    /// 认证材料缺失或无效时应返回的 HTTP 状态。
    ///
    /// 默认保持现有 RPC 授权拒绝语义（403）；bearer 等身份认证边界可覆盖为 401。
    fn rejection_status(&self) -> StatusCode {
        StatusCode::FORBIDDEN
    }
}

/// Axum RPC 状态。
///
/// 只持有认证解析器，方法实例由 `rpc_route!` 宏捕获。
pub struct AxumRpcState<A: Send + Sync + 'static> {
    pub access_resolver: Arc<A>,
}

impl<A: Send + Sync + 'static> Clone for AxumRpcState<A> {
    fn clone(&self) -> Self {
        Self {
            access_resolver: Arc::clone(&self.access_resolver),
        }
    }
}

/// Axum 专用的 RPC 方法契约，扩展传输无关的 [`RpcMethod`]。
///
/// 将 HTTP 相关配置（方法、路径）保留在适配器层，不污染契约层。实现者只需声明
/// [`HTTP_METHOD`] 和 [`RpcMethod::NAME`]，HTTP 路径由 [`http_path`] 默认实现从方法
/// 标识自动派生。
///
/// [`HTTP_METHOD`]: Self::HTTP_METHOD
/// [`http_path`]: Self::http_path
pub trait RpcAxumMethod: RpcMethod {
    /// HTTP 方法，影响 axum 路由注册方式。
    const HTTP_METHOD: RpcHttpMethod;

    /// 派生 Axum 应注册的 HTTP 路径。
    ///
    /// 默认实现将方法标识中的 `.` 映射为路径 `/`，并拼接 [`HTTP_RPC_PREFIX`] 前缀。
    /// 例如标识 `server.console.send` 会派生出 `/api/rpc/server/console/send`。
    ///
    /// 实现者可重写此方法以提供自定义路径，但必须保持以 `/` 开头。
    ///
    /// # 默认实现示例
    ///
    /// ```ignore
    /// // 通常在 rpc_route! 宏内部调用；也可在测试中直接使用：
    /// let path = <SendConsoleCommand<_> as RpcAxumMethod>::http_path();
    /// assert_eq!(path, "/api/rpc/server/console/send");
    /// ```
    fn http_path() -> String {
        let name = Self::NAME.as_str();
        let mut path = String::with_capacity(HTTP_RPC_PREFIX.len() + name.len() + 1);
        path.push_str(HTTP_RPC_PREFIX);
        path.push('/');
        for character in name.chars() {
            path.push(if character == '.' { '/' } else { character });
        }
        path
    }
}

/// HTTP 方法，对应 axum 路由的 HTTP 方法。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RpcHttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// 通过方法实例推导关联常量（路径、HTTP 方法）。
///
/// 仅在 `rpc_route!` 宏中被调用。
pub(crate) fn rpc_method_info<M: RpcAxumMethod>(_method: &Arc<M>) -> (String, RpcHttpMethod) {
    (M::http_path(), M::HTTP_METHOD)
}

/// 注册 RPC 方法到 Axum 路由。
///
/// 自动处理路径派生、HTTP 方法映射、鉴权、JSON 反序列化和响应序列化。
///
/// # 示例
///
/// ```ignore
/// let mut router = Router::new();
/// rpc_route!(router, SendConsoleCommand::new(service));
/// ```
#[macro_export]
macro_rules! rpc_route {
    ($router:ident, $method:expr) => {{
        let method: std::sync::Arc<_> = std::sync::Arc::new($method);
        let (__rpc_path, __rpc_method) = $crate::rpc::axum::rpc_method_info(&method);

        let __rpc_handler =
            move |state: axum::extract::State<$crate::rpc::axum::AxumRpcState<_>>,
                  headers: axum::http::HeaderMap,
                  payload: Result<
                axum::Json<serde_json::Value>,
                axum::extract::rejection::JsonRejection,
            >| {
                let method = std::sync::Arc::clone(&method);
                async move {
                    $crate::rpc::axum::handle_rpc(method.as_ref(), &state, &headers, payload).await
                }
            };

        match __rpc_method {
            $crate::rpc::axum::RpcHttpMethod::Get => {
                $router = $router.route(&__rpc_path, axum::routing::get(__rpc_handler));
            }
            $crate::rpc::axum::RpcHttpMethod::Post => {
                $router = $router.route(&__rpc_path, axum::routing::post(__rpc_handler));
            }
            $crate::rpc::axum::RpcHttpMethod::Put => {
                $router = $router.route(&__rpc_path, axum::routing::put(__rpc_handler));
            }
            $crate::rpc::axum::RpcHttpMethod::Patch => {
                $router = $router.route(&__rpc_path, axum::routing::patch(__rpc_handler));
            }
            $crate::rpc::axum::RpcHttpMethod::Delete => {
                $router = $router.route(&__rpc_path, axum::routing::delete(__rpc_handler));
            }
        }
    }};
}

/// 通用 RPC handler，处理鉴权、反序列化、调度和响应。
pub(crate) async fn handle_rpc<M, A>(
    method: &M,
    state: &AxumRpcState<A>,
    headers: &HeaderMap,
    payload: Result<Json<serde_json::Value>, axum::extract::rejection::JsonRejection>,
) -> Response
where
    M: RpcMethod + Sync,
    M::Request: DeserializeOwned,
    M::Response: Serialize,
    A: HttpRpcAccessResolver + Send + Sync + 'static,
{
    let request_id = match request_id(headers) {
        Ok(request_id) => request_id,
        Err((request_id, error)) => return reject(request_id, "invalid_request_id", error),
    };

    let access = match state.access_resolver.resolve(headers) {
        Ok(access) => access,
        Err(error) => {
            return reject_with_status(
                request_id,
                "authorization_rejected",
                error,
                state.access_resolver.rejection_status(),
            );
        }
    };

    let value = match payload {
        Ok(Json(value)) => value,
        Err(_) => {
            return reject(
                request_id,
                "invalid_json",
                RpcError::invalid_argument("request", "must be a valid JSON object"),
            );
        }
    };

    let params: M::Request = match serde_json::from_value(value) {
        Ok(params) => params,
        Err(_) => {
            return reject(
                request_id,
                "invalid_params",
                RpcError::invalid_argument("request", "failed to parse request body"),
            );
        }
    };

    let context = RpcContext::new(request_id, RpcTransport::Http).with_access(access);
    match dispatch(method, RpcRequest::new(context, params)).await {
        Ok(response) => respond(StatusCode::OK, response.request_id(), &response),
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
    let status = status_for(error.code());
    reject_with_status(request_id, reason, error, status)
}

fn reject_with_status(
    request_id: RpcRequestId,
    reason: &'static str,
    error: RpcError,
    status: StatusCode,
) -> Response {
    let error = error.with_request_id(request_id);
    observability::rpc_http_request_rejected(
        error
            .request_id()
            .expect("rejection must have a request ID")
            .as_str(),
        reason,
        error.code().as_str(),
    );
    let request_id = error
        .request_id()
        .expect("rejection must have a request ID")
        .clone();
    respond(status, &request_id, error)
}

fn rpc_error_response(error: RpcError) -> Response {
    let status = status_for(error.code());
    let request_id = error
        .request_id()
        .expect("all Axum RPC errors must have a request ID")
        .clone();
    respond(status, &request_id, error)
}

fn respond<T: Serialize>(status: StatusCode, request_id: &RpcRequestId, body: T) -> Response {
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
    use std::sync::Mutex;

    use axum::{
        Router,
        body::{Body, to_bytes},
        http::Request,
    };
    use serde_json::Value;
    use tower::ServiceExt;

    use super::*;
    use crate::rpc::{
        RpcMethodName, RpcPermission,
        methods::PERMISSION_SERVER_CONSOLE_SEND,
        methods::server::SendConsoleCommand,
        service::{ConsoleCommandService, ConsoleCommandServiceError},
    };

    struct RecordingConsoleService {
        commands: Mutex<Vec<(String, String)>>,
    }

    impl ConsoleCommandService for RecordingConsoleService {
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
        let svc = Arc::new(RecordingConsoleService { commands: Mutex::new(Vec::new()) });
        let state = AxumRpcState {
            access_resolver: Arc::new(AllowConsoleSend),
        };
        let mut router = Router::new();
        rpc_route!(router, SendConsoleCommand::new(svc.clone()));
        let app = router.with_state(state);
        let response = app
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
            *svc.commands.lock().expect("recording service lock"),
            vec![("alpha".into(), "say hello".into())]
        );
    }

    #[tokio::test]
    async fn rejects_an_unprivileged_request_without_calling_the_service() {
        let svc = Arc::new(RecordingConsoleService { commands: Mutex::new(Vec::new()) });
        let state = AxumRpcState { access_resolver: Arc::new(DenyAll) };
        let mut router = Router::new();
        rpc_route!(router, SendConsoleCommand::new(svc.clone()));
        let app = router.with_state(state);

        let response = app
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
        assert!(
            svc.commands
                .lock()
                .expect("recording service lock")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn maps_invalid_json_to_the_rpc_error_envelope() {
        let svc = Arc::new(RecordingConsoleService { commands: Mutex::new(Vec::new()) });
        let state = AxumRpcState {
            access_resolver: Arc::new(AllowConsoleSend),
        };
        let mut router = Router::new();
        rpc_route!(router, SendConsoleCommand::new(svc.clone()));
        let app = router.with_state(state);
        let response = app
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
        assert!(
            svc.commands
                .lock()
                .expect("recording service lock")
                .is_empty()
        );
    }

    #[test]
    fn maps_rpc_errors_to_stable_http_statuses() {
        assert_eq!(status_for(RpcErrorCode::Unavailable), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(status_for(RpcErrorCode::DeadlineExceeded), StatusCode::GATEWAY_TIMEOUT);
        assert_eq!(status_for(RpcErrorCode::Cancelled), StatusCode::REQUEST_TIMEOUT);
    }

    #[tokio::test]
    async fn build_router_dispatches_requests_through_the_public_entry() {
        let svc = Arc::new(RecordingConsoleService { commands: Mutex::new(Vec::new()) });
        let services = crate::rpc::service::RpcServices::new(svc.clone());
        let app = crate::rpc::router::build_router(services, AllowConsoleSend);

        let response = app
            .oneshot(request(r#"{"instanceId":"beta","command":"list"}"#))
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
            *svc.commands.lock().expect("recording service lock"),
            vec![("beta".into(), "list".into())]
        );
    }

    #[tokio::test]
    async fn build_router_rejects_unprivileged_requests() {
        let svc = Arc::new(RecordingConsoleService { commands: Mutex::new(Vec::new()) });
        let services = crate::rpc::service::RpcServices::new(svc.clone());
        let app = crate::rpc::router::build_router(services, DenyAll);

        let response = app
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
        assert!(
            svc.commands
                .lock()
                .expect("recording service lock")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn build_router_rejects_invalid_json() {
        let svc = Arc::new(RecordingConsoleService { commands: Mutex::new(Vec::new()) });
        let services = crate::rpc::service::RpcServices::new(svc.clone());
        let app = crate::rpc::router::build_router(services, AllowConsoleSend);

        let response = app
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
    }

    struct HttpPathTestMethod;

    impl RpcMethod for HttpPathTestMethod {
        const NAME: RpcMethodName = RpcMethodName::new("server.console.send");
        const REQUIRED_PERMISSION: Option<RpcPermission> = None;

        type Request = ();
        type Response = ();

        async fn call(
            &self,
            _context: &RpcContext,
            _request: Self::Request,
        ) -> RpcResult<Self::Response> {
            Ok(())
        }
    }

    impl RpcAxumMethod for HttpPathTestMethod {
        const HTTP_METHOD: RpcHttpMethod = RpcHttpMethod::Post;
    }

    #[test]
    fn derives_a_stable_http_path_from_the_rpc_method_name() {
        assert_eq!(HttpPathTestMethod::http_path(), "/api/rpc/server/console/send");
    }
}
