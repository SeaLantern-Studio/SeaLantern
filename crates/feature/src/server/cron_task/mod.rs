//! 服务器 Cron 定时任务。
//!
//! 此模块只负责任务模型、持久化和执行调度。宿主通过
//! [`CronTaskExecutor`] 注入实际的服务器操作，避免 `feature` 反向依赖
//! `core` 或特定的桌面运行时。

mod model;
mod service;

pub use model::{CronTask, CronTaskAction, CronTaskDraft, CronTaskList, CronTaskRun};
pub use service::{CronTaskError, CronTaskExecutor, CronTaskService};
