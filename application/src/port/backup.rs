//! 服务器备份服务端口。

use async_trait::async_trait;
use sealantern_contract::BackupServiceError;
use sealantern_contract::backup::{BackupItem, BackupSettings, CreateBackupRequest};

/// 服务器备份宿主能力端口。
///
/// 提供备份的查询、创建、删除、恢复与设置管理。创建 / 恢复需要实例
/// 已停止（冷备份），运行状态校验由实现方组合 [`ServerService`] 完成，
/// 不依赖任何具体宿主。
#[async_trait]
pub trait BackupService: Send + Sync {
    /// 列出指定服务器的全部备份项。
    async fn list(&self, server_id: &str) -> Result<Vec<BackupItem>, BackupServiceError>;

    /// 为指定服务器创建备份；服务器正在运行时拒绝。
    async fn create(&self, request: CreateBackupRequest) -> Result<BackupItem, BackupServiceError>;

    /// 删除指定备份。
    async fn delete(&self, backup_id: &str) -> Result<(), BackupServiceError>;

    /// 将备份恢复到指定服务器；服务器正在运行时拒绝。
    async fn restore(&self, backup_id: &str, server_id: &str) -> Result<(), BackupServiceError>;

    /// 获取指定服务器的备份设置。
    async fn settings(&self, server_id: &str) -> Result<BackupSettings, BackupServiceError>;

    /// 更新指定服务器的备份设置。
    async fn update_settings(
        &self,
        server_id: &str,
        settings: BackupSettings,
    ) -> Result<(), BackupServiceError>;
}
