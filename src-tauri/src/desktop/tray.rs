//! 系统托盘与轻量模式入口。

use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager, Wry};

use super::lightweight;
use super::window_state::{
    self as window_lifecycle, MainWindowMode, MainWindowState, MAIN_WINDOW_LABEL,
};

const SHOW_MENU_ID: &str = "tray-show";
const LIGHT_WEIGHT_MENU_ID: &str = "tray-light-weight";
const QUIT_MENU_ID: &str = "tray-quit";

struct TrayMenuState {
    _show: MenuItem<Wry>,
    _light_weight: CheckMenuItem<Wry>,
    _quit: MenuItem<Wry>,
}

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, SHOW_MENU_ID, "显示主窗口", true, None::<&str>)?;
    let light_weight =
        CheckMenuItem::with_id(app, LIGHT_WEIGHT_MENU_ID, "轻量模式", true, false, None::<&str>)?;
    let quit = MenuItem::with_id(app, QUIT_MENU_ID, "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &light_weight, &quit])?;
    app.state::<MainWindowState>()
        .set_tray_item(light_weight.clone());
    app.manage(TrayMenuState {
        _show: show,
        _light_weight: light_weight,
        _quit: quit,
    });

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| std::io::Error::other("default application icon is unavailable"))?;
    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("Sea Lantern")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SHOW_MENU_ID => show_main_window(app),
            LIGHT_WEIGHT_MENU_ID => {
                if let Err(error) = toggle_light_weight_mode(app) {
                    tracing::error!(%error, "failed to toggle lightweight mode");
                }
            }
            QUIT_MENU_ID => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

pub fn show_main_window(app: &AppHandle) {
    if let Err(error) = reveal_main_window(app) {
        tracing::error!(%error, "failed to show the main window from tray");
    }
}

pub fn show_when_ready(app: &AppHandle) {
    if app.state::<MainWindowState>().mode() == MainWindowMode::Hidden {
        show_main_window(app);
    }
}

fn reveal_main_window(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<MainWindowState>();
    let mut transition = state.begin_transition()?;
    if app.get_webview_window(MAIN_WINDOW_LABEL).is_none()
        && transition.mode() != MainWindowMode::Background
    {
        transition.move_to(MainWindowMode::Background)?;
    }
    if transition.mode() == MainWindowMode::Background {
        return lightweight::leave(app, &mut transition);
    }
    window_lifecycle::show(app, &mut transition)
}

fn toggle_light_weight_mode(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<MainWindowState>();
    let mut transition = state.begin_transition()?;
    if transition.mode() == MainWindowMode::Background {
        lightweight::leave(app, &mut transition)
    } else {
        lightweight::enter(app, &mut transition)
    }
}

/// 隐藏主窗口但保留 WebView，用于最小化到托盘。
#[tauri::command]
pub fn hide_main_window(app: AppHandle) -> Result<(), String> {
    window_lifecycle::hide(&app)
}

/// 显示主窗口；处于轻量模式时先重建 WebView。
#[tauri::command]
pub fn restore_main_window(app: AppHandle) -> Result<(), String> {
    reveal_main_window(&app)
}

/// 切换轻量模式：销毁主 WebView，同时保持后台服务运行。
#[tauri::command]
pub fn toggle_light_weight(app: AppHandle) -> Result<(), String> {
    toggle_light_weight_mode(&app)
}
