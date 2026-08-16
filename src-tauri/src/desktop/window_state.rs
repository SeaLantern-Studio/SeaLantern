//! 主窗口生命周期状态机。

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, MutexGuard};

use tauri::menu::CheckMenuItem;
use tauri::{AppHandle, Manager, Wry};

pub const MAIN_WINDOW_LABEL: &str = "main";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MainWindowMode {
    Visible = 0,
    Hidden = 1,
    Background = 2,
}

impl MainWindowMode {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Visible,
            2 => Self::Background,
            _ => Self::Hidden,
        }
    }

    fn allows(self, next: Self) -> bool {
        self == next
            || matches!(
                (self, next),
                (Self::Visible, Self::Hidden | Self::Background)
                    | (Self::Hidden, Self::Visible | Self::Background)
                    | (Self::Background, Self::Hidden)
            )
    }
}

/// 串行执行主窗口的显示、隐藏、销毁与重建操作。
pub struct MainWindowState {
    mode: AtomicU8,
    transition: Mutex<()>,
    tray_item: Mutex<Option<CheckMenuItem<Wry>>>,
}

impl MainWindowState {
    pub fn new() -> Self {
        Self {
            // tauri.conf.json 隐藏创建主窗口，前端完成首屏准备后再显示。
            mode: AtomicU8::new(MainWindowMode::Hidden as u8),
            transition: Mutex::new(()),
            tray_item: Mutex::new(None),
        }
    }

    pub fn mode(&self) -> MainWindowMode {
        MainWindowMode::from_u8(self.mode.load(Ordering::Acquire))
    }

    pub fn begin_transition(&self) -> Result<MainWindowTransition<'_>, String> {
        let guard = self
            .transition
            .lock()
            .map_err(|_| "main window transition lock is poisoned".to_owned())?;
        Ok(MainWindowTransition { state: self, _guard: guard })
    }

    pub(super) fn set_tray_item(&self, item: CheckMenuItem<Wry>) {
        let _ = item.set_checked(self.mode() == MainWindowMode::Background);
        if let Ok(mut tray_item) = self.tray_item.lock() {
            *tray_item = Some(item);
        }
    }

    fn set_mode(&self, mode: MainWindowMode) {
        self.mode.store(mode as u8, Ordering::Release);
        if let Ok(tray_item) = self.tray_item.lock() {
            if let Some(tray_item) = tray_item.as_ref() {
                let _ = tray_item.set_checked(mode == MainWindowMode::Background);
            }
        }
    }
}

impl Default for MainWindowState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MainWindowTransition<'a> {
    state: &'a MainWindowState,
    _guard: MutexGuard<'a, ()>,
}

impl MainWindowTransition<'_> {
    pub fn mode(&self) -> MainWindowMode {
        self.state.mode()
    }

    pub fn move_to(&mut self, next: MainWindowMode) -> Result<(), String> {
        let current = self.mode();
        if current == next {
            return Ok(());
        }
        if !current.allows(next) {
            tracing::warn!(?current, ?next, "main window transition rejected");
            return Err(format!("invalid main window transition: {current:?} -> {next:?}"));
        }
        self.state.set_mode(next);
        tracing::debug!(?current, ?next, "main window mode changed");
        Ok(())
    }
}

pub fn show(app: &AppHandle, transition: &mut MainWindowTransition<'_>) -> Result<(), String> {
    if transition.mode() == MainWindowMode::Background {
        return Err("background window must be restored to hidden before showing".to_owned());
    }
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        transition.move_to(MainWindowMode::Background)?;
        return Err("main window is unavailable".to_owned());
    };

    #[cfg(target_os = "windows")]
    window
        .set_skip_taskbar(false)
        .map_err(|error| error.to_string())?;
    window.unminimize().map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    transition.move_to(MainWindowMode::Visible)?;
    window.set_focus().map_err(|error| error.to_string())
}

pub fn hide(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<MainWindowState>();
    let mut transition = state.begin_transition()?;
    if transition.mode() == MainWindowMode::Background {
        return Ok(());
    }
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        transition.move_to(MainWindowMode::Background)?;
        return Ok(());
    };
    if transition.mode() == MainWindowMode::Hidden {
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    window
        .set_skip_taskbar(true)
        .map_err(|error| error.to_string())?;
    window.hide().map_err(|error| error.to_string())?;
    transition.move_to(MainWindowMode::Hidden)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_window_modes() {
        assert_eq!(MainWindowState::new().mode(), MainWindowMode::Hidden);
        assert_eq!(MainWindowMode::from_u8(0), MainWindowMode::Visible);
        assert_eq!(MainWindowMode::from_u8(1), MainWindowMode::Hidden);
        assert_eq!(MainWindowMode::from_u8(2), MainWindowMode::Background);
        assert_eq!(MainWindowMode::from_u8(u8::MAX), MainWindowMode::Hidden);
    }

    #[test]
    fn rejects_direct_background_restore() {
        assert!(MainWindowMode::Background.allows(MainWindowMode::Hidden));
        assert!(MainWindowMode::Hidden.allows(MainWindowMode::Visible));
        assert!(!MainWindowMode::Background.allows(MainWindowMode::Visible));
    }
}
