//! Sea Lantern 桌面端的 Tauri 宿主入口。

pub mod adapter;
pub mod desktop;
pub mod observability;

use sealantern_application::services::AppServices;
use tauri::Manager;

use adapter::tauri::commands::catalog::{catalog_details, catalog_server_types, catalog_versions};
use adapter::tauri::commands::cron::{
    create_cron_task, delete_cron_task, list_cron_tasks, run_cron_task, set_cron_task_enabled,
    update_cron_task,
};
use adapter::tauri::commands::download::{download_cancel, download_create, download_query};
use adapter::tauri::commands::instance::{
    create_instance, delete_instance, get_instance, list_instances, rename_instance,
    update_instance_path,
};
use adapter::tauri::commands::java::{java_detect, java_validate};
use adapter::tauri::commands::plugin::{
    plugin_v2_audit, plugin_v2_disable, plugin_v2_discover, plugin_v2_enable, plugin_v2_load,
    plugin_v2_plugins, plugin_v2_unload,
};
use adapter::tauri::commands::provisioning::{
    inspect_server, parse_startup_script, plan_existing_instance, plan_instance_copy,
    plan_modpack_provision,
};
use adapter::tauri::commands::server::{
    force_stop_server, restart_server, send_server_command, server_status, start_server,
    stop_server,
};
use adapter::tauri::commands::settings::{
    export_settings, get_settings, import_settings, reset_settings, settings_overview,
    update_settings, update_settings_partial,
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
            catalog_details,
            catalog_server_types,
            catalog_versions,
            //系统资源能力（由adapter/tauri/commands接入application）
            get_directory_usage,
            get_process_usage,
            get_system_snapshot,
            //实例与服务器进程服务
            create_instance,
            delete_instance,
            get_instance,
            list_instances,
            rename_instance,
            update_instance_path,
            java_detect,
            java_validate,
            force_stop_server,
            restart_server,
            send_server_command,
            server_status,
            start_server,
            stop_server,
            //下载与设置服务
            download_cancel,
            download_create,
            download_query,
            export_settings,
            get_settings,
            import_settings,
            reset_settings,
            settings_overview,
            update_settings,
            update_settings_partial,
            //服务端检查与供给计划
            inspect_server,
            parse_startup_script,
            plan_existing_instance,
            plan_instance_copy,
            plan_modpack_provision,
            //应用更新检查契约命令
            check_update,
            //插件 v2 宿主能力与策略管理
            plugin_v2_audit,
            plugin_v2_disable,
            plugin_v2_discover,
            plugin_v2_enable,
            plugin_v2_load,
            plugin_v2_plugins,
            plugin_v2_unload
        ])
        .setup(|app| {
            tauri::async_runtime::block_on(async {
                let services = AppServices::get().await.map_err(|error| {
                    std::io::Error::other(format!(
                        "failed to assemble application services: {error}"
                    ))
                })?;
                services
                    .initialize_network_settings()
                    .await
                    .map_err(|error| {
                        std::io::Error::other(format!(
                            "failed to initialize persisted network settings: {error}"
                        ))
                    })?;
                Ok::<(), std::io::Error>(())
            })?;

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

#[cfg(test)]
mod tests {
    const SNAKE_CASE_SERVICE_COMMANDS: &[&str] = &[
        "create_cron_task",
        "delete_cron_task",
        "list_cron_tasks",
        "run_cron_task",
        "set_cron_task_enabled",
        "update_cron_task",
        "get_directory_usage",
        "get_process_usage",
        "get_system_snapshot",
        "create_instance",
        "delete_instance",
        "get_instance",
        "list_instances",
        "rename_instance",
        "update_instance_path",
        "force_stop_server",
        "restart_server",
        "send_server_command",
        "server_status",
        "start_server",
        "stop_server",
        "download_cancel",
        "download_create",
        "download_query",
        "export_settings",
        "get_settings",
        "import_settings",
        "reset_settings",
        "settings_overview",
        "update_settings",
        "update_settings_partial",
        "check_update",
    ];

    #[test]
    fn snake_case_service_commands_are_registered() {
        let source = include_str!("lib.rs");
        let (_, handler) = source
            .split_once(".invoke_handler(tauri::generate_handler![")
            .expect("Tauri handler must exist");
        let (handler, _) = handler
            .split_once("])\n        .setup")
            .expect("Tauri handler must close before setup");

        for command in SNAKE_CASE_SERVICE_COMMANDS {
            assert!(handler.contains(command), "snake_case command {command} must be registered");
        }
    }
}
