use std::ops::Deref;
use std::path::{Component, Path, PathBuf};

use super::FsError;

/// Windows 保留设备名，去掉扩展名后作大写比较。
///
/// 在 Windows 上，`CON`、`PRN`、`AUX`、`NUL` 以及 `COM1`-`COM9`、`LPT1`-`LPT9`
/// 无论是否带扩展名（如 `NUL.txt`）都指向设备而非普通文件，写入会被静默丢弃
/// 或失败。归档里出现这样的条目名会让用户以为文件已解压，实际却是空设备。
/// 为保持归档可移植性，所有平台统一拒绝，不做 `cfg` 分支。
const WINDOWS_RESERVED_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

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
            let Component::Normal(component) = component else {
                return Err(invalid(path, "path contains a traversal or root component"));
            };
            validate_component(
                path,
                component
                    .to_str()
                    .ok_or_else(|| invalid(path, "path contains a non-Unicode component"))?,
            )?;
        }
        Ok(Self(path.to_path_buf()))
    }

    /// 返回已验证的相对路径。
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// 校验单个路径组件在 Windows 上不会产生歧义。
///
/// 组件里的 ASCII 字符按小写比较（Windows 文件系统大小写不敏感）：
/// - 去掉首个 `.` 之后的部分若命中保留设备名则拒绝
/// - 以 `.` 或空格结尾的组件在 Windows 上会被截断（`a.` 与 `a` 指向同一
///   文件），与归档内的另一个 `a` 无法区分，拒绝
fn validate_component(path: &Path, component: &str) -> Result<(), FsError> {
    let stem = component.split('.').next().unwrap_or(component);
    if WINDOWS_RESERVED_NAMES
        .iter()
        .any(|name| name.eq_ignore_ascii_case(stem))
    {
        return Err(invalid(path, "path component is a Windows reserved device name"));
    }
    if component.ends_with('.') || component.ends_with(' ') {
        return Err(invalid(path, "path component ends with a dot or space"));
    }
    Ok(())
}

impl AsRef<Path> for SafeRelativePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

/// 允许直接调用 `Path` 的方法（如 `parent()`、`join()`、`file_name()`），
/// 无需手动写 `.as_path()`。
impl Deref for SafeRelativePath {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

fn invalid(path: &Path, reason: &'static str) -> FsError {
    FsError::InvalidPath { path: path.to_path_buf(), reason }
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
    fn rejects_windows_reserved_device_names() {
        for path in ["CON", "nul.txt", "COM1", "NUL.txt", "a/CON/b", "lpt9/server.log"] {
            assert!(SafeRelativePath::parse(path).is_err(), "{path} should be rejected");
        }
    }

    #[test]
    fn accepts_names_with_a_reserved_prefix() {
        // 前缀相同但并非保留名，不应误拒。
        for path in ["console.log", "nullable.txt", "command/run.sh", "auxx.txt"] {
            assert!(SafeRelativePath::parse(path).is_ok(), "{path} should be accepted");
        }
    }

    #[test]
    fn rejects_components_ending_with_a_dot_or_space() {
        for path in ["evil.", "evil ", "dir/name.", "dir/name "] {
            assert!(SafeRelativePath::parse(path).is_err(), "{path} should be rejected");
        }
    }
}
