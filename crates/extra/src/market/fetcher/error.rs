//! 资源获取器（Fetcher）模块的错误类型。
//!
//! 定义了 [`FetcherError`] 枚举，用于表示在资源获取过程中可能出现的各类错误，
//! 并提供了向 [`MarketError`] 的自动转换。

use std::fmt;

/// 资源获取器的错误类型。
#[derive(Debug)]
pub enum FetcherError {
    /// 文件下载失败。
    ///
    /// 可能的原因包括：网络连接异常、URL 无效、磁盘写入失败等。
    /// 内部包含具体的错误描述信息。
    Download(String),
}

impl fmt::Display for FetcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetcherError::Download(msg) => write!(f, "download error: {}", msg),
        }
    }
}

impl std::error::Error for FetcherError {}

impl From<FetcherError> for crate::market::error::MarketError {
    fn from(err: FetcherError) -> Self {
        match err {
            FetcherError::Download(msg) => crate::market::error::MarketError::Download(msg),
        }
    }
}
