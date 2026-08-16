//! 桌面端命令模块。
//!
//! 按职责拆分各子模块，并将子模块的公开命令统一重新导出，
//! 便于宿主在 `invoke_handler` 中集中注册。

pub mod dialog;
pub mod window_effect;

pub use dialog::{
    desktop_pick_archive_file, desktop_pick_folder, desktop_pick_image_file, desktop_pick_jar_file,
    desktop_pick_java_file, desktop_pick_save_file, desktop_pick_server_executable,
    desktop_pick_startup_file,
};

pub use window_effect::apply_acrylic;
