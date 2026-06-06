use crate::runtime::command_catalog;
use crate::runtime::desktop_setup;
use crate::runtime::desktop_shell;
use crate::runtime::plugin_bridge;

use crate::services::download::download_manager::DownloadManager;

use std::sync::Arc;
use tauri::{Emitter, Manager};

/// 启动桌面模式。
pub(crate) fn run_desktop() {
    let download_manager = DownloadManager::new();

    tauri::Builder::default()
        .manage(download_manager)
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            desktop_shell::handle_single_instance(app, args, cwd.into());
        }))
        .on_tray_icon_event(|app, event| {
            desktop_shell::handle_builder_tray_click(app, &event);
        })
        .invoke_handler(command_catalog::desktop_handler())
        .on_window_event(|window, event| {
            // 处理文件拖放事件，发送到前端
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Enter { .. }) = event {
                let _ = window.emit("tauri://drag", ());
            }
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) = event {
                let _ = window.emit("tauri://drop", paths);
            }
            if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Leave) = event {
                let _ = window.emit("tauri://drag-cancelled", ());
            }

            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                desktop_shell::handle_close_requested(window, api);
            }
        })
        .setup(|app| {
            desktop_setup::apply_platform_window_style(app);

            let plugin_setup = desktop_setup::initialize_plugins();
            let shared_runtimes_for_server_ready = Arc::clone(&plugin_setup.shared_runtimes);

            plugin_bridge::install_plugin_bridge(
                app,
                plugin_setup.shared_runtimes,
                shared_runtimes_for_server_ready,
                plugin_setup.api_registry,
            );

            app.manage(plugin_setup.manager);

            desktop_setup::install_frontend_watchdog(app.handle().clone());

            desktop_shell::setup_tray(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Sea Lantern");
}
