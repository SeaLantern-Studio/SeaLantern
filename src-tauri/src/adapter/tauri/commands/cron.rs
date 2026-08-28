//! 服务器定时任务 Tauri 命令。

use std::sync::Arc;

use sealantern_application::port::CronTaskService;
use sealantern_application::service::CoreCronTaskService;
use sealantern_application::services::AppServices;
use sealantern_contract::CronTaskServiceError;
use sealantern_contract::cron::{CronTask, CronTaskDraft, CronTaskRun};
use tauri::State;

fn cron_service(services: &AppServices) -> Arc<CoreCronTaskService> {
    services.cron().clone()
}

/// 列出全部定时任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn list_cron_tasks(
    services: State<'_, AppServices>,
) -> Result<Vec<CronTask>, CronTaskServiceError> {
    cron_service(&services).list().await
}

/// 创建定时任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn create_cron_task(
    services: State<'_, AppServices>,
    draft: CronTaskDraft,
) -> Result<CronTask, CronTaskServiceError> {
    cron_service(&services).create(draft).await
}

/// 更新定时任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_cron_task(
    services: State<'_, AppServices>,
    id: String,
    draft: CronTaskDraft,
) -> Result<CronTask, CronTaskServiceError> {
    cron_service(&services).update(&id, draft).await
}

/// 删除定时任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_cron_task(
    services: State<'_, AppServices>,
    id: String,
) -> Result<(), CronTaskServiceError> {
    cron_service(&services).delete(&id).await
}

/// 启用或禁用定时任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn set_cron_task_enabled(
    services: State<'_, AppServices>,
    id: String,
    enabled: bool,
) -> Result<CronTask, CronTaskServiceError> {
    cron_service(&services).set_enabled(&id, enabled).await
}

/// 立即执行一次定时任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn run_cron_task(
    services: State<'_, AppServices>,
    id: String,
) -> Result<CronTaskRun, CronTaskServiceError> {
    cron_service(&services).run_now(&id).await
}
