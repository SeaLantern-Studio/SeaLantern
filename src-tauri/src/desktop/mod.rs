//! 桌面端命令模块。
//!
//! 按职责拆分各子模块，并将子模块的公开命令统一重新导出，
//! 便于宿主在 `invoke_handler` 中集中注册。

mod auto_lightweight;
pub mod desktop_jelly;
pub mod dialog;
mod effects;
pub mod lightweight;
pub mod pet;
mod theme;
pub mod tray;
pub mod window_state;

pub use dialog::{
    desktop_open_folder, desktop_pick_archive_file, desktop_pick_folder, desktop_pick_image_file,
    desktop_pick_jar_file, desktop_pick_java_file, desktop_pick_save_file,
    desktop_pick_server_executable, desktop_pick_startup_file,
};

pub use auto_lightweight::AutoLightweightState;
pub use desktop_jelly::{
    DesktopJellyState, desktop_jelly_start, desktop_jelly_status, desktop_jelly_stop,
};
pub use effects::{apply_acrylic, set_window_material, supports_liquid_glass};
pub use pet::{
    PetWindowState, pet_frontend_ready, pet_hide, pet_open_main_window, pet_set_click_through,
};
pub use theme::DesktopAppearanceState;
pub use tray::{hide_main_window, restore_main_window, toggle_light_weight};
pub use window_state::MainWindowState;
