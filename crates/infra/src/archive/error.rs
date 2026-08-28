use std::fmt;
use std::path::PathBuf;

use zip::result::ZipError;

/// 归档操作返回的错误。
///
/// 大部分变体与具体归档格式无关（路径校验、资源上限、目标冲突等），
/// 仅 [`ArchiveError::Zip`] 承载 ZIP 特有的底层错误。新增格式时应
/// 追加各自的底层错误变体，而不是复用 `Zip`。
#[derive(Debug)]
pub enum ArchiveError {
    /// 操作系统文件操作失败。
    Io {
        operation: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    /// ZIP 格式无法读取或写入。
    Zip {
        operation: &'static str,
        path: PathBuf,
        source: ZipError,
    },
    /// 请求的归档源不是一个常规目录。
    InvalidSource { path: PathBuf, reason: &'static str },
    /// 解压目标不是一个全新的普通目录。
    InvalidDestination { path: PathBuf, reason: &'static str },
    /// 输出归档或解压目标已存在。
    DestinationExists { path: PathBuf },
    /// 源条目无法安全地表示在可移植的归档中。
    UnsupportedSourceEntry { path: PathBuf, kind: &'static str },
    /// 归档条目名会逃逸或以其他方式违反解压根目录。
    UnsafeEntry {
        archive: PathBuf,
        entry: String,
        reason: String,
    },
    /// 归档条目使用了此 API 有意不支持的类型。
    UnsupportedEntry {
        archive: PathBuf,
        entry: String,
        kind: &'static str,
    },
    /// 归档元数据或流式内容超过了配置的资源限制。
    LimitExceeded {
        archive: PathBuf,
        limit: &'static str,
        observed: u64,
        maximum: u64,
    },
    /// 符号链接载荷不是一个可移植的安全相对路径。
    InvalidSymbolicLinkTarget { reason: &'static str },
    /// 符号链接载荷对于特定的归档条目不安全。
    InvalidSymbolicLinkTargetEntry {
        archive: PathBuf,
        entry: String,
        reason: &'static str,
    },
    /// 无法从归档中读取符号链接载荷。
    SymbolicLinkTargetRead {
        archive: PathBuf,
        entry: String,
        source: std::io::Error,
    },
}

impl ArchiveError {
    pub(crate) fn io(
        operation: &'static str,
        path: impl Into<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::Io { operation, path: path.into(), source }
    }

    pub(crate) fn zip(operation: &'static str, path: impl Into<PathBuf>, source: ZipError) -> Self {
        Self::Zip { operation, path: path.into(), source }
    }

    pub(crate) fn entry(&self) -> Option<&str> {
        match self {
            Self::UnsafeEntry { entry, .. }
            | Self::UnsupportedEntry { entry, .. }
            | Self::InvalidSymbolicLinkTargetEntry { entry, .. }
            | Self::SymbolicLinkTargetRead { entry, .. } => Some(entry),
            _ => None,
        }
    }
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, path, source } => {
                write!(formatter, "failed to {operation} '{}': {source}", path.display())
            }
            Self::Zip { operation, path, source } => write!(
                formatter,
                "failed to {operation} ZIP archive '{}': {source}",
                path.display()
            ),
            Self::InvalidSource { path, reason } => {
                write!(formatter, "invalid archive source '{}': {reason}", path.display())
            }
            Self::InvalidDestination { path, reason } => {
                write!(formatter, "invalid archive destination '{}': {reason}", path.display())
            }
            Self::DestinationExists { path } => {
                write!(formatter, "archive destination already exists: '{}'", path.display())
            }
            Self::UnsupportedSourceEntry { path, kind } => write!(
                formatter,
                "cannot add {kind} source entry '{}' to a portable archive",
                path.display()
            ),
            Self::UnsafeEntry { archive, entry, reason } => write!(
                formatter,
                "unsafe archive entry '{entry}' in '{}': {reason}",
                archive.display()
            ),
            Self::UnsupportedEntry { archive, entry, kind } => write!(
                formatter,
                "unsupported {kind} archive entry '{entry}' in '{}'",
                archive.display()
            ),
            Self::LimitExceeded { archive, limit, observed, maximum } => write!(
                formatter,
                "archive '{}' exceeds the {limit} limit: {observed} > {maximum}",
                archive.display()
            ),
            Self::InvalidSymbolicLinkTarget { reason } => {
                write!(formatter, "invalid archive symbolic-link target: {reason}")
            }
            Self::InvalidSymbolicLinkTargetEntry { archive, entry, reason } => {
                write!(
                    formatter,
                    "invalid symbolic-link target for archive entry '{entry}' in '{}': {reason}",
                    archive.display()
                )
            }
            Self::SymbolicLinkTargetRead { archive, entry, source } => {
                write!(
                    formatter,
                    "failed to read symbolic-link target for archive entry '{entry}' in '{}': {source}",
                    archive.display()
                )
            }
        }
    }
}

impl std::error::Error for ArchiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Zip { source, .. } => Some(source),
            Self::SymbolicLinkTargetRead { source, .. } => Some(source),
            _ => None,
        }
    }
}
