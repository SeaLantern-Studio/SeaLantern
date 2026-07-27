//! Sea Lantern Desktop - Tauri 应用入口

mod commands;
mod models;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化设置（从文件加载）
    let initial_settings = commands::load_settings_from_file();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(commands::SettingsState {
            settings: std::sync::Mutex::new(initial_settings),
        })
        .manage(commands::ServerState::default())
        .manage(commands::JavaState::default())
        .manage(commands::PluginState::default())
        .invoke_handler(tauri::generate_handler![
            // Settings commands
            get_settings,
            save_settings,
            save_settings_with_diff,
            update_settings_partial,
            reset_settings,
            export_settings,
            import_settings,
            // Server commands
            get_server_list,
            get_server_status,
            start_server,
            stop_server,
            delete_server,
            update_server_name,
            // Java commands
            detect_java,
            validate_java_path,
            install_java,
            cancel_java_install,
            // System commands
            get_system_info,
            pick_jar_file,
            pick_archive_file,
            pick_java_file,
            pick_save_file,
            get_system_fonts,
            check_developer_mode,
            frontend_heartbeat,
            // Config commands
            read_server_properties,
            write_server_properties,
            read_server_properties_source,
            write_server_properties_source,
            parse_server_properties_source,
            preview_server_properties_write,
            preview_server_properties_write_from_source,
            read_config,
            write_config,
            read_sl_config,
            write_sl_config,
            // Plugin commands
            list_plugins,
            scan_plugins,
            enable_plugin,
            disable_plugin,
            get_plugin_nav_items,
            install_plugin,
            install_plugins_batch,
            get_plugin_icon,
            get_plugin_settings,
            set_plugin_settings,
            get_plugin_css,
            get_all_plugin_css,
            delete_plugin,
            delete_plugins,
            check_plugin_update,
            check_all_plugin_updates,
            fetch_market_plugins,
            fetch_market_plugin_detail,
            fetch_market_categories,
            install_from_market,
            on_locale_changed,
            on_page_changed,
            component_mirror_clear,
            component_mirror_register,
            component_mirror_unregister,
            context_menu_show_notify,
            context_menu_hide_notify,
            context_menu_callback,
            get_plugin_ui_snapshot,
            get_plugin_sidebar_snapshot,
            get_plugin_context_menu_snapshot,
            get_plugin_component_snapshot,
            get_plugin_permission_logs,
            // Greeting (保留用于测试)
            greet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Sea Lantern.", name)
}