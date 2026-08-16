//! Sea Lantern 桌面端的 Tauri 宿主入口。

pub mod adapter;
pub mod desktop;
pub mod observability;

use sealantern_application::services::AppServices;
use sealantern_interface::OnlineTunnelService;
use tauri::{AppHandle, Manager};

use adapter::tauri::commands::catalog::{catalog_details, catalog_server_types, catalog_versions};
use adapter::tauri::commands::console::get_server_logs;
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
use adapter::tauri::commands::logging::share_logs;
use adapter::tauri::commands::online_tunnel::{
    online_tunnel_host, online_tunnel_join, online_tunnel_status, online_tunnel_stop,
    OnlineTunnelEventForwarder,
};
use adapter::tauri::commands::plugin::{
    plugin_v2_approve_session, plugin_v2_audit, plugin_v2_disable, plugin_v2_discover,
    plugin_v2_enable, plugin_v2_end_session, plugin_v2_grant_persistent, plugin_v2_grant_session,
    plugin_v2_invoke, plugin_v2_issue_approval_token, plugin_v2_load, plugin_v2_plugins,
    plugin_v2_revoke_persistent, plugin_v2_set_trust, plugin_v2_unload,
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
    get_default_run_path, get_server_resource_usage, get_system_snapshot,
};
use adapter::tauri::commands::update::check_update;
use adapter::tauri::commands::update_install::{
    update_clear_pending, update_download, update_install, update_pending,
};
use adapter::tauri::events::LogSenderState;
use desktop::{
    desktop_pick_archive_file, desktop_pick_folder, desktop_pick_image_file, desktop_pick_jar_file,
    desktop_pick_java_file, desktop_pick_save_file, desktop_pick_server_executable,
    desktop_pick_startup_file, apply_acrylic,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// 启动桌面应用。
pub fn run() {
    // 初始化 tracing 日志（在 Tauri 构建之前）
    observability::init();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .manage(OnlineTunnelEventForwarder::default())
        .manage(LogSenderState::new())
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
            //窗口亚克力透明效果（由desktop/window_effect提供）
            apply_acrylic,
            //服务器定时任务契约命令
            create_cron_task,
            delete_cron_task,
            list_cron_tasks,
            run_cron_task,
            set_cron_task_enabled,
            update_cron_task,
            //服务器类型目录契约命令
            catalog_details,
            catalog_server_types,
            catalog_versions,
            //服务器控制台日志契约命令
            get_server_logs,
            //系统资源能力（由adapter/tauri/commands接入application）
            get_default_run_path,
            get_server_resource_usage,
            get_system_snapshot,
            //日志分享能力（上传到 mclo.gs）
            share_logs,
            //实例与服务器进程服务
            create_instance,
            delete_instance,
            get_instance,
            list_instances,
            rename_instance,
            update_instance_path,
            //Java 运行时检测与校验
            java_detect,
            java_validate,
            //在线隧道（联机）契约命令
            online_tunnel_host,
            online_tunnel_join,
            online_tunnel_status,
            online_tunnel_stop,
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
            //应用更新下载与安装契约命令
            update_clear_pending,
            update_download,
            update_install,
            update_pending,
            //插件 v2 宿主能力与策略管理
            plugin_v2_approve_session,
            plugin_v2_audit,
            plugin_v2_disable,
            plugin_v2_discover,
            plugin_v2_enable,
            plugin_v2_end_session,
            plugin_v2_grant_persistent,
            plugin_v2_grant_session,
            plugin_v2_invoke,
            plugin_v2_issue_approval_token,
            plugin_v2_load,
            plugin_v2_plugins,
            plugin_v2_revoke_persistent,
            plugin_v2_set_trust,
            plugin_v2_unload
        ])
        .setup(setup)
        .build(tauri::generate_context!())
        .expect("error while building Sea Lantern");

    // 全局退出钩子：覆盖窗口销毁、`app.exit`、操作系统关闭等所有退出路径，
    // 保证异步服务（日志转发、在线隧道）在进程退出前统一清理。
    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            on_shutdown(app_handle.clone());
        }
    });
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    //这里提供app_handle，便于使用时直接clone
    let app_handle = app.handle().clone();

    tauri::async_runtime::block_on(async {
        let services = AppServices::get().await.map_err(|error| {
            std::io::Error::other(format!("failed to assemble application services: {error}"))
        })?;
        if let Err(error) = services.initialize_network_settings().await {
            // 网络设置同步失败不阻止启动：网络运行时保持默认直连，
            // 系统代理恢复后由轮询与后续设置操作的重试自动跟上。
            tracing::error!(
                error = %error,
                "failed to initialize persisted network settings; continuing with direct network"
            );
        }
        Ok::<(), std::io::Error>(())
    })?;

    // 前端提供自定义标题栏；macOS 仍使用 Overlay 承载系统交通灯。
    #[cfg(not(target_os = "macos"))]
    if let Some(window) = app.get_webview_window("main") {
        window.set_decorations(false)?;
    }

    let handle_for_server_log = app_handle.clone();
    let log_sender: tauri::State<'_, LogSenderState> = app_handle.state();
    tauri::async_runtime::block_on(async { log_sender.start(handle_for_server_log).await });

    Ok(())
}

/// 应用退出时关闭后台异步服务（当前为在线隧道和日志提交器）。
fn on_shutdown(app_handle: AppHandle) {
    let log_sender: tauri::State<'_, LogSenderState> = app_handle.state();
    tauri::async_runtime::block_on(async { log_sender.stop().await });

    let Some(services) = AppServices::try_get() else {
        return;
    };

    tauri::async_runtime::block_on(async move {
        if let Err(error) = services.online_tunnel().shutdown().await {
            tracing::error!(
                target: "sealantern.tauri.online_tunnel",
                error = %error,
                "failed to shut down online tunnel"
            );
        }
    });
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
        "get_default_run_path",
        "get_server_resource_usage",
        "get_system_snapshot",
        "create_instance",
        "delete_instance",
        "get_instance",
        "list_instances",
        "rename_instance",
        "update_instance_path",
        "online_tunnel_host",
        "online_tunnel_join",
        "online_tunnel_status",
        "online_tunnel_stop",
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
