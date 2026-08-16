//! 下载任务管理 REST handler。
//!
//! 提供下载任务创建、进度查询与取消接口，薄转发到
//! [`CoreDownloadService`](sealantern_application::service::CoreDownloadService)
//! 并收敛错误为 [`HttpError`](super::super::error::HttpError)。

use axum::extract::{Path, State};
use axum::Json;

use sealantern_interface::download::{DownloadRequest, DownloadTaskInfo};
use sealantern_interface::DownloadService;

use super::super::error::HttpError;
use super::super::state::AppState;

/// 创建下载任务请求体（snake_case 与后端契约一致）。
#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateDownloadBody {
    url: String,
    save_path: String,
    #[serde(default = "default_thread_count")]
    thread_count: usize,
}

fn default_thread_count() -> usize {
    32
}

/// `POST /api/downloads` — 创建下载任务，返回任务信息。
pub async fn create_download(
    State(state): State<AppState>,
    Json(body): Json<CreateDownloadBody>,
) -> Result<Json<DownloadTaskInfo>, HttpError> {
    let id = state
        .download()
        .create(DownloadRequest {
            url: body.url,
            save_path: body.save_path,
            thread_count: body.thread_count,
        })
        .await
        .map_err(HttpError::from)?;
    state
        .download()
        .poll(&id)
        .await?
        .ok_or(HttpError::from(sealantern_interface::DownloadServiceError::TaskNotFound))
        .map(Json)
}

/// `GET /api/downloads/{id}` — 查询下载任务进度。
pub async fn query_download(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DownloadTaskInfo>, HttpError> {
    state
        .download()
        .poll(&id)
        .await?
        .ok_or(HttpError::from(sealantern_interface::DownloadServiceError::TaskNotFound))
        .map(Json)
}

/// `DELETE /api/downloads/{id}` — 取消下载任务。
pub async fn cancel_download(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(), HttpError> {
    state.download().cancel(&id).await.map_err(HttpError::from)
}
