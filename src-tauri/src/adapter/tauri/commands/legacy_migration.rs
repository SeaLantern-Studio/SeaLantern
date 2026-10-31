//! 旧版实例配置迁移 Tauri 命令。
//!
//! 前端在启动后（`list_instances` 为空时）调用 `legacy_migration_status` 检测，
//! 由弹窗决定是否调用 `migrate_legacy_config`；用户选择「不再提示」时调用
//! `dismiss_legacy_migration`。
//!
//! 错误统一为接口契约错误 [`LegacyMigrationServiceError`]，可序列化回前端。

use std::sync::Arc;

use sealantern_application::port::LegacyMigrationService;
use sealantern_application::service::CoreLegacyMigrationService;
use sealantern_application::services::AppServices;
use sealantern_contract::LegacyMigrationServiceError;
use sealantern_contract::legacy_migration::{LegacyMigrationReport, LegacyMigrationStatus};
use tauri::State;

/// 获取宿主注入的迁移服务句柄。
fn migration_service(services: &AppServices) -> Arc<CoreLegacyMigrationService> {
    services.legacy_migration().clone()
}

/// 检测是否存在可迁移的旧版实例配置。
#[tauri::command(rename_all = "snake_case")]
pub async fn legacy_migration_status(
    services: State<'_, AppServices>,
) -> Result<LegacyMigrationStatus, LegacyMigrationServiceError> {
    migration_service(&services).status().await
}

/// 迁移旧版实例配置（写 `sl.json` + 登记信任名单）。
#[tauri::command(rename_all = "snake_case")]
pub async fn migrate_legacy_config(
    services: State<'_, AppServices>,
) -> Result<LegacyMigrationReport, LegacyMigrationServiceError> {
    migration_service(&services).migrate().await
}

/// 记录用户「不再提示」的选择。
#[tauri::command(rename_all = "snake_case")]
pub async fn dismiss_legacy_migration(
    services: State<'_, AppServices>,
) -> Result<(), LegacyMigrationServiceError> {
    migration_service(&services).dismiss().await
}
