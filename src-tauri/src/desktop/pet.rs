//! Sea Lantern 桌面桌宠窗口。
//!
//! 桌宠使用独立的透明、无边框、置顶窗口，不挤占主窗口生命周期。
//! 前端通过 query 参数 `window=pet` 走轻量入口，主窗口仍使用完整 App。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

use tauri::menu::CheckMenuItem;
use tauri::window::Color;
use tauri::{
    AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Wry,
};

use super::tray;

pub const PET_WINDOW_LABEL: &str = "pet";
const PET_WIDTH: f64 = 520.0;
const PET_HEIGHT: f64 = 440.0;
const PET_WINDOW_MARGIN: i32 = 18;

/// 桌宠可见性状态，同时同步托盘里的“显示桌宠”勾选项。
pub struct PetWindowState {
    visible: AtomicBool,
    tray_item: Mutex<Option<CheckMenuItem<Wry>>>,
}

impl PetWindowState {
    pub fn new() -> Self {
        Self {
            visible: AtomicBool::new(false),
            tray_item: Mutex::new(None),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::Acquire)
    }

    pub(super) fn set_tray_item(&self, item: CheckMenuItem<Wry>) {
        let _ = item.set_checked(self.is_visible());
        if let Ok(mut tray_item) = self.lock_tray_item() {
            *tray_item = Some(item);
        }
    }

    fn set_visible(&self, visible: bool) {
        self.visible.store(visible, Ordering::Release);
        if let Ok(tray_item) = self.lock_tray_item()
            && let Some(tray_item) = tray_item.as_ref()
        {
            let _ = tray_item.set_checked(visible);
        }
    }

    fn lock_tray_item(&self) -> Result<MutexGuard<'_, Option<CheckMenuItem<Wry>>>, ()> {
        self.tray_item.lock().map_err(|_| ())
    }
}

impl Default for PetWindowState {
    fn default() -> Self {
        Self::new()
    }
}

/// 在应用 setup 阶段创建桌宠窗口；先隐藏，等前端首帧就绪后再显示。
pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window = create_window(app.handle())?;
    position_bottom_right(&window)?;
    // 默认让透明区域鼠标穿透；前端轮询到鼠标进入桌宠本体后会临时恢复交互。
    window.set_ignore_cursor_events(true)?;
    Ok(())
}

fn create_window(manager: &AppHandle) -> Result<WebviewWindow<Wry>, Box<dyn std::error::Error>> {
    let window = WebviewWindowBuilder::new(
        manager,
        PET_WINDOW_LABEL,
        WebviewUrl::App("index.html?window=pet".into()),
    )
    .title("Sea Lantern Pet")
    .inner_size(PET_WIDTH, PET_HEIGHT)
    .min_inner_size(PET_WIDTH, PET_HEIGHT)
    .max_inner_size(PET_WIDTH, PET_HEIGHT)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .background_color(Color(0, 0, 0, 0))
    .always_on_top(true)
    .skip_taskbar(true)
    .shadow(false)
    .visible(false)
    .focused(false)
    .build()?;
    Ok(window)
}

/// 初始放在主屏右下角，避开任务栏与固定边距。
fn position_bottom_right(window: &WebviewWindow<Wry>) -> Result<(), Box<dyn std::error::Error>> {
    let Some(monitor) = window.current_monitor()?.or(window.primary_monitor()?) else {
        return Ok(());
    };
    let size = window.outer_size()?;
    let work = monitor.work_area();
    let x = work.position.x + work.size.width as i32 - size.width as i32 - PET_WINDOW_MARGIN;
    let y = work.position.y + work.size.height as i32 - size.height as i32 - PET_WINDOW_MARGIN;
    window.set_position(PhysicalPosition::new(x, y))?;
    Ok(())
}

/// 显示桌宠。前端首帧就绪与托盘菜单都会走这里。
pub fn show(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<PetWindowState>();
    let window = match app.get_webview_window(PET_WINDOW_LABEL) {
        Some(window) => window,
        None => create_window(app).map_err(|error| error.to_string())?,
    };

    window.show().map_err(|error| error.to_string())?;
    window
        .set_always_on_top(true)
        .map_err(|error| error.to_string())?;
    state.set_visible(true);
    Ok(())
}

pub fn hide(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<PetWindowState>();
    if let Some(window) = app.get_webview_window(PET_WINDOW_LABEL) {
        window.hide().map_err(|error| error.to_string())?;
    }
    state.set_visible(false);
    Ok(())
}

pub fn toggle(app: &AppHandle) -> Result<(), String> {
    if app.state::<PetWindowState>().is_visible() {
        hide(app)
    } else {
        show(app)
    }
}

/// 桌宠前端完成首帧渲染后调用，避免透明窗口先闪出空白区域。
#[tauri::command]
pub fn pet_frontend_ready(app: AppHandle) -> Result<(), String> {
    show(&app)
}

#[tauri::command]
pub fn pet_hide(app: AppHandle) -> Result<(), String> {
    hide(&app)
}

#[tauri::command]
pub fn pet_open_main_window(app: AppHandle) -> Result<(), String> {
    tray::show_main_window(&app);
    Ok(())
}

/// 透明窗口大部分区域不承载内容；按鼠标位置切换穿透，避免挡住桌面点击。
#[tauri::command]
pub fn pet_set_click_through(app: AppHandle, ignore: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(PET_WINDOW_LABEL) {
        window
            .set_ignore_cursor_events(ignore)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pet_window_is_hidden_before_frontend_ready() {
        assert!(!PetWindowState::new().is_visible());
    }

    #[test]
    fn pet_window_uses_the_lightweight_frontend_entry() {
        let source = include_str!("pet.rs");
        assert!(source.contains("index.html?window=pet"));
        assert!(source.contains(".transparent(true)"));
        assert!(source.contains(".always_on_top(true)"));
        assert!(source.contains(".skip_taskbar(true)"));
    }
}
