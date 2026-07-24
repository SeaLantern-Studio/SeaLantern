use std::path::{Component, Path, PathBuf};

use super::FsError;

/// 一个保证为相对路径且不含遍历组件的路径。
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SafeRelativePath(PathBuf);

impl SafeRelativePath {
    /// 解析一个可用于文件系统存储的可移植相对路径。
    pub fn parse(path: impl AsRef<Path>) -> Result<Self, FsError> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() {
            return Err(invalid(path, "path is empty"));
        }
        if path.is_absolute() || path.to_string_lossy().contains('\\') {
            return Err(invalid(path, "path must be portable and relative"));
        }
        for component in path.components() {
            if !matches!(component, Component::Normal(_)) {
                return Err(invalid(path, "path contains a traversal or root component"));
            }
        }
        Ok(Self(path.to_path_buf()))
    }

    /// 返回已验证的相对路径。
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for SafeRelativePath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

fn invalid(path: &Path, reason: &'static str) -> FsError {
    FsError::InvalidPath { path: path.to_path_buf(), reason }
}

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
/// - Windows MSI 安装：%AppData%\SeaLantern
/// - Windows 便携版：程序所在目录
/// - macOS: ~/Library/Application Support/SeaLantern
/// - Linux: ~/.local/share/sea-lantern
///
/// 这个函数确保 MSI 安装的应用将数据存储在用户目录而非安装目录
pub fn get_app_data_dir() -> PathBuf {
    // Docker 环境检测 - 优先返回容器内数据目录
    let is_docker = std::path::Path::new("/.dockerenv").exists();
    eprintln!("[DEBUG] path.rs: /.dockerenv exists = {}", is_docker);
    if is_docker {
        let docker_path = PathBuf::from("./data");
        eprintln!("[DEBUG] path.rs: Docker mode, returning path: {:?}", docker_path);
        return docker_path;
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: 检查是否为 MSI 安装
        if is_msi_installation() {
            // MSI 安装：使用 %AppData%
            if let Some(data_dir) = dirs::data_dir() {
                eprintln!(
                    "[DEBUG] path.rs: Windows MSI mode, returning path: {:?}",
                    data_dir.join("SeaLantern")
                );
                return data_dir.join("SeaLantern");
            }
            // 回退到主目录
            if let Some(home_dir) = dirs::home_dir() {
                eprintln!(
                    "[DEBUG] path.rs: Windows fallback to home, returning path: {:?}",
                    home_dir.join(".sea-lantern")
                );
                return home_dir.join(".sea-lantern");
            }
        }
        // 便携版或其他安装：使用程序所在目录
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                eprintln!("[DEBUG] path.rs: Windows portable mode, returning path: {:?}", exe_dir);
                return exe_dir.to_path_buf();
            }
        }
        // 最后的回退方案
        if let Some(home_dir) = dirs::home_dir() {
            eprintln!(
                "[DEBUG] path.rs: Windows final fallback, returning path: {:?}",
                home_dir.join(".sea-lantern")
            );
            return home_dir.join(".sea-lantern");
        }
        eprintln!("[DEBUG] path.rs: Windows ultimate fallback, returning path: .");
        PathBuf::from(".")
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: ~/Library/Application Support/SeaLantern
        if let Some(data_dir) = dirs::data_dir() {
            eprintln!(
                "[DEBUG] path.rs: macOS mode, returning path: {:?}",
                data_dir.join("SeaLantern")
            );
            return data_dir.join("SeaLantern");
        }
        if let Some(home_dir) = dirs::home_dir() {
            eprintln!(
                "[DEBUG] path.rs: macOS fallback, returning path: {:?}",
                home_dir
                    .join("Library")
                    .join("Application Support")
                    .join("SeaLantern")
            );
            return home_dir
                .join("Library")
                .join("Application Support")
                .join("SeaLantern");
        }
        eprintln!("[DEBUG] path.rs: macOS ultimate fallback, returning path: .");
        PathBuf::from(".")
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: ~/.local/share/sea-lantern
        if let Some(data_dir) = dirs::data_dir() {
            eprintln!(
                "[DEBUG] path.rs: Linux mode, returning path: {:?}",
                data_dir.join("sea-lantern")
            );
            return data_dir.join("sea-lantern");
        }
        if let Some(home_dir) = dirs::home_dir() {
            eprintln!(
                "[DEBUG] path.rs: Linux fallback, returning path: {:?}",
                home_dir.join(".sea-lantern")
            );
            return home_dir.join(".sea-lantern");
        }
        eprintln!("[DEBUG] path.rs: Linux ultimate fallback, returning path: .");
        PathBuf::from(".")
    }
}

/// 获取应用数据目录的字符串表示，如果目录不存在则创建
pub fn get_or_create_app_data_dir() -> String {
    let data_dir = get_app_data_dir();
    eprintln!("[DEBUG] path.rs: get_or_create_app_data_dir called, path: {:?}", data_dir);
    // 创建目录（如果不存在）
    if let Err(e) = std::fs::create_dir_all(&data_dir) {
        eprintln!("警告：无法创建数据目录：{}", e);
    } else {
        eprintln!("[DEBUG] path.rs: Directory created/verified successfully");
    }
    let result = data_dir.to_string_lossy().to_string();
    eprintln!("[DEBUG] path.rs: Returning data dir string: {}", result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_nested_relative_path() {
        assert_eq!(
            SafeRelativePath::parse("cache/manifest.json")
                .unwrap()
                .as_path(),
            Path::new("cache/manifest.json")
        );
    }

    #[test]
    fn rejects_traversal_and_absolute_paths() {
        for path in ["../secret", "/etc/passwd", "folder\\\\file", "."] {
            assert!(SafeRelativePath::parse(path).is_err(), "{path} should be rejected");
        }
    }

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
