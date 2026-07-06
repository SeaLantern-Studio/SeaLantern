use crate::runtime::command_catalog;
use crate::runtime::desktop_setup;
use crate::runtime::desktop_shell;
use crate::runtime::frontend_runtime_event_bridge;
use crate::runtime::plugin_bridge;

use crate::services::download::download_manager::DownloadManager;
use crate::utils::logger::log_fatal_ctx;

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

            #[cfg(feature = "plugin-runtime-bridge")]
            {
                let shared_runtimes_for_server_ready =
                    std::sync::Arc::clone(&plugin_setup.runtime.shared_runtimes);

                plugin_bridge::install_plugin_runtime_bridge(
                    app,
                    std::sync::Arc::clone(&plugin_setup.runtime.shared_runtimes),
                    shared_runtimes_for_server_ready,
                    plugin_setup.runtime.api_registry.clone(),
                );
            }

            frontend_runtime_event_bridge::install_frontend_runtime_event_bridge(app);

            app.manage(plugin_setup.manager);

            let desktop_web_enabled = crate::services::global::settings_manager()
                .get()
                .enable_desktop_web_ui;
            crate::services::desktop_web::sync_desktop_web_server(
                app.handle(),
                desktop_web_enabled,
            )?;

            desktop_setup::install_frontend_watchdog(app.handle().clone());

            desktop_shell::setup_tray(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| {
            log_fatal_ctx(
                "runtime.bootstrap",
                "run_desktop",
                &format!("SeaLantern desktop runtime exited with error: {}", error),
            );
            std::process::exit(1);
        });
}
