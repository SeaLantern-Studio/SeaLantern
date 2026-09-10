//! 设置信息服务端口。

use async_trait::async_trait;
use sealantern_contract::SettingsServiceError;
use sealantern_contract::settings::{
    AppSettings, PartialAppSettings, SettingsOverview, UpdateResult,
};

/// 设置管理宿主能力端口。
///
/// 配置模型及持久化语义由 `contract` 定义；本端口统一暴露设置概览、读取、更新、
/// 重置与导入导出能力，供不同宿主复用。
#[async_trait]
pub trait SettingsService: Send + Sync {
    /// 获取设置概览（所有分组及其设置项列表）。
    async fn settings_overview(&self) -> Result<SettingsOverview, SettingsServiceError>;

    /// 获取当前完整设置。
    ///
    /// 默认返回 [`SettingsServiceError::Unsupported`]，允许宿主分阶段接入新契约。
    async fn get(&self) -> Result<AppSettings, SettingsServiceError> {
        Err(SettingsServiceError::Unsupported)
    }

    /// 全量替换并持久化当前设置。
    ///
    /// 默认返回 [`SettingsServiceError::Unsupported`]，允许宿主分阶段接入新契约。
    async fn update(&self, _settings: AppSettings) -> Result<UpdateResult, SettingsServiceError> {
        Err(SettingsServiceError::Unsupported)
    }

    /// 部分更新并持久化当前设置。
    ///
    /// 默认返回 [`SettingsServiceError::Unsupported`]，允许宿主分阶段接入新契约。
    async fn update_partial(
        &self,
        _partial: PartialAppSettings,
    ) -> Result<UpdateResult, SettingsServiceError> {
        Err(SettingsServiceError::Unsupported)
    }

    /// 将当前设置恢复为默认值。
    ///
    /// 默认返回 [`SettingsServiceError::Unsupported`]，允许宿主分阶段接入新契约。
    async fn reset(&self) -> Result<AppSettings, SettingsServiceError> {
        Err(SettingsServiceError::Unsupported)
    }

    /// 将当前设置导出为 JSON 字符串。
    ///
    /// 默认返回 [`SettingsServiceError::Unsupported`]，允许宿主分阶段接入新契约。
    async fn export_json(&self) -> Result<String, SettingsServiceError> {
        Err(SettingsServiceError::Unsupported)
    }

    /// 从 JSON 字符串导入并持久化设置。
    ///
    /// 默认返回 [`SettingsServiceError::Unsupported`]，允许宿主分阶段接入新契约。
    async fn import_json(&self, _json: &str) -> Result<UpdateResult, SettingsServiceError> {
        Err(SettingsServiceError::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct OverviewOnlySettingsService;

    #[async_trait]
    impl SettingsService for OverviewOnlySettingsService {
        async fn settings_overview(&self) -> Result<SettingsOverview, SettingsServiceError> {
            Ok(SettingsOverview {
                groups: Vec::new(),
                total_entries: 0,
                configured_entries: 0,
            })
        }
    }

    #[tokio::test]
    async fn unimplemented_settings_operations_are_explicitly_unsupported() {
        let service = OverviewOnlySettingsService;

        assert!(matches!(service.get().await, Err(SettingsServiceError::Unsupported)));
        assert!(matches!(
            service.update(AppSettings::default()).await,
            Err(SettingsServiceError::Unsupported)
        ));
        assert!(matches!(
            service.update_partial(PartialAppSettings::default()).await,
            Err(SettingsServiceError::Unsupported)
        ));
        assert!(matches!(service.reset().await, Err(SettingsServiceError::Unsupported)));
        assert_eq!(service.export_json().await, Err(SettingsServiceError::Unsupported));
        assert!(matches!(
            service.import_json("{}").await,
            Err(SettingsServiceError::Unsupported)
        ));
    }
}
