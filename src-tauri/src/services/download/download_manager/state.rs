//! 下载任务状态子模块

use crate::models::download::{TaskProgressResponse, TaskStatus};
use crate::utils::downloader::DownloadStatus;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

pub(super) type SharedDownloadTasks = Arc<RwLock<HashMap<Uuid, Arc<DownloadTaskState>>>>;

/// 创建共享任务表
pub(super) fn new_shared_download_tasks() -> SharedDownloadTasks {
    Arc::new(RwLock::new(HashMap::new()))
}

/// 单个下载任务的内部状态
pub(super) struct DownloadTaskState {
    pub(super) file_path: String,
    pub(super) status_handle: tokio::sync::Mutex<Option<Arc<DownloadStatus>>>,
    pub(super) internal_status: RwLock<TaskStatus>,
    pub(super) join_handle: tokio::sync::Mutex<Option<JoinHandle<()>>>,
}

/// 创建一个初始下载任务状态
pub(super) fn create_task_state(url: &str, path: &str) -> Arc<DownloadTaskState> {
    let _ = url;
    Arc::new(DownloadTaskState {
        file_path: path.to_string(),
        status_handle: tokio::sync::Mutex::new(None),
        internal_status: RwLock::new(TaskStatus::Pending),
        join_handle: tokio::sync::Mutex::new(None),
    })
}

/// 读取单个任务的当前快照
async fn snapshot_task_state(state: &Arc<DownloadTaskState>) -> (TaskStatus, f64, u64, u64) {
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

    (status, progress, total_size, downloaded)
}

/// 组装前端需要的任务进度结构
pub(super) async fn build_task_progress_response(
    id: Uuid,
    state: &Arc<DownloadTaskState>,
) -> TaskProgressResponse {
    let (status, progress, total_size, downloaded) = snapshot_task_state(state).await;
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

/// 批量读取任务进度，并返回需要清理的任务 ID
pub(super) async fn collect_task_progress_responses(
    task_entries: Vec<(Uuid, Arc<DownloadTaskState>)>,
) -> (Vec<TaskProgressResponse>, Vec<Uuid>) {
    let mut results = Vec::new();
    let mut to_remove = Vec::new();

    for (id, state) in task_entries {
        let response = build_task_progress_response(id, &state).await;
        if response.is_finished {
            to_remove.push(id);
        }
        results.push(response);
    }

    (results, to_remove)
}
