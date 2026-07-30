//! 服务器列表持久化存储。
//!
//! 基于 `infra::persistence::config::ConfigFile` 实现，
//! 提供服务器元数据的原子读写和备份能力。

use sealantern_infra::persistence::config::ConfigFile;

use super::types::ServerList;

/// 服务器列表管理器的持久化句柄
pub struct ServerStore {
    inner: ConfigFile<ServerList>,
}

impl ServerStore {
    /// 加载或创建服务器列表文件
    pub async fn load(
        path: impl Into<std::path::PathBuf>,
    ) -> Result<Self, sealantern_infra::fs::FsError> {
        let inner = ConfigFile::load_or_create(path, ServerList::default()).await?;
        Ok(Self { inner })
    }

    /// 获取当前服务器列表（只读快照）
    pub fn get(&self) -> &ServerList {
        self.inner.get()
    }

    /// 获取可变引用并持久化
    pub async fn save(&self) -> Result<(), sealantern_infra::fs::FsError> {
        self.inner.save(false).await
    }

    /// 更新服务器列表并持久化
    pub async fn update(
        &mut self,
        f: impl FnOnce(&mut ServerList),
    ) -> Result<(), sealantern_infra::fs::FsError> {
        self.inner.update(f);
        self.inner.save(false).await
    }

    /// 创建备份
    pub async fn backup(&self) -> Result<std::path::PathBuf, sealantern_infra::fs::FsError> {
        self.inner.backup().await
    }
}
