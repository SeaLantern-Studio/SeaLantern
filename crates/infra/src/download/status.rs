//! 下载状态和错误类型。
//!
//! 提供下载任务的共享状态 `DownloadStatus`、进度快照 `DownloadSnapshot`
//! 和统一的错误类型 `DownloadError`。

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::net::error::NetError;
use crate::observability;

/// 下载被主动取消时携带的默认说明。
///
/// `Cancelled` 保留字符串载荷以维持公开 API 兼容；各调用点都只需这一句恒定
/// 说明，因此集中在此避免重复字面量。
pub const CANCELLED_MESSAGE: &str = "download cancelled";

/// 下载错误的类别。
///
/// 与 `DownloadError` 的具体错误载荷解耦：载荷是给日志和排查用的英文诊断
/// 文本，类别才是宿主做本地化的依据。宿主按类别查自己的文案表，不要去
/// 匹配诊断文本。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadErrorKind {
    /// 下载被主动取消（用户操作，不是故障）。
    Cancelled,
    /// 网络传输失败：请求错误或网络层错误。
    Network,
    /// 服务端返回了意外的状态码。
    Server,
    /// 本地文件读写失败。
    Io,
    /// 其他未能细分的失败。
    Other,
}

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
            DownloadError::Reqwest(e) => write!(f, "request error: {}", e),
            DownloadError::Io(e) => write!(f, "I/O error: {}", e),
            DownloadError::Response(code, body) => write!(f, "server returned {}: {}", code, body),
            DownloadError::Net(e) => write!(f, "network error: {}", e),
            DownloadError::Cancelled(msg) => write!(f, "download cancelled: {}", msg),
            DownloadError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DownloadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DownloadError::Reqwest(err) => Some(err),
            DownloadError::Io(err) => Some(err),
            DownloadError::Net(err) => Some(err),
            DownloadError::Response(..)
            | DownloadError::Message(_)
            | DownloadError::Cancelled(_) => None,
        }
    }
}

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

