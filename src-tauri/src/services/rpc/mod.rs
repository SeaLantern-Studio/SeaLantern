//! Tauri 侧的 RPC 方法实现与传输适配。
//!
//! 本模块在桌面端实现传输无关的 `server::rpc::RpcMethod` 方法（[`instances`]），
//! 并提供 Tauri `invoke` 到 `dispatch` 的传输适配（[`adapter`]），使桌面命令与
//! server 的 HTTP(axum) 端共用同一套 RPC 契约。
//!
//! Tauri 作为本地可信进程，其传输适配（权限解析、请求来源、错误呈现）由
//! [`adapter`] 独立实现，不复用 HTTP 的 resolver / header 机制。

pub mod adapter;
pub mod instances;

pub use adapter::{rpc_error_message, tauri_access, tauri_request, tauri_request_id};
