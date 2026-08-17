//! 服务器实例记录管理服务实现。
//!
//! 实现 [`sealantern_interface::InstanceService`] 能力端口，用 `extra` 的
//! [`InstanceRegistry`](sealantern_extra::config::InstanceRegistry)（持久化到
//! `sea_lantern_servers.json`）驱动实例记录的查询与 CRUD。服务器进程生命周期
//! 由 [`sealantern_interface::ServerService`] 负责（见 `service/server.rs`）。
//!
//! 错误分层：内部以应用层主错误 [`InstanceError`] 为源头，沿
//! `core/extra/infra` → `application::error::instance` → `interface::error`
//! 收敛；暴露 [`InstanceService`] 时统一转为接口契约错误 [`InstanceServiceError`]。

use async_trait::async_trait;
use std::path::PathBuf;

use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_core::provisioning::{
    ImportExistingServerRequest, build_import_spec, plan_existing_instance,
    source_directories_equal, validate_source_directory,
};
use sealantern_extra::config::InstanceRegistry;
use sealantern_infra::platform::get_app_data_dir;
use sealantern_interface::{
    ImportExistingServerError as InterfaceImportError, InstanceService, InstanceServiceError,
};

use crate::error::ImportExistingServerError;
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

    /// 更新实例的最后启动时间（服务器进程成功拉起后调用）。
    pub async fn update_last_started(&self, id: &InstanceId) -> Result<(), InstanceError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut registry = self.registry.lock().await;
        let edited = registry
            .edit_instance(id, |instance| instance.last_started_at_unix_secs = Some(now))
            .await?;
        if !edited {
            return Err(InstanceError::NotFound);
        }
        Ok(())
    }

    /// 内部创建实例：core 校验规范化后持久化登记，返回应用层主错误。
    async fn create_inner(&self, spec: InstanceSpec) -> Result<Instance, InstanceError> {
        let instance = Instance::new(spec)?;
        let mut registry = self.registry.lock().await;
        // 持锁检查 ID 是否已存在，避免 save_instance 的 upsert 语义静默覆盖旧数据。
        if registry.get(&instance.id).is_some() {
            return Err(InstanceError::AlreadyExists);
        }
        registry.save_instance(&instance).await?;
        Ok(instance)
    }

    /// 内部删除实例，返回应用层主错误。
    async fn delete_inner(&self, id: &InstanceId) -> Result<(), InstanceError> {
        let mut registry = self.registry.lock().await;
        let deleted = registry.delete(id).await?;
        if !deleted {
            return Err(InstanceError::NotFound);
        }
        Ok(())
    }

    /// 内部重命名实例：改字段后经 core 校验再写回，返回应用层主错误。
    async fn rename_inner(&self, id: &InstanceId, name: &str) -> Result<(), InstanceError> {
        let mut registry = self.registry.lock().await;
        let mut instance = registry.get(id).cloned().ok_or(InstanceError::NotFound)?;
        instance.name = name.to_owned();
        // 写回前复用 core 字段校验，拒绝空名/纯空白名。
        instance.validate()?;
        registry.save_instance(&instance).await?;
        Ok(())
    }

    /// 内部更新实例目录路径：改字段后经 core 校验再写回，返回应用层主错误。
    async fn update_path_inner(&self, id: &InstanceId, path: &str) -> Result<(), InstanceError> {
        let mut registry = self.registry.lock().await;
        let mut instance = registry.get(id).cloned().ok_or(InstanceError::NotFound)?;
        instance.directory = PathBuf::from(path);
        // 写回前复用 core 字段校验，拒绝空路径。
        instance.validate()?;
        registry.save_instance(&instance).await?;
        Ok(())
    }

    /// 导入已有服务器目录：校验源目录 → 去重 → 构建导入规格 → 供给计划 → 持久化登记。
    ///
    /// 返回应用层富错误（携带 PathBuf 用于日志详情）；契约层（[`InstanceService`]
    /// 的 `import_existing_server`）在此之上收敛为薄契约错误。
    /// 编排逻辑收口于 service 层，命令层（Tauri / Axum）只负责参数转发与错误映射。
    /// 导入的实例直接引用原始目录（FR-5：不复制文件）；检查与构建规格为同步文件系统
    /// 扫描，经 `spawn_blocking` 调度到阻塞线程池，避免阻塞异步运行时核心线程。
    async fn import_existing_server_inner(
        &self,
        request: ImportExistingServerRequest,
    ) -> Result<Instance, ImportExistingServerError> {
        validate_source_directory(&request.source_directory)?;

        let instances = self
            .list()
            .await
            .map_err(ImportExistingServerError::ListFailed)?;
        if instances.iter().any(|instance| {
            source_directories_equal(
                instance.directory.as_path(),
                request.source_directory.as_path(),
            )
        }) {
            return Err(ImportExistingServerError::AlreadyImported);
        }

        let import_request = tokio::task::spawn_blocking({
            let request = request.clone();
            move || build_import_spec(&request)
        })
        .await
        .map_err(|_| ImportExistingServerError::InspectionPanicked)?
        .map_err(ImportExistingServerError::from)?;

        let plan = plan_existing_instance(import_request)
            .map_err(|_| ImportExistingServerError::PlanInvalid)?;

        self.create(plan.instance.spec())
            .await
            .map_err(ImportExistingServerError::CreateFailed)
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

    async fn create(&self, spec: InstanceSpec) -> Result<Instance, InstanceServiceError> {
        self.create_inner(spec).await.map_err(Into::into)
    }

    async fn delete(&self, id: &InstanceId) -> Result<(), InstanceServiceError> {
        self.delete_inner(id).await.map_err(Into::into)
    }

    async fn rename(&self, id: &InstanceId, name: &str) -> Result<(), InstanceServiceError> {
        self.rename_inner(id, name).await.map_err(Into::into)
    }

    async fn update_path(&self, id: &InstanceId, path: &str) -> Result<(), InstanceServiceError> {
        self.update_path_inner(id, path).await.map_err(Into::into)
    }

    async fn import_existing_server(
        &self,
        request: ImportExistingServerRequest,
    ) -> Result<Instance, InterfaceImportError> {
        self.import_existing_server_inner(request)
            .await
            .map_err(|rich| {
                tracing::warn!(error = ?rich, "import existing server failed");
                rich.into()
            })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use sealantern_core::instance::{LocalLaunch, StartupMode};
    use sealantern_core::provisioning::ImportExistingServerRequest;

    use sealantern_interface::ImportExistingServerError as InterfaceImportError;

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

        service.delete(&created.id).await.expect("delete");
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

    #[tokio::test]
    async fn create_with_duplicate_id_reports_already_exists() {
        let path = test_path("duplicate");
        let service = CoreInstanceService::with_path(&path)
            .await
            .expect("load service");

        service
            .create(sample_spec("dup"))
            .await
            .expect("first create");

        // 相同 ID 再次创建应被拒绝，而不是覆盖原实例。
        let result = service.create(sample_spec("dup")).await;
        assert_eq!(result, Err(InstanceServiceError::AlreadyExists));

        // 原实例未被覆盖。
        let listed = service.list().await.expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "服务器-dup");

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[tokio::test]
    async fn rename_with_blank_name_is_rejected() {
        let path = test_path("rename-blank");
        let service = CoreInstanceService::with_path(&path)
            .await
            .expect("load service");
        let created = service.create(sample_spec("a")).await.expect("create");

        for blank in ["", "   "] {
            let result = service.rename(&created.id, blank).await;
            assert_eq!(result, Err(InstanceServiceError::InvalidInput));
        }

        // 原名称未被修改。
        let found = service.find(&created.id).await.expect("find");
        assert_eq!(found.expect("exists").name, "服务器-a");

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[tokio::test]
    async fn update_path_with_blank_path_is_rejected() {
        let path = test_path("path-blank");
        let service = CoreInstanceService::with_path(&path)
            .await
            .expect("load service");
        let created = service.create(sample_spec("a")).await.expect("create");

        let result = service.update_path(&created.id, "").await;
        assert_eq!(result, Err(InstanceServiceError::InvalidInput));

        // 原路径未被修改。
        let found = service.find(&created.id).await.expect("find");
        assert_eq!(found.expect("exists").directory, PathBuf::from("/tmp/server-a"));

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    /// 写一个最小可识别服务器 jar：含 `Main-Class` 清单即可被通用检测器识别为可启动 jar。
    fn write_test_jar(path: &Path, manifest: &str) {
        use zip::write::FileOptions;
        let file = File::create(path).expect("create test JAR");
        let mut archive = zip::ZipWriter::new(file);
        archive
            .start_file("META-INF/MANIFEST.MF", FileOptions::<()>::default())
            .expect("create manifest entry");
        archive
            .write_all(manifest.as_bytes())
            .expect("write manifest");
        archive.finish().expect("finish test JAR");
    }

    fn import_request(
        source_directory: PathBuf,
        jvm_arguments: Option<Vec<String>>,
    ) -> ImportExistingServerRequest {
        ImportExistingServerRequest {
            source_directory,
            name: None,
            port: None,
            max_memory_mib: None,
            min_memory_mib: None,
            java_executable: None,
            jvm_arguments,
            selected_launch_profile_id: None,
        }
    }

    #[tokio::test]
    async fn import_existing_server_applies_explicit_jvm_override() {
        let root = tempfile::tempdir().expect("temp dir");
        let source = root.path().join("demo-server");
        fs::create_dir_all(&source).expect("create source dir");
        write_test_jar(
            &source.join("server.jar"),
            "Manifest-Version: 1.0\r\nMain-Class: com.example.DemoServer\r\n\r\n",
        );

        let service = CoreInstanceService::with_path(root.path().join("servers.json"))
            .await
            .expect("load service");

        let instance = service
            .import_existing_server(import_request(source, Some(vec!["-Xmx4G".to_string()])))
            .await
            .expect("import should succeed");

        // 用户显式 JVM 参数必须无条件覆盖检查识别所得（existing.rs:177-182 的覆盖语义）。
        assert_eq!(instance.launch.jvm_arguments, vec!["-Xmx4G".to_string()]);
    }

    #[tokio::test]
    async fn import_existing_server_rejects_unavailable_source() {
        let root = tempfile::tempdir().expect("temp dir");
        let missing = root.path().join("does-not-exist");

        let service = CoreInstanceService::with_path(root.path().join("servers.json"))
            .await
            .expect("load service");

        let result = service
            .import_existing_server(import_request(missing, None))
            .await;

        assert!(matches!(result, Err(InterfaceImportError::SourceUnavailable)));
    }
}
