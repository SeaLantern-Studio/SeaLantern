//! 系统剪贴板访问。
//!
//! 供上层把票据等文本写入系统剪贴板。剪贴板是宿主平台能力，桌面端可用；
//! 无图形环境的宿主（如 headless Docker）会返回 [`PlatformError::ClipboardFailed`]。

use super::error::PlatformError;

/// 把文本写入系统剪贴板。
pub fn copy_text(text: &str) -> Result<(), PlatformError> {
    arboard::Clipboard::new()
        .and_then(|mut clipboard| clipboard.set_text(text.to_owned()))
        .map_err(|error| PlatformError::ClipboardFailed { message: error.to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_reads_back_through_the_clipboard() {
        // 无图形环境的 CI 上剪贴板不可用，此时跳过（返回 ClipboardFailed）。
        match copy_text("sealantern-clipboard-probe") {
            Ok(()) => {}
            Err(PlatformError::ClipboardFailed { .. }) => return,
            Err(other) => panic!("unexpected error: {other}"),
        }

        let mut clipboard = arboard::Clipboard::new().expect("clipboard available after write");
        let value = clipboard.get_text().expect("read back clipboard");
        assert_eq!(value, "sealantern-clipboard-probe");
    }
}
