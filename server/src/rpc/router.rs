//! RPC 方法的路由注册和 Axum 应用组装。

use std::sync::Arc;

use axum::Router;

use crate::register_methods;

use super::axum::{AxumRpcState, HttpRpcAccessResolver};
use super::service::RpcServices;

/// 组装当前所有已实现 RPC 方法的 Axum 路由。
///
/// 调用方创建 [`RpcServices`] 容器并传入此函数，返回的 [`Router`] 可直接嵌套进
/// 更大的 Axum 应用，或通过 `tower::ServiceExt::oneshot` 测试。
///
/// # 示例
///
/// ```ignore
/// let services = RpcServices::new(Arc::new(MyConsoleService));
/// let router = build_router(services, MyAccessResolver);
/// ```
pub fn build_router<A>(services: RpcServices, access_resolver: A) -> Router
where
    A: HttpRpcAccessResolver,
{
    let mut router = Router::new();

    // ── 服务器实例管理 ──
    register_methods!(
        router,
        services,
        [
            (crate::rpc::methods::server::SendConsoleCommand, console),
            // 格式：(crate::rpc::methods::<模块>::<方法>, <services 字段>)
            // 示例：(crate::rpc::methods::config::GetConfig, config),
        ]
    );

    router.with_state(AxumRpcState {
        access_resolver: Arc::new(access_resolver),
    })
}
