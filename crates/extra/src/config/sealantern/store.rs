//! 服务器列表持久化存储。
//!
//! 基于 `infra::persistence::config::ConfigFile` 实现，
//! 提供服务器元数据的原子读写和备份能力。
//!
//! `update` 使用 `ConfigFile::update_persisted` 在单个文件锁内完成
//! "加载 → 修改 → 保存"，避免多个 `ServerStore` 实例并发修改时
//! 基于过期快照互相覆盖。

use std::path::PathBuf;

use sealantern_infra::persistence::config::ConfigFile;

use super::types::ServerList;

/// 服务器列表管理器的持久化句柄
pub struct ServerStore {
    inner: ConfigFile<ServerList>,
    path: PathBuf,
}

impl ServerStore {
    /// 加载或创建服务器列表文件
    pub async fn load(
        path: impl Into<std::path::PathBuf>,
    ) -> Result<Self, sealantern_infra::fs::FsError> {
        let path = path.into();
        let inner = ConfigFile::load_or_create(&path, ServerList::default()).await?;
        Ok(Self { inner, path })
    }

    /// 获取当前服务器列表（只读快照）
    pub fn get(&self) -> &ServerList {
        self.inner.get()
    }

    /// 获取可变引用并持久化
    pub async fn save(&self) -> Result<(), sealantern_infra::fs::FsError> {
        self.inner.save(false).await
    }

    /// 更新服务器列表并持久化。
    ///
    /// 使用底层锁内"读-改-写"原语 `ConfigFile::update_persisted`，
    /// 在单个文件锁内重新加载最新状态、应用修改并保存，
    /// 防止基于过期快照覆盖其它实例的修改。
    /// 更新完成后同步内存快照，保持与磁盘一致。
    pub async fn update(
        &mut self,
        f: impl FnOnce(&mut ServerList),
    ) -> Result<(), sealantern_infra::fs::FsError> {
        let updated =
            ConfigFile::update_persisted(&self.path, ServerList::default(), false, f).await?;
        self.inner.set(updated);
        Ok(())
    }

    /// 创建备份
    pub async fn backup(&self) -> Result<std::path::PathBuf, sealantern_infra::fs::FsError> {
        self.inner.backup().await
    }
}
