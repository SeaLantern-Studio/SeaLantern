//! 服务器实例管理服务实现。
//!
//! 实现 [`sealantern_interface::InstanceService`] 能力端口，用 `extra` 的
//! [`InstanceRegistry`](sealantern_extra::config::InstanceRegistry)（持久化到
//! `sea_lantern_servers.json`）驱动查询与 CRUD。进程管理（Daemon）接入后补全
//! 生命周期（当前返回 [`InstanceError::Unsupported`]）。
//!
//! 错误分层：内部以应用层主错误 [`InstanceError`] 为源头，沿
//! `core/extra/infra` → `application::error::instance` → `interface::error`
//! 收敛；暴露 [`InstanceService`] 时统一转为接口契约错误 [`InstanceServiceError`]。

use async_trait::async_trait;
use std::path::PathBuf;

use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_core::server::ServerStatus;
use sealantern_extra::config::InstanceRegistry;
use sealantern_infra::platform::get_app_data_dir;
use sealantern_interface::{InstanceService, InstanceServiceError};

use crate::error::InstanceError;

/// 实例列表数据文件名，置于应用数据根目录。
///
/// 沿用历史版本使用的文件名，以保证旧数据文件可被读取迁移。
const INSTANCES_FILE: &str = "sea_lantern_servers.json";

/// 基于 `core` + `extra` 的实例管理宿主能力实现。
pub struct CoreInstanceService {
    registry: tokio::sync::Mutex<InstanceRegistry>,
}

impl CoreInstanceService {
    /// 从应用数据根目录加载实例注册表。
    pub async fn new() -> Result<Self, InstanceError> {
        let path = get_app_data_dir().join(INSTANCES_FILE);
        Self::with_path(path).await
    }

    /// 从指定路径加载实例注册表（便于测试注入）。
    pub async fn with_path(path: impl Into<PathBuf>) -> Result<Self, InstanceError> {
        let registry = InstanceRegistry::load(path).await?;
        Ok(Self {
            registry: tokio::sync::Mutex::new(registry),
        })
    }

    /// 内部创建实例：core 校验规范化后持久化登记，返回应用层主错误。
    async fn create_inner(&self, spec: InstanceSpec) -> Result<Instance, InstanceError> {
        let instance = Instance::new(spec)?;
        let mut registry = self.registry.lock().await;
        registry.save_instance(&instance).await?;
        Ok(instance)
    }

    /// 内部删除实例，返回应用层主错误。
    async fn delete_inner(&self, id: &InstanceId) -> Result<bool, InstanceError> {
        let mut registry = self.registry.lock().await;
        let deleted = registry.delete(id).await?;
        if !deleted {
            return Err(InstanceError::NotFound);
        }
        Ok(true)
    }

    /// 内部重命名实例，返回应用层主错误。
    async fn rename_inner(&self, id: &InstanceId, name: &str) -> Result<(), InstanceError> {
        let mut registry = self.registry.lock().await;
        let edited = registry
            .edit_instance(id, |instance| instance.name = name.to_owned())
            .await?;
        if !edited {
            return Err(InstanceError::NotFound);
        }
        Ok(())
    }

    /// 内部更新实例目录路径，返回应用层主错误。
    async fn update_path_inner(
        &self,
        id: &InstanceId,
        path: &str,
    ) -> Result<(), InstanceError> {
        let mut registry = self.registry.lock().await;
        let edited = registry
            .edit_instance(id, |instance| instance.directory = PathBuf::from(path))
            .await?;
        if !edited {
            return Err(InstanceError::NotFound);
        }
        Ok(())
    }
}

#[async_trait]
impl InstanceService for CoreInstanceService {
    async fn list(&self) -> Result<Vec<Instance>, InstanceServiceError> {
        let registry = self.registry.lock().await;
        Ok(registry.list().to_vec())
    }

    async fn find(&self, id: &InstanceId) -> Result<Option<Instance>, InstanceServiceError> {
        let registry = self.registry.lock().await;
        Ok(registry.get(id).cloned())
    }

    async fn status(&self, _id: &InstanceId) -> Result<ServerStatus, InstanceServiceError> {
        Err(InstanceError::Unsupported.into())
    }

    async fn start(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
        Err(InstanceError::Unsupported.into())
    }

    async fn stop(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
        Err(InstanceError::Unsupported.into())
    }

    async fn force_stop(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
        Err(InstanceError::Unsupported.into())
    }

    async fn create(&self, spec: InstanceSpec) -> Result<Instance, InstanceServiceError> {
        self.create_inner(spec).await.map_err(Into::into)
    }

    async fn delete(&self, id: &InstanceId) -> Result<bool, InstanceServiceError> {
        self.delete_inner(id).await.map_err(Into::into)
    }

    async fn rename(&self, id: &InstanceId, name: &str) -> Result<(), InstanceServiceError> {
        self.rename_inner(id, name).await.map_err(Into::into)
    }

    async fn update_path(&self, id: &InstanceId, path: &str) -> Result<(), InstanceServiceError> {
        self.update_path_inner(id, path).await.map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use sealantern_core::instance::{LocalLaunch, StartupMode};

    use super::*;

    fn sample_spec(id: &str) -> InstanceSpec {
        InstanceSpec {
            id: InstanceId::new(id).expect("valid id"),
            name: format!("服务器-{id}"),
            aliases: Vec::new(),
            core_type: "paper".into(),
            core_version: "1.20.4".into(),
            game_version: "1.20.4".into(),
            directory: PathBuf::from(format!("/tmp/server-{id}")),
            port: 25565,
            max_memory_mib: 2048,
            min_memory_mib: 512,
            created_at_unix_secs: 0,
            last_started_at_unix_secs: None,
            server_metadata: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(PathBuf::from(format!("/tmp/server-{id}/server.jar"))),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        }
    }

    fn test_path(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sealantern-inst-svc-{label}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir.join("sea_lantern_servers.json")
    }

    #[tokio::test]
    async fn persists_reads_and_deletes_an_instance() {
        let path = test_path("crud");
        let service = CoreInstanceService::with_path(&path)
            .await
            .expect("load service");

        let created = service.create(sample_spec("a")).await.expect("create");
        assert_eq!(created.name, "服务器-a");

        let listed = service.list().await.expect("list");
        assert_eq!(listed.len(), 1);

        let found = service.find(&created.id).await.expect("find");
        assert!(found.is_some());

        service.rename(&created.id, "改名").await.expect("rename");
        assert_eq!(service.list().await.expect("list")[0].name, "改名");

        assert!(service.delete(&created.id).await.expect("delete"));
        assert!(service.list().await.expect("list").is_empty());

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[tokio::test]
    async fn rename_missing_instance_reports_not_found() {
        let path = test_path("rename-missing");
        let service = CoreInstanceService::with_path(&path)
            .await
            .expect("load service");
        let missing = InstanceId::new("missing").expect("valid id");
        let result = service.rename(&missing, "x").await;
        assert_eq!(result, Err(InstanceServiceError::InstanceNotFound));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}