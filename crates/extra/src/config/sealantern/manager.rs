//! 应用设置管理器。
//!
//! 管理 `AppSettings` 的加载、保存、分组 diff 和部分更新。
//! 底层复用 `infra::persistence::config::ConfigFile` 实现原子写入。

use sealantern_infra::persistence::config::ConfigFile;
use std::path::PathBuf;

use super::types::{AppSettings, PartialAppSettings, UpdateResult};

/// 应用设置管理器
pub struct SettingsManager {
    inner: ConfigFile<AppSettings>,
}

impl SettingsManager {
    /// 加载或创建设置文件
    pub async fn load(path: impl Into<PathBuf>) -> Result<Self, sealantern_infra::fs::FsError> {
        let inner = ConfigFile::load_or_create(path, AppSettings::default()).await?;
        Ok(Self { inner })
    }

    /// 获取当前设置的只读引用
    pub fn get(&self) -> &AppSettings {
        self.inner.get()
    }

    /// 全量替换设置并持久化
    pub async fn update(
        &mut self,
        new: AppSettings,
    ) -> Result<UpdateResult, sealantern_infra::fs::FsError> {
        let old = self.inner.get().clone();
        let changed_groups = old.changed_groups(&new);
        self.inner.set(new);
        self.inner.save(false).await?;
        Ok(UpdateResult {
            settings: self.inner.get().clone(),
            changed_groups,
        })
    }

    /// 全量替换 + 计算变更分组
    pub async fn update_with_diff(
        &mut self,
        new: AppSettings,
    ) -> Result<UpdateResult, sealantern_infra::fs::FsError> {
        self.update(new).await
    }

    /// 部分更新（只传需要改的字段）
    pub async fn update_partial(
        &mut self,
        partial: PartialAppSettings,
    ) -> Result<UpdateResult, sealantern_infra::fs::FsError> {
        let old = self.inner.get().clone();
        self.inner.update(|s| partial.merge_into(s));
        let changed_groups = old.changed_groups(self.inner.get());
        self.inner.save(false).await?;
        Ok(UpdateResult {
            settings: self.inner.get().clone(),
            changed_groups,
        })
    }

    /// 重置为默认设置
    pub async fn reset(&mut self) -> Result<AppSettings, sealantern_infra::fs::FsError> {
        let default = AppSettings::default();
        self.inner.set(default.clone());
        self.inner.save(false).await?;
        Ok(default)
    }

    /// 导出设置为 JSON 字符串
    pub fn export_json(&self) -> Result<String, sealantern_infra::fs::FsError> {
        serde_json::to_string_pretty(self.inner.get()).map_err(|e| {
            sealantern_infra::fs::FsError::Serialization {
                format: "json",
                operation: "serialize settings",
                path: "".into(),
                message: e.to_string(),
            }
        })
    }

    /// 从 JSON 字符串导入设置
    pub async fn import_json(
        &mut self,
        json: &str,
    ) -> Result<UpdateResult, sealantern_infra::fs::FsError> {
        let imported: AppSettings = serde_json::from_str(json).map_err(|e| {
            sealantern_infra::fs::FsError::Serialization {
                format: "json",
                operation: "deserialize settings",
                path: "".into(),
                message: e.to_string(),
            }
        })?;
        self.update(imported).await
    }
}
