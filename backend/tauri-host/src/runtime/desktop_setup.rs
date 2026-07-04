use crate::plugins::manager::PluginManager;
use crate::runtime::desktop_shell;
use crate::services::global;

use std::sync::{Arc, Mutex};

#[cfg(any(feature = "plugin-local-runtime", feature = "plugin-runtime-bridge"))]
use crate::plugins::api::PluginApiRegistry;
#[cfg(any(feature = "plugin-local-runtime", feature = "plugin-runtime-bridge"))]
use crate::plugins::runtime::PluginRuntime;
#[cfg(any(feature = "plugin-local-runtime", feature = "plugin-runtime-bridge"))]
use std::collections::HashMap;
#[cfg(any(feature = "plugin-local-runtime", feature = "plugin-runtime-bridge"))]
use std::sync::RwLock;
use tauri::Manager;
#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;

#[cfg(any(feature = "plugin-local-runtime", feature = "plugin-runtime-bridge"))]
pub(crate) struct PluginRuntimeSetup {
    pub shared_runtimes: Arc<RwLock<HashMap<String, PluginRuntime>>>,
    pub api_registry: PluginApiRegistry,
}

pub(crate) struct PluginSetup {
    pub manager: Arc<Mutex<PluginManager>>,
    #[cfg(any(feature = "plugin-local-runtime", feature = "plugin-runtime-bridge"))]
    pub runtime: PluginRuntimeSetup,
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

        let settings = crate::services::global::settings_manager().get();
        let theme_pref = match settings.theme.as_str() {
            "dark" => Some(true),
            "light" => Some(false),
            _ => None,
        };

        let native_effect_result = crate::commands::app::settings::sync_native_window_effect(
            &window,
            &settings.window_effect,
            theme_pref,
        );

        if let Err(e) = native_effect_result {
            eprintln!("Failed to sync native macOS vibrancy effect: {}", e);
        }
    }

    #[cfg(target_os = "windows")]
    if let Some(window) = app.get_webview_window("main") {
        let settings = crate::services::global::settings_manager().get();
        let theme_pref = match settings.theme.as_str() {
            "dark" => Some(true),
            "light" => Some(false),
            _ => None,
        };

        // Windows native blur/acrylic/mica flickers while dragging a transparent
        // window with no app background. Keep the startup entry disabled for now
        // and let the shared sync path clear effects and restore a solid fallback.
        if let Err(e) = crate::commands::app::settings::sync_native_window_effect(
            &window,
            crate::models::settings::WINDOW_EFFECT_OFF,
            theme_pref,
        ) {
            eprintln!("Failed to apply fallback Windows window effect: {}", e);
        }
    }
}

#[cfg(feature = "plugin-local-runtime")]
fn maybe_auto_enable_plugins(manager: &Arc<Mutex<PluginManager>>) {
    if is_safe_mode() {
        eprintln!("Safe mode enabled: plugins will be disabled");
        return;
    }

    let manager_for_auto_enable = Arc::clone(manager);
    tauri::async_runtime::spawn(async move {
        let result = tauri::async_runtime::spawn_blocking(move || {
            let mut plugin_manager = manager_for_auto_enable
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            plugin_manager.auto_enable_plugins_checked()
        })
        .await;

        match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                eprintln!("[WARN] Failed to auto-enable plugins: {}", error);
            }
            Err(error) => {
                eprintln!("[WARN] Failed to auto-enable plugins in background: {}", error);
            }
        }
    });
}

#[cfg(not(feature = "plugin-local-runtime"))]
fn maybe_auto_enable_plugins(_manager: &Arc<Mutex<PluginManager>>) {
    if is_safe_mode() {
        eprintln!("Safe mode enabled: plugins will be disabled");
    } else {
        eprintln!("Plugin runtime disabled in this build: metadata-only desktop startup");
    }
}

/// 初始化插件管理和可选运行时共享状态。
pub(crate) fn initialize_plugins() -> PluginSetup {
    let manager = Arc::clone(global::plugin_manager());

    maybe_auto_enable_plugins(&manager);

    #[cfg(any(feature = "plugin-local-runtime", feature = "plugin-runtime-bridge"))]
    let runtime = {
        let plugin_manager = manager.lock().unwrap_or_else(|e| e.into_inner());
        PluginRuntimeSetup {
            shared_runtimes: plugin_manager.get_shared_runtimes(),
            api_registry: plugin_manager.get_api_registry(),
        }
    };

    PluginSetup {
        manager,
        #[cfg(any(feature = "plugin-local-runtime", feature = "plugin-runtime-bridge"))]
        runtime,
    }
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
