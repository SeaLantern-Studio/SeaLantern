mod commands;
mod models;
mod services;
mod utils;

use commands::config as config_commands;
use commands::java as java_commands;
use commands::join as join_commands;
use commands::mods as mods_commands;
use commands::player as player_commands;
use commands::server as server_commands;
use commands::server_id as server_id_commands;
use commands::settings as settings_commands;
use commands::system as system_commands;
use commands::update as update_commands;
use tauri::{tray::MouseButton, tray::MouseButtonState, tray::TrayIconEvent, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Handle CLI mode
    utils::cli::handle_cli();

    // Fix white screen issue on Wayland desktop environments (tested on Arch Linux + KDE Plasma)
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
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
            server_commands::import_modpack,
            server_commands::start_server,
            server_commands::stop_server,
            server_commands::force_stop_all_servers,
            server_commands::send_command,
            server_commands::get_server_list,
            server_commands::get_server_status,
            server_commands::delete_server,
            server_commands::get_server_logs,
            server_commands::add_server_command,
            server_commands::update_server_command,
            server_commands::delete_server_command,
            server_commands::update_server_name,
            java_commands::detect_java,
            java_commands::validate_java_path,
            config_commands::read_config,
            config_commands::write_config,
            config_commands::read_server_properties,
            config_commands::write_server_properties,
            system_commands::get_system_info,
            system_commands::pick_jar_file,
            system_commands::pick_startup_file,
            system_commands::pick_java_file,
            system_commands::pick_folder,
            system_commands::pick_image_file,
            system_commands::open_folder,
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
            settings_commands::reset_settings,
            settings_commands::export_settings,
            settings_commands::import_settings,
            settings_commands::check_acrylic_support,
            settings_commands::apply_acrylic,
            settings_commands::get_system_fonts,
            update_commands::check_update,
            update_commands::open_download_url,
            update_commands::download_update,
            update_commands::install_update,
            update_commands::check_pending_update,
            update_commands::clear_pending_update,
            update_commands::restart_and_install,
            update_commands::download_update_from_debug_url,
            mods_commands::search_mods,
            mods_commands::install_mod,
            join_commands::resolve_join_server_id,
            join_commands::join_server_by_id,
            server_id_commands::create_server_id,
            server_id_commands::resolve_server_id,
            server_id_commands::get_server_id,
            server_id_commands::list_server_ids,
            server_id_commands::update_server_id,
            server_id_commands::deactivate_server_id,
            server_id_commands::delete_server_id,
            server_id_commands::search_server_ids,
        ])
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let settings = services::global::settings_manager().get();
                if settings.close_servers_on_exit {
                    services::global::server_manager().stop_all_servers();
                }
            }
        })
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running Sea Lantern");
}
