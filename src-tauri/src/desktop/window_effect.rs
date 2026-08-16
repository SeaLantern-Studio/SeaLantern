//! 窗口透明效果命令。
//!
//! `apply_acrylic`：开启后窗口背景透出后方桌面内容。
//! Windows 走 acrylic，旧系统回退 blur；macOS 走 NSVisualEffectView；
//! Linux 无原生支持，静默跳过（前端半透明层仍可提供近似观感）。

use tauri::Manager;

/// 应用或移除主窗口的亚克力透明效果。
#[tauri::command(rename_all = "snake_case")]
pub fn apply_acrylic(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    // 仅作用于主窗口，找不到时视为无需处理
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };

    #[cfg(target_os = "windows")]
    {
        use window_vibrancy::{apply_acrylic as set_acrylic, apply_blur, clear_acrylic};
        if enabled {
            // acrylic 需 Win10 1803+，失败则回退 blur
            if set_acrylic(&window, None).is_err() {
                apply_blur(&window, None).map_err(|e| format!("apply blur failed: {e}"))?;
            }
        } else {
            clear_acrylic(&window).map_err(|e| format!("clear acrylic failed: {e}"))?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        use window_vibrancy::{apply_vibrancy, clear_vibrancy, NSVisualEffectView};
        if enabled {
            apply_vibrancy(&window, NSVisualEffectView::Sidebar, None, None)
                .map_err(|e| format!("apply vibrancy failed: {e}"))?;
        } else {
            clear_vibrancy(&window).map_err(|e| format!("clear vibrancy failed: {e}"))?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        let _ = (&window, enabled);
    }

    Ok(())
}
