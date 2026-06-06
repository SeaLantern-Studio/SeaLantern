//! SeaLantern services 层入口模块。
//!
//! - 按领域导出子模块：`server` / `download` / `online`；
//! - 顶层仅保留少量横切模块：`global`、`i18n`、`panic_report` 等。
pub mod download;
pub mod global;
pub mod i18n;
pub mod java_detector;
pub(crate) mod locale_json;
pub mod mod_manager;
pub mod online;
pub mod panic_report;
pub mod server;
pub mod settings_manager;
