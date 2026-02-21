use std::path::PathBuf;

/// 检查是否为 MSI 安装（程序安装在 Program Files 目录）
#[cfg(target_os = "windows")]
fn is_msi_installation() -> bool {
    // 检查可执行文件是否在 Program Files 目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let exe_str = parent.to_string_lossy().to_lowercase();
            // 检查路径是否包含 Program Files
            if exe_str.contains(r"\program files\") {
                return true;
            }
        }
    }
    false
}

/// 获取应用程序数据目录
///
/// 根据不同平台和安装方式返回合适的存储路径：
/// - Windows MSI 安装：%AppData%\Sea Lantern
/// - Windows 便携版：程序所在目录
/// - macOS: ~/Library/Application Support/Sea Lantern
/// - Linux: ~/.local/share/sea-lantern
///
/// 这个函数确保 MSI 安装的应用将数据存储在用户目录而非安装目录
pub fn get_app_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Windows: 检查是否为 MSI 安装
        if is_msi_installation() {
            // MSI 安装：使用 %AppData%
            if let Some(data_dir) = dirs_next::data_dir() {
                return data_dir.join("Sea Lantern");
            }
            // 回退到主目录
            if let Some(home_dir) = dirs_next::home_dir() {
                return home_dir.join(".sea-lantern");
            }
        }

        // 便携版或其他安装：使用程序所在目录
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.to_path_buf();
            }
        }

        // 最后的回退方案
        if let Some(home_dir) = dirs_next::home_dir() {
            return home_dir.join(".sea-lantern");
        }
        PathBuf::from(".")
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: ~/Library/Application Support/Sea Lantern
        if let Some(data_dir) = dirs_next::data_dir() {
            return data_dir.join("Sea Lantern");
        }
        if let Some(home_dir) = dirs_next::home_dir() {
            return home_dir
                .join("Library")
                .join("Application Support")
                .join("Sea Lantern");
        }
        PathBuf::from(".")
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: ~/.local/share/sea-lantern
        if let Some(data_dir) = dirs_next::data_dir() {
            return data_dir.join("sea-lantern");
        }
        if let Some(home_dir) = dirs_next::home_dir() {
            return home_dir.join(".sea-lantern");
        }
        PathBuf::from(".")
    }
}

/// 获取应用数据目录的字符串表示，如果目录不存在则创建
pub fn get_or_create_app_data_dir() -> String {
    let data_dir = get_app_data_dir();

    // 创建目录（如果不存在）
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        eprintln!("警告：无法创建数据目录：{}", e);
    }

    data_dir.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_app_data_dir_not_empty() {
        let dir = get_app_data_dir();
        assert!(!dir.as_path().as_os_str().is_empty());
    }

    #[test]
    fn test_get_or_create_app_data_dir() {
        let dir_str = get_or_create_app_data_dir();
        assert!(!dir_str.is_empty());

        // 验证目录存在
        let path = PathBuf::from(&dir_str);
        assert!(path.exists());
        assert!(path.is_dir());
    }
}
