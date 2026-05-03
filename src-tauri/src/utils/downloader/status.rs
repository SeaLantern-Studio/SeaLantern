use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

/// 下载过程中的错误类型
#[derive(Debug)]
pub enum DownloadError {
    /// HTTP 请求失败
    Reqwest(reqwest::Error),
    /// 本地文件读写失败
    Io(std::io::Error),
    /// 下载被主动取消
    Cancelled(String),
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::Reqwest(e) => write!(f, "Request error: {}", e),
            DownloadError::Io(e) => write!(f, "IO error: {}", e),
            DownloadError::Cancelled(msg) => write!(f, "Download cancelled: {}", msg),
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

/// 实时进度快照
#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadSnapshot {
    /// 已下载字节数
    pub downloaded: u64,
    /// 文件总大小
    pub total_size: u64,
    /// 当前百分比进度
    pub progress_percentage: f64,
    /// 是否已经结束
    pub is_finished: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 单个下载任务的共享状态
pub struct DownloadStatus {
    /// 文件总大小
    pub total_size: u64,
    /// 已下载字节数
    pub downloaded: AtomicU64,
    /// 错误信息
    pub error_message: RwLock<Option<String>>,
    pub(crate) cancel_token: CancellationToken,
}

impl DownloadStatus {
    /// 创建下载状态对象
    pub fn new(total_size: u64) -> Self {
        Self {
            total_size,
            downloaded: AtomicU64::new(0),
            error_message: RwLock::new(None),
            cancel_token: CancellationToken::new(),
        }
    }

    /// 取消下载
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// 判断是否已取消
    pub fn cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// 设置错误信息
    pub async fn set_error(&self, msg: String) {
        let mut lock = self.error_message.write().await;
        *lock = Some(msg);
    }

    /// 获取当前快照，用于传递给前端
    pub async fn snapshot(&self) -> DownloadSnapshot {
        let downloaded = self.downloaded.load(Ordering::Relaxed);
        let error = self.error_message.read().await.clone();

        DownloadSnapshot {
            downloaded,
            total_size: self.total_size,
            progress_percentage: if self.total_size > 0 {
                (downloaded as f64 / self.total_size as f64) * 100.0
            } else {
                0.0
            },
            is_finished: downloaded >= self.total_size || error.is_some(),
            error,
        }
    }
}
