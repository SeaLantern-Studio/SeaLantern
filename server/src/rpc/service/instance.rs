//! 服务器实例管理的宿主能力端口。

use async_trait::async_trait;
use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_core::server::ServerStatus;

/// 实例管理操作失败的错误类别。
///
/// 与 [`ConsoleCommandServiceError`](super::ConsoleCommandServiceError) 一致，
/// 采用分类式错误：不携带主机路径、实例内容等敏感细节，底层失败详情应写入受控日志。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceServiceError {
    /// 指定的实例不存在。
    InstanceNotFound,
    /// 目标实例标识已存在（创建冲突）。
    AlreadyExists,
    /// 实例当前状态不允许该操作（如未运行时停止、已运行时重复启动）。
    InvalidState,
    /// 底层 IO / 供给 / 进程操作失败。
    OperationFailed,
    /// 该能力尚未实现（占位）。
    Unsupported,
}

/// 管理服务器实例的宿主能力端口。
///
/// 覆盖实例的查询、生命周期（启动/停止/强制停止）与 CRUD（创建/删除/重命名/改路径）。
/// 方法均为异步：内部涉及持久化 IO 与进程管理。供给（导入、整合包、扫描等）方法在
/// 后续迭代补充。实现方负责组合 `core` 的 repository、provisioning plan 与 process 能力，
/// 不依赖任何具体传输。
#[async_trait]
pub trait InstanceService: Send + Sync {
    /// 列出全部实例。
    async fn list(&self) -> Result<Vec<Instance>, InstanceServiceError>;

    /// 按 ID 查找实例，不存在时返回 `None`。
    async fn find(&self, id: &InstanceId) -> Result<Option<Instance>, InstanceServiceError>;

    /// 查询实例的当前运行状态。
    async fn status(&self, id: &InstanceId) -> Result<ServerStatus, InstanceServiceError>;

    /// 启动实例。
    async fn start(&self, id: &InstanceId) -> Result<(), InstanceServiceError>;

    /// 优雅停止实例。
    async fn stop(&self, id: &InstanceId) -> Result<(), InstanceServiceError>;

    /// 强制停止实例（终止进程树）。
    async fn force_stop(&self, id: &InstanceId) -> Result<(), InstanceServiceError>;

    /// 创建新实例并持久化。
    async fn create(&self, spec: InstanceSpec) -> Result<Instance, InstanceServiceError>;

    /// 删除实例，返回是否确实删除了某个实例。
    async fn delete(&self, id: &InstanceId) -> Result<bool, InstanceServiceError>;

    /// 重命名实例。
    async fn rename(&self, id: &InstanceId, name: &str) -> Result<(), InstanceServiceError>;

    /// 更新实例目录路径。
    async fn update_path(&self, id: &InstanceId, path: &str) -> Result<(), InstanceServiceError>;
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

        async fn status(&self, _id: &InstanceId) -> Result<ServerStatus, InstanceServiceError> {
            self.calls.lock().expect("lock").push("status");
            Ok(ServerStatus {
                process_id: 0,
                state: sealantern_core::server::ServerProcessState::Running,
            })
        }

        async fn start(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
            self.calls.lock().expect("lock").push("start");
            Ok(())
        }

        async fn stop(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
            self.calls.lock().expect("lock").push("stop");
            Ok(())
        }

        async fn force_stop(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
            self.calls.lock().expect("lock").push("force_stop");
            Ok(())
        }

        async fn create(&self, _spec: InstanceSpec) -> Result<Instance, InstanceServiceError> {
            self.calls.lock().expect("lock").push("create");
            Ok(sample_instance())
        }

        async fn delete(&self, _id: &InstanceId) -> Result<bool, InstanceServiceError> {
            self.calls.lock().expect("lock").push("delete");
            Ok(true)
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
    }

    #[tokio::test]
    async fn service_contract_supports_query_lifecycle_and_crud_calls() {
        let service = FakeInstanceService { calls: Mutex::new(Vec::new()) };
        let id = InstanceId::new("server-42").expect("valid id");

        assert_eq!(service.list().await.expect("list").len(), 1);
        assert!(service.find(&id).await.expect("find").is_some());
        assert!(matches!(
            service.status(&id).await.expect("status").state,
            sealantern_core::server::ServerProcessState::Running
        ));
        service.start(&id).await.expect("start");
        service.stop(&id).await.expect("stop");
        service.force_stop(&id).await.expect("force_stop");
        service.create(sample_spec()).await.expect("create");
        assert!(service.delete(&id).await.expect("delete"));
        service.rename(&id, "新名字").await.expect("rename");
        service
            .update_path(&id, "/new/path")
            .await
            .expect("update_path");

        assert_eq!(
            *service.calls.lock().expect("lock"),
            vec![
                "list",
                "find",
                "status",
                "start",
                "stop",
                "force_stop",
                "create",
                "delete",
                "rename",
                "update_path"
            ]
        );
    }
}
