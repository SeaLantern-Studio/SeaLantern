//! 服务器实例注册表。
//!
//! 管理所有已创建实例的元数据（CRUD），
//! 底层通过 [`InstanceStore`] 持久化到 JSON 文件。
//!
//! 所有操作都会记录带操作类型、配置路径和实例 ID 的 tracing 事件，
//! 便于排查持久化错误与并发覆盖问题。

use std::path::PathBuf;

use sealantern_core::instance::{Instance, InstanceId};
use sealantern_infra::fs::FsError;

use crate::observability;

use super::store::InstanceStore;

/// 服务器实例注册表
pub struct InstanceRegistry {
    store: InstanceStore,
    path: PathBuf,
}

impl InstanceRegistry {
    /// 从指定路径加载实例列表
    pub async fn load(path: impl Into<PathBuf>) -> Result<Self, FsError> {
        let path = path.into();
        let store = InstanceStore::load(&path).await?;
        let count = store.get().instances.len();
        observability::config_registry_loaded(&path, count);
        Ok(Self { store, path })
    }

    /// 获取全部实例
    pub fn list(&self) -> &[Instance] {
        &self.store.get().instances
    }

    /// 按 ID 查找实例
    pub fn get(&self, id: &InstanceId) -> Option<&Instance> {
        self.store.get().instances.iter().find(|i| i.id == *id)
    }

    /// 保存（覆盖写入）实例。
    ///
    /// 已存在同 ID 实例时整体替换，否则追加——即 upsert 语义。
    /// 写入在锁内闭包中执行，避免"检查-动作"竞态窗口。
    pub async fn save_instance(&mut self, instance: &Instance) -> Result<(), FsError> {
        let id = instance.id.clone();
        let name = instance.name.clone();
        let result = self
            .store
            .update(|list| {
                if let Some(existing) = list.instances.iter_mut().find(|i| i.id == id) {
                    *existing = instance.clone();
                } else {
                    list.instances.push(instance.clone());
                }
            })
            .await;

        match &result {
            Ok(()) => observability::config_registry_server_added(&self.path, id.as_str(), &name),
            Err(e) => observability::config_registry_operation_failed(
                &self.path,
                "save",
                Some(id.as_str()),
                e,
            ),
        }
        result
    }

    /// 编辑实例，ID 不存在时静默跳过（不触发写入）。
    ///
    /// 返回是否确实编辑了某个实例。
    pub async fn edit_instance(
        &mut self,
        id: &InstanceId,
        f: impl FnOnce(&mut Instance),
    ) -> Result<bool, FsError> {
        let id = id.clone();
        if !self.store.get().instances.iter().any(|i| i.id == id) {
            observability::config_registry_server_not_found(&self.path, "edit", id.as_str());
            return Ok(false);
        }
        let mut updated = false;
        let result = self
            .store
            .update(|list| {
                if let Some(instance) = list.instances.iter_mut().find(|i| i.id == id) {
                    f(instance);
                    updated = true;
                }
            })
            .await;
        match result {
            Ok(()) if updated => {
                observability::config_registry_server_updated(&self.path, id.as_str());
            }
            Ok(()) => {}
            Err(ref e) => observability::config_registry_operation_failed(
                &self.path,
                "edit",
                Some(id.as_str()),
                e,
            ),
        }
        result.map(|_| updated)
    }

    /// 删除实例，ID 不存在时静默跳过（不触发写入）。
    ///
    /// 返回是否确实删除了某个实例。
    pub async fn delete(&mut self, id: &InstanceId) -> Result<bool, FsError> {
        let id = id.clone();
        if !self.store.get().instances.iter().any(|i| i.id == id) {
            observability::config_registry_server_not_found(&self.path, "delete", id.as_str());
            return Ok(false);
        }
        let mut removed = false;
        let result = self
            .store
            .update(|list| {
                let before = list.instances.len();
                list.instances.retain(|i| i.id != id);
                removed = list.instances.len() < before;
            })
            .await;
        match result {
            Ok(()) if removed => {
                observability::config_registry_server_deleted(&self.path, id.as_str());
            }
            Ok(()) => {}
            Err(ref e) => observability::config_registry_operation_failed(
                &self.path,
                "delete",
                Some(id.as_str()),
                e,
            ),
        }
        result.map(|_| removed)
    }
}
