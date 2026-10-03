//! 服务器实例管理服务实现。
//!
//! 实现 [`crate::port::InstanceService`] 能力端口。实例以 `sl.json` 为权威
//! 存储（`DocumentStore` 锁内读写）；目录布局、信任名单与忽略名单统一来自
//! [`CoreSettingsService`]（懒加载 `SettingsManager`）——服务不再持有自己的
//! 注册表文件，「哪些实例受管理」由发现层 + 名单合成：
//!
//! - `discover`/`classify`：按 `AppLayout` + 注册表分区只读扫描与分类；
//! - `trust_instance`/`ignore_dir`/`register_extra_dir` 等：经
//!   `SettingsManager::update_registry` 做设置文件锁内的名单维护；
//! - `DocumentStore`：`sl.json` 的创建/重载/局部更新。
//!
//! `instances.json`（`sea_lantern_servers.json`）体系已随阶段 7 移除。
//! 错误分层：内部以应用层主错误 [`InstanceError`] 为源头，沿
//! `core/feature/infra` → `application::error::instance` → `contract::error`
//! 收敛；暴露 [`InstanceService`] 时统一转为接口契约错误 [`InstanceServiceError`]。

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sealantern_contract::InstanceServiceError;
use sealantern_contract::instance::{
    ClassifiedProblemEntry, DiscoveryProblemEntry, DiscoveryProblemKind, PendingInstance,
};
use sealantern_core::instance::{Instance, InstanceId, InstanceSpec};
use sealantern_core::provisioning::{
    ImportExistingServerRequest, ImportModpackError as CoreModpackError, ImportModpackRequest,
    SourceType, build_import_spec, infer_source_type, plan_existing_instance, plan_import_modpack,
    source_directories_equal, validate_source_directory,
};
use sealantern_feature::config::instance::{
    ClassifiedInstance, DiscoveryOrigin, DiscoveryProblem, DocumentError, DocumentStore,
    InstanceDocument, TrustReport, TrustState, classify, discover, ignore_dir, orphan_trusted_ids,
    probe_dir, register_extra_dir, trust_instance, unignore_dir, unregister_extra_dir,
    untrust_instance,
};
use sealantern_infra::archive::{
    ArchiveFormat, detect_archive_format, extract_tar_gz, extract_zip,
};
use sealantern_infra::platform::AppLayout;
use uuid::Uuid;

use crate::error::InstanceError;
use crate::port::{InstanceDiscoveryView, InstanceService};
use crate::service::settings::CoreSettingsService;

/// 实例目录名长度：v4 UUID 去除连字符后取前 30 位（沿用 v1.2.0 的目录命名形态）。
const INSTANCE_DIRECTORY_NAME_LEN: usize = 30;

/// 一次「布局 + 名单快照 + 只读发现 + 信任分类」的扫描结果。
///
/// 布局与名单是读取时刻的快照（发现层纯只读，快照即可满足查询）；
/// 写路径（信任/忽略/目录注册）由调用方经 `lock_manager` 单独持锁完成，
/// 不与本快照共享锁。
struct InstanceScan {
    /// 当前生效的目录布局。
    layout: AppLayout,
    /// 发现结果与名单合成后的分类报告（含 Unmarked/Ignored 条目）。
    classified: TrustReport,
    /// 登记为受信任、但本次扫描中不存在对应实例的 id。
    orphan_trusted_ids: Vec<String>,
}

/// 基于 `core` + `feature` 的实例管理宿主能力实现。
pub struct CoreInstanceService {
    settings: Arc<CoreSettingsService>,
}

impl CoreInstanceService {
    /// 以共享的设置服务构造实例服务（设置必须先于实例装配）。
    pub fn new(settings: Arc<CoreSettingsService>) -> Self {
        Self { settings }
    }

    /// 测试/独立装配用：以独占的设置服务构造。
    pub fn with_settings(settings: CoreSettingsService) -> Self {
        Self::new(Arc::new(settings))
    }

    /// 完整扫描：布局 + 名单快照 → 只读发现 → 信任分类 + 孤儿记录。
    async fn scan(&self) -> Result<InstanceScan, InstanceError> {
        let layout = self.settings.layout().await.map_err(internal_error)?;
        let registry = self.settings.registry().await.map_err(internal_error)?;
        let report = discover(&layout, &registry).await;
        let orphans = orphan_trusted_ids(&report, &registry);
        let classified = classify(report, &registry);
        Ok(InstanceScan {
            layout,
            classified,
            orphan_trusted_ids: orphans,
        })
    }

