//! 应用更新检查 REST handler。

use axum::Json;
use axum::extract::State;

use sealantern_interface::UpdateCheckService;
use sealantern_interface::update::UpdateInfo;

use super::super::error::HttpError;
use super::super::state::AppState;

/// `GET /api/update` — 检查当前平台是否存在应用更新。
pub async fn check_update(State(state): State<AppState>) -> Result<Json<UpdateInfo>, HttpError> {
    state
        .update()
        .check()
        .await
        .map(Json)
        .map_err(HttpError::from)
}
