mod commands;
mod models;
mod services;
mod utils;

use commands::config as config_commands;
use commands::java as java_commands;
use commands::player as player_commands;
use commands::server as server_commands;
use commands::settings as settings_commands;
use commands::system as system_commands;
use commands::update as update_commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        .invoke_handler(tauri::generate_handler![
            server_commands::create_server,
            server_commands::import_server,
            server_commands::import_modpack,
            server_commands::start_server,
            server_commands::stop_server,
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
