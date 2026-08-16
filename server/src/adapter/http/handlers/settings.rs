//! 设置信息 REST handler。
//!
//! 提供设置概览等接口，薄转发到
//! [`CoreSettingsService`](sealantern_application::service::CoreSettingsService)
//! 并收敛错误为 [`HttpError`](super::super::error::HttpError)。

use axum::extract::State;
use axum::Json;

use sealantern_interface::settings::SettingsOverview;
use sealantern_interface::SettingsService;

use super::super::error::HttpError;
use super::super::state::AppState;

/// `GET /api/settings` — 获取设置概览。
pub async fn settings_overview(
    State(state): State<AppState>,
) -> Result<Json<SettingsOverview>, HttpError> {
    state
        .settings()
        .settings_overview()
        .await
        .map(Json)
        .map_err(HttpError::from)
}
