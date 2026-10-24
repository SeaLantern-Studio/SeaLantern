//! 跨平台的应用程序目录位置策略。
//!
//! 本模块决定"应用程序的数据、缓存、配置等文件应该存放在哪里"——这是平台感知问题，
//! 不是数据持久化问题。`persistence` 等消费者通过本模块获取目录路径，不自行推导
//! 路径规则。
//!
//! 路径优先级因平台而异（Docker → MSI → 便携版 → 标准目录），各平台内部的
//! fallback 链通过 `Option::or_else` 表达，优先级由调用顺序决定。

use std::path::{Path, PathBuf};

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

/// 备份目录名（主资源目录下）。
const BACKUPS_DIR_NAME: &str = "backups";

/// 实例容器目录名（主资源目录下）。
const INSTANCES_DIR_NAME: &str = "instances";

/// 临时下载目录名（主资源目录下）；与实例容器分离，避免下载暂存被实例
/// 发现层误识别为实例目录。
const TEMP_DIR_NAME: &str = "temp";

/// 宿主插件目录名（主资源目录下）。
const PLUGINS_DIR_NAME: &str = "plugins";

/// 共享资源目录名（主资源目录下）。
const RESOURCES_DIR_NAME: &str = "resources";

/// 应用目录布局。
///
/// 主配置目录存放应用设置；主资源目录存放服务器实例、备份与共享资源。
/// 主资源目录默认与主配置目录同址，可由应用设置覆盖。
///
/// 本类型只描述"目录应该在哪"，不做任何 IO、也不读取设置——覆盖值由调用方
/// （应用层）从设置中取出后传入，因此它应当保持为纯数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppLayout {
    config_dir: PathBuf,
    resource_dir: PathBuf,
}

impl AppLayout {
    /// 按显式路径构造布局；`resource_override` 为 `None` 时主资源目录与主配置目录同址。
    pub fn new(config_dir: impl Into<PathBuf>, resource_override: Option<PathBuf>) -> Self {
        let config_dir = config_dir.into();
        let resource_dir = match resource_override {
            Some(dir) => dir,
            None => config_dir.clone(),
        };
        Self { config_dir, resource_dir }
    }

    /// 按本机默认数据目录构造布局。
    pub fn native(resource_override: Option<PathBuf>) -> Self {
        Self::new(get_app_data_dir(), resource_override)
    }

    /// 主配置目录。
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// 主资源目录。
    pub fn resource_dir(&self) -> &Path {
        &self.resource_dir
    }

    /// 主配置目录下的某个文件。
    pub fn config_file(&self, name: &str) -> PathBuf {
        self.config_dir.join(name)
    }

    /// 主资源目录下的某个文件。
    pub fn resource_file(&self, name: &str) -> PathBuf {
        self.resource_dir.join(name)
    }

    /// 备份目录。
    pub fn backups_dir(&self) -> PathBuf {
        self.resource_dir.join(BACKUPS_DIR_NAME)
    }

    /// 服务器实例容器目录。
    pub fn instances_dir(&self) -> PathBuf {
        self.resource_dir.join(INSTANCES_DIR_NAME)
    }

    /// 临时下载目录（下载暂存等非实例数据）。
    ///
    /// 与 `instances_dir()` 同级但独立于实例容器：下载暂存目录若放进
    /// `instances/`，会被实例发现层当成缺失 `sl.json` 的问题目录上报。
    pub fn temp_dir(&self) -> PathBuf {
        self.resource_dir.join(TEMP_DIR_NAME)
    }

    /// 宿主插件目录。
    pub fn plugins_dir(&self) -> PathBuf {
        self.resource_dir.join(PLUGINS_DIR_NAME)
    }

    /// 共享资源目录。
    pub fn resources_dir(&self) -> PathBuf {
        self.resource_dir.join(RESOURCES_DIR_NAME)
    }
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

    #[test]
    fn app_layout_defaults_the_resource_dir_to_the_config_dir() {
        let layout = AppLayout::new("config", None);

        assert_eq!(layout.resource_dir(), layout.config_dir());
        assert_eq!(layout.instances_dir(), Path::new("config").join("instances"));
    }

    #[test]
    fn app_layout_honours_a_resource_dir_override() {
        let layout = AppLayout::new("config", Some(PathBuf::from("data")));

        assert_eq!(layout.config_dir(), Path::new("config"));
        assert_eq!(layout.resource_dir(), Path::new("data"));
        assert_eq!(layout.instances_dir(), Path::new("data").join("instances"));
        assert_eq!(layout.backups_dir(), Path::new("data").join("backups"));
        assert_eq!(layout.plugins_dir(), Path::new("data").join("plugins"));
        assert_eq!(layout.resources_dir(), Path::new("data").join("resources"));
    }

    #[test]
    fn app_layout_keeps_config_files_apart_from_resource_files() {
        let layout = AppLayout::new("config", Some(PathBuf::from("data")));

        assert_eq!(layout.config_file("settings.json"), Path::new("config").join("settings.json"));
        assert_eq!(layout.resource_file("archive.zip"), Path::new("data").join("archive.zip"));
    }
}
