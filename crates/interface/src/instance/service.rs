use crate::error::InstanceServiceError;
use async_trait::async_trait;
use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_core::provisioning::{ImportExistingServerRequest, ImportModpackRequest};

/// 管理服务器实例记录的宿主能力端口。
///
/// 覆盖实例记录的查询与 CRUD（创建/删除/重命名/改路径），不涉及进程操作。
/// 实例的启动/停止/状态等进程生命周期由 [`ServerService`](crate::ServerService)
/// 提供。方法均为异步：内部涉及持久化 IO。供给（导入、整合包、扫描等）方法在
/// 后续迭代补充。实现方负责组合 `core` 的 repository 能力，不依赖任何具体传输。
#[async_trait]
pub trait InstanceService: Send + Sync {
    /// 列出全部实例。
    async fn list(&self) -> Result<Vec<Instance>, InstanceServiceError>;

    /// 按 ID 查找实例，不存在时返回 `None`。
    async fn find(&self, id: &InstanceId) -> Result<Option<Instance>, InstanceServiceError>;

    /// 创建新实例并持久化。
    async fn create(&self, spec: InstanceSpec) -> Result<Instance, InstanceServiceError>;

    /// 删除实例；实例不存在时返回 [`InstanceServiceError::InstanceNotFound`]。
    async fn delete(&self, id: &InstanceId) -> Result<(), InstanceServiceError>;

    /// 重命名实例。
    async fn rename(&self, id: &InstanceId, name: &str) -> Result<(), InstanceServiceError>;

    /// 更新实例目录路径。
    async fn update_path(&self, id: &InstanceId, path: &str) -> Result<(), InstanceServiceError>;

    /// 导入已有服务器目录：校验源目录 → 去重 → 构建导入规格 → 供给计划 → 持久化登记。
    ///
    /// 返回薄契约错误（无主机路径载荷）；底层失败详情由实现层写入受控日志。
    async fn import_existing_server(
        &self,
        request: ImportExistingServerRequest,
    ) -> Result<Instance, InstanceServiceError>;

    /// 导入整合包为受管实例。
    ///
    /// 支持三种来源：压缩包解压到运行目录、jar 单文件复制到运行目录、文件夹直接引用。
    /// 文件操作与实例注册均在此方法内完成。
    async fn import_modpack(
        &self,
        request: ImportModpackRequest,
    ) -> Result<Instance, InstanceServiceError>;
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Mutex;

    use sealantern_core::instance::StartupMode;

    use super::*;

    fn sample_spec() -> InstanceSpec {
        InstanceSpec {
            id: InstanceId::new("server-42").expect("valid id"),
            name: "测试服".into(),
            aliases: Vec::new(),
            core_type: "paper".into(),
            core_version: "1.20.4".into(),
            game_version: "1.20.4".into(),
            directory: PathBuf::from("/tmp/server-42"),
            port: 25565,
            max_memory_mib: 2048,
            min_memory_mib: 512,
            created_at_unix_secs: 0,
            last_started_at_unix_secs: None,
            server_metadata: None,
            launch: sealantern_core::instance::LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(PathBuf::from("/tmp/server-42/server.jar")),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        }
    }

    fn sample_instance() -> Instance {
        Instance::new(sample_spec()).expect("valid instance")
    }

    struct FakeInstanceService {
        calls: Mutex<Vec<&'static str>>,
    }

    #[async_trait]
    impl InstanceService for FakeInstanceService {
        async fn list(&self) -> Result<Vec<Instance>, InstanceServiceError> {
            self.calls.lock().expect("lock").push("list");
            Ok(vec![sample_instance()])
        }

        async fn find(&self, _id: &InstanceId) -> Result<Option<Instance>, InstanceServiceError> {
            self.calls.lock().expect("lock").push("find");
            Ok(Some(sample_instance()))
        }

        async fn create(&self, _spec: InstanceSpec) -> Result<Instance, InstanceServiceError> {
            self.calls.lock().expect("lock").push("create");
            Ok(sample_instance())
        }

        async fn delete(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
            self.calls.lock().expect("lock").push("delete");
            Ok(())
        }

        async fn rename(&self, _id: &InstanceId, _name: &str) -> Result<(), InstanceServiceError> {
            self.calls.lock().expect("lock").push("rename");
            Ok(())
        }

        async fn update_path(
            &self,
            _id: &InstanceId,
            _path: &str,
        ) -> Result<(), InstanceServiceError> {
            self.calls.lock().expect("lock").push("update_path");
            Ok(())
        }

        async fn import_existing_server(
            &self,
            _request: ImportExistingServerRequest,
        ) -> Result<Instance, InstanceServiceError> {
            self.calls
                .lock()
                .expect("lock")
                .push("import_existing_server");
            Ok(sample_instance())
        }

        async fn import_modpack(
            &self,
            _request: ImportModpackRequest,
        ) -> Result<Instance, InstanceServiceError> {
            self.calls.lock().expect("lock").push("import_modpack");
            Ok(sample_instance())
        }
    }

    #[tokio::test]
    async fn service_contract_supports_query_and_crud_calls() {
        let service = FakeInstanceService { calls: Mutex::new(Vec::new()) };
        let id = InstanceId::new("server-42").expect("valid id");

        assert_eq!(service.list().await.expect("list").len(), 1);
        assert!(service.find(&id).await.expect("find").is_some());
        service.create(sample_spec()).await.expect("create");
        service.delete(&id).await.expect("delete");
        service.rename(&id, "新名字").await.expect("rename");
        service
            .update_path(&id, "/new/path")
            .await
            .expect("update_path");

        assert_eq!(
            *service.calls.lock().expect("lock"),
            vec!["list", "find", "create", "delete", "rename", "update_path"]
        );
    }
}
