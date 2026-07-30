//! 应用设置管理器。
//!
//! 管理 `AppSettings` 的加载、保存、分组 diff 和部分更新。
//! 底层复用 `infra::persistence::config::ConfigFile` 实现原子写入。

use sealantern_infra::persistence::config::ConfigFile;
use std::path::PathBuf;

use super::types::{AppSettings, PartialAppSettings, UpdateResult, CURRENT_CONFIG_VERSION};

/// 应用设置管理器
pub struct SettingsManager {
    inner: ConfigFile<AppSettings>,
}

impl SettingsManager {
    /// 加载或创建设置文件，检测版本号并执行迁移
    pub async fn load(path: impl Into<PathBuf>) -> Result<Self, sealantern_infra::fs::FsError> {
        let path = path.into();
        let inner = match ConfigFile::load(&path).await {
            Ok(cf) => cf,
            Err(_) => {
                // 文件不存在或格式错误，创建默认配置
                ConfigFile::load_or_create(&path, AppSettings::default()).await?
            }
        };

        let mut mgr = Self { inner };

        // 版本迁移：如果配置版本落后于当前版本，执行升级并保存
        let version = mgr.inner.get().config_version;
        if version < CURRENT_CONFIG_VERSION {
            tracing::info!(
                target: "sealantern.config",
                "配置版本升级: {} → {}",
                version, CURRENT_CONFIG_VERSION
            );
            mgr.inner
                .update(|s| s.config_version = CURRENT_CONFIG_VERSION);
            mgr.inner.save(false).await?;
        }

        Ok(mgr)
    }

    /// 获取当前设置的只读引用
    pub fn get(&self) -> &AppSettings {
        self.inner.get()
    }

    /// 全量替换设置并持久化
    /// 持久化失败时回滚内存状态
    pub async fn update(
        &mut self,
        new: AppSettings,
    ) -> Result<UpdateResult, sealantern_infra::fs::FsError> {
        let old = self.inner.get().clone();
        let changed_groups = old.changed_groups(&new);
        self.inner.set(new);
        match self.inner.save(false).await {
            Ok(()) => Ok(UpdateResult {
                settings: self.inner.get().clone(),
                changed_groups,
            }),
            Err(e) => {
                self.inner.set(old);
                Err(e)
            }
        }
    }

    /// 全量替换 + 计算变更分组
    pub async fn update_with_diff(
        &mut self,
        new: AppSettings,
    ) -> Result<UpdateResult, sealantern_infra::fs::FsError> {
        self.update(new).await
    }

    /// 部分更新（只传需要改的字段）
    /// 持久化失败时回滚内存状态
    pub async fn update_partial(
        &mut self,
        partial: PartialAppSettings,
    ) -> Result<UpdateResult, sealantern_infra::fs::FsError> {
        let old = self.inner.get().clone();
        self.inner.update(|s| partial.merge_into(s));
        let changed_groups = old.changed_groups(self.inner.get());
        match self.inner.save(false).await {
            Ok(()) => Ok(UpdateResult {
                settings: self.inner.get().clone(),
                changed_groups,
            }),
            Err(e) => {
                self.inner.set(old);
                Err(e)
            }
        }
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