    /// 在分类结果中按 id 查找条目（不限信任状态）。
    fn find_classified<'a>(
        classified: &'a TrustReport,
        id: &InstanceId,
    ) -> Option<&'a ClassifiedInstance> {
        classified
            .instances
            .iter()
            .find(|item| item.instance.document.id.as_str() == id.as_str())
    }

    /// 受信任实例的目录列表，供实例文档级访问者（cron/备份等）按
    /// `DocumentStore` 打开 `sl.json`。未标记/已忽略的实例不在列——
    /// 它们的任务不应被自动调度执行。
    pub(crate) async fn trusted_instance_dirs(&self) -> Result<Vec<PathBuf>, InstanceError> {
        let scan = self.scan().await?;
        Ok(scan
            .classified
            .instances
            .iter()
            .filter(|item| item.state == TrustState::Trusted)
            .map(|item| item.instance.dir.clone())
            .collect())
    }

    /// 更新实例的最后启动时间（服务器进程成功拉起后调用）。
    ///
    /// 目标须是已发现的实例（`sl.json` 权威）；找不到时按 `NotFound` 上报。
    pub async fn update_last_started(&self, id: &InstanceId) -> Result<(), InstanceError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let scan = self.scan().await?;
        let item = Self::find_classified(&scan.classified, id).ok_or(InstanceError::NotFound)?;
        let mut store = DocumentStore::load(DocumentStore::path_for(&item.instance.dir))
            .await
            .map_err(doc_error)?;
        store
            .update(|document| {
                document.last_started_at_unix_secs = Some(now);
                Ok(true)
            })
            .await
            .map_err(doc_error)?;
        Ok(())
    }

    /// 内部创建实例：校验 → 冲突检查 → 写 `sl.json` → 名单登记，返回应用层主错误。
    ///
    /// 冲突检查针对「同 id」与「同目录」：同目录已有实例、或该目录存在不可用
    /// 文档的问题条目时拒绝；`MissingDocument` 不在此列——裸目录允许通过创建
    /// 接管（与 `import_existing_server` 的语义一致）。
    async fn create_inner(&self, spec: InstanceSpec) -> Result<Instance, InstanceError> {
        let instance = Instance::new(spec)?;
        let scan = self.scan().await?;

        if scan
            .classified
            .instances
            .iter()
            .any(|item| item.instance.document.id.as_str() == instance.id.as_str())
        {
            return Err(InstanceError::AlreadyExists);
        }
        if scan
            .classified
            .instances
            .iter()
            .any(|item| source_directories_equal(&item.instance.dir, &instance.directory))
        {
            return Err(InstanceError::AlreadyExists);
        }
        for classified_problem in &scan.classified.problems {
            if matches!(classified_problem.problem, DiscoveryProblem::MissingDocument { .. }) {
                continue;
            }
            if problem_dirs(&classified_problem.problem)
                .iter()
                .any(|dir| source_directories_equal(dir, &instance.directory))
            {
                return Err(InstanceError::AlreadyExists);
            }
        }

        // 写 sl.json（目录位置即实例身份；启动目标在文档层换算为相对路径）。
        let document = InstanceDocument::try_from_instance(&instance).map_err(doc_error)?;
        DocumentStore::create(DocumentStore::path_for(&instance.directory), document)
            .await
            .map_err(doc_error)?;

        // 名单登记（设置文件锁内）：信任 id；非 `instances/` 下的目录登记为
        // 附加服务器目录；目录曾被忽略时先解除忽略（显式创建即用户处置）。
        let mut manager = self.settings.lock_manager().await.map_err(internal_error)?;
        let register_extra = !is_managed_dir(&instance.directory, &manager.layout());
        if register_extra {
            register_extra_dir(&mut manager, &instance.directory)
                .await
                .map_err(internal_error)?;
        }
        unignore_dir(&mut manager, &instance.directory)
            .await
            .map_err(internal_error)?;
        trust_instance(&mut manager, instance.id.as_str())
            .await
            .map_err(internal_error)?;
        drop(manager);

        tracing::info!(
            target: "sealantern.application.instance",
            id = %instance.id.as_str(),
            dir = %instance.directory.display(),
            "instance created"
        );
        Ok(instance)
    }

    /// 内部删除实例：撤销信任登记并把目录加入忽略名单（「移除但保留目录」语义）。
    async fn delete_inner(&self, id: &InstanceId) -> Result<(), InstanceError> {
        let scan = self.scan().await?;
        let item = scan
            .classified
            .instances
            .iter()
            .find(|item| {
                item.state == TrustState::Trusted
                    && item.instance.document.id.as_str() == id.as_str()
            })
            .ok_or(InstanceError::NotFound)?;
        let dir = item.instance.dir.clone();

        let mut manager = self.settings.lock_manager().await.map_err(internal_error)?;
        untrust_instance(&mut manager, id.as_str())
            .await
            .map_err(internal_error)?;
        ignore_dir(&mut manager, &dir)
            .await
            .map_err(internal_error)?;
        drop(manager);

        tracing::info!(
            target: "sealantern.application.instance",
            id = %id.as_str(),
            dir = %dir.display(),
            "instance removed (directory kept)"
        );
        Ok(())
    }

    /// 内部重命名实例：先经 `Instance::new` 校验与规范化，再写回 `sl.json` 名称。
    async fn rename_inner(&self, id: &InstanceId, name: &str) -> Result<(), InstanceError> {
        let scan = self.scan().await?;
        let item = Self::find_classified(&scan.classified, id).ok_or(InstanceError::NotFound)?;

        let mut spec = item.instance.document.to_spec(item.instance.dir.clone());
        spec.name = name.to_owned();
        let instance = Instance::new(spec)?;

        let mut store = DocumentStore::load(DocumentStore::path_for(&item.instance.dir))
            .await
            .map_err(doc_error)?;
        store
            .update(|document| {
                if document.name == instance.name {
                    return Ok(false);
                }
                document.name = instance.name.clone();
                Ok(true)
            })
            .await
            .map_err(doc_error)?;
        Ok(())
    }

    /// 内部更新实例目录：验证目标目录归属本实例后同步附加目录登记。
    ///
    /// `sl.json` 随目录移动，登记层只需维护 `extra_server_dirs`：目标目录
    /// 探查出的实例 id 必须等于本实例 id（目录已搬迁完毕的场景）；其他
    /// 结果一律拒绝。目标在 `instances/` 外时登记为附加目录，旧位置的
    /// 附加登记同步收回。
    async fn update_path_inner(&self, id: &InstanceId, path: &str) -> Result<(), InstanceError> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err(InstanceError::InvalidInput);
        }
        let target = PathBuf::from(trimmed);

        let scan = self.scan().await?;
        let item = Self::find_classified(&scan.classified, id).ok_or(InstanceError::NotFound)?;
        if source_directories_equal(&item.instance.dir, &target) {
            return Ok(());
        }
        let old_extra = item.instance.origin == DiscoveryOrigin::Extra;

        let origin = if is_managed_dir(&target, &scan.layout) {
            DiscoveryOrigin::Managed
        } else {
            DiscoveryOrigin::Extra
        };
        match probe_dir(target.clone(), origin).await {
            Ok(discovered) if discovered.document.id.as_str() == id.as_str() => {}
            Ok(_) => return Err(InstanceError::InvalidInput),
            Err(problem) => return Err(discovery_problem_error(&problem)),
        }

        let new_extra = !is_managed_dir(&target, &scan.layout);
        let mut manager = self.settings.lock_manager().await.map_err(internal_error)?;
        if new_extra {
            register_extra_dir(&mut manager, &target)
                .await
                .map_err(internal_error)?;
        }
        if old_extra {
            unregister_extra_dir(&mut manager, &item.instance.dir)
                .await
                .map_err(internal_error)?;
        }
        drop(manager);
        Ok(())
    }

    /// 导入已有服务器目录：校验 → 探查 →（已建档：登记 / 裸目录：识别构建）→ 持久化。
    ///
    /// 返回应用层富错误（携带 PathBuf 用于日志详情）；契约层
    /// （[`InstanceService`] 的 `import_existing_server`）在此之上收敛为薄契约错误。
    /// 编排逻辑收口于 service 层，命令层（Tauri / Axum）只负责参数转发与错误映射。
    /// 导入的实例直接引用原始目录（FR-5：不复制文件）；检查与构建规格为同步文件
    /// 系统扫描，经 `spawn_blocking` 调度到阻塞线程池。
    async fn import_existing_server_inner(
        &self,
        request: ImportExistingServerRequest,
    ) -> Result<Instance, InstanceError> {
        validate_source_directory(&request.source_directory)?;

        let scan = self.scan().await?;
        let origin = if is_managed_dir(&request.source_directory, &scan.layout) {
            DiscoveryOrigin::Managed
        } else {
            DiscoveryOrigin::Extra
        };

        match probe_dir(request.source_directory.clone(), origin).await {
            Ok(discovered) => {
                // 目录已有合法 sl.json：核实归属后直接登记信任/忽略/附加名单。
                // 去重只拦 Trusted——Unmarked/Ignored 目录经此转正（unignore+trust），
                // 否则被忽略或待处理的实例将没有任何恢复入口。
                if scan.classified.instances.iter().any(|item| {
                    item.state == TrustState::Trusted
                        && source_directories_equal(&item.instance.dir, &request.source_directory)
                }) {
                    return Err(InstanceError::SourceAlreadyImported);
                }
                // 同一 id 已在其它目录受信任：不同目录的同名实例才算冲突。
                if scan.classified.instances.iter().any(|item| {
                    item.state == TrustState::Trusted
                        && item.instance.document.id.as_str() == discovered.document.id.as_str()
                        && !source_directories_equal(&item.instance.dir, &request.source_directory)
                }) {
                    return Err(InstanceError::AlreadyExists);
                }

                let mut manager = self.settings.lock_manager().await.map_err(internal_error)?;
                if origin == DiscoveryOrigin::Extra {
                    register_extra_dir(&mut manager, &request.source_directory)
                        .await
                        .map_err(internal_error)?;
                }
                unignore_dir(&mut manager, &request.source_directory)
                    .await
                    .map_err(internal_error)?;
                trust_instance(&mut manager, discovered.document.id.as_str())
                    .await
                    .map_err(internal_error)?;
                drop(manager);

                let spec = discovered.document.to_spec(discovered.dir.clone());
                Instance::new(spec).map_err(InstanceError::from)
            }
            Err(DiscoveryProblem::MissingDocument { .. }) => {
                // 裸目录接管：识别服务器构件并构建导入规格（同步文件系统
                // 扫描经 spawn_blocking 调度到阻塞线程池）。
                let import_request = tokio::task::spawn_blocking({
                    let request = request.clone();
                    move || build_import_spec(&request)
                })
                .await
                .map_err(|_| InstanceError::InspectionPanicked)?
                .map_err(InstanceError::from)?;

                let plan = plan_existing_instance(import_request).map_err(InstanceError::from)?;

                self.create_inner(plan.instance.spec())
                    .await
                    .map_err(|e| InstanceError::ImportCreateFailed { source: e.into() })
            }
            Err(problem) => Err(discovery_problem_error(&problem)),
        }
    }
}

