//! Download status and error types.
//!
//! Provides shared state `DownloadStatus` for download tasks, progress snapshot `DownloadSnapshot`
//! and unified error type `DownloadError`.

use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::net::error::NetError;
use crate::observability;

/// Errors that may occur during download.
#[derive(Debug)]
pub enum DownloadError {
    /// HTTP request failed
    Reqwest(reqwest::Error),
    /// Local file read/write failed
    Io(std::io::Error),
    /// Server returned unexpected status code
    Response(u16, String),
    /// Network layer error
    Net(NetError),
    /// Download was actively cancelled
    Cancelled(String),
    /// Other error messages
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

/// Real-time snapshot of download progress, used to pass to the frontend.
pub struct DownloadSnapshot {
    /// Bytes downloaded
    pub downloaded: u64,
    /// Total file size
    pub total_size: u64,
    /// Current progress percentage (0.0 ~ 100.0)
    pub progress_percentage: f64,
    /// Whether it has finished (completed, errored, or cancelled)
    pub is_finished: bool,
    /// Error message (includes cancellation message when cancelled)
    pub error: Option<String>,
}

/// Shared state for a single download task.
///
/// Uses `AtomicU64` to track downloaded bytes; safe for concurrent accumulation by multiple segment threads.
/// Implements cancellation signal via `CancellationToken`.
///
/// Fields are visible within the `download` module (directly accessed by segment and transport layers),
/// external code interacts via public methods like `snapshot()` / `cancel()`.
pub struct DownloadStatus {
    /// Total file size
    pub(super) total_size: u64,
    /// Bytes downloaded (atomic counter, thread-safe)
    pub(super) downloaded: AtomicU64,
    /// Error message
    pub(super) error_message: RwLock<Option<String>>,
    /// Cancellation token
    pub(super) cancel_token: CancellationToken,
}

impl DownloadStatus {
    /// Creates a new download status.
    ///
    /// # Parameters
    ///
    /// - `total_size`: Total file size
    pub fn new(total_size: u64) -> Self {
        Self {
            total_size,
            downloaded: AtomicU64::new(0),
            error_message: RwLock::new(None),
            cancel_token: CancellationToken::new(),
        }
    }

    /// Cancels the download.
    pub fn cancel(&self) {
        observability::download_cancelled();
        self.cancel_token.cancel();
    }

    /// Checks if the download has been cancelled.
    pub fn cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// Sets the error message.
    ///
    /// # Parameters
    ///
    /// - `msg`: Error description
    pub async fn set_error(&self, msg: String) {
        observability::download_error(&msg);
        let mut lock = self.error_message.write().await;
        *lock = Some(msg);
    }

    /// Gets the current progress snapshot.
    ///
    /// When cancelled without a specific error set, the `error` field returns the cancellation message,
    /// ensuring the frontend can detect the cancelled state via `is_finished`.
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
