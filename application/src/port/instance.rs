use async_trait::async_trait;
use sealantern_contract::InstanceServiceError;
use sealantern_contract::instance::{ClassifiedProblemEntry, PendingInstance};
use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_core::provisioning::{ImportExistingServerRequest, ImportModpackRequest};

/// 实例发现视图：已信任实例 + 待处理实例 + 问题清单 + 孤儿信任记录。
///
/// 由 `InstanceService::discovery` 返回，供宿主展示「哪些实例已纳入管理、
/// 哪些目录等待用户处置、哪些信任记录已失去对应实例」。字段名对外序列化为
/// `snake_case`，与契约 DTO 风格保持一致。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InstanceDiscoveryView {
    /// 已登记信任的实例（`list` 的同名集合）。
    pub trusted: Vec<Instance>,
    /// 身份有效但未登记的实例（待用户决定信任/忽略）。
    pub pending: Vec<PendingInstance>,
    /// 发现过程中记录的问题（含忽略标记）。
    pub problems: Vec<ClassifiedProblemEntry>,
    /// 登记为受信任、但磁盘上已不存在对应实例的 id。
    pub orphan_trusted_ids: Vec<String>,
}

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

    /// 实例发现视图：可信实例 + 待处理实例 + 问题 + 孤儿信任记录。
    async fn discovery(&self) -> Result<InstanceDiscoveryView, InstanceServiceError>;

    /// 创建新实例并持久化。
    async fn create(&self, spec: InstanceSpec) -> Result<Instance, InstanceServiceError>;

    /// 删除实例；实例不存在时返回 [`InstanceServiceError::InstanceNotFound`]。
    async fn delete(&self, id: &InstanceId) -> Result<(), InstanceServiceError>;

    /// 重命名实例。
    async fn rename(&self, id: &InstanceId, name: &str) -> Result<(), InstanceServiceError>;

    /// 更新实例目录路径。
    async fn update_path(&self, id: &InstanceId, path: &str) -> Result<(), InstanceServiceError>;

    /// 把实例登记为受信任（`trusted_instances`）。幂等。
    ///
    /// 目标实例须是本次扫描中身份有效（`sl.json` 可解析且通过校验）的条目；
    /// 实例不存在或文档无效时返回 [`InstanceServiceError::InstanceNotFound`] /
    /// [`InstanceServiceError::InvalidInput`]。
    async fn trust(&self, id: &InstanceId) -> Result<(), InstanceServiceError>;

    /// 撤销实例的信任登记。幂等：未登记时视为成功。
    async fn untrust(&self, id: &InstanceId) -> Result<(), InstanceServiceError>;

    /// 把目录加入忽略名单（`ignored_dirs`），此后该目录不再作为待处理项提示。
    ///
    /// 目录不必是有效实例：被忽略的目录本就可能没有 `sl.json`。
    /// 路径为空或不是目录时返回 [`InstanceServiceError::InvalidInput`]。
    async fn ignore_dir(&self, path: &str) -> Result<(), InstanceServiceError>;

    /// 把目录移出忽略名单。幂等：不在名单内时视为成功。
    async fn unignore_dir(&self, path: &str) -> Result<(), InstanceServiceError>;

    /// 为指定目录的实例重新分配 id（修复「同 id 出现在多个目录」的冲突）。
    ///
    /// **按目录定位**而非按 id：重复组里 id 是歧义的，发现层已把整组移出
    /// 实例列表，`find`/`trust`/`delete` 等按 id 的命令都够不着它们。
    ///
    /// `new_id` 为 `None` 时自动生成 v4 UUID；只改 `sl.json` 的 id，不动信任
    /// 名单——原 id 的信任自动归属「保留原 id 的那个目录」，本目录落回未标记态。
    async fn reassign_id(
        &self,
        dir: &str,
        new_id: Option<&str>,
    ) -> Result<(), InstanceServiceError>;

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
            required_java: None,
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

        async fn discovery(&self) -> Result<InstanceDiscoveryView, InstanceServiceError> {
            self.calls.lock().expect("lock").push("discovery");
            Ok(InstanceDiscoveryView::default())
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

        async fn trust(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
            self.calls.lock().expect("lock").push("trust");
            Ok(())
        }

        async fn untrust(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
            self.calls.lock().expect("lock").push("untrust");
            Ok(())
        }

        async fn ignore_dir(&self, _path: &str) -> Result<(), InstanceServiceError> {
            self.calls.lock().expect("lock").push("ignore_dir");
            Ok(())
        }

        async fn unignore_dir(&self, _path: &str) -> Result<(), InstanceServiceError> {
            self.calls.lock().expect("lock").push("unignore_dir");
            Ok(())
        }

        async fn reassign_id(
            &self,
            _dir: &str,
            _new_id: Option<&str>,
        ) -> Result<(), InstanceServiceError> {
            self.calls.lock().expect("lock").push("reassign_id");
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
        service.discovery().await.expect("discovery");

        assert_eq!(
            *service.calls.lock().expect("lock"),
            vec!["list", "find", "create", "delete", "rename", "update_path", "discovery"]
        );
    }
}
