//! 服务器实例管理的 RPC 方法实现（Tauri 侧）。
//!
//! 每个方法实现 `server::rpc::RpcMethod`，内部借用 [`CoreInstanceService`]，
//! 让 Tauri 命令与 server 的 HTTP(axum) 端真正共用同一套 `dispatch` 契约。
//! 方法本身不接触任何传输细节；权限由命令层统一经 [`super::tauri_request`] 注入。

use std::future::Future;
use std::sync::Arc;

use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_server::rpc::traits::instance::InstanceServiceError;
use sealantern_server::rpc::traits::InstanceService;
use sealantern_server::rpc::{
    RpcContext, RpcError, RpcMethod, RpcMethodName, RpcPermission, RpcResult,
};

use crate::services::instance::CoreInstanceService;

/// 实例管理的调读权限（只读操作）。
pub const PERMISSION_INSTANCE_READ: RpcPermission = RpcPermission::new("server.instance.read");
/// 实例管理的写权限（创建/删除/改名/改路径）。
pub const PERMISSION_INSTANCE_WRITE: RpcPermission = RpcPermission::new("server.instance.write");

/// 把实例服务错误映射为稳定的 RPC 错误，不携带底层路径或凭据。
fn map_error(error: InstanceServiceError) -> RpcError {
    match error {
        InstanceServiceError::InstanceNotFound => RpcError::not_found("server instance"),
        InstanceServiceError::AlreadyExists => RpcError::conflict("create a server instance"),
        InstanceServiceError::InvalidState => RpcError::conflict("perform this instance operation"),
        InstanceServiceError::OperationFailed => RpcError::unavailable("server instance"),
        InstanceServiceError::Unsupported => RpcError::invalid_argument(
            "operation",
            "the requested server instance operation is not supported",
        ),
    }
}

/// 列出全部实例。
pub(crate) struct InstanceList {
    service: Arc<CoreInstanceService>,
}

impl InstanceList {
    pub(crate) fn new(service: Arc<CoreInstanceService>) -> Self {
        Self { service }
    }
}

impl RpcMethod for InstanceList {
    const NAME: RpcMethodName = RpcMethodName::new("server.instance.list");
    const REQUIRED_PERMISSION: Option<RpcPermission> = Some(PERMISSION_INSTANCE_READ);

    type Request = ();
    type Response = Vec<Instance>;

    fn call(
        &self,
        _context: &RpcContext,
        _request: Self::Request,
    ) -> impl Future<Output = RpcResult<Self::Response>> + Send {
        let service = Arc::clone(&self.service);
        async move { service.list().await.map_err(map_error) }
    }
}

/// 按 ID 查找实例。
pub(crate) struct InstanceGet {
    service: Arc<CoreInstanceService>,
}

impl InstanceGet {
    pub(crate) fn new(service: Arc<CoreInstanceService>) -> Self {
        Self { service }
    }
}

impl RpcMethod for InstanceGet {
    const NAME: RpcMethodName = RpcMethodName::new("server.instance.get");
    const REQUIRED_PERMISSION: Option<RpcPermission> = Some(PERMISSION_INSTANCE_READ);

    type Request = InstanceId;
    type Response = Option<Instance>;

    fn call(
        &self,
        _context: &RpcContext,
        request: Self::Request,
    ) -> impl Future<Output = RpcResult<Self::Response>> + Send {
        let service = Arc::clone(&self.service);
        async move { service.find(&request).await.map_err(map_error) }
    }
}

/// 创建新实例。
pub(crate) struct InstanceCreate {
    service: Arc<CoreInstanceService>,
}

impl InstanceCreate {
    pub(crate) fn new(service: Arc<CoreInstanceService>) -> Self {
        Self { service }
    }
}

impl RpcMethod for InstanceCreate {
    const NAME: RpcMethodName = RpcMethodName::new("server.instance.create");
    const REQUIRED_PERMISSION: Option<RpcPermission> = Some(PERMISSION_INSTANCE_WRITE);

    type Request = InstanceSpec;
    type Response = Instance;

    fn call(
        &self,
        _context: &RpcContext,
        request: Self::Request,
    ) -> impl Future<Output = RpcResult<Self::Response>> + Send {
        let service = Arc::clone(&self.service);
        async move { service.create(request).await.map_err(map_error) }
    }
}

/// 删除实例。
pub(crate) struct InstanceDelete {
    service: Arc<CoreInstanceService>,
}

impl InstanceDelete {
    pub(crate) fn new(service: Arc<CoreInstanceService>) -> Self {
        Self { service }
    }
}

impl RpcMethod for InstanceDelete {
    const NAME: RpcMethodName = RpcMethodName::new("server.instance.delete");
    const REQUIRED_PERMISSION: Option<RpcPermission> = Some(PERMISSION_INSTANCE_WRITE);

    type Request = InstanceId;
    type Response = bool;

    fn call(
        &self,
        _context: &RpcContext,
        request: Self::Request,
    ) -> impl Future<Output = RpcResult<Self::Response>> + Send {
        let service = Arc::clone(&self.service);
        async move { service.delete(&request).await.map_err(map_error) }
    }
}

/// 重命名请求参数。
pub(crate) struct RenameRequest {
    pub(crate) id: InstanceId,
    pub(crate) name: String,
}

/// 重命名实例。
pub(crate) struct InstanceRename {
    service: Arc<CoreInstanceService>,
}

impl InstanceRename {
    pub(crate) fn new(service: Arc<CoreInstanceService>) -> Self {
        Self { service }
    }
}

impl RpcMethod for InstanceRename {
    const NAME: RpcMethodName = RpcMethodName::new("server.instance.rename");
    const REQUIRED_PERMISSION: Option<RpcPermission> = Some(PERMISSION_INSTANCE_WRITE);

    type Request = RenameRequest;
    type Response = ();

    fn call(
        &self,
        _context: &RpcContext,
        request: Self::Request,
    ) -> impl Future<Output = RpcResult<Self::Response>> + Send {
        let service = Arc::clone(&self.service);
        async move {
            service
                .rename(&request.id, &request.name)
                .await
                .map_err(map_error)
        }
    }
}

/// 更新路径请求参数。
pub(crate) struct UpdatePathRequest {
    pub(crate) id: InstanceId,
    pub(crate) path: String,
}

/// 更新实例目录路径。
pub(crate) struct InstanceUpdatePath {
    service: Arc<CoreInstanceService>,
}

impl InstanceUpdatePath {
    pub(crate) fn new(service: Arc<CoreInstanceService>) -> Self {
        Self { service }
    }
}

impl RpcMethod for InstanceUpdatePath {
    const NAME: RpcMethodName = RpcMethodName::new("server.instance.update");
    const REQUIRED_PERMISSION: Option<RpcPermission> = Some(PERMISSION_INSTANCE_WRITE);

    type Request = UpdatePathRequest;
    type Response = ();

    fn call(
        &self,
        _context: &RpcContext,
        request: Self::Request,
    ) -> impl Future<Output = RpcResult<Self::Response>> + Send {
        let service = Arc::clone(&self.service);
        async move {
            service
                .update_path(&request.id, &request.path)
                .await
                .map_err(map_error)
        }
    }
}
