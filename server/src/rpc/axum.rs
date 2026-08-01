//! Axum HTTP 到 RPC 契约的传输适配器。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::{
    http::{header::HeaderName, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::observability;

use super::{
    dispatch, RpcAccess, RpcContext, RpcError, RpcErrorCode, RpcMethod, RpcRequest, RpcRequestId,
    RpcResult, RpcTransport,
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
/// 标识自动派生；需要 RESTful 路径时可声明 [`HTTP_PATH_TEMPLATE`] 覆盖。
///
/// [`HTTP_METHOD`]: Self::HTTP_METHOD
/// [`http_path`]: Self::http_path
/// [`HTTP_PATH_TEMPLATE`]: Self::HTTP_PATH_TEMPLATE
pub trait RpcAxumMethod: RpcMethod {
    /// HTTP 方法，影响 axum 路由注册方式。
    const HTTP_METHOD: RpcHttpMethod;

    /// 可选的 REST 路径模板，如 `"/api/servers/{id}/status"`。
    ///
    /// 模板中的 `{param}` 与 [`RpcMethod::Request`] 的字段名一一对应，
    /// 由 [`handle_rpc`] 从 URL 路径提取并合并进请求参数（Tauri 等其它传输
    /// 无需感知路径概念，参数以 args 全量传递）。为 `None` 时回退默认
    /// RPC 风格路径（见 [`http_path`]）。
    ///
    /// [`handle_rpc`]: crate::rpc::axum::handle_rpc
    const HTTP_PATH_TEMPLATE: Option<&'static str> = None;

    /// 派生 Axum 应注册的 HTTP 路径。
    ///
    /// 默认实现优先返回 [`HTTP_PATH_TEMPLATE`]；未声明时将方法标识中的 `.`
    /// 映射为路径 `/`，并拼接 [`HTTP_RPC_PREFIX`] 前缀。例如标识
    /// `server.console.send` 会派生出 `/api/rpc/server/console/send`。
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
    ///
    /// [`HTTP_PATH_TEMPLATE`]: Self::HTTP_PATH_TEMPLATE
    /// [`HTTP_RPC_PREFIX`]: crate::rpc::axum::HTTP_RPC_PREFIX
    fn http_path() -> String {
        if let Some(template) = Self::HTTP_PATH_TEMPLATE {
            return template.to_string();
        }
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
                  path: Option<axum::extract::Path<std::collections::HashMap<String, String>>>,
                  payload: Result<
                axum::Json<serde_json::Value>,
                axum::extract::rejection::JsonRejection,
            >| {
                let method = std::sync::Arc::clone(&method);
                async move {
                    $crate::rpc::axum::handle_rpc(method.as_ref(), &state, &headers, path, payload)
                        .await
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
///
/// 路径参数（REST 模板 `{param}`）与请求体合并后反序列化为 `M::Request`；
/// 无请求体（如 REST GET）时以空对象兜底。
pub(crate) async fn handle_rpc<M, A>(
    method: &M,
    state: &AxumRpcState<A>,
    headers: &HeaderMap,
    path_params: Option<axum::extract::Path<std::collections::HashMap<String, String>>>,
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
        Err(error) => return reject(request_id, "authorization_rejected", error),
    };

    let params = match build_params::<M>(path_params, payload) {
        Ok(params) => params,
        Err(error) => return reject(request_id, "invalid_params", error),
    };

    let context = RpcContext::new(request_id, RpcTransport::Http).with_access(access);
    match dispatch(method, RpcRequest::new(context, params)).await {
        Ok(response) => respond(StatusCode::OK, response.request_id(), &response),
        Err(error) => rpc_error_response(error),
    }
}

/// 将路径参数与请求体合并后反序列化为 `M::Request`。
///
/// - 请求体：合法 JSON 对象直接使用；缺少 JSON 请求体（REST GET 等）视为空对象；
///   请求体存在但不是合法 JSON 时拒绝。
/// - 路径参数：来自 REST 模板 `{param}` 的值以字符串形式覆盖合并进请求体
///   （字段类型由 `M::Request` 反序列化时决定）。
fn build_params<M: RpcMethod>(
    path_params: Option<axum::extract::Path<std::collections::HashMap<String, String>>>,
    payload: Result<Json<serde_json::Value>, axum::extract::rejection::JsonRejection>,
) -> RpcResult<M::Request>
where
    M::Request: DeserializeOwned,
{
    let mut value = match payload {
        Ok(Json(value)) => value,
        // 无 JSON 请求体（如 REST GET）：以空对象兜底，参数全部来自路径
        Err(axum::extract::rejection::JsonRejection::MissingJsonContentType(_)) => {
            serde_json::Value::Object(Default::default())
        }
        Err(_) => {
            return Err(RpcError::invalid_argument("request", "must be a valid JSON object"));
        }
    };

    if let Some(axum::extract::Path(path_params)) = path_params {
        let object = value.as_object_mut().ok_or_else(|| {
            RpcError::invalid_argument("request", "path parameters require a JSON object")
        })?;
        for (key, param) in path_params {
            object.insert(key, serde_json::Value::String(param));
        }
    }

    serde_json::from_value(value)
        .map_err(|_| RpcError::invalid_argument("request", "failed to parse request body"))
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
#[path = "axum_tests.rs"]
mod tests;
