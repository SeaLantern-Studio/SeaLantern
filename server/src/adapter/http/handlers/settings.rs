//! 设置信息 REST handler。
//!
//! 提供设置概览等接口，薄转发到
//! [`CoreSettingsService`](sealantern_application::service::CoreSettingsService)
//! 并收敛错误为 [`HttpError`](super::super::error::HttpError)。

use axum::Json;
use axum::extract::State;

use sealantern_extra::models::AppSettings;
use sealantern_interface::SettingsService;
use sealantern_interface::settings::SettingsOverview;

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

/// `GET /api/settings/all` — 获取当前完整设置。
pub async fn get_settings(State(state): State<AppState>) -> Result<Json<AppSettings>, HttpError> {
    state
        .settings()
        .get()
        .await
        .map(Json)
        .map_err(HttpError::from)
}
