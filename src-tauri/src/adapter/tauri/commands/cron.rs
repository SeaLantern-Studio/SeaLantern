//! 服务器定时任务 Tauri 命令。

use std::sync::Arc;

use sealantern_application::service::CoreCronTaskService;
use sealantern_application::services::AppServices;
use sealantern_interface::cron::{CronTask, CronTaskDraft, CronTaskRun};
use sealantern_interface::{CronTaskService, CronTaskServiceError};

async fn cron_service() -> Result<Arc<CoreCronTaskService>, CronTaskServiceError> {
    let services = AppServices::get()
        .await
        .map_err(|_| CronTaskServiceError::OperationFailed)?;
    Ok(services.cron().clone())
}

/// 列出全部定时任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn list_cron_tasks() -> Result<Vec<CronTask>, CronTaskServiceError> {
    cron_service().await?.list().await
}

/// 创建定时任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn create_cron_task(draft: CronTaskDraft) -> Result<CronTask, CronTaskServiceError> {
    cron_service().await?.create(draft).await
}

/// 更新定时任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn update_cron_task(
    id: String,
    draft: CronTaskDraft,
) -> Result<CronTask, CronTaskServiceError> {
    cron_service().await?.update(&id, draft).await
}

/// 删除定时任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn delete_cron_task(id: String) -> Result<(), CronTaskServiceError> {
    cron_service().await?.delete(&id).await
}

/// 启用或禁用定时任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn set_cron_task_enabled(
    id: String,
    enabled: bool,
) -> Result<CronTask, CronTaskServiceError> {
    cron_service().await?.set_enabled(&id, enabled).await
}

/// 立即执行一次定时任务。
#[tauri::command(rename_all = "snake_case")]
pub async fn run_cron_task(id: String) -> Result<CronTaskRun, CronTaskServiceError> {
    cron_service().await?.run_now(&id).await
}
