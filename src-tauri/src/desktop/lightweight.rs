//! 轻量模式会销毁主 WebView，同时保留进程级后台服务。

#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use tauri::{AppHandle, Manager, WebviewWindowBuilder};
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};

use super::theme::{self, DesktopAppearanceState};
use super::window_state::{
    self as window_lifecycle, MainWindowMode, MainWindowTransition, MAIN_WINDOW_LABEL,
};

fn window_state_flags() -> StateFlags {
    StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED
}

pub fn leave(app: &AppHandle, transition: &mut MainWindowTransition<'_>) -> Result<(), String> {
    if transition.mode() != MainWindowMode::Background {
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    app.set_activation_policy(ActivationPolicy::Regular)
        .map_err(|error| error.to_string())?;

    let window = if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window
    } else {
        let config = app
            .config()
            .app
            .windows
            .iter()
            .find(|config| config.label == MAIN_WINDOW_LABEL)
            .ok_or_else(|| "main window configuration is unavailable".to_owned())?;
        WebviewWindowBuilder::from_config(app, config)
            .map_err(|error| error.to_string())?
            .visible(false)
            .build()
            .map_err(|error| error.to_string())?
    };

    // 状态机要求按 Background -> Hidden -> Visible 的顺序恢复。
    window.hide().map_err(|error| error.to_string())?;
    let _ = window.restore_state(window_state_flags());

    let appearance = app.state::<DesktopAppearanceState>().snapshot();
    if let Err(error) = theme::apply_material(app, &appearance.material, &appearance.theme) {
        tracing::error!(%error, "failed to restore native material after lightweight mode");
    }
    #[cfg(target_os = "windows")]
    window
        .set_skip_taskbar(false)
        .map_err(|error| error.to_string())?;
    transition.move_to(MainWindowMode::Hidden)?;
    window_lifecycle::show(app, transition)
}

pub fn enter(app: &AppHandle, transition: &mut MainWindowTransition<'_>) -> Result<(), String> {
    if transition.mode() == MainWindowMode::Background {
        return Ok(());
    }

    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = app.save_window_state(window_state_flags());
        #[cfg(target_os = "windows")]
        window
            .set_skip_taskbar(true)
            .map_err(|error| error.to_string())?;
        window.destroy().map_err(|error| error.to_string())?;
    }

    transition.move_to(MainWindowMode::Background)?;
    #[cfg(target_os = "macos")]
    app.set_activation_policy(ActivationPolicy::Accessory)
        .map_err(|error| error.to_string())?;
    Ok(())
}
