//! 旧版实例配置迁移 REST handler。
//!
//! 提供检测、执行与「不再提示」三个接口，薄转发到
//! [`LegacyMigrationService`](sealantern_application::port::LegacyMigrationService)
//! 并收敛错误为 [`HttpError`](super::super::error::HttpError)。

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use sealantern_application::port::LegacyMigrationService;
use sealantern_contract::legacy_migration::{LegacyMigrationReport, LegacyMigrationStatus};

use super::super::error::HttpError;
use super::super::state::AppState;

/// `GET /api/legacy-migration` — 检测是否存在可迁移的旧版配置。
pub async fn legacy_migration_status(
    State(state): State<AppState>,
) -> Result<Json<LegacyMigrationStatus>, HttpError> {
    state
        .legacy_migration()
        .status()
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `POST /api/legacy-migration` — 执行迁移。
pub async fn migrate_legacy_config(
    State(state): State<AppState>,
) -> Result<Json<LegacyMigrationReport>, HttpError> {
    state
        .legacy_migration()
        .migrate()
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `POST /api/legacy-migration/dismiss` — 记录用户「不再提示」。
pub async fn dismiss_legacy_migration(
    State(state): State<AppState>,
) -> Result<StatusCode, HttpError> {
    state.legacy_migration().dismiss().await?;
    Ok(StatusCode::NO_CONTENT)
}
