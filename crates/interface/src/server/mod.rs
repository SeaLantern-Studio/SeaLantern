//! 服务器进程管理服务。
//!
//! 提供服务器进程生命周期（启动/停止/强制停止/状态/控制台命令）的宿主能力端口，
//! 供 tauri / server 等宿主统一消费。实例记录管理见 [`crate::instance`]。

mod models;
mod service;

pub use models::{ServerSnapshot, ServerState};
pub use service::ServerService;
