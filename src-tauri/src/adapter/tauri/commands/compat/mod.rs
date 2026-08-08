//! 前端旧命令名兼容子层。
//!
//! 前端 `src/api/*.ts` 仍使用旧命令名（如 `create_server`、`get_server_list`），
//! 后端 service 层已迁移到新接口（如 `create_instance`、`list_instances`）。
//! 本子模块注册前端旧命令名作为 Tauri 命令，内部做参数/响应适配后调用新 service，
//! 让前端零改动即可对接新后端。
//!
//! - [`adapter`]：纯转换函数（Instance ↔ 前端形态、SystemSnapshot ↔ 前端形态），可单测。
//! - [`models`]：前端形态的请求/响应 DTO（serde 结构体，camelCase 反序列化）。
//! - [`error`]：跨域错误映射辅助（instance_err ↔ system_err）。
//! - [`instance_compat`]：`src/api/server.ts` 对应的兼容命令。
//! - [`system_compat`]：`src/api/system.ts` 对应的兼容命令。
//!
//! 未对接后端的命令一律返回 `Unsupported`，绝不静默 no-op。

pub mod adapter;
pub mod error;
pub mod instance_compat;
pub mod models;
pub mod system_compat;
