//! 实例列表持久化存储。
//!
//! 基于 `infra::persistence::config::ConfigFile` 实现，
//! 提供实例元数据的原子读写和备份能力。
//!
//! `update` 使用 `ConfigFile::update_persisted` 在单个文件锁内完成
//! "加载 → 修改 → 保存"，避免多个 `InstanceStore` 实例并发修改时
//! 基于过期快照互相覆盖。

use std::path::PathBuf;

use sealantern_infra::persistence::config::ConfigFile;

use super::types::InstanceList;

/// 实例列表管理器的持久化句柄
pub struct InstanceStore {
    inner: ConfigFile<InstanceList>,
    path: PathBuf,
}

impl InstanceStore {
    /// 加载或创建实例列表文件
    ///
    /// 若文件是 1.2.0 旧版裸数组格式，先迁移为新版 `InstanceList`
    /// 并写回磁盘，保证升级后首次启动即可正常解析。
    pub async fn load(
        path: impl Into<std::path::PathBuf>,
    ) -> Result<Self, sealantern_infra::fs::FsError> {
        let path = path.into();
        migrate_legacy_servers_file(&path).await?;
        let inner = ConfigFile::load_or_create(&path, InstanceList::default()).await?;
        Ok(Self { inner, path })
    }

    /// 获取当前实例列表（只读快照）
    pub fn get(&self) -> &InstanceList {
        self.inner.get()
    }

    /// 获取可变引用并持久化
    pub async fn save(&self) -> Result<(), sealantern_infra::fs::FsError> {
        self.inner.save(false).await
    }

    /// 更新实例列表并持久化。
    ///
    /// 使用底层锁内"读-改-写"原语 `ConfigFile::update_persisted`，
    /// 在单个文件锁内重新加载最新状态、应用修改并保存，
    /// 防止基于过期快照覆盖其它实例的修改。
    /// 更新完成后同步内存快照，保持与磁盘一致。
    pub async fn update(
        &mut self,
        f: impl FnOnce(&mut InstanceList),
    ) -> Result<(), sealantern_infra::fs::FsError> {
        let updated =
            ConfigFile::update_persisted(&self.path, InstanceList::default(), false, f).await?;
        self.inner.set(updated);
        Ok(())
    }

    /// 创建备份
    pub async fn backup(&self) -> Result<std::path::PathBuf, sealantern_infra::fs::FsError> {
        self.inner.backup().await
    }
}

/// 检测 1.2.0 旧版裸数组格式并迁移为新版 `InstanceList`，写回磁盘。
///
/// 旧版文件以 `[` 开头（数组）；新版以 `{` 开头（对象）。仅当文件存在且
/// 以数组形式开头时才触发迁移，避免对空文件或已迁移文件做多余写入。
async fn migrate_legacy_servers_file(
    path: &std::path::Path,
) -> Result<(), sealantern_infra::fs::FsError> {
    if !path.exists() {
        return Ok(());
    }
    let raw = match tokio::fs::read(path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            return Err(sealantern_infra::fs::FsError::Io {
                operation: "read legacy servers",
                path: path.to_path_buf(),
                source: e,
            })
        }
    };
    let text = match String::from_utf8(raw) {
        Ok(text) => text,
        Err(e) => {
            return Err(sealantern_infra::fs::FsError::Encoding {
                path: path.to_path_buf(),
                encoding: "UTF-8",
                message: e.to_string(),
            })
        }
    };
    let trimmed = text.trim_start();
    if !trimmed.starts_with('[') {
        return Ok(());
    }

    let records: Vec<crate::models::LegacyServerInstance> = match serde_json::from_str(&text) {
        Ok(records) => records,
        Err(e) => {
            return Err(sealantern_infra::fs::FsError::Serialization {
                format: "json",
                operation: "decode legacy servers",
                path: path.to_path_buf(),
                message: e.to_string(),
            })
        }
    };
    let list = InstanceList::migrate_legacy(records);
    let content = match serde_json::to_string_pretty(&list) {
        Ok(content) => content,
        Err(e) => {
            return Err(sealantern_infra::fs::FsError::Serialization {
                format: "json",
                operation: "encode migrated servers",
                path: path.to_path_buf(),
                message: e.to_string(),
            })
        }
    };
    if let Err(e) = tokio::fs::write(path, content).await {
        return Err(sealantern_infra::fs::FsError::Io {
            operation: "write migrated servers",
            path: path.to_path_buf(),
            source: e,
        });
    }
    Ok(())
}
