//! 平台原生窗口材质。

use super::theme::SystemTheme;
use super::window_state::MAIN_WINDOW_LABEL;
#[cfg(target_os = "macos")]
use std::ffi::c_void;
use tauri::{AppHandle, Manager};

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn sealantern_supports_liquid_glass() -> i32;
    fn sealantern_set_liquid_glass(window: *mut c_void, enabled: i32) -> i32;
}

#[cfg(target_os = "macos")]
fn set_liquid_glass(app: &AppHandle, enabled: bool) -> Result<bool, String> {
    if enabled && unsafe { sealantern_supports_liquid_glass() } != 1 {
        tracing::debug!("Liquid Glass is unavailable; falling back to vibrancy");
        return Ok(false);
    }
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        tracing::warn!("Liquid Glass update skipped because the main window is missing");
        return Ok(true);
    };
    let native_window = window.ns_window().map_err(|error| error.to_string())?;
    // NSWindow 由 Tauri 持有；Swift 仅在 AppKit 主线程同步借用该指针。
    let status = unsafe { sealantern_set_liquid_glass(native_window, i32::from(enabled)) };
    if status == 1 {
        Ok(true)
    } else {
        Err("failed to update macOS Liquid Glass".to_owned())
    }
}

pub(super) fn set_material(
    app: &AppHandle,
    material: &str,
    theme: SystemTheme,
) -> Result<(), String> {
    tracing::debug!(material, ?theme, "updating native window material");

    #[cfg(target_os = "windows")]
    {
        use tauri::window::{Color, Effect, EffectsBuilder};

        let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
            return Ok(());
        };
        let effects = match material {
            "acrylic" => Some(
                EffectsBuilder::new()
                    .effect(Effect::Acrylic)
                    .color(match theme {
                        SystemTheme::Dark => Color(32, 32, 32, 225),
                        SystemTheme::Light => Color(245, 245, 245, 215),
                    })
                    .build(),
            ),
            _ => None,
        };
        window
            .set_effects(effects)
            .map_err(|error| error.to_string())
    }

    #[cfg(target_os = "macos")]
    {
        use tauri::window::{Effect, EffectsBuilder};

        let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
            return Ok(());
        };
        window
            .set_effects(None)
            .map_err(|error| error.to_string())?;
        if material == "liquid_glass" && set_liquid_glass(app, true)? {
            return Ok(());
        }
        set_liquid_glass(app, false)?;

        let effects = matches!(material, "vibrancy" | "liquid_glass").then(|| {
            EffectsBuilder::new()
                .effect(Effect::UnderWindowBackground)
                .build()
        });
        window
            .set_effects(effects)
            .map_err(|error| error.to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (app, material, theme);
        Ok(())
    }
}

/// 返回当前 macOS 系统是否支持液态玻璃。
#[tauri::command]
pub fn supports_liquid_glass() -> bool {
    #[cfg(target_os = "macos")]
    return unsafe { sealantern_supports_liquid_glass() == 1 };

    #[cfg(not(target_os = "macos"))]
    false
}

/// 应用指定的原生窗口材质与主题。
#[tauri::command(rename_all = "snake_case")]
pub fn set_window_material(app: AppHandle, material: String, theme: String) -> Result<(), String> {
    super::theme::apply_material(&app, &material, &theme)
}

/// 保留对旧版高级材质开关的兼容。
#[tauri::command(rename_all = "snake_case")]
pub fn apply_acrylic(app: AppHandle, enabled: bool) -> Result<(), String> {
    let material = if enabled {
        advanced_material()
    } else {
        "solid"
    };
    super::theme::apply_material(&app, material, "auto")
}

#[cfg(target_os = "windows")]
const fn advanced_material() -> &'static str {
    "acrylic"
}

#[cfg(target_os = "macos")]
const fn advanced_material() -> &'static str {
    "liquid_glass"
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
const fn advanced_material() -> &'static str {
    "solid"
}