impl DownloadError {
    /// 返回该错误所属的类别，供宿主做本地化。
    ///
    /// 取消与真实失败在类别上就是分开的，调用方不需要比较任何文本。
    pub fn kind(&self) -> DownloadErrorKind {
        match self {
            DownloadError::Cancelled(_) => DownloadErrorKind::Cancelled,
            DownloadError::Reqwest(_) | DownloadError::Net(_) => DownloadErrorKind::Network,
            DownloadError::Response(..) => DownloadErrorKind::Server,
            DownloadError::Io(_) => DownloadErrorKind::Io,
            DownloadError::Message(_) => DownloadErrorKind::Other,
        }
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
    /// 错误诊断文本，**固定为英文**，仅用于日志与排查。
    ///
    /// 不要把它直接展示给用户：宿主应改用 [`DownloadSnapshot::error_kind`]
    /// 查自己的文案表，不要去匹配这里的文本。
    pub error: Option<String>,
    /// 错误的类别，宿主做本地化的依据；无错误时为 `None`。
    ///
    /// 取消与真实失败在此已分开，调用方无需比较任何文本。
    pub error_kind: Option<DownloadErrorKind>,
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
    /// 是否已下载完成（下载主体结束后置位，与 total_size 无关）
    pub(super) completed: AtomicBool,
    /// 错误信息
    pub(super) error_message: RwLock<Option<String>>,
    /// 错误类别（与 `error_message` 同源写入，保证二者一致）
    pub(super) error_kind: RwLock<Option<DownloadErrorKind>>,
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
            completed: AtomicBool::new(false),
            error_message: RwLock::new(None),
            error_kind: RwLock::new(None),
            cancel_token: CancellationToken::new(),
        }
    }

    /// 标记下载已完成（下载主体结束后调用，供进度查询判定）。
    pub fn mark_completed(&self) {
        self.completed.store(true, Ordering::Relaxed);
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
        self.set_error_with_kind(msg, DownloadErrorKind::Other)
            .await;
    }

    /// 设置错误信息并记录其类别。
    ///
    /// 调用方有结构化错误时应走这里，让宿主能按类别本地化；只有拿不到
    /// 类别时才退化为 [`DownloadStatus::set_error`]（落为
    /// [`DownloadErrorKind::Other`]）。
    ///
    /// # Parameters
    ///
    /// - `msg`: 错误描述
    /// - `kind`: 错误类别
    pub async fn set_error_with_kind(&self, msg: String, kind: DownloadErrorKind) {
        observability::download_error(&msg);
        // 两把锁始终按「类别 → 文本」的固定顺序获取，读取方 ``snapshot``
        // 也按同一顺序获取，因此不会出现「有文本无类别」的中间态。
        let mut kind_guard = self.error_kind.write().await;
        let mut msg_guard = self.error_message.write().await;
        *kind_guard = Some(kind);
        *msg_guard = Some(msg);
    }

    /// 获取当前进度快照。
    ///
    /// 当取消时未设置特定错误时，`error` 字段返回取消消息，
    /// 确保前端可以通过 `is_finished` 检测到已取消的状态。
    pub async fn snapshot(&self) -> DownloadSnapshot {
        let downloaded = self.downloaded.load(Ordering::Relaxed);
        // 与 ``set_error_with_kind`` 保持同一加锁顺序，读到的一定是成对结果。
        let error_kind = *self.error_kind.read().await;
        let error = self.error_message.read().await.clone();

        let is_cancelled = self.cancelled() && error.is_none();

        DownloadSnapshot {
            downloaded,
            total_size: self.total_size,
            progress_percentage: if self.total_size > 0 {
                (downloaded as f64 / self.total_size as f64) * 100.0
            } else {
                // 未知总大小时：已完成显示 100，否则显示 0（避免瞬时误判完成）。
                if self.completed.load(Ordering::Relaxed) {
                    100.0
                } else {
                    0.0
                }
            },
            is_finished: self.completed.load(Ordering::Relaxed) || error.is_some() || is_cancelled,
            error: if is_cancelled {
                Some(CANCELLED_MESSAGE.to_string())
            } else {
                error
            },
            // 取消状态即使没有显式错误文本也带类别，宿主因此能统一按类别取文案。
            error_kind: if is_cancelled {
                Some(DownloadErrorKind::Cancelled)
            } else {
                error_kind
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
        status.mark_completed();
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
        assert_eq!(snap.error.unwrap(), CANCELLED_MESSAGE);
        assert_eq!(snap.error_kind, Some(DownloadErrorKind::Cancelled));
    }

    #[tokio::test]
    async fn snapshot_without_error_has_no_kind() {
        let status = DownloadStatus::new(100);
        let snap = status.snapshot().await;
        assert!(snap.error.is_none());
        assert!(snap.error_kind.is_none());
    }

    #[tokio::test]
    async fn snapshot_carries_error_kind() {
        let status = DownloadStatus::new(100);
        status
            .set_error_with_kind("server returned 500: boom".to_string(), DownloadErrorKind::Server)
            .await;
        let snap = status.snapshot().await;
        assert_eq!(snap.error.as_deref(), Some("server returned 500: boom"));
        assert_eq!(snap.error_kind, Some(DownloadErrorKind::Server));
    }

    #[tokio::test]
    async fn set_error_without_kind_defaults_to_other() {
        let status = DownloadStatus::new(100);
        status.set_error("something odd".to_string()).await;
        let snap = status.snapshot().await;
        assert_eq!(snap.error_kind, Some(DownloadErrorKind::Other));
    }

    #[test]
    fn download_error_kind_classifies_each_variant() {
        assert_eq!(
            DownloadError::Cancelled(CANCELLED_MESSAGE.to_string()).kind(),
            DownloadErrorKind::Cancelled
        );
        assert_eq!(
            DownloadError::Response(404, "missing".to_string()).kind(),
            DownloadErrorKind::Server
        );
        assert_eq!(DownloadError::Message("boom".to_string()).kind(), DownloadErrorKind::Other);
        assert_eq!(DownloadError::Io(std::io::Error::other("disk")).kind(), DownloadErrorKind::Io);
        assert_eq!(
            DownloadError::Net(NetError::Config("bad proxy".to_string())).kind(),
            DownloadErrorKind::Network
        );
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
        assert_eq!(snap_after.error.unwrap(), CANCELLED_MESSAGE);
        assert_eq!(snap_after.error_kind, Some(DownloadErrorKind::Cancelled));
        assert_eq!(snap_after.downloaded, 300);
    }
}
