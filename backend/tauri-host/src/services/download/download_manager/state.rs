//! 下载任务状态子模块

use crate::models::download::{TaskProgressResponse, TaskStatus};
use crate::utils::downloader::DownloadStatus;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

/// 单个下载任务的内部状态
pub(super) struct DownloadTaskState {
    pub(super) file_path: String,
    pub(super) status_handle: tokio::sync::Mutex<Option<Arc<DownloadStatus>>>,
    pub(super) internal_status: RwLock<TaskStatus>,
    pub(super) join_handle: tokio::sync::Mutex<Option<JoinHandle<()>>>,
}

/// 组装前端需要的任务进度结构
pub(super) async fn build_task_progress_response(
    id: Uuid,
    state: &Arc<DownloadTaskState>,
) -> TaskProgressResponse {
    let mut status = state.internal_status.read().await.clone();
    let mut progress = 0.0;
    let mut total_size = 0;
    let mut downloaded = 0;

    let status_handle_opt = {
        let handle = state.status_handle.lock().await;
        handle.as_ref().cloned()
    };

    if let Some(handle) = status_handle_opt {
        let snap = handle.snapshot().await;

        if let Some(err_msg) = snap.error {
            status = TaskStatus::Error(err_msg);
        } else {
            progress = snap.progress_percentage;
            total_size = snap.total_size;
            downloaded = snap.downloaded;
            if snap.is_finished {
                status = TaskStatus::Completed;
            }
        }
    }
    let is_finished = matches!(status, TaskStatus::Completed | TaskStatus::Error(_));

    TaskProgressResponse {
        id,
        total_size,
        downloaded,
        progress,
        status,
        is_finished,
    }
}
