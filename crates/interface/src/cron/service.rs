use async_trait::async_trait;

use crate::error::CronTaskServiceError;

use super::models::{CronTask, CronTaskDraft, CronTaskRun};

/// 服务器定时任务宿主能力端口。
///
/// 实现方负责 JSON 持久化、Cron 表达式校验和服务器动作执行；传输层只消费
/// 本契约，不直接依赖具体调度或存储实现。
#[async_trait]
pub trait CronTaskService: Send + Sync {
    /// 列出全部定时任务。
    async fn list(&self) -> Result<Vec<CronTask>, CronTaskServiceError>;

    /// 创建并持久化定时任务。
    async fn create(&self, draft: CronTaskDraft) -> Result<CronTask, CronTaskServiceError>;

    /// 更新并持久化定时任务。
    async fn update(
        &self,
        id: &str,
        draft: CronTaskDraft,
    ) -> Result<CronTask, CronTaskServiceError>;

    /// 删除定时任务。
    async fn delete(&self, id: &str) -> Result<(), CronTaskServiceError>;

    /// 启用或禁用定时任务。
    async fn set_enabled(&self, id: &str, enabled: bool) -> Result<CronTask, CronTaskServiceError>;

    /// 立即执行一次指定任务，并更新运行记录和下次执行时间。
    async fn run_now(&self, id: &str) -> Result<CronTaskRun, CronTaskServiceError>;
}
