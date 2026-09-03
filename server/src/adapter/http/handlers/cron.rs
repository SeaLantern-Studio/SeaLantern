//! 服务器定时任务 REST handler。

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;

use sealantern_interface::CronTaskService;
use sealantern_interface::cron::{CronTask, CronTaskDraft, CronTaskRun};

use super::super::error::HttpError;
use super::super::state::AppState;

/// 启停定时任务请求体。
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SetEnabledRequest {
    pub enabled: bool,
}

/// `GET /api/cron-tasks` — 列出全部定时任务。
pub async fn list_cron_tasks(
    State(state): State<AppState>,
) -> Result<Json<Vec<CronTask>>, HttpError> {
    state.cron().list().await.map(Json).map_err(HttpError::from)
}

/// `POST /api/cron-tasks` — 创建定时任务。
pub async fn create_cron_task(
    State(state): State<AppState>,
    Json(draft): Json<CronTaskDraft>,
) -> Result<(StatusCode, Json<CronTask>), HttpError> {
    let task = state.cron().create(draft).await?;
    Ok((StatusCode::CREATED, Json(task)))
}

/// `PUT /api/cron-tasks/{id}` — 更新定时任务。
pub async fn update_cron_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(draft): Json<CronTaskDraft>,
) -> Result<Json<CronTask>, HttpError> {
    state
        .cron()
        .update(&id, draft)
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `DELETE /api/cron-tasks/{id}` — 删除定时任务。
pub async fn delete_cron_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, HttpError> {
    state.cron().delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `PUT /api/cron-tasks/{id}/enabled` — 启用或禁用定时任务。
pub async fn set_cron_task_enabled(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<SetEnabledRequest>,
) -> Result<Json<CronTask>, HttpError> {
    state
        .cron()
        .set_enabled(&id, request.enabled)
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `POST /api/cron-tasks/{id}/run` — 立即执行一次定时任务。
pub async fn run_cron_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CronTaskRun>, HttpError> {
    state
        .cron()
        .run_now(&id)
        .await
        .map(Json)
        .map_err(HttpError::from)
}
