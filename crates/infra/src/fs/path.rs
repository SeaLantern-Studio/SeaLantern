use std::path::{Component, Path, PathBuf};

use super::FsError;

/// A path guaranteed to be relative and free of traversal components.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SafeRelativePath(PathBuf);

impl SafeRelativePath {
    /// Parses a portable relative path accepted for file system storage.
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

    /// Returns the validated relative path.
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
