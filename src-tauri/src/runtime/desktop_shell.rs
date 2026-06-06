use crate::services;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager,
};

fn reveal_main_window<R: tauri::Runtime, M: Manager<R>>(manager: &M) {
    if let Some(window) = manager.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

pub(crate) fn stop_servers_and_disable_plugins(app: &tauri::AppHandle) {
    let settings = services::global::settings_manager().get();
    if settings.close_servers_on_exit {
        services::global::server_manager().stop_all_servers();
    }

    if let Some(manager) =
        app.try_state::<std::sync::Arc<std::sync::Mutex<crate::plugins::manager::PluginManager>>>()
    {
        if let Ok(mut m) = manager.lock() {
            m.disable_all_plugins_for_shutdown();
        }
    }
}

pub(crate) fn handle_single_instance(
    app: &tauri::AppHandle,
    args: Vec<String>,
    cwd: std::path::PathBuf,
) {
    reveal_main_window(app);
    print!("Received second instance with args: {:?}, cwd: {:?}", args, cwd);
}

pub(crate) fn handle_builder_tray_click(app: &tauri::AppHandle, event: &TrayIconEvent) {
    if let TrayIconEvent::Click { button, button_state, .. } = event {
        if *button == MouseButton::Left && *button_state == MouseButtonState::Up {
            if let Some(window) = app.get_webview_window("main") {
                match window.is_visible() {
                    Ok(is_visible) => {
                        if is_visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    Err(_) => {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        }
    }
}

pub(crate) fn handle_close_requested(window: &tauri::Window, api: &tauri::CloseRequestApi) {
    let settings = services::global::settings_manager().get();

    match settings.close_action.as_str() {
        "minimize" => {
            api.prevent_close();
            let _ = window.hide();
        }
        "close" => {
            stop_servers_and_disable_plugins(window.app_handle());
            window.app_handle().exit(0);
        }
        _ => {
            api.prevent_close();
            let _ = window.emit("close-requested", ());
        }
    }
}

use crate::hardcode_data::app_files::{APP_DIRECTORY_NAME, APP_EXECUTABLE_NAME_WINDOWS};

fn restart_in_safe_mode(app: &tauri::AppHandle) {
    stop_servers_and_disable_plugins(app);

    let default_name = if cfg!(windows) {
        APP_EXECUTABLE_NAME_WINDOWS
    } else {
        APP_DIRECTORY_NAME
    };
    let app_path = std::env::current_exe()
        .or_else(|_| {
            std::env::args()
                .next()
                .map(std::path::PathBuf::from)
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Executable path not found",
                ))
        })
        .unwrap_or_else(|_| std::path::PathBuf::from(default_name));

    #[cfg(target_os = "macos")]
    {
        if let Some(app_bundle_path) = app_path
            .ancestors()
            .find(|p| p.extension().is_some_and(|ext| ext == "app"))
        {
            match std::process::Command::new("open")
                .arg("-n")
                .arg(app_bundle_path)
                .arg("--args")
                .arg("--safe-mode")
                .spawn()
            {
                Ok(_) => app.exit(0),
                Err(e) => {
                    eprintln!("Failed to restart in safe mode using open command: {}", e);
                    match std::process::Command::new(&app_path)
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
            match std::process::Command::new(&app_path)
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
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;

        if let Ok(metadata) = app_path.metadata() {
            let perms = metadata.permissions();
            if (perms.mode() & 0o111) == 0 {
                if let Ok(()) = std::fs::set_permissions(
                    &app_path,
                    Permissions::from_mode(perms.mode() | 0o111),
                ) {
                    eprintln!("Added execute permissions to {}", app_path.display());
                } else {
                    eprintln!("Warning: No execute permissions on {}", app_path.display());
                }
            }
        }

        match std::process::Command::new(&app_path)
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
        match std::process::Command::new(&app_path)
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

pub(crate) fn setup_tray<M: Manager<tauri::Wry>>(app: &M) -> tauri::Result<()> {
    let safe_mode = std::env::args().any(|arg| arg == "--safe-mode");

    let show_item = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = if safe_mode {
        Menu::with_items(app, &[&show_item, &quit_item])?
    } else {
        let safe_mode_item =
            MenuItem::with_id(app, "restart-safe-mode", "以安全模式重启", true, None::<&str>)?;
        Menu::with_items(app, &[&show_item, &safe_mode_item, &quit_item])?
    };

    let icon_bytes = include_bytes!("../../icons/icon.png");
    let img = image::load_from_memory(icon_bytes)
        .map_err(|e| tauri::Error::from(std::io::Error::other(e.to_string())))?
        .into_rgba8();
    let (width, height) = img.dimensions();
    let icon = tauri::image::Image::new_owned(img.into_raw(), width, height);

    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("Sea Lantern")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => reveal_main_window(app),
            "restart-safe-mode" => restart_in_safe_mode(app),
            "quit" => {
                stop_servers_and_disable_plugins(app);
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
                reveal_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}
