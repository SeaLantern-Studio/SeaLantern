use std::path::PathBuf;

use crate::hardcode_data::app_files::{APP_DOCKER_DATA_DIR, APP_HIDDEN_DIRECTORY_NAME};

#[cfg(any(target_os = "windows", target_os = "macos"))]
use crate::hardcode_data::app_files::APP_DIRECTORY_NAME;

#[cfg(target_os = "linux")]
use crate::hardcode_data::app_files::APP_DIRECTORY_NAME_LOWERCASE;

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
/// - Docker 环境：./data
/// - Windows MSI 安装：%AppData%\Sea Lantern
/// - Windows 便携版：程序所在目录
/// - macOS: ~/Library/Application Support/Sea Lantern
/// - Linux: ~/.local/share/sea-lantern
///
/// 这个函数确保 MSI 安装的应用将数据存储在用户目录而非安装目录
pub fn get_app_data_dir() -> PathBuf {
    if std::path::Path::new("/.dockerenv").exists() {
        return PathBuf::from(APP_DOCKER_DATA_DIR);
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: 检查是否为 MSI 安装
        if is_msi_installation() {
            if let Some(data_dir) = dirs_next::data_dir() {
                return data_dir.join(APP_DIRECTORY_NAME);
            }
            if let Some(home_dir) = dirs_next::home_dir() {
                return home_dir.join(APP_HIDDEN_DIRECTORY_NAME);
            }
        }

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.to_path_buf();
            }
        }

        if let Some(home_dir) = dirs_next::home_dir() {
            return home_dir.join(APP_HIDDEN_DIRECTORY_NAME);
        }
        PathBuf::from(".")
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(data_dir) = dirs_next::data_dir() {
            return data_dir.join(APP_DIRECTORY_NAME);
        }
        if let Some(home_dir) = dirs_next::home_dir() {
            return home_dir
                .join("Library")
                .join("Application Support")
                .join(APP_DIRECTORY_NAME);
        }
        PathBuf::from(".")
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(data_dir) = dirs_next::data_dir() {
            return data_dir.join(APP_DIRECTORY_NAME_LOWERCASE);
        }
        if let Some(home_dir) = dirs_next::home_dir() {
            return home_dir.join(APP_HIDDEN_DIRECTORY_NAME);
        }
        PathBuf::from(".")
    }
}

/// 获取应用数据目录的字符串表示，如果目录不存在则创建
pub fn get_or_create_app_data_dir() -> String {
    let data_dir = get_app_data_dir();

    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        eprintln!("警告：无法创建数据目录：{}", e);
    }

    data_dir.to_string_lossy().to_string()
}

/// 只允许传入单纯的文件名
pub fn validate_file_name_only(file_name: &str) -> Result<&str, String> {
    let trimmed = file_name.trim();
    if trimmed.is_empty() {
        return Err("文件名不能为空".to_string());
    }

    let path = std::path::Path::new(trimmed);
    if path.is_absolute() {
        return Err("文件名不能是绝对路径".to_string());
    }

    if trimmed == "." || trimmed == ".." {
        return Err("文件名不合法".to_string());
    }

    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err("文件名里不能包含路径分隔符".to_string());
    }

    let base_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "文件名不合法".to_string())?;

    if base_name != trimmed {
        return Err("文件名不合法".to_string());
    }

    Ok(trimmed)
}

#[cfg(test)]
#[path = "../../tests/unit/utils_path_tests.rs"]
mod tests;
