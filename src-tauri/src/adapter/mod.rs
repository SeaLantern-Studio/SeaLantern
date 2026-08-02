//! 桌面端 RPC 适配层。
//!
//! 把 `server::rpc` 的宿主能力端口（如实例管理）包装为 Tauri 命令，
//! 供前端以 `invoke` 调用。按领域拆分到各文件（`instances` 等）。

pub mod instances;

pub use instances::{server_instance_get, server_instance_list};
