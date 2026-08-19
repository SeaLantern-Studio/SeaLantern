//! Tauri 命令适配模块。
//!
//! 每个子模块承载一类宿主能力，命令以 snake_case 命名直接暴露给前端
//! `invoke` 调用，内部统一经应用装配层组合对应的应用服务。

pub mod backup;
pub mod catalog;
pub mod console;
pub mod cron;
pub mod download;
pub mod instance;
pub mod java;
pub mod logging;
pub mod online_tunnel;
pub mod plugin;
pub mod provisioning;
pub mod server;
pub mod server_config;
pub mod settings;
pub mod system;
pub mod update;
pub mod update_install;
