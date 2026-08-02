//! 服务器实例管理服务实现。
//!
//! 实现 `server::rpc::service::InstanceService`，用 `extra` 的
//! [`InstanceRegistry`]（持久化到 `sea_lantern_servers.json`）驱动查询与 CRUD，
//! 进程管理（Daemon）接入后补全生命周期（当前返回 [`InstanceServiceError::Unsupported`]）。

use async_trait::async_trait;
use std::path::PathBuf;

use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_core::server::ServerStatus;
use sealantern_extra::config::InstanceRegistry;
use sealantern_infra::fs::FsError;
use sealantern_infra::platform::get_app_data_dir;
use sealantern_server::rpc::service::{InstanceService, InstanceServiceError};

/// 实例列表数据文件名（沿用 v1.2.0 命名，置于应用数据根目录）。
const INSTANCES_FILE: &str = "sea_lantern_servers.json";

/// 基于 `core` + `extra` 的实例管理宿主能力实现。
pub struct CoreInstanceService {
    registry: tokio::sync::Mutex<InstanceRegistry>,
}

impl CoreInstanceService {
    /// 从应用数据根目录加载实例注册表。
    pub async fn new() -> Result<Self, FsError> {
        let path = get_app_data_dir().join(INSTANCES_FILE);
        Self::with_path(path).await
    }

    /// 从指定路径加载实例注册表（便于测试注入）。
    pub async fn with_path(path: impl Into<PathBuf>) -> Result<Self, FsError> {
        let registry = InstanceRegistry::load(path).await?;
        Ok(Self {
            registry: tokio::sync::Mutex::new(registry),
        })
    }
}

/// 把 `extra` 的存储/序列化错误映射为实例服务错误。
///
/// 分类错误不携带底层细节（IO、锁、序列化等统一视为操作失败），细节写入宿主日志。
fn map_store_error(_error: FsError) -> InstanceServiceError {
    InstanceServiceError::OperationFailed
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
        Err(InstanceServiceError::Unsupported)
    }

    async fn start(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
        Err(InstanceServiceError::Unsupported)
    }

    async fn stop(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
        Err(InstanceServiceError::Unsupported)
    }

    async fn force_stop(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
        Err(InstanceServiceError::Unsupported)
    }

    async fn create(&self, spec: InstanceSpec) -> Result<Instance, InstanceServiceError> {
        // 先由 core 校验并规范化实例，再持久化登记。
        let instance = sealantern_core::instance::Instance::new(spec)
            .map_err(|_| InstanceServiceError::InvalidState)?;
        let mut registry = self.registry.lock().await;
        registry
            .save_instance(&instance)
            .await
            .map_err(map_store_error)?;
        Ok(instance)
    }

    async fn delete(&self, id: &InstanceId) -> Result<bool, InstanceServiceError> {
        let mut registry = self.registry.lock().await;
        let deleted = registry.delete(id).await.map_err(map_store_error)?;
        if !deleted {
            return Err(InstanceServiceError::InstanceNotFound);
        }
        Ok(true)
    }

    async fn rename(&self, id: &InstanceId, name: &str) -> Result<(), InstanceServiceError> {
        let mut registry = self.registry.lock().await;
        let edited = registry
            .edit_instance(id, |instance| instance.name = name.to_owned())
            .await
            .map_err(map_store_error)?;
        if !edited {
            return Err(InstanceServiceError::InstanceNotFound);
        }
        Ok(())
    }

    async fn update_path(&self, id: &InstanceId, path: &str) -> Result<(), InstanceServiceError> {
        let mut registry = self.registry.lock().await;
        let edited = registry
            .edit_instance(id, |instance| instance.directory = PathBuf::from(path))
            .await
            .map_err(map_store_error)?;
        if !edited {
            return Err(InstanceServiceError::InstanceNotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use sealantern_core::instance::LocalLaunch;
    use sealantern_core::instance::StartupMode;

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
