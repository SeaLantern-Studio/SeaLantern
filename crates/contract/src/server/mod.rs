//! 服务器进程管理契约模型。
//!
//! 提供服务器进程生命周期（启动/停止/强制停止/状态/控制台命令）的宿主能力端口，
//! 供 tauri / server 等宿主统一消费。实例记录管理由
//! `application/src/port/instance.rs` 定义。

mod models;

pub use models::{ServerSnapshot, ServerState};
