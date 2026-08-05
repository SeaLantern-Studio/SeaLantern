//! 服务器管理领域的主错误。
//!
//! 当前占位：待服务器控制台/生命周期服务接入后补充领域错误与契约转换。

use std::fmt;

/// 服务器管理操作失败的应用层主错误（占位）。
#[derive(Debug)]
pub enum ServerError {
    /// 该能力尚未实现（占位）。
    Unsupported,
}

impl fmt::Display for ServerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported => write!(formatter, "operation not supported"),
        }
    }
}

impl std::error::Error for ServerError {}