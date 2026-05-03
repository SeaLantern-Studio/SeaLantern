use std::sync::Arc;

use crate::models::download::TaskStatus;
use crate::utils::downloader::DownloadStatus;
use uuid::Uuid;

use super::state::{create_task_state, DownloadTaskState};
use super::DownloadManager;

pub(super) async fn create_task(
    manager: &DownloadManager,
    url: &str,
    path: &str,
    thread_count: usize,
) -> Uuid {
    let id = Uuid::new_v4();
    let state = create_task_state(url, path);

    manager.tasks.write().await.insert(id, state.clone());

    let downloader = manager.downloader.clone();
    let url_str = url.to_string();
    let path_str = path.to_string();
    let state_clone = state.clone();

    let handle = tokio::spawn(async move {
        match downloader.download(&url_str, &path_str, thread_count).await {
            Ok(handle) => {
                bind_status_handle(&state_clone, handle).await;
                watch_download_finish(&state_clone).await;
            }
            Err(err) => {
                let mut status = state_clone.internal_status.write().await;
                *status = TaskStatus::Error(err);
            }
        }
    });

    {
        let mut join_handle_lock = state.join_handle.lock().await;
        *join_handle_lock = Some(handle);
    }

    id
}

pub(super) async fn cancel_task(manager: &DownloadManager, id: Uuid) -> Result<(), String> {
    let (state, file_path) = {
        let tasks = manager.tasks.read().await;
        if let Some(state) = tasks.get(&id) {
            (Some(state.clone()), Some(state._file_path.clone()))
        } else {
            (None, None)
        }
    };

    let Some(state) = state else {
        return Ok(());
    };
    let file_path = file_path.expect("file_path must exist when state exists");

    {
        let handle = state.status_handle.lock().await;
        if let Some(ref status_handle) = *handle {
            status_handle.cancel();
        }
    }

    {
        let mut join_handle_guard = state.join_handle.lock().await;
        if let Some(handle) = join_handle_guard.take() {
            handle.abort();
        }
    }

    {
        let mut tasks = manager.tasks.write().await;
        tasks.remove(&id);
    }

    if let Err(err) = tokio::fs::remove_file(&file_path).await {
        if err.kind() != std::io::ErrorKind::NotFound {
            return Err(format!("删除临时文件失败: {}", err));
        }
    }

    Ok(())
}

async fn bind_status_handle(state: &Arc<DownloadTaskState>, handle: Arc<DownloadStatus>) {
    let mut status_handle = state.status_handle.lock().await;
    *status_handle = Some(handle);

    let mut status = state.internal_status.write().await;
    *status = TaskStatus::Downloading;
}

async fn watch_download_finish(state: &Arc<DownloadTaskState>) {
    loop {
        let status_handle_opt = {
            let handle = state.status_handle.lock().await;
            handle.as_ref().cloned()
        };

        let mut is_done = false;

        if let Some(status_handle) = status_handle_opt {
            let snap = status_handle.snapshot().await;

            if let Some(err_msg) = snap.error {
                let mut status = state.internal_status.write().await;
                *status = TaskStatus::Error(err_msg);
                break;
            }

            if snap.is_finished {
                is_done = true;
            }
        }

        if is_done {
            let mut status = state.internal_status.write().await;
            if let TaskStatus::Downloading = *status {
                *status = TaskStatus::Completed;
            }
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
}
