//! 旧版实例配置迁移端口。
//!
//! 迁移完全由用户在设置页**手动触发**，没有任何启动期自动调用：`status`
//! 只读检测，`migrate` 把旧文件中仍可定位的服务器登记为新版受管实例，
//! `dismiss` 记录「不再提示」。三者的副作用边界刻意分开。

use async_trait::async_trait;
use sealantern_contract::LegacyMigrationServiceError;
use sealantern_contract::legacy_migration::{LegacyMigrationReport, LegacyMigrationStatus};

/// 管理旧版实例配置迁移的宿主能力端口。
#[async_trait]
pub trait LegacyMigrationService: Send + Sync {
    /// 检测设置页是否应提供「迁移旧版实例」入口。
    ///
    /// 不写盘；只反映「旧配置里是否还有可迁移条目」，与当前是否已有实例无关。
    async fn status(&self) -> Result<LegacyMigrationStatus, LegacyMigrationServiceError>;

    /// 执行迁移：为可迁移条目写入 `sl.json` 并登记信任名单。
    ///
    /// 已迁移 / 目录缺失 / 内容非法的条目会被跳过并在报告中列出；全部条目
    /// 都不可迁移时返回 [`LegacyMigrationServiceError::NothingToMigrate`]。
    /// 迁移成功后旧配置文件被重命名归档，从此不再触发提示。
    async fn migrate(&self) -> Result<LegacyMigrationReport, LegacyMigrationServiceError>;

    /// 记录用户选择「不再提示」。
    ///
    /// 幂等；不删除旧配置文件，用户之后仍可通过设置页重新触发检测。
    async fn dismiss(&self) -> Result<(), LegacyMigrationServiceError>;
}
