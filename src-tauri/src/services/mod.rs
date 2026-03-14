//! SeaLantern services 层入口模块。
//!
//! - 按领域导出子模块：`server` / `http` / `download`；
//! - 顶层仅保留少量横切模块：`global`、`i18n`、`panic_report`、`async_loader` 等；
//! - 通过 `pub use` 为历史模块名提供别名（如 `server_manager`、`download_manager`），
//!   以便在未来大版本中按计划移除这些别名而不影响当前调用方。
pub mod async_loader;
pub mod download;
pub mod global;
pub mod http;
pub mod i18n;
pub mod java_detector;
pub mod mcs_plugin_manager;
pub mod mod_manager;
pub mod panic_report;
pub mod server;
pub mod settings_manager;

pub use download::download_manager;
pub use download::java_installer;
pub use download::starter_installer_links;

pub use server::config as config_parser;
#[allow(unused_imports)]
pub use server::downloader as server_downloader;
pub use server::id_manager as server_id_manager;
pub use server::installer as server_installer;
pub use server::join as join_manager;
pub use server::log_pipeline as server_log_pipeline;
pub use server::manager as server_manager;
pub use server::player as player_manager;
