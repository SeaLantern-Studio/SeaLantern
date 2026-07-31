//! 服务器实例注册表。
//!
//! 管理所有已创建服务器的元数据（CRUD），
//! 底层通过 [`ServerStore`] 持久化到 JSON 文件。
//!
//! 所有操作都会记录带操作类型、配置路径和实例 ID 的 tracing 事件，
//! 便于排查持久化错误与并发覆盖问题。

use std::path::PathBuf;

use sealantern_infra::fs::FsError;

use crate::observability;

use super::store::ServerStore;
use super::types::ServerInstance;

/// 服务器注册表
pub struct ServerRegistry {
    store: ServerStore,
    path: PathBuf,
}

impl ServerRegistry {
    /// 从指定路径加载服务器列表
    pub async fn load(path: impl Into<PathBuf>) -> Result<Self, FsError> {
        let path = path.into();
        let store = ServerStore::load(&path).await?;
        let count = store.get().servers.len();
        observability::config_registry_loaded(&path, count);
        Ok(Self { store, path })
    }

    /// 获取所有服务器
    pub fn list(&self) -> &[ServerInstance] {
        &self.store.get().servers
    }

    /// 按 ID 查找服务器
    pub fn get(&self, id: &str) -> Option<&ServerInstance> {
        self.store.get().servers.iter().find(|s| s.id == id)
    }

    /// 添加服务器，拒绝重复 ID。
    ///
    /// 重复 ID 检查在锁内闭包中执行，与写入共享同一把文件锁，
    /// 避免"检查-动作"竞态窗口。
    pub async fn add(
        &mut self,
        instance: ServerInstance,
    ) -> Result<(), sealantern_infra::fs::FsError> {
        let id = instance.id.clone();
        let name = instance.name.clone();
        let mut duplicate = false;
        let result = self
            .store
            .update(|list| {
                if list.servers.iter().any(|s| s.id == id) {
                    duplicate = true;
                } else {
                    list.servers.push(instance);
                }
            })
            .await;

        match &result {
            Ok(()) if duplicate => {
                observability::config_registry_duplicate_id(&self.path, &id);
                return Err(FsError::Task {
                    operation: "add server",
                    message: format!("duplicate server id: {id}"),
                });
            }
            Ok(()) => observability::config_registry_server_added(&self.path, &id, &name),
            Err(e) => {
                observability::config_registry_operation_failed(&self.path, "add", Some(&id), e)
            }
        }
        result
    }

    /// 更新服务器，ID 不存在时静默跳过（不触发写入）
    pub async fn update(
        &mut self,
        id: &str,
        f: impl FnOnce(&mut ServerInstance),
    ) -> Result<bool, sealantern_infra::fs::FsError> {
        let id = id.to_string();
        if !self.store.get().servers.iter().any(|s| s.id == id) {
            observability::config_registry_server_not_found(&self.path, "update", &id);
            return Ok(false);
        }
        let mut updated = false;
        let result = self
            .store
            .update(|list| {
                if let Some(server) = list.servers.iter_mut().find(|s| s.id == id) {
                    f(server);
                    updated = true;
                }
            })
            .await;
        match result {
            Ok(()) if updated => {
                observability::config_registry_server_updated(&self.path, &id);
            }
            Ok(()) => {}
            Err(ref e) => {
                observability::config_registry_operation_failed(&self.path, "update", Some(&id), e)
            }
        }
        result.map(|_| updated)
    }

    /// 删除服务器，ID 不存在时静默跳过（不触发写入）
    pub async fn delete(&mut self, id: &str) -> Result<bool, sealantern_infra::fs::FsError> {
        let id = id.to_string();
        if !self.store.get().servers.iter().any(|s| s.id == id) {
            observability::config_registry_server_not_found(&self.path, "delete", &id);
            return Ok(false);
        }
        let mut removed = false;
        let result = self
            .store
            .update(|list| {
                let before = list.servers.len();
                list.servers.retain(|s| s.id != id);
                removed = list.servers.len() < before;
            })
            .await;
        match result {
            Ok(()) if removed => {
                observability::config_registry_server_deleted(&self.path, &id);
            }
            Ok(()) => {}
            Err(ref e) => {
                observability::config_registry_operation_failed(&self.path, "delete", Some(&id), e)
            }
        }
        result.map(|_| removed)
    }
}
