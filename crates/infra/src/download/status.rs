//! 下载状态和错误类型。
//!
//! 提供下载任务的共享状态 `DownloadStatus`、进度快照 `DownloadSnapshot`
//! 和统一的错误类型 `DownloadError`。

use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::net::error::NetError;
use crate::observability;

/// 下载过程中可能发生的错误。
#[derive(Debug)]
pub enum DownloadError {
    /// HTTP 请求失败
    Reqwest(reqwest::Error),
    /// 本地文件读写失败
    Io(std::io::Error),
    /// 服务端返回了意外的状态码
    Response(u16, String),
    /// 网络层错误
    Net(NetError),
    /// 下载被主动取消
    Cancelled(String),
    /// 其他错误信息
    Message(String),
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::Reqwest(e) => write!(f, "请求错误: {}", e),
            DownloadError::Io(e) => write!(f, "IO 错误: {}", e),
            DownloadError::Response(code, body) => write!(f, "服务端返回 {}: {}", code, body),
            DownloadError::Net(e) => write!(f, "网络错误: {}", e),
            DownloadError::Cancelled(msg) => write!(f, "下载取消: {}", msg),
            DownloadError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DownloadError {}

impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        DownloadError::Reqwest(err)
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(err: std::io::Error) -> Self {
        DownloadError::Io(err)
    }
}

impl From<NetError> for DownloadError {
    fn from(err: NetError) -> Self {
        DownloadError::Net(err)
    }
}

/// 下载进度的实时快照，用于传递给前端。
pub struct DownloadSnapshot {
    /// 已下载的字节数
    pub downloaded: u64,
    /// 文件总大小
    pub total_size: u64,
    /// 当前进度百分比（0.0 ~ 100.0）
    pub progress_percentage: f64,
    /// 是否已完成（完成、出错或取消）
    pub is_finished: bool,
    /// 错误信息（取消时包含取消消息）
    pub error: Option<String>,
}

/// 单个下载任务的共享状态。
///
/// 使用 `AtomicU64` 跟踪已下载字节数，多个分段线程可安全地并发累加。
/// 通过 `CancellationToken` 实现取消信号。
///
/// 字段在 `download` 模块内可见（由分段层和传输层直接访问），
/// 外部代码通过 `snapshot()` / `cancel()` 等公共方法交互。
pub struct DownloadStatus {
    /// 文件总大小
    pub(super) total_size: u64,
    /// 已下载的字节数（原子计数器，线程安全）
    pub(super) downloaded: AtomicU64,
    /// 错误信息
    pub(super) error_message: RwLock<Option<String>>,
    /// 取消令牌
    pub(super) cancel_token: CancellationToken,
}

impl DownloadStatus {
    /// 创建新的下载状态。
    ///
    /// # Parameters
    ///
    /// - `total_size`: 文件总大小
    pub fn new(total_size: u64) -> Self {
        Self {
            total_size,
            downloaded: AtomicU64::new(0),
            error_message: RwLock::new(None),
            cancel_token: CancellationToken::new(),
        }
    }

    /// 取消下载。
    pub fn cancel(&self) {
        observability::download_cancelled();
        self.cancel_token.cancel();
    }

    /// 检查下载是否已被取消。
    pub fn cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// 设置错误信息。
    ///
    /// # Parameters
    ///
    /// - `msg`: 错误描述
    pub async fn set_error(&self, msg: String) {
        observability::download_error(&msg);
        let mut lock = self.error_message.write().await;
        *lock = Some(msg);
    }

    /// 获取当前进度快照。
    ///
    /// 当取消时未设置特定错误时，`error` 字段返回取消消息，
    /// 确保前端可以通过 `is_finished` 检测到已取消的状态。
    pub async fn snapshot(&self) -> DownloadSnapshot {
        let downloaded = self.downloaded.load(Ordering::Relaxed);
        let error = self.error_message.read().await.clone();

        let is_cancelled = self.cancelled() && error.is_none();

        DownloadSnapshot {
            downloaded,
            total_size: self.total_size,
            progress_percentage: if self.total_size > 0 {
                (downloaded as f64 / self.total_size as f64) * 100.0
            } else {
                0.0
            },
            is_finished: downloaded >= self.total_size || error.is_some() || is_cancelled,
            error: if is_cancelled {
                Some("下载已取消".to_string())
            } else {
                error
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn snapshot_normal() {
        let status = DownloadStatus::new(100);
        let snap = status.snapshot().await;
        assert!(!snap.is_finished);
        assert_eq!(snap.total_size, 100);
        assert_eq!(snap.downloaded, 0);
    }

    #[tokio::test]
    async fn snapshot_completed() {
        let status = DownloadStatus::new(100);
        status.downloaded.store(100, Ordering::Relaxed);
        let snap = status.snapshot().await;
        assert!(snap.is_finished);
        assert!(snap.error.is_none());
    }

    #[tokio::test]
    async fn snapshot_cancelled_reflects_finished() {
        let status = DownloadStatus::new(100);
        status.cancel();
        let snap = status.snapshot().await;
        assert!(snap.is_finished);
        assert_eq!(snap.error.unwrap(), "下载已取消");
    }

    #[tokio::test]
    async fn snapshot_partial_download_then_cancel() {
        let status = DownloadStatus::new(1000);
        status.downloaded.store(300, Ordering::Relaxed);
        let snap_before = status.snapshot().await;
        assert!(!snap_before.is_finished);

        status.cancel();
        let snap_after = status.snapshot().await;
        assert!(snap_after.is_finished);
        assert_eq!(snap_after.error.unwrap(), "下载已取消");
        assert_eq!(snap_after.downloaded, 300);
    }
}
