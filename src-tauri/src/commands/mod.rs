// 仅在 debug 构建下编译调试命令模块（发布包中不包含）
#[cfg(debug_assertions)]
pub mod debug;

pub mod config;
pub mod downloader;
pub mod java;
pub mod logging;
pub mod mcs_plugin;
pub mod player;
pub mod plugin;
pub mod server;
pub mod settings;
pub mod system;
pub mod tunnel;
pub mod update;

// 更新功能子模块
mod update_arch;
mod update_checksum;
mod update_cnb;
mod update_download;
mod update_github;
mod update_install;
mod update_types;
mod update_version;
