//! 宿主能力实现与 RPC 方法层。
//!
//! - `instance.rs` 用 `core` / `extra` 的能力实现 `server` crate 定义的
//!   `InstanceService` 端口（持久化到 `sea_lantern_servers.json`）；
//! - `rpc/` 承载桌面端 RPC 方法实现与 Tauri 传输适配；
//! - `app_service.rs` 承载应用级自托管容器。
//!
//! `adapter` 负责 Tauri 命令的传输适配，`services` 负责能力与 RPC 实现，分层明确。

pub mod app_service;
pub mod instance;
pub mod rpc;

pub use app_service::AppServices;
pub use instance::CoreInstanceService;
