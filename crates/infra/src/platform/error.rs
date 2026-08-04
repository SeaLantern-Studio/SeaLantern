use std::fmt;
use std::path::PathBuf;

/// 平台基础设施操作返回的错误。
#[derive(Debug)]
pub enum PlatformError {
    /// 无法读取 CA 证书文件。
    ReadCertificate {
        path: PathBuf,
        source: std::io::Error,
    },
    /// PEM 内容不是有效的 CA 证书束。
    InvalidCertificate { message: String },
    /// 当前平台不支持请求的系统操作。
    Unsupported { operation: &'static str },
    /// 系统命令无法启动或以失败状态退出。
    Command {
        operation: &'static str,
        source: std::io::Error,
    },
    /// 系统命令返回了无法解释的输出。
    InvalidCommandOutput { operation: &'static str },
}

impl fmt::Display for PlatformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadCertificate { path, source } => {
                write!(formatter, "failed to read CA certificate '{}': {source}", path.display())
            }
            Self::InvalidCertificate { message } => {
                write!(formatter, "invalid CA certificate bundle: {message}")
            }
            Self::Unsupported { operation } => {
                write!(formatter, "{operation} is not supported on this platform")
            }
            Self::Command { operation, source } => {
                write!(formatter, "failed to {operation}: {source}")
            }
            Self::InvalidCommandOutput { operation } => {
                write!(formatter, "{operation} returned an unexpected result")
            }
        }
    }
}

impl std::error::Error for PlatformError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadCertificate { source, .. } | Self::Command { source, .. } => Some(source),
            _ => None,
        }
    }
}
