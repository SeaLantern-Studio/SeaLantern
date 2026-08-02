use std::sync::Mutex;

use axum::{
    body::{to_bytes, Body},
    http::Request,
    Router,
};
use serde_json::Value;
use tower::ServiceExt;

use super::*;
use crate::rpc::{
    methods::server::SendConsoleCommand,
    methods::PERMISSION_SERVER_CONSOLE_SEND,
    service::{ConsoleCommandService, ConsoleCommandServiceError, InstanceServiceError},
    RpcMethodName, RpcPermission,
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

/// 供 `build_router` 测试使用的空实例管理服务。
struct NoopInstanceService;

#[async_trait::async_trait]
impl crate::rpc::service::InstanceService for NoopInstanceService {
    async fn list(&self) -> Result<Vec<sealantern_core::instance::Instance>, InstanceServiceError> {
        Ok(Vec::new())
    }

    async fn find(
        &self,
        _id: &sealantern_core::instance::InstanceId,
    ) -> Result<Option<sealantern_core::instance::Instance>, InstanceServiceError> {
        Ok(None)
    }

    async fn status(
        &self,
        _id: &sealantern_core::instance::InstanceId,
    ) -> Result<sealantern_core::server::ServerStatus, InstanceServiceError> {
        Ok(sealantern_core::server::ServerStatus {
            process_id: 0,
            state: sealantern_core::server::ServerProcessState::Running,
        })
    }

    async fn start(
        &self,
        _id: &sealantern_core::instance::InstanceId,
    ) -> Result<(), InstanceServiceError> {
        Ok(())
    }

    async fn stop(
        &self,
        _id: &sealantern_core::instance::InstanceId,
    ) -> Result<(), InstanceServiceError> {
        Ok(())
    }

    async fn force_stop(
        &self,
        _id: &sealantern_core::instance::InstanceId,
    ) -> Result<(), InstanceServiceError> {
        Ok(())
    }

    async fn create(
        &self,
        _spec: sealantern_core::instance::InstanceSpec,
    ) -> Result<sealantern_core::instance::Instance, InstanceServiceError> {
        Err(InstanceServiceError::Unsupported)
    }

    async fn delete(
        &self,
        _id: &sealantern_core::instance::InstanceId,
    ) -> Result<bool, InstanceServiceError> {
        Ok(false)
    }

    async fn rename(
        &self,
        _id: &sealantern_core::instance::InstanceId,
        _name: &str,
    ) -> Result<(), InstanceServiceError> {
        Ok(())
    }

    async fn update_path(
        &self,
        _id: &sealantern_core::instance::InstanceId,
        _path: &str,
    ) -> Result<(), InstanceServiceError> {
        Ok(())
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
    assert!(svc
        .commands
        .lock()
        .expect("recording service lock")
        .is_empty());
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
    assert!(svc
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

#[tokio::test]
async fn build_router_dispatches_requests_through_the_public_entry() {
    let svc = Arc::new(RecordingConsoleService { commands: Mutex::new(Vec::new()) });
    let services = crate::rpc::service::RpcServices::new(
        svc.clone(),
        std::sync::Arc::new(NoopInstanceService),
    );
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
    let services = crate::rpc::service::RpcServices::new(
        svc.clone(),
        std::sync::Arc::new(NoopInstanceService),
    );
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
    assert!(svc
        .commands
        .lock()
        .expect("recording service lock")
        .is_empty());
}

#[tokio::test]
async fn build_router_rejects_invalid_json() {
    let svc = Arc::new(RecordingConsoleService { commands: Mutex::new(Vec::new()) });
    let services = crate::rpc::service::RpcServices::new(
        svc.clone(),
        std::sync::Arc::new(NoopInstanceService),
    );
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
