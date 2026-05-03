use crate::plugins::manager::{PluginManager, SharedRuntimes};
use crate::runtime::desktop_shell;

use std::sync::{Arc, Mutex};
use tauri::Manager;
#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;
#[cfg(target_os = "macos")]
use window_vibrancy::{
    apply_vibrancy, clear_vibrancy, NSVisualEffectMaterial, NSVisualEffectState,
};

pub(crate) struct PluginSetup {
    pub manager: Arc<Mutex<PluginManager>>,
    pub shared_runtimes: SharedRuntimes,
    pub api_registry: crate::plugins::api::ApiRegistry,
}

fn is_safe_mode() -> bool {
    std::env::args().any(|arg| arg == "--safe-mode")
}

/// 同步桌面窗口的原生样式。
pub(crate) fn apply_platform_window_style(app: &tauri::App) {
    #[cfg(not(target_os = "macos"))]
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.set_decorations(false) {
            eprintln!("Failed to disable native window decorations: {}", e);
        }
    }

    #[cfg(target_os = "macos")]
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.set_decorations(true) {
            eprintln!("Failed to enable native macOS window decorations: {}", e);
        }

        if let Err(e) = window.set_title_bar_style(TitleBarStyle::Overlay) {
            eprintln!("Failed to set macOS title bar style to overlay: {}", e);
        }

        let acrylic_enabled = crate::services::global::settings_manager()
            .get()
            .acrylic_enabled;

        let native_effect_result = if acrylic_enabled {
            apply_vibrancy(
                &window,
                NSVisualEffectMaterial::UnderWindowBackground,
                Some(NSVisualEffectState::Active),
                None,
            )
            .map(|_| ())
        } else {
            clear_vibrancy(&window).map(|_| ())
        };

        if let Err(e) = native_effect_result {
            eprintln!("Failed to sync native macOS vibrancy effect: {}", e);
        }
    }
}

/// 初始化插件管理和运行时共享状态。
pub(crate) fn initialize_plugins() -> PluginSetup {
    let app_data_dir = crate::utils::path::get_app_data_dir();
    let plugins_dir = app_data_dir.join("plugins");
    let data_dir = app_data_dir.join("plugin_data");

    let plugin_manager = PluginManager::new(plugins_dir, data_dir);
    let shared_runtimes = plugin_manager.get_shared_runtimes();
    let api_registry = plugin_manager.get_api_registry();
    let manager = Arc::new(Mutex::new(plugin_manager));

    {
        let mut plugin_manager = manager.lock().unwrap_or_else(|e| e.into_inner());

        if let Err(e) = plugin_manager.scan_plugins() {
            eprintln!("Failed to scan plugins: {}", e);
        }
    }

    if is_safe_mode() {
        eprintln!("Safe mode enabled: plugins will be disabled");
    } else {
        let manager_for_auto_enable = Arc::clone(&manager);
        tauri::async_runtime::spawn(async move {
            let result = tauri::async_runtime::spawn_blocking(move || {
                let mut plugin_manager = manager_for_auto_enable
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                plugin_manager.auto_enable_plugins();
            })
            .await;

            if let Err(e) = result {
                eprintln!("[WARN] Failed to auto-enable plugins in background: {}", e);
            }
        });
    }

    PluginSetup { manager, shared_runtimes, api_registry }
}

/// 启动前端心跳看门狗。
pub(crate) fn install_frontend_watchdog(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        use std::time::{Duration, SystemTime, UNIX_EPOCH};
        use tokio::time::sleep;

        loop {
            sleep(Duration::from_secs(5)).await;

            let last = crate::services::global::last_frontend_heartbeat();
            if last == 0 {
                continue;
            }

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if now.saturating_sub(last) > 30 {
                eprintln!("[Watchdog] frontend heartbeat lost, shutting down Sea Lantern");
                desktop_shell::stop_servers_and_disable_plugins(&app_handle);
                app_handle.exit(0);
                break;
            }
        }
    });
}