#[async_trait]
impl InstanceService for CoreInstanceService {
    async fn list(&self) -> Result<Vec<Instance>, InstanceServiceError> {
        let scan = self.scan().await.map_err(InstanceServiceError::from)?;
        let mut instances = Vec::new();
        for item in &scan.classified.instances {
            if item.state == TrustState::Trusted {
                instances.push(to_instance(item).map_err(InstanceServiceError::from)?);
            }
        }
        instances.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(instances)
    }

    async fn find(&self, id: &InstanceId) -> Result<Option<Instance>, InstanceServiceError> {
        let scan = self.scan().await.map_err(InstanceServiceError::from)?;
        match Self::find_classified(&scan.classified, id) {
            Some(item) if item.state == TrustState::Trusted => {
                Ok(Some(to_instance(item).map_err(InstanceServiceError::from)?))
            }
            _ => Ok(None),
        }
    }

    async fn discovery(&self) -> Result<InstanceDiscoveryView, InstanceServiceError> {
        let scan = self.scan().await.map_err(InstanceServiceError::from)?;
        let mut trusted = Vec::new();
        let mut pending = Vec::new();
        for item in &scan.classified.instances {
            match item.state {
                TrustState::Trusted => {
                    trusted.push(to_instance(item).map_err(InstanceServiceError::from)?)
                }
                TrustState::Unmarked => pending.push(PendingInstance {
                    dir: item.instance.dir.clone(),
                    instance_id: item.instance.document.id.as_str().to_string(),
                    name: item.instance.document.name.clone(),
                }),
                TrustState::Ignored => {}
            }
        }
        let problems = scan
            .classified
            .problems
            .iter()
            .map(|entry| ClassifiedProblemEntry {
                problem: problem_entry(&entry.problem),
                ignored: entry.ignored,
            })
            .collect();
        Ok(InstanceDiscoveryView {
            trusted,
            pending,
            problems,
            orphan_trusted_ids: scan.orphan_trusted_ids,
        })
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
    ) -> Result<Instance, InstanceServiceError> {
        // 目录从 request 摘出后再移交 inner——失败日志需要带上源目录便于定位。
        let source_directory = request.source_directory.clone();
        self.import_existing_server_inner(request)
            .await
            .map_err(|e| {
                tracing::warn!(
                    directory = %source_directory.display(),
                    error = ?e,
                    "import existing server failed"
                );
                e.into()
            })
    }

