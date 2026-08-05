//! Sea Lantern 桌面端的 Tauri 宿主入口。

pub mod desktop;
pub mod observability;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// 启动桌面应用。
pub fn run() {
    // 初始化 tracing 日志（在 Tauri 构建之前）
    observability::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            desktop::dialog::pick_archive_file,
            desktop::dialog::pick_folder,
            desktop::dialog::pick_image_file,
            desktop::dialog::pick_jar_file,
            desktop::dialog::pick_java_file,
            desktop::dialog::pick_save_file,
            desktop::dialog::pick_server_executable,
            desktop::dialog::pick_startup_file,
            desktop::download::download_file,
            desktop::download::poll_task,
            desktop::download::cancel_download_task,
        ])
        .setup(|app| {
            // 前端提供自定义标题栏；macOS 仍使用 Overlay 承载系统交通灯。
            #[cfg(not(target_os = "macos"))]
            if let Some(window) = app.get_webview_window("main") {
                window.set_decorations(false)?;
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Sea Lantern");
}
