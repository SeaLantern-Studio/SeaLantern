//! 系统剪贴板写入。

/// 将文本写入系统剪贴板，成功返回 `true`。
///
/// Linux Wayland 下优先调用 `wl-copy`，X11 下调用 `xclip`，其余平台使用 `arboard`。
pub fn clipboard_copy(text: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};

        if std::env::var_os("WAYLAND_DISPLAY").is_some()
            && let Ok(mut child) = Command::new("wl-copy")
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            return child.wait().is_ok_and(|s| s.success());
        }

        if std::env::var_os("DISPLAY").is_some()
            && let Ok(mut child) = Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            return child.wait().is_ok_and(|s| s.success());
        }
    }

    arboard::Clipboard::new()
        .and_then(|mut cb| cb.set_text(text))
        .is_ok()
}
