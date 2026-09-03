//! 桌面端库目标的共享模块。
//!
//! 实际应用入口由 `main.rs` 提供；该库目标仅保留可复用的桌面模块，避免
//! 与当前 adapter Tauri 宿主重复注册已经迁移的命令。

pub mod desktop;
pub mod observability;
