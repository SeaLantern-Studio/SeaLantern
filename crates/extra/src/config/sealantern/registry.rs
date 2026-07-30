//! 服务器实例注册表。
//!
//! 管理所有已创建服务器的元数据（CRUD），
//! 底层通过 [`ServerStore`] 持久化到 JSON 文件。

use super::store::ServerStore;
use super::types::ServerInstance;

/// 服务器注册表
pub struct ServerRegistry {
    store: ServerStore,
}

impl ServerRegistry {
    /// 从指定路径加载服务器列表
    pub async fn load(
        path: impl Into<std::path::PathBuf>,
    ) -> Result<Self, sealantern_infra::fs::FsError> {
        let store = ServerStore::load(path).await?;
        Ok(Self { store })
    }

    /// 获取所有服务器
    pub fn list(&self) -> &[ServerInstance] {
        &self.store.get().servers
    }

    /// 按 ID 查找服务器
    pub fn get(&self, id: &str) -> Option<&ServerInstance> {
        self.store.get().servers.iter().find(|s| s.id == id)
    }

    /// 添加服务器
    pub async fn add(
        &mut self,
        instance: ServerInstance,
    ) -> Result<(), sealantern_infra::fs::FsError> {
        self.store.update(|list| list.servers.push(instance)).await
    }

    /// 更新服务器
    pub async fn update(
        &mut self,
        id: &str,
        f: impl FnOnce(&mut ServerInstance),
    ) -> Result<bool, sealantern_infra::fs::FsError> {
        let id = id.to_string();
        let mut updated = false;
        self.store
            .update(|list| {
                if let Some(server) = list.servers.iter_mut().find(|s| s.id == id) {
                    f(server);
                    updated = true;
                }
            })
            .await?;
        Ok(updated)
    }

    /// 删除服务器
    pub async fn delete(&mut self, id: &str) -> Result<bool, sealantern_infra::fs::FsError> {
        let id = id.to_string();
        let mut removed = false;
        self.store
            .update(|list| {
                let before = list.servers.len();
                list.servers.retain(|s| s.id != id);
                removed = list.servers.len() < before;
            })
            .await?;
        Ok(removed)
    }
}
