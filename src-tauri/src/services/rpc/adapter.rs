//! Tauri 传输适配辅助。
//!
//! 提供 Tauri `invoke` 到 RPC 契约所需的上文构造、权限提供和错误映射，
//! 与 server 的 `rpc::axum`（HTTP 适配器）对位，但因 Tauri 是本地可信进程，
//! 采用独立的适配实现，不复用 HTTP 的 header / `HttpRpcAccessResolver` 机制。

use sealantern_server::rpc::{
    RpcAccess, RpcContext, RpcError, RpcRequest, RpcRequestId, RpcTransport,
};

/// Tauri 端权限解析的集中入口。
///
/// 当前桌面进程视为本地可信调用方，授予全部权限；未来引入插件调用方时，可在此
/// 依据调用来源（`RpcContext.transport` 或请求标识）收敛为调用方对应的权限集合。
/// 命令层不在各处散落 `RpcAccess` 构造，统一经由本函数获取。
pub fn tauri_access() -> RpcAccess {
    // 本地受信任进程：授予全部权限（尚未引入插件调用方）
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
