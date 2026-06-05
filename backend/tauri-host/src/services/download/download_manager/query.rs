use std::sync::Arc;

use crate::models::download::TaskProgressResponse;
use uuid::Uuid;

use super::state::{
    build_task_progress_response, collect_task_progress_responses, DownloadTaskState,
};
use super::DownloadManager;

pub(super) async fn get_progress(
    manager: &DownloadManager,
    id: Uuid,
) -> Option<TaskProgressResponse> {
    let state = {
        let tasks = manager.tasks.read().await;
        tasks.get(&id).cloned()?
    };

    Some(build_task_progress_response(id, &state).await)
}

pub(super) async fn get_all_progress(manager: &DownloadManager) -> Vec<TaskProgressResponse> {
    let task_entries: Vec<(Uuid, Arc<DownloadTaskState>)> = {
        let tasks = manager.tasks.read().await;
        tasks
            .iter()
            .map(|(id, state)| (*id, state.clone()))
            .collect()
    };
    let (results, to_remove) = collect_task_progress_responses(task_entries).await;

    if !to_remove.is_empty() {
        let mut tasks_write = manager.tasks.write().await;
        for id in to_remove {
            tasks_write.remove(&id);
        }
    }

    results
}

pub(super) async fn get_progress_and_remove(
    manager: &DownloadManager,
    id: Uuid,
) -> Option<TaskProgressResponse> {
    let resp = get_progress(manager, id).await?;

    if resp.is_finished {
        let mut tasks = manager.tasks.write().await;
        tasks.remove(&id);
    }

    Some(resp)
}