    async fn import_modpack(
        &self,
        mut request: ImportModpackRequest,
    ) -> Result<Instance, InstanceServiceError> {
        // run_path 缺省时落到主资源 `instances/` 容器（子目录由派生步生成）。
        if request.run_path.as_os_str().is_empty() {
            request.run_path = self
                .settings
                .layout()
                .await
                .map_err(|e| InstanceServiceError::from(internal_error(e)))?
                .instances_dir();
        }
        // run_path 的语义是「实例存放位置」而非最终目录：先派生实例独占子目录再规划。
        // 文件夹来源直接引用源目录、不使用 run_path，派生结果对其自然无影响。
        request.run_path = derive_instance_directory(&request.run_path)?;

        // 规划导入，构建实例规格（不执行文件操作）。
        let result = plan_import_modpack(&request).map_err(|error| {
            tracing::warn!(
                target: "sealantern.application.instance",
                error = %error,
                "modpack import plan failed"
            );
            match error {
                CoreModpackError::InvalidStartupMode(_) => InstanceError::InvalidInput,
                CoreModpackError::InvalidInstanceId(source) => InstanceError::Invalid { source },
                CoreModpackError::ExtractFailed(_)
                | CoreModpackError::CreateDirectoryFailed(_)
                | CoreModpackError::CopyFailed(_) => InstanceError::ImportFailed,
            }
        })?;

        match infer_source_type(&request.modpack_path) {
            SourceType::Archive => {
                // 解压是耗时同步 IO，放到 blocking 线程执行。
                let archive = request.modpack_path.clone();
                let destination = result.directory.clone();
                tokio::task::spawn_blocking(move || extract_archive(&archive, destination))
                    .await
                    .map_err(|join_error| {
                        tracing::error!(
                            target: "sealantern.application.instance",
                            error = %join_error,
                            "modpack extract task failed"
                        );
                        InstanceError::ImportFailed
                    })?
                    .map_err(|error| {
                        tracing::warn!(
                            target: "sealantern.application.instance",
                            error = %error,
                            "modpack extract failed"
                        );
                        InstanceError::ImportFailed
                    })?;
            }
            SourceType::JarFile => {
                // 创建运行目录并复制 jar。
                std::fs::create_dir_all(&result.directory).map_err(|error| {
                    tracing::warn!(
                        target: "sealantern.application.instance",
                        error = %error,
                        "failed to create run directory"
                    );
                    InstanceError::ImportFailed
                })?;
                if let Some(ref dest_path) = result.startup_target {
                    std::fs::copy(&request.modpack_path, dest_path).map_err(|error| {
                        tracing::warn!(
                            target: "sealantern.application.instance",
                            error = %error,
                            "failed to copy jar"
                        );
                        InstanceError::ImportFailed
                    })?;
                }
            }
            SourceType::Folder => {
                // 直接引用原目录。
            }
        }

        // 写 sl.json + 名单登记（含必要的附加目录注册）。
        self.create_inner(result.spec).await.map_err(Into::into)
    }
}

/// 实例目录是否位于主资源 `instances/` 容器下（词法比较，不做 IO）。
fn is_managed_dir(dir: &Path, layout: &AppLayout) -> bool {
    dir.parent()
        .map(|parent| parent == layout.instances_dir().as_path())
        .unwrap_or(false)
}

/// `ClassifiedInstance` → `Instance`：`to_spec` 已含全部字段，重建经 core 校验。
fn to_instance(item: &ClassifiedInstance) -> Result<Instance, InstanceError> {
    let spec = item.instance.document.to_spec(item.instance.dir.clone());
    Instance::new(spec).map_err(InstanceError::from)
}

/// `DiscoveryProblem`（feature 层）→ `DiscoveryProblemEntry`（契约层）。
fn problem_entry(problem: &DiscoveryProblem) -> DiscoveryProblemEntry {
    let mut entry = DiscoveryProblemEntry {
        kind: DiscoveryProblemKind::InvalidDocument,
        dir: None,
        dirs: Vec::new(),
        id: None,
        schema_version: None,
        alias_of: None,
        message: None,
    };
    match problem {
        DiscoveryProblem::UnreadableDir { dir, message } => {
            entry.kind = DiscoveryProblemKind::UnreadableDir;
            entry.dir = Some(dir.clone());
            entry.message = Some(message.clone());
        }
        DiscoveryProblem::MissingDocument { dir } => {
            entry.kind = DiscoveryProblemKind::MissingDocument;
            entry.dir = Some(dir.clone());
        }
        DiscoveryProblem::InvalidDocument { dir, message } => {
            entry.kind = DiscoveryProblemKind::InvalidDocument;
            entry.dir = Some(dir.clone());
            entry.message = Some(message.clone());
        }
        DiscoveryProblem::UnsupportedSchemaVersion { dir, found } => {
            entry.kind = DiscoveryProblemKind::UnsupportedSchemaVersion;
            entry.dir = Some(dir.clone());
            entry.schema_version = Some(*found);
        }
        DiscoveryProblem::DuplicateId { id, dirs } => {
            entry.kind = DiscoveryProblemKind::DuplicateId;
            entry.id = Some(id.clone());
            entry.dirs = dirs.clone();
        }
        DiscoveryProblem::DuplicateDirAlias { dir, alias_of } => {
            entry.kind = DiscoveryProblemKind::DuplicateDirAlias;
            entry.dir = Some(dir.clone());
            entry.alias_of = Some(alias_of.clone());
        }
    }
    entry
}

