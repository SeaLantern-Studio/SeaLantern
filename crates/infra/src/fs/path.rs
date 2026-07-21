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
}
