//! SeaLantern services 层入口模块。
//!
//! - 按领域导出子模块：`server` / `download` / `online`；
//! - 顶层仅保留少量横切模块：`global`、`i18n`、`panic_report` 等。
pub mod data_dir;
pub mod download;
pub mod event_consumer_registry;
pub mod events;
pub mod global;
pub mod i18n;
pub mod java_detector;
pub mod mod_manager;
pub mod online;
pub mod plugin_dir;
pub mod plugin_trusted_catalog;
pub mod server;
pub mod settings_manager;
