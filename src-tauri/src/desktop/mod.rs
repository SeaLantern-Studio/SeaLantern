//! 桌面端命令模块。
//!
//! 按职责拆分各子模块，并将子模块的公开命令统一重新导出，
//! 便于宿主在 `invoke_handler` 中集中注册。

pub mod dialog;
pub mod download;

pub use dialog::{
    pick_archive_file, pick_folder, pick_image_file, pick_jar_file, pick_java_file,
    pick_save_file, pick_server_executable, pick_startup_file,
};

pub use download::{cancel_download_task, download_file, poll_task};
