mod commands;
mod models;
mod plugins;
mod services;
mod utils;

use commands::config as config_commands;
use commands::downloader as download_commands;
use commands::java as java_commands;
use commands::mcs_plugin as mcs_plugin_commands;
use commands::player as player_commands;
use commands::plugin as plugin_commands;
use commands::server as server_commands;
use commands::settings as settings_commands;
use commands::system as system_commands;
use commands::update as update_commands;

use crate::services::download_manager::DownloadManager;
use plugins::manager::PluginManager;

use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Listener, Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    // Fix white screen issue on Wayland desktop environments (tested on Arch Linux + KDE Plasma)
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    #[cfg(target_os = "linux")]
    {
        let _handle = tokio::spawn(async {
            services::panic_report::panic_report().await;
            println!("panic_report 注册完成");
        });
    }

    let download_manager = DownloadManager::new();

    tauri::Builder::default()
        .manage(download_manager)
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            if let Some(window) = app.get_webview_window("main") {
                // 先显示窗口（处理隐藏状态）
                let _ = window.show();
                // 恢复窗口（处理最小化状态）
                let _ = window.unminimize();
                // 设置焦点
                let _ = window.set_focus();
            }
            print!("Received second instance with args: {:?}, cwd: {:?}", args, cwd);
        }))
        .on_tray_icon_event(|app, event| {
            if let TrayIconEvent::Click { button, button_state, .. } = event {
                // 只处理鼠标释放事件，确保只触发一次
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    // 左键点击切换主界面显示/隐藏
                    if let Some(window) = app.get_webview_window("main") {
                        // 先尝试获取窗口可见性状态
                        match window.is_visible() {
                            Ok(is_visible) => {
                                if is_visible {
                                    // 如果窗口可见，则隐藏它
                                    let _ = window.hide();
                                } else {
                                    // 如果窗口隐藏，则显示它
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            Err(_) => {
                                // 如果获取状态失败，默认显示窗口
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            server_commands::create_server,
            server_commands::import_server,
            server_commands::add_existing_server,
            server_commands::import_modpack,
            server_commands::parse_server_core_type,
            server_commands::scan_startup_candidates,
            server_commands::collect_copy_conflicts,
            server_commands::copy_directory_contents,
            server_commands::start_server,
            server_commands::stop_server,
            server_commands::send_command,
            server_commands::get_server_list,
            server_commands::get_server_status,
            server_commands::delete_server,
            server_commands::get_server_logs,
            server_commands::update_server_name,
            java_commands::detect_java,
            java_commands::validate_java_path,
            java_commands::install_java,
            java_commands::cancel_java_install,
            config_commands::read_config,
            config_commands::write_config,
            config_commands::read_server_properties,
            config_commands::write_server_properties,
            system_commands::get_system_info,
            system_commands::pick_jar_file,
            system_commands::pick_archive_file,
            system_commands::pick_startup_file,
            system_commands::pick_server_executable,
            system_commands::pick_java_file,
            system_commands::pick_save_file,
            system_commands::pick_folder,
            system_commands::pick_image_file,
            system_commands::open_file,
            system_commands::open_folder,
            system_commands::get_default_run_path,
            system_commands::get_safe_mode_status,
            player_commands::get_whitelist,
            player_commands::get_banned_players,
            player_commands::get_ops,
            player_commands::add_to_whitelist,
            player_commands::remove_from_whitelist,
            player_commands::ban_player,
            player_commands::unban_player,
            player_commands::add_op,
            player_commands::remove_op,
            player_commands::kick_player,
            player_commands::export_logs,
            settings_commands::get_settings,
            settings_commands::save_settings,
            settings_commands::save_settings_with_diff,
            settings_commands::update_settings_partial,
            settings_commands::reset_settings,
            settings_commands::export_settings,
            settings_commands::import_settings,
            settings_commands::get_system_fonts,
            update_commands::check_update,
            update_commands::open_download_url,
            update_commands::download_update,
            update_commands::install_update,
            update_commands::check_pending_update,
            update_commands::clear_pending_update,
            update_commands::restart_and_install,
            update_commands::download_update_from_debug_url,
            download_commands::download_file,
            download_commands::poll_task,
            download_commands::poll_all_downloads,
            download_commands::remove_download_task,
            plugin_commands::list_plugins,
            plugin_commands::scan_plugins,
            plugin_commands::enable_plugin,
            plugin_commands::disable_plugin,
            plugin_commands::get_plugin_nav_items,
            plugin_commands::install_plugin,
            plugin_commands::get_plugin_icon,
            plugin_commands::get_plugin_settings,
            plugin_commands::set_plugin_settings,
            plugin_commands::get_plugin_css,
            plugin_commands::get_all_plugin_css,
            plugin_commands::delete_plugin,
            plugin_commands::delete_plugins,
            plugin_commands::check_plugin_update,
            plugin_commands::check_all_plugin_updates,
            plugin_commands::fetch_market_plugins,
            plugin_commands::fetch_market_categories,
            plugin_commands::fetch_market_plugin_detail,
            plugin_commands::install_from_market,
            plugin_commands::install_plugins_batch,
            plugin_commands::context_menu_callback,
            plugin_commands::context_menu_show_notify,
            plugin_commands::context_menu_hide_notify,
            plugin_commands::on_locale_changed,
            plugin_commands::component_mirror_register,
            plugin_commands::component_mirror_unregister,
            plugin_commands::component_mirror_clear,
            plugin_commands::on_page_changed,
            plugin_commands::get_plugin_component_snapshot,
            plugin_commands::get_plugin_ui_snapshot,
            plugin_commands::get_plugin_sidebar_snapshot,
            plugin_commands::get_plugin_context_menu_snapshot,
            mcs_plugin_commands::m_get_plugins,
            mcs_plugin_commands::m_toggle_plugin,
            mcs_plugin_commands::m_delete_plugin,
            mcs_plugin_commands::m_install_plugin,
            mcs_plugin_commands::m_get_plugin_config_files
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let settings = services::global::settings_manager().get();

                match settings.close_action.as_str() {
                    "minimize" => {
                        // 最小化到托盘
                        api.prevent_close();
                        let _ = window.hide();
                    }
                    "close" => {
                        // 直接关闭
                        if settings.close_servers_on_exit {
                            services::global::server_manager().stop_all_servers();
                        }
                        // 关闭时禁用插件
                        if let Some(manager) =
                            window.app_handle().try_state::<std::sync::Arc<
                                std::sync::Mutex<crate::plugins::manager::PluginManager>,
                            >>()
                        {
                            if let Ok(mut m) = manager.lock() {
                                m.disable_all_plugins_for_shutdown();
                            }
                        }
                    }
                    _ => {
                        // 显示对话框（ask 或其他值）
                        api.prevent_close();
                        let _ = window.emit("close-requested", ());
                    }
                }
            }
        })
        .setup(|app| {
            // 初始化插件管理
            // 插件目录与其他模块共用同一套数据目录选择规则
            let app_data_dir = crate::utils::path::get_app_data_dir();
            let plugins_dir = app_data_dir.join("plugins");
            let data_dir = app_data_dir.join("plugin_data");

            let mut plugin_manager = PluginManager::new(plugins_dir, data_dir);

            // Check for safe mode
            let safe_mode = std::env::args().any(|arg| arg == "--safe-mode");

            // Always scan plugins to load the list, even in safe mode
            if let Err(e) = plugin_manager.scan_plugins() {
                eprintln!("Failed to scan plugins: {}", e);
            }

            if safe_mode {
                eprintln!("Safe mode enabled: plugins will be disabled");
                // In safe mode, don't enable any plugins
            } else {
                // 自动启用上启用的插件
                plugin_manager.auto_enable_plugins();
            }

            let shared_runtimes = plugin_manager.get_shared_runtimes();
            let shared_runtimes_for_server_ready = Arc::clone(&shared_runtimes);
            let api_registry = plugin_manager.get_api_registry();

            let manager = Arc::new(Mutex::new(plugin_manager));

            plugins::api::set_api_call_handler(Arc::new(move |_source, target, api_name, args| {
                use crate::plugins::api::ApiRegistryOps;

                // 检查api是否存在
                let lua_fn_name = api_registry
                    .get_api_fn_name(target, api_name)
                    .ok_or_else(|| format!("插件 '{}' 没有注册 API '{}'", target, api_name))?;

                // 获取目标插件的runtime
                let runtimes = shared_runtimes.read().unwrap_or_else(|e| e.into_inner());
                let runtime = runtimes
                    .get(target)
                    .ok_or_else(|| format!("插件 '{}' 的运行时不存在", target))?;
                runtime.call_registered_api(&lua_fn_name, args)
            }));

            let app_handle = app.handle().clone();
            plugins::api::set_ui_event_handler(Arc::new(
                move |plugin_id, action, element_id, html| {
                    use serde::Serialize;

                    #[derive(Serialize, Clone)]
                    struct PluginUiEvent {
                        plugin_id: String,
                        action: String,
                        element_id: String,
                        html: String,
                    }

                    let event = PluginUiEvent {
                        plugin_id: plugin_id.to_string(),
                        action: action.to_string(),
                        element_id: element_id.to_string(),
                        html: html.to_string(),
                    };

                    app_handle
                        .emit("plugin-ui-event", event)
                        .map_err(|e| format!("Failed to emit UI event: {}", e))
                },
            ));

            let app_handle = app.handle().clone();
            plugins::api::set_log_event_handler(Arc::new(move |plugin_id, level, message| {
                use serde::Serialize;

                #[derive(Serialize, Clone)]
                struct PluginLogEvent {
                    plugin_id: String,
                    level: String,
                    message: String,
                }

                let event = PluginLogEvent {
                    plugin_id: plugin_id.to_string(),
                    level: level.to_string(),
                    message: message.to_string(),
                };

                app_handle
                    .emit("plugin-log-event", event)
                    .map_err(|e| format!("Failed to emit log event: {}", e))
            }));

            let app_handle = app.handle().clone();
            plugins::api::set_context_menu_handler(Arc::new(
                move |plugin_id, action, context, items_json| {
                    use serde::Serialize;

                    #[derive(Serialize, Clone)]
                    struct PluginContextMenuEvent {
                        plugin_id: String,
                        action: String,
                        context: String,
                        items: String,
                    }

                    let event = PluginContextMenuEvent {
                        plugin_id: plugin_id.to_string(),
                        action: action.to_string(),
                        context: context.to_string(),
                        items: items_json.to_string(),
                    };

                    app_handle
                        .emit("plugin-context-menu-event", event)
                        .map_err(|e| format!("Failed to emit context menu event: {}", e))
                },
            ));

            let app_handle = app.handle().clone();
            plugins::api::set_sidebar_event_handler(Arc::new(
                move |plugin_id, action, label, icon| {
                    use serde::Serialize;

                    #[derive(Serialize, Clone)]
                    struct PluginSidebarEvent {
                        plugin_id: String,
                        action: String,
                        label: String,
                        icon: String,
                    }

                    let event = PluginSidebarEvent {
                        plugin_id: plugin_id.to_string(),
                        action: action.to_string(),
                        label: label.to_string(),
                        icon: icon.to_string(),
                    };

                    app_handle
                        .emit("plugin-sidebar-event", event)
                        .map_err(|e| format!("Failed to emit sidebar event: {}", e))
                },
            ));

            let app_handle = app.handle().clone();
            plugins::api::set_permission_log_handler(Arc::new(
                move |plugin_id, log_type, action, detail, timestamp| {
                    use serde::Serialize;

                    #[derive(Serialize, Clone)]
                    struct PluginPermissionLog {
                        plugin_id: String,
                        log_type: String,
                        action: String,
                        detail: String,
                        timestamp: u64,
                    }

                    let event = PluginPermissionLog {
                        plugin_id: plugin_id.to_string(),
                        log_type: log_type.to_string(),
                        action: action.to_string(),
                        detail: detail.to_string(),
                        timestamp,
                    };

                    app_handle
                        .emit("plugin-permission-log", event)
                        .map_err(|e| format!("Failed to emit permission log: {}", e))
                },
            ));

            let app_handle = app.handle().clone();
            plugins::api::set_component_event_handler(Arc::new(move |_plugin_id, payload_json| {
                let val: serde_json::Value =
                    serde_json::from_str(payload_json).unwrap_or(serde_json::Value::Null);
                app_handle
                    .emit("plugin:ui:component", val)
                    .map_err(|e| format!("Failed to emit component event: {}", e))
            }));

            let app_handle = app.handle().clone();
            plugins::api::set_i18n_event_handler(Arc::new(
                move |plugin_id, action, locale, payload| {
                    use serde::Serialize;

                    #[derive(Serialize, Clone)]
                    struct PluginI18nEvent {
                        plugin_id: String,
                        action: String,
                        locale: String,
                        payload: String,
                    }

                    let event = PluginI18nEvent {
                        plugin_id: plugin_id.to_string(),
                        action: action.to_string(),
                        locale: locale.to_string(),
                        payload: payload.to_string(),
                    };

                    app_handle
                        .emit("plugin-i18n-event", event)
                        .map_err(|e| format!("Failed to emit i18n event: {}", e))
                },
            ));

            {
                plugins::api::set_server_ready_handler(Arc::new(move |server_id| {
                    let shared_runtimes = &shared_runtimes_for_server_ready;
                    let runtimes = shared_runtimes.read().unwrap_or_else(|e| e.into_inner());
                    for (plugin_id, runtime) in runtimes.iter() {
                        if let Err(e) = runtime.call_lifecycle_with_arg("onServerReady", server_id)
                        {
                            eprintln!("[WARN] plugin '{}' onServerReady failed: {}", plugin_id, e);
                        }
                    }
                    Ok(())
                }));
            }

            {
                let app_handle = app.handle().clone();
                app_handle.listen("plugin-element-response", |event| {
                    eprintln!("[Element] Received response event");
                    if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload())
                    {
                        if let (Some(request_id), Some(data)) = (
                            payload.get("request_id").and_then(|v| v.as_u64()),
                            payload.get("data").and_then(|v| v.as_str()),
                        ) {
                            plugins::api::element_response_resolve(request_id, data.to_string());
                        }
                    }
                });
            }

            {
                use serde::Serialize;

                #[derive(Serialize, Clone)]
                struct ServerLogLineEvent {
                    server_id: String,
                    line: String,
                }

                let app_handle = app.handle().clone();
                let _ = services::server_log_pipeline::set_server_log_event_handler(Arc::new(
                    move |server_id, line| {
                        let event = ServerLogLineEvent {
                            server_id: server_id.to_string(),
                            line: line.to_string(),
                        };
                        app_handle
                            .emit("server-log-line", event)
                            .map_err(|e| format!("Failed to emit server log line event: {}", e))
                    },
                ));
            }

            app.manage(manager);

            // Check if currently in safe mode
            let safe_mode = std::env::args().any(|arg| arg == "--safe-mode");

            let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = if safe_mode {
                Menu::with_items(app, &[&show_item, &quit_item])?
            } else {
                let safe_mode_item = MenuItem::with_id(
                    app,
                    "restart-safe-mode",
                    "以安全模式重启",
                    true,
                    None::<&str>,
                )?;
                Menu::with_items(app, &[&show_item, &safe_mode_item, &quit_item])?
            };

            let icon_bytes = include_bytes!("../icons/icon.png");
            let img = image::load_from_memory(icon_bytes)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
                .into_rgba8();
            let (width, height) = img.dimensions();
            let icon = tauri::image::Image::new_owned(img.into_raw(), width, height);

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("Sea Lantern")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            // 先显示窗口（处理隐藏状态）
                            let _ = window.show();
                            // 恢复窗口（处理最小化状态）
                            let _ = window.unminimize();
                            // 设置焦点
                            let _ = window.set_focus();
                        }
                    }
                    "restart-safe-mode" => {
                        // Restart in safe mode
                        let settings = services::global::settings_manager().get();
                        if settings.close_servers_on_exit {
                            services::global::server_manager().stop_all_servers();
                        }
                        if let Some(manager) =
                            app.try_state::<std::sync::Arc<
                                std::sync::Mutex<crate::plugins::manager::PluginManager>,
                            >>()
                        {
                            if let Ok(mut m) = manager.lock() {
                                m.disable_all_plugins_for_shutdown();
                            }
                        }

                        // Restart with --safe-mode flag
                        let default_name = if cfg!(windows) {
                            "SeaLantern.exe"
                        } else {
                            "SeaLantern"
                        };
                        let app_path = std::env::current_exe()
                            .or_else(|_| {
                                std::env::args().next().map(std::path::PathBuf::from).ok_or(
                                    std::io::Error::new(
                                        std::io::ErrorKind::NotFound,
                                        "Executable path not found",
                                    ),
                                )
                            })
                            .unwrap_or_else(|_| std::path::PathBuf::from(default_name));

                        // Handle platform-specific restart logic
                        #[cfg(target_os = "macos")]
                        {
                            // On macOS, try to find the .app bundle and use open command
                            if let Some(app_bundle_path) = app_path
                                .ancestors()
                                .find(|p| p.extension().map_or(false, |ext| ext == "app"))
                            {
                                match std::process::Command::new("open")
                                        .arg("-n") // Open a new instance
                                        .arg(app_bundle_path)
                                        .arg("--args")
                                        .arg("--safe-mode")
                                        .spawn()
                                {
                                    Ok(_) => app.exit(0),
                                    Err(e) => {
                                        eprintln!(
                                            "Failed to restart in safe mode using open command: {}",
                                            e
                                        );
                                        // Fallback to direct execution if open command fails
                                        match std::process::Command::new(app_path)
                                            .arg("--safe-mode")
                                            .spawn()
                                        {
                                            Ok(_) => app.exit(0),
                                            Err(e) => {
                                                eprintln!("Failed to restart in safe mode: {}", e);
                                                app.exit(1);
                                            }
                                        }
                                    }
                                }
                            } else {
                                // If not in an app bundle, use direct execution
                                match std::process::Command::new(app_path)
                                    .arg("--safe-mode")
                                    .spawn()
                                {
                                    Ok(_) => app.exit(0),
                                    Err(e) => {
                                        eprintln!("Failed to restart in safe mode: {}", e);
                                        app.exit(1);
                                    }
                                }
                            }
                        }

                        #[cfg(target_os = "linux")]
                        {
                            // On Linux, check execute permissions
                            use std::fs::Permissions;
                            use std::os::unix::fs::PermissionsExt;

                            if let Ok(metadata) = app_path.metadata() {
                                let perms = metadata.permissions();
                                if (perms.mode() & 0o111) == 0 {
                                    // Try to add execute permissions
                                    if let Ok(()) = std::fs::set_permissions(
                                        &app_path,
                                        Permissions::from_mode(perms.mode() | 0o111),
                                    ) {
                                        eprintln!(
                                            "Added execute permissions to {}",
                                            app_path.display()
                                        );
                                    } else {
                                        eprintln!(
                                            "Warning: No execute permissions on {}",
                                            app_path.display()
                                        );
                                    }
                                }
                            }

                            match std::process::Command::new(app_path)
                                .arg("--safe-mode")
                                .spawn()
                            {
                                Ok(_) => app.exit(0),
                                Err(e) => {
                                    eprintln!("Failed to restart in safe mode: {}", e);
                                    app.exit(1);
                                }
                            }
                        }

                        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
                        {
                            // For Windows and other platforms
                            match std::process::Command::new(app_path)
                                .arg("--safe-mode")
                                .spawn()
                            {
                                Ok(_) => app.exit(0),
                                Err(e) => {
                                    eprintln!("Failed to restart in safe mode: {}", e);
                                    app.exit(1);
                                }
                            }
                        }
                    }
                    "quit" => {
                        let settings = services::global::settings_manager().get();
                        if settings.close_servers_on_exit {
                            services::global::server_manager().stop_all_servers();
                        }
                        if let Some(manager) =
                            app.try_state::<std::sync::Arc<
                                std::sync::Mutex<crate::plugins::manager::PluginManager>,
                            >>()
                        {
                            if let Ok(mut m) = manager.lock() {
                                m.disable_all_plugins_for_shutdown();
                            }
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            // 先显示窗口（处理隐藏状态）
                            let _ = window.show();
                            // 恢复窗口（处理最小化状态）
                            let _ = window.unminimize();
                            // 设置焦点
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Sea Lantern");
}
