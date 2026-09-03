//! 跨平台的应用程序目录位置策略。
//!
//! 本模块决定"应用程序的数据、缓存、配置等文件应该存放在哪里"——这是平台感知问题，
//! 不是数据持久化问题。`persistence` 等消费者通过本模块获取目录路径，不自行推导
//! 路径规则。
//!
//! 路径优先级因平台而异（Docker → MSI → 便携版 → 标准目录），各平台内部的
//! fallback 链通过 `Option::or_else` 表达，优先级由调用顺序决定。

use std::path::PathBuf;

use super::error::PlatformError;
use crate::observability;

const APP_DATA_DIR_ENV: &str = "SEALANTERN_DATA_DIR";

/// 标准安装的应用目录名（macOS 和 Windows MSI 安装使用）。
#[cfg(any(target_os = "windows", target_os = "macos"))]
const APP_DIR_NAME: &str = "SeaLantern";

/// Linux 平台的应用目录名（遵循 XDG 规范使用小写）。
#[cfg(target_os = "linux")]
const APP_DIR_NAME_LOWERCASE: &str = "sea-lantern";

/// 回退方案使用的隐藏目录名（Linux `$HOME` 回退、Windows 非 MSI 最终回退）。
#[cfg(any(target_os = "windows", target_os = "linux"))]
const APP_DIR_HIDDEN: &str = ".sea-lantern";

/// Docker 容器内的数据目录。
const APP_DOCKER_DATA_DIR: &str = "./data";

/// 检查是否为 MSI 安装（程序安装在 Program Files 目录）。
#[cfg(target_os = "windows")]
fn is_msi_installation() -> bool {
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(parent) = exe_path.parent()
    {
        let exe_str = parent.to_string_lossy().to_lowercase();
        if exe_str.contains(r"\program files\") {
            return true;
        }
    }
    false
}

/// 获取应用程序数据目录。
///
/// 根据不同平台和运行环境返回合适的存储路径：
/// - Docker：`./data`
/// - Windows MSI 安装：`%APPDATA%\SeaLantern`
/// - Windows 便携版：程序所在目录
/// - macOS：`~/Library/Application Support/SeaLantern`
/// - Linux：`~/.local/share/sea-lantern`
pub fn get_app_data_dir() -> PathBuf {
    if let Some(dir) = env_override() {
        return dir;
    }

    if std::path::Path::new("/.dockerenv").exists() {
        return PathBuf::from(APP_DOCKER_DATA_DIR);
    }

    #[cfg(target_os = "windows")]
    {
        if is_msi_installation()
            && let Some(dir) = dirs::data_dir()
                .map(|d| d.join(APP_DIR_NAME))
                .or_else(|| dirs::home_dir().map(|h| h.join(APP_DIR_HIDDEN)))
        {
            return dir;
        }
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .or_else(|| dirs::home_dir().map(|h| h.join(APP_DIR_HIDDEN)))
            .unwrap_or_else(|| PathBuf::from("."))
    }

    #[cfg(target_os = "macos")]
    {
        dirs::data_dir()
            .map(|d| d.join(APP_DIR_NAME))
            .or_else(|| {
                dirs::home_dir().map(|h| {
                    h.join("Library")
                        .join("Application Support")
                        .join(APP_DIR_NAME)
                })
            })
            .unwrap_or_else(|| PathBuf::from("."))
    }

    #[cfg(target_os = "linux")]
    {
        dirs::data_dir()
            .map(|d| d.join(APP_DIR_NAME_LOWERCASE))
            .or_else(|| dirs::home_dir().map(|h| h.join(APP_DIR_HIDDEN)))
            .unwrap_or_else(|| PathBuf::from("."))
    }
}

/// 通过环境变量覆盖数据目录。
///
/// 环境变量存在但为空时记录警告并回退默认路径，方便诊断配置错误。
fn env_override() -> Option<PathBuf> {
    let value = match std::env::var(APP_DATA_DIR_ENV) {
        Ok(v) => v,
        Err(std::env::VarError::NotPresent) => return None,
        // 变量存在但值不是合法 UTF-8，视为无效配置
        Err(std::env::VarError::NotUnicode(_)) => {
            observability::platform_env_override_invalid(APP_DATA_DIR_ENV);
            return None;
        }
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        observability::platform_env_override_invalid(APP_DATA_DIR_ENV);
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

/// 获取应用数据目录，如果不存在则创建。
///
/// 如果目录创建失败，仅记录警告不阻断流程——调用方仍可拿到路径自行处理。
pub fn get_or_create_app_data_dir() -> String {
    let data_dir = get_app_data_dir();
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        observability::app_data_dir_create_failed(&data_dir, &e);
    }
    data_dir.to_string_lossy().to_string()
}

/// 获取默认运行路径。
///
/// 路径优先级：标准数据目录 → 文档目录 → 当前工作目录。
///
/// 返回原始 `PathBuf` 而非字符串；目录全部不可用时以
/// [`PlatformError::ResolveDefaultRunPath`] 返回原始错误。
pub fn get_default_run_path() -> Result<PathBuf, PlatformError> {
    if let Some(base) = dirs_next::data_dir().or_else(dirs_next::document_dir) {
        return Ok(base.join("SeaLantern"));
    }

    std::env::current_dir()
        .map(|cwd| cwd.join("SeaLantern"))
        .map_err(|source| PlatformError::ResolveDefaultRunPath { source })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_data_dir_is_not_empty() {
        let dir = get_app_data_dir();
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn test_app_data_dir_ends_with_app_name() {
        let dir = get_app_data_dir();
        let file_name = dir.file_name().expect("path should have a file name");
        let name = file_name.to_string_lossy();

        // 预期路径末端包含应用目录名（SeaLantern / sea-lantern / .sea-lantern）。
        // Windows 便携版返回 exe 所在目录，不一定以应用名结尾，跳过。
        if cfg!(not(target_os = "windows")) {
            assert!(
                name.contains("SeaLantern") || name.contains("sea-lantern"),
                "expected app directory name in path, got: {name}"
            );
        }
    }

    #[test]
    fn test_get_or_create_app_data_dir_returns_non_empty() {
        let dir_str = get_or_create_app_data_dir();
        assert!(!dir_str.is_empty());
    }

    #[test]
    fn test_get_default_run_path_returns_non_empty() {
        let path = get_default_run_path().expect("default run path should resolve");
        assert!(!path.as_os_str().is_empty());
    }

    #[test]
    fn test_get_default_run_path_ends_with_app_name() {
        let path = get_default_run_path().expect("default run path should resolve");
        let name = path
            .file_name()
            .expect("path should have a file name")
            .to_string_lossy();

        assert_eq!(name, "SeaLantern", "expected SeaLantern directory name, got: {name}");
    }
}
