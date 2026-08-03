//! Tauri 侧 RPC 方法实现与传输适配。
//!
//! 这里实现 `server::rpc::contract::RpcMethod` 传输无关方法（`instances`），
//! 并提供 Tauri 传输的适配辅助（`adapter`）。设计原则：
//!
//! - **该对齐的对齐**：方法走统一的 `dispatch`（校验 → 权限 → call → 包络），
//!   与 server 的 HTTP(axum) 端共用同一套契约。
//! - **不该对齐的坚决不对齐**：Tauri 是本地可信进程，权限解析、请求参数来源、
//!   错误呈现都由本地适配器决定，不套用 HTTP 的 `HttpRpcAccessResolver`/header 机制。
//! - **为插件铺路**：权限在一个集中点（[`adapter::tauri_access`]）按调用方决定，
//!   未来插件接入时只需在此按调用方身份注入权限，而不是在每条命令里散落
//!   硬编码 `RpcAccess::allow`。

pub mod adapter;
pub mod instances;

pub use adapter::{rpc_error_message, tauri_access, tauri_request, tauri_request_id};
