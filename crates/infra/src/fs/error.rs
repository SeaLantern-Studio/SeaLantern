use std::fmt;
use std::path::PathBuf;

/// 文件系统基础设施操作返回的错误。
#[derive(Debug)]
pub enum FsError {
    /// 底层的文件系统操作失败。
    Io {
        /// 正在尝试的操作。
        operation: &'static str,
        /// 操作涉及的路径。
        path: PathBuf,
        /// 底层的操作系统错误。
        source: std::io::Error,
    },
    /// 用户提供的路径不是安全的相对路径。
    InvalidPath { path: PathBuf, reason: &'static str },
    /// 读取操作超过了配置的最大大小限制。
    DataLimitExceeded {
        path: PathBuf,
        limit: usize,
        observed_at_least: usize,
    },
    /// 另一个进程或任务持有请求的锁。
    AlreadyLocked(PathBuf),
    /// 序列化或反序列化失败。
    Serialization {
        format: &'static str,
        operation: &'static str,
        path: PathBuf,
        message: String,
    },
    /// 文本无法以 UTF-8 解码。
    Encoding {
        path: PathBuf,
        encoding: &'static str,
        message: String,
    },
    /// 阻塞的文件系统任务无法完成。
    Task {
        operation: &'static str,
        message: String,
    },
}

impl fmt::Display for FsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, path, source } => {
                write!(f, "failed to {operation} '{}': {source}", path.display())
            }
            Self::InvalidPath { path, reason } => {
                write!(f, "unsafe path '{}': {reason}", path.display())
            }
            Self::DataLimitExceeded { path, limit, observed_at_least } => {
                write!(
                    f,
                    "file '{}' exceeds the {limit}-byte read limit (observed at least {observed_at_least} bytes)",
                    path.display()
                )
            }
            Self::AlreadyLocked(path) => {
                write!(f, "file lock is already held: '{}'", path.display())
            }
            Self::Serialization { format, operation, path, message } => {
                write!(f, "failed to {operation} {format} file '{}': {message}", path.display())
            }
            Self::Encoding { path, encoding, message } => {
                write!(f, "failed to decode '{}' as {encoding}: {message}", path.display())
            }
            Self::Task { operation, message } => {
                write!(f, "file system task failed while attempting to {operation}: {message}")
            }
        }
    }
}

impl std::error::Error for FsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl FsError {
    /// 将 I/O 错误与操作名称和受影响的路径一同包装。
    pub(crate) fn io(
        operation: &'static str,
        path: impl Into<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::Io { operation, path: path.into(), source }
    }

    /// 构建一个详细的结构化数据错误。
    pub(crate) fn serialization(
        format: &'static str,
        operation: &'static str,
        path: impl Into<PathBuf>,
        message: impl Into<String>,
    ) -> Self {
        Self::Serialization {
            format,
            operation,
            path: path.into(),
            message: message.into(),
        }
    }

    /// 为失败的阻塞操作构建错误。
    pub(crate) fn task(operation: &'static str, message: impl Into<String>) -> Self {
        Self::Task { operation, message: message.into() }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;

    #[test]
    fn io_error_includes_operation_and_path() {
        let error = FsError::io(
            "read file",
            "cache/state.json",
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
        );

        assert_eq!(error.to_string(), "failed to read file 'cache/state.json': missing");
        assert!(error.source().is_some());
    }
}
