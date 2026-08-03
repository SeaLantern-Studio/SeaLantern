//! Tauri 传输适配辅助。
//!
//! 提供 Tauri `invoke` 到 RPC 契约所需的上下文构造、权限解析和错误映射，
//! 角色对位 server 的 `rpc::axum`（HTTP 传输适配器），但因 Tauri 本地可信本质
//! 而采用独立实现：不套用 HTTP 的 header/`HttpRpcAccessResolver` 机制。

use sealantern_server::rpc::{
    RpcAccess, RpcContext, RpcError, RpcRequest, RpcRequestId, RpcTransport,
};

/// Tauri 端权限解析的集中入口。
///
/// 当前本地进程调用视为受信任，授予全部权限；未来插件桥接时在此按调用方身份
/// （`RpcContext.transport` / 请求来源）收敛为对应权限集合。命令层不直接构造
/// `RpcAccess`，避免权限逻辑散落。
pub fn tauri_access() -> RpcAccess {
    // 本地受信任进程：允许全部（尚未引入插件调用方）
    RpcAccess::allow_all()
}

/// 生成一个 Tauri 调用使用的请求关联标识。
///
/// Tauri 没有 HTTP 头来传递 request_id，这里用一个进程级自增序列模拟，
/// 保持与 HTTP 端一致的关联日志能力。
pub fn tauri_request_id() -> RpcRequestId {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    RpcRequestId::new(format!("tauri-{sequence}")).expect("generated request id must be valid")
}

/// 构造一个带本地授权上下文的 Tauri RPC 请求。
pub fn tauri_request<T>(params: T) -> RpcRequest<T> {
    let context =
        RpcContext::new(tauri_request_id(), RpcTransport::TauriInvoke).with_access(tauri_access());
    RpcRequest::new(context, params)
}

/// 把 dispatch 返回的 RpcError 映射为 Tauri 命令的 `Err(String)`。
pub fn rpc_error_message(error: RpcError) -> String {
    error.to_string()
}
