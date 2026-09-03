//! 原生窗口材质与系统主题协调。

use std::sync::Mutex;

use tauri::{AppHandle, Manager};

use super::effects;
use super::window_state::MAIN_WINDOW_LABEL;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SystemTheme {
    Light,
    Dark,
}

#[derive(Clone, Debug)]
pub(super) struct AppearanceSnapshot {
    pub(super) material: String,
    pub(super) theme: String,
}

/// 保存最近一次应用的外观，以便窗口重建后恢复原生状态。
pub struct DesktopAppearanceState {
    current: Mutex<AppearanceSnapshot>,
}

impl DesktopAppearanceState {
    pub fn new() -> Self {
        Self {
            current: Mutex::new(AppearanceSnapshot {
                material: "solid".to_owned(),
                theme: "auto".to_owned(),
            }),
        }
    }

    pub(super) fn snapshot(&self) -> AppearanceSnapshot {
        self.current
            .lock()
            .map(|current| current.clone())
            .unwrap_or_else(|_| AppearanceSnapshot {
                material: "solid".to_owned(),
                theme: "auto".to_owned(),
            })
    }

    fn update(&self, material: &str, theme: &str) -> Result<(), String> {
        let mut current = self
            .current
            .lock()
            .map_err(|_| "desktop appearance lock is poisoned".to_owned())?;
        current.material.clear();
        current.material.push_str(material);
        current.theme.clear();
        current.theme.push_str(theme);
        Ok(())
    }
}

impl Default for DesktopAppearanceState {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn apply_material(
    app: &AppHandle,
    material: &str,
    preference: &str,
) -> Result<(), String> {
    validate_material(material)?;
    validate_theme(preference)?;
    let effective = match preference {
        "dark" => SystemTheme::Dark,
        "light" => SystemTheme::Light,
        _ => current(app),
    };

    // 先应用匹配主题的材质，避免清除显式主题时短暂闪现白色窗口。
    effects::set_material(app, material, effective)?;
    set_window_theme(app, preference)?;
    app.state::<DesktopAppearanceState>()
        .update(material, preference)
}

fn validate_material(material: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let supported = matches!(material, "solid" | "acrylic");
    #[cfg(target_os = "macos")]
    let supported = matches!(material, "solid" | "vibrancy" | "liquid_glass");
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let supported = material == "solid";

    if supported {
        Ok(())
    } else {
        Err("window material is unsupported on this platform".to_owned())
    }
}

fn validate_theme(theme: &str) -> Result<(), String> {
    if matches!(theme, "auto" | "light" | "dark") {
        Ok(())
    } else {
        Err("invalid theme preference".to_owned())
    }
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn set_window_theme(app: &AppHandle, preference: &str) -> Result<(), String> {
    use tauri::Theme;

    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Ok(());
    };
    let theme = match preference {
        "dark" => Some(Theme::Dark),
        "light" => Some(Theme::Light),
        _ => None,
    };
    window.set_theme(theme).map_err(|error| error.to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn set_window_theme(_app: &AppHandle, _preference: &str) -> Result<(), String> {
    Ok(())
}

fn current(app: &AppHandle) -> SystemTheme {
    #[cfg(target_os = "windows")]
    {
        let _ = app;
        windows_theme()
    }

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL)
            && let Ok(theme) = window.theme()
        {
            return match theme {
                tauri::Theme::Dark => SystemTheme::Dark,
                _ => SystemTheme::Light,
            };
        }
        SystemTheme::Light
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = app;
        SystemTheme::Light
    }
}

#[cfg(target_os = "windows")]
fn windows_theme() -> SystemTheme {
    use std::mem::MaybeUninit;
    use windows_sys::Win32::System::Registry::{
        HKEY_CURRENT_USER, KEY_READ, REG_DWORD, RegCloseKey, RegOpenKeyExW, RegQueryValueExW,
    };

    let key_path: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let value_name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();
    let mut key = std::ptr::null_mut();
    let opened =
        unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, key_path.as_ptr(), 0, KEY_READ, &mut key) };
    if opened != 0 {
        return SystemTheme::Light;
    }

    let mut value_type = 0;
    let mut value = MaybeUninit::<u32>::uninit();
    let mut value_size = std::mem::size_of::<u32>() as u32;
    let queried = unsafe {
        RegQueryValueExW(
            key,
            value_name.as_ptr(),
            std::ptr::null(),
            &mut value_type,
            value.as_mut_ptr().cast(),
            &mut value_size,
        )
    };
    unsafe { RegCloseKey(key) };
    if queried == 0 && value_type == REG_DWORD && value_size == 4 {
        return if unsafe { value.assume_init() } == 0 {
            SystemTheme::Dark
        } else {
            SystemTheme::Light
        };
    }
    SystemTheme::Light
}
