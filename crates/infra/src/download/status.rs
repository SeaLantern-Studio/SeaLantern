//! 下载状态与错误类型。
//!
//! 提供下载任务的共享状态 `DownloadStatus`、进度快照 `DownloadSnapshot`
//! 以及统一的错误类型 `DownloadError`。

use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::observability;

/// 下载过程中可能出现的错误。
#[derive(Debug)]
pub enum DownloadError {
    /// HTTP 请求失败
    Reqwest(reqwest::Error),
    /// 本地文件读写失败
    Io(std::io::Error),
    /// 服务端返回非预期状态码
    Response(u16, String),
    /// 下载被主动取消
    Cancelled(String),
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::Reqwest(e) => write!(f, "请求错误: {}", e),
            DownloadError::Io(e) => write!(f, "IO 错误: {}", e),
            DownloadError::Response(code, body) => write!(f, "服务端返回 {}: {}", code, body),
            DownloadError::Cancelled(msg) => write!(f, "下载取消: {}", msg),
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

/// 下载进度的实时快照，用于传递给前端。
pub struct DownloadSnapshot {
    /// 已下载字节数
    pub downloaded: u64,
    /// 文件总大小
    pub total_size: u64,
    /// 当前百分比进度（0.0 ~ 100.0）
    pub progress_percentage: f64,
    /// 是否已经结束（完成或出错或取消）
    pub is_finished: bool,
    /// 错误信息（取消时也会包含取消提示）
    pub error: Option<String>,
}

/// 单个下载任务的共享状态。
///
/// 使用 `AtomicU64` 记录已下载量，多个分片线程可安全并发累加。
/// 通过 `CancellationToken` 实现取消信号。
///
/// 字段对 `download` 模块内可见（分片、传输层直接访问），
/// 外部通过 `snapshot()` / `cancel()` 等公开方法交互。
pub struct DownloadStatus {
    /// 文件总大小
    pub(super) total_size: u64,
    /// 已下载字节数（原子计数器，多线程安全）
    pub(super) downloaded: AtomicU64,
    /// 错误信息
    pub(super) error_message: RwLock<Option<String>>,
    /// 取消令牌
    pub(super) cancel_token: CancellationToken,
}

impl DownloadStatus {
    /// 创建下载状态对象。
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
        tracing::warn!(
            target: observability::DOWNLOAD_TARGET,
            event_name = "download_cancelled",
            "download cancelled by user"
        );
        self.cancel_token.cancel();
    }

    /// 判断是否已取消。
    pub fn cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// 设置错误信息。
    ///
    /// # Parameters
    ///
    /// - `msg`: 错误描述
    pub async fn set_error(&self, msg: String) {
        tracing::error!(
            target: observability::DOWNLOAD_TARGET,
            event_name = "download_error",
            error = %msg,
            "download error"
        );
        let mut lock = self.error_message.write().await;
        *lock = Some(msg);
    }

    /// 获取当前进度快照。
    ///
    /// 已取消但未设置具体错误时，`error` 字段会返回"下载已取消"，
    /// 确保前端能通过 `is_finished` 感知到取消状态。
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
