//! Sea Lantern 桌面端的 Tauri 宿主入口。

pub mod adapter;
pub mod desktop;
pub mod observability;

use tauri::Manager;

use adapter::tauri::commands::compat::download_compat::{
    cancel_download_task, download_file, poll_task,
};
use adapter::tauri::commands::compat::instance_compat::{
    add_existing_server, collect_copy_conflicts, copy_directory_contents, create_server,
    delete_server, force_stop_server, get_server_list, get_server_logs, get_server_status,
    import_modpack, import_server, parse_server_core_type, prepare_force_stop_server,
    restart_server, scan_startup_candidates, send_command, start_server, stop_server,
    update_server_name, update_server_path, validate_server_path,
};
use adapter::tauri::commands::compat::settings_compat::{
    export_settings, get_settings, import_settings, reset_settings, save_settings,
    save_settings_with_diff, update_settings_partial,
};
use adapter::tauri::commands::compat::system_compat::{
    get_default_run_path, get_safe_mode_status, get_server_resource_usage, get_system_info,
    open_file, open_folder, remove_file, test_ipv6_connectivity,
};
use adapter::tauri::commands::cron::{
    create_cron_task, delete_cron_task, list_cron_tasks, run_cron_task, set_cron_task_enabled,
    update_cron_task,
};
use adapter::tauri::commands::system::{
    get_directory_usage, get_process_usage, get_system_snapshot,
};
use adapter::tauri::commands::update::check_update;
use desktop::{
    desktop_pick_archive_file, desktop_pick_folder, desktop_pick_image_file, desktop_pick_jar_file,
    desktop_pick_java_file, desktop_pick_save_file, desktop_pick_server_executable,
    desktop_pick_startup_file,
};

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
            //桌面端能力（由desktop/dialog提供）
            desktop_pick_archive_file,
            desktop_pick_folder,
            desktop_pick_image_file,
            desktop_pick_jar_file,
            desktop_pick_java_file,
            desktop_pick_save_file,
            desktop_pick_server_executable,
            desktop_pick_startup_file,
            //服务器定时任务契约命令
            create_cron_task,
            delete_cron_task,
            list_cron_tasks,
            run_cron_task,
            set_cron_task_enabled,
            update_cron_task,
            //系统资源能力（由adapter/tauri/commands接入application）
            get_directory_usage,
            get_process_usage,
            get_system_snapshot,
            //应用更新检查契约命令
            check_update,
            //兼容层（前端旧命令名 → 新服务，由adapter/tauri/commands/compat提供）
            add_existing_server,
            cancel_download_task,
            collect_copy_conflicts,
            copy_directory_contents,
            create_server,
            delete_server,
            download_file,
            export_settings,
            force_stop_server,
            get_default_run_path,
            get_safe_mode_status,
            get_server_list,
            get_server_logs,
            get_server_resource_usage,
            get_server_status,
            get_settings,
            get_system_info,
            import_modpack,
            import_server,
            import_settings,
            open_file,
            open_folder,
            parse_server_core_type,
            poll_task,
            prepare_force_stop_server,
            remove_file,
            reset_settings,
            restart_server,
            save_settings,
            save_settings_with_diff,
            scan_startup_candidates,
            send_command,
            start_server,
            stop_server,
            test_ipv6_connectivity,
            update_server_name,
            update_server_path,
            update_settings_partial,
            validate_server_path
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