/// 问题条目涉及的目录集合（`DuplicateId` 携带多个目录；别名条目含本体与
/// 首次出现位置，冲突检查须同时覆盖两侧）。
fn problem_dirs(problem: &DiscoveryProblem) -> Vec<PathBuf> {
    match problem {
        DiscoveryProblem::UnreadableDir { dir, .. }
        | DiscoveryProblem::MissingDocument { dir }
        | DiscoveryProblem::InvalidDocument { dir, .. }
        | DiscoveryProblem::UnsupportedSchemaVersion { dir, .. } => vec![dir.clone()],
        DiscoveryProblem::DuplicateId { dirs, .. } => dirs.clone(),
        DiscoveryProblem::DuplicateDirAlias { dir, alias_of } => {
            vec![dir.clone(), alias_of.clone()]
        }
    }
}

/// 设置管理器错误收敛为 `Internal`：布局/名单的读取与写入失败都属于
/// 基础设施层面问题，不向调用方区分细节（细节由日志承载）。
fn internal_error(error: impl std::fmt::Display) -> InstanceError {
    InstanceError::Internal(error.to_string())
}

/// `sl.json` 文档错误 → 应用层实例错误。
fn doc_error(error: DocumentError) -> InstanceError {
    match error {
        DocumentError::Storage { source } => InstanceError::OperationFailed { source },
        DocumentError::UnsupportedSchemaVersion { .. } => {
            InstanceError::Internal(error.to_string())
        }
        DocumentError::AlreadyExists { .. } => InstanceError::AlreadyExists,
        DocumentError::Invalid { .. } => InstanceError::InvalidInput,
    }
}

/// 发现问题 → 应用层实例错误（`import_existing_server`/`update_path` 的
/// 单目录探查失败分类）。`MissingDocument` 在上层已被单独处理，此处兜底
/// 为 `InvalidInput`；重复类问题在单目录探查中不会出现，同样兜底。
fn discovery_problem_error(problem: &DiscoveryProblem) -> InstanceError {
    match problem {
        DiscoveryProblem::UnreadableDir { dir, .. } => {
            InstanceError::SourceUnavailable(dir.clone())
        }
        // `MissingDocument` 在上层被单独处理（裸目录接管）；重复类问题在单目录
        // 探查中不会出现——到达此处都按「输入无效」兜底。
        DiscoveryProblem::MissingDocument { .. }
        | DiscoveryProblem::InvalidDocument { .. }
        | DiscoveryProblem::UnsupportedSchemaVersion { .. }
        | DiscoveryProblem::DuplicateId { .. }
        | DiscoveryProblem::DuplicateDirAlias { .. } => InstanceError::InvalidInput,
    }
}

/// 依据请求中的运行目录派生实例实际落盘目录。
///
/// `run_path` 的语义是「实例存放位置」而非最终目录：
/// - 末级已是实例目录标识（30/32/36 位十六进制 UUID）时原样复用，
///   保证前端已自行拼出完整路径（Docker 分支）时不会二次嵌套；
/// - 其余情况在 `run_path` 下生成一个 30 位十六进制 UUID 子目录，
///   避免多个实例共用同一父目录互相覆盖，也避免默认开服路径
///   （与应用数据目录同为 `%APPDATA%/SeaLantern`）被服务器文件直接铺满。
///
/// 命名形态与 v1.2.0 的 `import_modpack` 一致（v4 UUID 去连字符取前 30 位）；
/// 额外补上对 30 位目录的识别——v1.2.0 只认 32/36 位，会导致自己生成的
/// 目录在重复调用时被再套一层。
fn derive_instance_directory(run_path: &Path) -> Result<PathBuf, InstanceError> {
    if run_path.as_os_str().is_empty() {
        return Err(InstanceError::InvalidInput);
    }
    if is_instance_directory_name(run_path) {
        return Ok(run_path.to_path_buf());
    }
    let folder_name = Uuid::new_v4().to_string().replace('-', "");
    Ok(run_path.join(&folder_name[..INSTANCE_DIRECTORY_NAME_LEN]))
}

/// 判断路径末级是否形如实例目录标识。
///
/// 兼容三种形态：30 位（本模块生成）、32 位无连字符 UUID、36 位带连字符 UUID。
fn is_instance_directory_name(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let is_hex = |text: &str| text.chars().all(|c| c.is_ascii_hexdigit());
    match name.len() {
        // 30 位实例目录片段、32 位无连字符 UUID。
        INSTANCE_DIRECTORY_NAME_LEN | 32 => is_hex(name),
        // 36 位带连字符 UUID：8-4-4-4-12。
        36 => {
            let groups: Vec<&str> = name.split('-').collect();
            groups.len() == 5
                && groups.iter().map(|group| group.len()).eq([8, 4, 4, 4, 12])
                && groups.iter().all(|group| is_hex(group))
        }
        _ => false,
    }
}

