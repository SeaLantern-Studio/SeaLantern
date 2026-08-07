//! 设置信息服务端口。

use async_trait::async_trait;

use crate::error::SettingsServiceError;

use super::models::SettingsOverview;

/// 设置信息宿主能力端口。
///
/// 提供设置分组、设置项列表等查询能力，不涉及具体配置读写。
#[async_trait]
pub trait SettingsService: Send + Sync {
    /// 获取设置概览（所有分组及其设置项列表）。
    async fn settings_overview(&self) -> Result<SettingsOverview, SettingsServiceError>;
}