/// 按归档实际格式解压整合包。
///
/// 格式由文件魔数判定而非扩展名：`infer_source_type` 已按扩展名把 `.zip`、
/// `.tar.gz`、`.tgz` 都归为归档，但用户提供的文件扩展名可能与内容不符，
/// 按内容分派才能避免用错误的解析器读取。
fn extract_archive(
    archive: &Path,
    destination: PathBuf,
) -> Result<(), sealantern_infra::archive::ArchiveError> {
    match detect_archive_format(archive)? {
        ArchiveFormat::Zip => extract_zip(archive, destination)?,
        ArchiveFormat::TarGz => extract_tar_gz(archive, destination)?,
        // 未来新增的格式：本调用点尚未实现解压，按不受支持处理。
        _ => {
            return Err(sealantern_infra::archive::ArchiveError::InvalidSource {
                path: archive.to_path_buf(),
                reason: "archive format is not supported by the modpack importer",
            });
        }
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use sealantern_core::instance::{LocalLaunch, StartupMode};
    use sealantern_core::provisioning::ImportExistingServerRequest;
    use sealantern_feature::config::SettingsManager;

    use sealantern_contract::InstanceServiceError;

    use super::*;

    /// 以临时根目录装配服务：`settings.json` 落在根下，`instances/` 容器即
    /// `root/instances`——实例目录默认放进该容器（受管），无需附加目录登记。
    async fn test_service(root: &Path) -> CoreInstanceService {
        let manager = SettingsManager::load(root.join("settings.json"))
            .await
            .expect("load settings manager");
        CoreInstanceService::with_settings(CoreSettingsService::with_manager(manager))
    }

    /// 构造落位真实目录的实例规格：`directory`/`startup_target` 一致且位于
    /// `root/instances/{id}`（实例目录存在与否不影响创建——`sl.json` 写入会
    /// 自补父目录；但 `Instance::new` 校验要求启动目标在目录内）。
    fn sample_spec(root: &Path, id: &str) -> InstanceSpec {
        let dir = root.join("instances").join(id);
        InstanceSpec {
            id: InstanceId::new(id).expect("valid id"),
            name: format!("server-{id}"),
            aliases: Vec::new(),
            core_type: "paper".into(),
            core_version: "1.20.4".into(),
            game_version: "1.20.4".into(),
            directory: dir.clone(),
            port: 25565,
            max_memory_mib: 2048,
            min_memory_mib: 512,
            created_at_unix_secs: 0,
            last_started_at_unix_secs: None,
            server_metadata: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(dir.join("server.jar")),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        }
    }

    #[tokio::test]
    async fn persists_reads_and_deletes_an_instance() {
        let root = tempfile::tempdir().expect("temp dir");
        let service = test_service(root.path()).await;

        let created = service
            .create(sample_spec(root.path(), "a"))
            .await
            .expect("create");
        assert_eq!(created.name, "server-a");

        let listed = service.list().await.expect("list");
        assert_eq!(listed.len(), 1);

        let found = service.find(&created.id).await.expect("find");
        assert!(found.is_some());

        service
            .rename(&created.id, "renamed")
            .await
            .expect("rename");
        assert_eq!(service.list().await.expect("list")[0].name, "renamed");

        service.delete(&created.id).await.expect("delete");
        assert!(service.list().await.expect("list").is_empty());
    }

    #[tokio::test]
    async fn rename_missing_instance_reports_not_found() {
        let root = tempfile::tempdir().expect("temp dir");
        let service = test_service(root.path()).await;
        let missing = InstanceId::new("missing").expect("valid id");
        let result = service.rename(&missing, "x").await;
        assert_eq!(result, Err(InstanceServiceError::InstanceNotFound));
    }

    #[tokio::test]
    async fn create_with_duplicate_id_reports_already_exists() {
        let root = tempfile::tempdir().expect("temp dir");
        let service = test_service(root.path()).await;

        service
            .create(sample_spec(root.path(), "dup"))
            .await
            .expect("first create");

        // 相同 ID 再次创建应被拒绝，而不是覆盖原实例。
        let result = service.create(sample_spec(root.path(), "dup")).await;
        assert_eq!(result, Err(InstanceServiceError::AlreadyExists));

        // 原实例未被覆盖。
        let listed = service.list().await.expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "server-dup");
    }

    #[tokio::test]
    async fn rename_with_blank_name_is_rejected() {
        let root = tempfile::tempdir().expect("temp dir");
        let service = test_service(root.path()).await;
        let created = service
            .create(sample_spec(root.path(), "a"))
            .await
            .expect("create");

        for blank in ["", "   "] {
            let result = service.rename(&created.id, blank).await;
            assert_eq!(result, Err(InstanceServiceError::InvalidInput));
        }

        // 原名称未被修改。
        let found = service.find(&created.id).await.expect("find");
        assert_eq!(found.expect("exists").name, "server-a");
    }

    #[tokio::test]
    async fn update_path_with_blank_path_is_rejected() {
        let root = tempfile::tempdir().expect("temp dir");
        let service = test_service(root.path()).await;
        let created = service
            .create(sample_spec(root.path(), "a"))
            .await
            .expect("create");

        let result = service.update_path(&created.id, "").await;
        assert_eq!(result, Err(InstanceServiceError::InvalidInput));

        // 原路径未被修改。
        let found = service.find(&created.id).await.expect("find");
        assert_eq!(found.expect("exists").directory, root.path().join("instances").join("a"));
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

        let service = test_service(root.path()).await;

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

        let service = test_service(root.path()).await;

        let result = service
            .import_existing_server(import_request(missing, None))
            .await;

        assert!(matches!(result, Err(InstanceServiceError::SourceUnavailable)));
    }

    #[tokio::test]
    async fn import_existing_server_restores_ignored_instance() {
        // 「移除但保留目录」的实例（untrust + ignore）必须能经 import 恢复：
        // 去重只拦 Trusted，Ignored/Unmarked 目录走 unignore + trust 转正。
        let root = tempfile::tempdir().expect("temp dir");
        let service = test_service(root.path()).await;

        let created = service
            .create(sample_spec(root.path(), "revive"))
            .await
            .expect("create");
        let dir = created.directory.clone();
        service.delete(&created.id).await.expect("delete");
        assert!(service.list().await.expect("list").is_empty());

        let restored = service
            .import_existing_server(import_request(dir, None))
            .await
            .expect("import should restore ignored instance");

        // 身份沿用原 sl.json 的 id，名单状态回到 trusted。
        assert_eq!(restored.id, created.id);
        let listed = service.list().await.expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, created.id);
        let discovery = service.discovery().await.expect("discovery");
        assert_eq!(discovery.trusted.len(), 1);
        assert!(discovery.pending.is_empty());
        assert!(discovery.orphan_trusted_ids.is_empty());
    }

    #[tokio::test]
    async fn import_existing_server_rejects_trusted_source_directory() {
        // 已受信任的实例目录重复导入仍然被拒绝。
        let root = tempfile::tempdir().expect("temp dir");
        let service = test_service(root.path()).await;

        let created = service
            .create(sample_spec(root.path(), "registered"))
            .await
            .expect("create");

        let result = service
            .import_existing_server(import_request(created.directory.clone(), None))
            .await;
        assert_eq!(result, Err(InstanceServiceError::SourceAlreadyImported));
    }

    /// 在指定目录写入一份合法的 `sl.json`（实例 id 固定为 `id`，目录/启动
    /// 目标按 `dir` 重写），供「目录已有建档」的导入/发现场景使用。
    async fn write_sl_json(dir: &Path, id: &str) {
        let mut spec = sample_spec(dir.parent().expect("parent"), id);
        spec.directory = dir.to_path_buf();
        spec.launch.startup_target = Some(dir.join("server.jar"));
        let instance = Instance::new(spec).expect("valid instance");
        let document = InstanceDocument::try_from_instance(&instance).expect("document");
        DocumentStore::create(DocumentStore::path_for(dir), document)
            .await
            .expect("write sl.json");
    }

    #[tokio::test]
    async fn import_existing_server_rejects_duplicate_id_after_first_import() {
        // 两个未登记目录携带同一实例 id：前者导入转正后，后者按 id 冲突拒绝。
        // 去重只拦 Trusted——第二个目录的探查结果与首个实例同 id、不同目录，
        // 命中 AlreadyExists（而不是被静默并入受管实例）。
        let root = tempfile::tempdir().expect("temp dir");
        let dir_a = root.path().join("external-a");
        let dir_b = root.path().join("external-b");
        write_sl_json(&dir_a, "dup-ext").await;
        write_sl_json(&dir_b, "dup-ext").await;

        let service = test_service(root.path()).await;

        service
            .import_existing_server(import_request(dir_a, None))
            .await
            .expect("first import should succeed");

        let result = service
            .import_existing_server(import_request(dir_b, None))
            .await;
        assert_eq!(result, Err(InstanceServiceError::AlreadyExists));
    }

    #[tokio::test]
    async fn discovery_reports_duplicate_id_for_untrusted_dirs() {
        // 同一实例 id 出现在 managed 容器下的两个目录（均未登记信任）：发现层
        // 将整组目录移出 instances 并归入 problems——trusted/pending 都不含
        // 冲突成员，orphan 名单也不应有（无人登记过该 id）。
        let root = tempfile::tempdir().expect("temp dir");
        let dir_a = root.path().join("instances").join("dup-a");
        let dir_b = root.path().join("instances").join("dup-b");
        write_sl_json(&dir_a, "dup-managed").await;
        write_sl_json(&dir_b, "dup-managed").await;

        let service = test_service(root.path()).await;

        let discovery = service.discovery().await.expect("discovery");
        assert!(discovery.trusted.is_empty());
        assert!(discovery.pending.is_empty());
        assert!(discovery.orphan_trusted_ids.is_empty());
        let problem = discovery
            .problems
            .iter()
            .find(|entry| entry.problem.kind == DiscoveryProblemKind::DuplicateId)
            .expect("duplicate id problem should be reported");
        assert_eq!(problem.problem.id.as_deref(), Some("dup-managed"));
        assert_eq!(problem.problem.dirs.len(), 2);
        assert!(problem.problem.dirs.contains(&dir_a));
        assert!(problem.problem.dirs.contains(&dir_b));
    }

    #[tokio::test]
    async fn discovery_reports_and_clears_orphan_trusted_ids() {
        // 孤儿信任记录链路：登记了信任但磁盘上没有对应实例的 id 出现在
        // discovery 视图的 orphan_trusted_ids；撤销登记后不再报告（孤儿
        // 只报告不自动清理，清除动作由调用方决定——这里模拟用户处置）。
        let root = tempfile::tempdir().expect("temp dir");
        let service = test_service(root.path()).await;

        // 直接登记一个不存在的实例 id（等价于实例目录被外部删除后残留）。
        {
            let mut manager = service.settings.lock_manager().await.expect("lock manager");
            trust_instance(&mut manager, "gone-instance")
                .await
                .expect("trust should succeed");
        }

        let discovery = service.discovery().await.expect("discovery");
        assert_eq!(discovery.orphan_trusted_ids, vec!["gone-instance".to_string()]);

        // 处置：撤销信任后孤儿报告清空。
        {
            let mut manager = service.settings.lock_manager().await.expect("lock manager");
            untrust_instance(&mut manager, "gone-instance")
                .await
                .expect("untrust should succeed");
        }

        let discovery = service.discovery().await.expect("discovery");
        assert!(discovery.orphan_trusted_ids.is_empty());
    }

    #[tokio::test]
    async fn import_modpack_from_folder_registers_instance() {
        let root = tempfile::tempdir().expect("temp dir");
        let service = test_service(root.path()).await;

        let source = root.path().join("modpack-source");
        std::fs::create_dir_all(&source).expect("create source dir");
        std::fs::write(source.join("server.jar"), b"fake jar").expect("write jar");

        let request = ImportModpackRequest {
            name: "整合包测试".into(),
            modpack_path: source.clone(),
            java_path: PathBuf::from("java"),
            max_memory: 2048,
            min_memory: 1024,
            port: 25565,
            startup_mode: "jar".into(),
            startup_file_path: Some(PathBuf::from("server.jar")),
            core_type: Some("paper".into()),
            mc_version: Some("1.20.4".into()),
            custom_command: None,
            // 文件夹来源直接引用源目录，run_path 不参与；空路径走缺省分支。
            run_path: PathBuf::new(),
        };

        let instance = service
            .import_modpack(request)
            .await
            .expect("import modpack should succeed");
        assert_eq!(instance.name, "整合包测试");
        assert_eq!(instance.game_version, "1.20.4");
        // 文件夹来源直接引用原目录，不复制文件。
        assert_eq!(instance.directory, source);
    }

    /// 用 infra 的写入侧构造一个含 `server.jar` 的整合包归档。
    fn write_test_modpack(destination: &Path, format: ArchiveFormat) {
        let staging = destination
            .parent()
            .expect("archive has a parent")
            .join(format!("staging-{}", format.extension().replace('.', "-")));
        fs::create_dir_all(&staging).expect("create staging dir");
        fs::write(staging.join("server.jar"), b"fake jar").expect("write jar");
        match format {
            ArchiveFormat::Zip => {
                sealantern_infra::archive::create_zip(&staging, destination).expect("create zip");
            }
            ArchiveFormat::TarGz => {
                sealantern_infra::archive::create_tar_gz(&staging, destination)
                    .expect("create tar.gz");
            }
            // 测试只覆盖当前两种格式。
            _ => panic!("unexpected archive format in test helper"),
        }
        fs::remove_dir_all(&staging).expect("remove staging dir");
    }

    /// 归档来源的导入应把内容解压到运行目录，且格式按内容而非扩展名判定。
    async fn assert_modpack_archive_import(label: &str, filename: &str, format: ArchiveFormat) {
        let root = tempfile::tempdir().expect("temp dir");
        let service = test_service(root.path()).await;

        let archive = root.path().join(filename);
        write_test_modpack(&archive, format);
        let run_path = root.path().join("run");

        let request = ImportModpackRequest {
            name: format!("整合包-{label}"),
            modpack_path: archive,
            java_path: PathBuf::from("java"),
            max_memory: 2048,
            min_memory: 1024,
            port: 25565,
            startup_mode: "jar".into(),
            startup_file_path: Some(PathBuf::from("server.jar")),
            core_type: Some("paper".into()),
            mc_version: Some("1.20.4".into()),
            custom_command: None,
            run_path,
        };

        let instance = service
            .import_modpack(request)
            .await
            .expect("import modpack should succeed");
        assert_eq!(
            fs::read(instance.directory.join("server.jar")).expect("extracted jar"),
            b"fake jar"
        );
    }

    #[tokio::test]
    async fn import_modpack_from_zip_extracts_contents() {
        assert_modpack_archive_import("zip", "modpack.zip", ArchiveFormat::Zip).await;
    }

    #[tokio::test]
    async fn import_modpack_from_tar_gz_extracts_contents() {
        assert_modpack_archive_import("tar-gz", "modpack.tar.gz", ArchiveFormat::TarGz).await;
    }

    #[tokio::test]
    async fn import_modpack_detects_format_from_content_not_extension() {
        // 扩展名声称是 ZIP，内容实际是 tar.gz：分派须按内容进行。
        assert_modpack_archive_import("mislabeled", "modpack.zip", ArchiveFormat::TarGz).await;
    }

    #[tokio::test]
    async fn import_modpack_places_instance_below_run_path() {
        let root = tempfile::tempdir().expect("temp dir");
        let service = test_service(root.path()).await;

        let archive = root.path().join("modpack.zip");
        write_test_modpack(&archive, ArchiveFormat::Zip);
        let run_path = root.path().join("run");

        let request = ImportModpackRequest {
            name: "整合包-派生目录".into(),
            modpack_path: archive,
            java_path: PathBuf::from("java"),
            max_memory: 2048,
            min_memory: 1024,
            port: 25565,
            startup_mode: "jar".into(),
            startup_file_path: Some(PathBuf::from("server.jar")),
            core_type: Some("paper".into()),
            mc_version: Some("1.20.4".into()),
            custom_command: None,
            run_path: run_path.clone(),
        };

        let instance = service
            .import_modpack(request)
            .await
            .expect("import modpack");

        // 实例必须落在 run_path 的子目录里，而不是把服务器文件直接铺在 run_path。
        assert_eq!(instance.directory.parent(), Some(run_path.as_path()));
        assert_eq!(
            fs::read(instance.directory.join("server.jar")).expect("extracted jar"),
            b"fake jar"
        );
        assert!(!run_path.join("server.jar").exists());
    }

    #[test]
    fn derive_instance_directory_appends_uuid_segment() {
        let derived = derive_instance_directory(Path::new("/servers/root")).expect("derive");

        assert_eq!(derived.parent(), Some(Path::new("/servers/root")));
        let name = derived
            .file_name()
            .and_then(|n| n.to_str())
            .expect("dir name");
        assert_eq!(name.len(), INSTANCE_DIRECTORY_NAME_LEN);
        assert!(name.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn derive_instance_directory_reuses_existing_instance_directory() {
        // 幂等：末级已是实例目录标识时不得再嵌套一层。
        for name in [
            "0123456789abcdef0123456789abcd",       // 30 位
            "0123456789abcdef0123456789abcdef",     // 32 位
            "01234567-89ab-cdef-0123-456789abcdef", // 36 位
        ] {
            let path = Path::new("/servers").join(name);
            assert_eq!(derive_instance_directory(&path).expect("derive"), path);
        }
    }

    #[test]
    fn derive_instance_directory_rejects_empty_path() {
        assert!(matches!(
            derive_instance_directory(Path::new("")),
            Err(InstanceError::InvalidInput)
        ));
    }
}
