//! 资源管理服务实现。
//!
//! 组合 `feature` 的资源管理器（实例资源管理）与市场获取器（市场查询），
//! 并向宿主提供统一端口。下载动作由前端驱动：后端只负责"解析下载目标"
//! 与"安装已下载文件"，进度显示复用前端 TaskPill 机制。
//!
//! # 市场下载临时文件
//!
//! [`market_resolve_download`](ResourceService::market_resolve_download) 解析出的
//! `save_path` 位于应用数据根目录的 `market-tmp/` 下，文件名带
//! `项目-ID-版本-ID` 前缀以避免同名冲突。前端把文件下载到该路径后调
//! [`install`](ResourceService::install)：安装成功即由本服务清理临时文件
//! （失败时保留，便于用户重试）。
//!
//! 错误分层：`ResourceManagerError` / `MarketError` 经映射收敛为契约错误
//! [`ResourceServiceError`]，不向宿主泄露底层细节。

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use sealantern_contract::{InstanceServiceError, ResourceServiceError};
use sealantern_core::instance::InstanceId;
use sealantern_feature::resource::manager::{
    InstanceExtension, InstanceExtensionKind, ManagedResource, ReconcileReport,
    ResourceManagerError, ResourceProvenance, ResourceTarget, install, list, remove,
    resource_targets, set_enabled, sync as sync_resources,
};
use sealantern_feature::resource::market::{
    Fetcher, MarketError, MarketSource, ModrinthFetcher, ResourceInfo, ResourceType, SearchResult,
    SpigetFetcher, Version, VersionFile,
};
use sealantern_infra::platform::get_app_data_dir;

use crate::port::{InstanceService, ResolvedDownload, ResourceService};

/// 市场下载临时目录名（位于应用数据根目录下）。
const MARKET_TMP_DIR: &str = "market-tmp";

/// 基于 `feature` 资源管理器与市场获取器的资源管理宿主能力实现。
pub struct CoreResourceService {
    instance: Arc<dyn InstanceService>,
    modrinth: Arc<dyn Fetcher>,
    spiget: Arc<dyn Fetcher>,
}

impl CoreResourceService {
    /// 以全局网络配置装配（生产路径）。
    pub fn new(instance: Arc<dyn InstanceService>) -> Self {
        Self {
            instance,
            modrinth: Arc::new(ModrinthFetcher::global()),
            spiget: Arc::new(SpigetFetcher::global()),
        }
    }

    /// 按来源选择市场获取器。
    fn fetcher(&self, source: MarketSource) -> &Arc<dyn Fetcher> {
        match source {
            MarketSource::Modrinth => &self.modrinth,
            MarketSource::Spiget => &self.spiget,
        }
    }

    /// 解析实例目录与资源目标（由服务端检查得到的生态标记推导）。
    async fn resolve_targets(
        &self,
        instance_id: &str,
    ) -> Result<(PathBuf, Vec<ResourceTarget>), ResourceServiceError> {
        let id = InstanceId::new(instance_id).map_err(|_| ResourceServiceError::InvalidInput)?;
        let instance = self
            .instance
            .find(&id)
            .await
            .map_err(map_instance_error)?
            .ok_or(ResourceServiceError::InstanceNotFound)?;

        let ecosystems = instance
            .server_metadata
            .as_ref()
            .and_then(|metadata| metadata.identity.as_ref())
            .map(|identity| identity.ecosystems.clone())
            .unwrap_or_default();
        let targets = resource_targets(&ecosystems);

        if targets.is_empty() {
            // 纯原版等：没有 mods/plugins 目录可管理。
            return Err(ResourceServiceError::NoResourceDirs);
        }
        Ok((instance.directory, targets))
    }
}

#[async_trait]
impl ResourceService for CoreResourceService {
    async fn list(&self, instance_id: &str) -> Result<ReconcileReport, ResourceServiceError> {
        let (directory, targets) = self.resolve_targets(instance_id).await?;
        list(&directory, &targets).await.map_err(map_manager_error)
    }

    async fn install(
        &self,
        instance_id: &str,
        source_path: &str,
        kind: ResourceType,
        provenance: Option<ResourceProvenance>,
    ) -> Result<ManagedResource, ResourceServiceError> {
        let (directory, targets) = self.resolve_targets(instance_id).await?;
        let target = select_target(&targets, kind)?;
        let installed = install(&directory, target, Path::new(source_path), provenance)
            .await
            .map_err(map_manager_error)?;

        // 市场下载的临时文件安装成功后清理；失败保留便于重试。
        cleanup_downloaded_tmp(source_path).await;
        Ok(installed)
    }

    async fn remove(
        &self,
        instance_id: &str,
        kind: ResourceType,
        file_name: &str,
    ) -> Result<(), ResourceServiceError> {
        let (directory, targets) = self.resolve_targets(instance_id).await?;
        let kind = extension_kind(kind)?;
        remove(&directory, &targets, kind, file_name)
            .await
            .map_err(map_manager_error)
    }

    async fn set_enabled(
        &self,
        instance_id: &str,
        kind: ResourceType,
        file_name: &str,
        enabled: bool,
    ) -> Result<InstanceExtension, ResourceServiceError> {
        let (directory, targets) = self.resolve_targets(instance_id).await?;
        let kind = extension_kind(kind)?;
        set_enabled(&directory, &targets, kind, file_name, enabled)
            .await
            .map_err(map_manager_error)
    }

    async fn sync(&self, instance_id: &str) -> Result<ReconcileReport, ResourceServiceError> {
        let (directory, targets) = self.resolve_targets(instance_id).await?;
        sync_resources(&directory, &targets)
            .await
            .map_err(map_manager_error)
    }

    async fn market_search(
        &self,
        source: MarketSource,
        query: &str,
        page: u32,
        page_size: u32,
    ) -> Result<SearchResult, ResourceServiceError> {
        self.fetcher(source)
            .search(query, page, page_size)
            .await
            .map_err(map_market_error)
    }

    async fn market_resource(
        &self,
        source: MarketSource,
        id: &str,
    ) -> Result<ResourceInfo, ResourceServiceError> {
        self.fetcher(source)
            .get_resource(id)
            .await
            .map_err(map_market_error)
    }

    async fn market_versions(
        &self,
        source: MarketSource,
        id: &str,
    ) -> Result<Vec<Version>, ResourceServiceError> {
        self.fetcher(source)
            .get_resource_versions(id)
            .await
            .map_err(map_market_error)
    }

    async fn market_resolve_download(
        &self,
        source: MarketSource,
        project_id: &str,
        version_id: &str,
    ) -> Result<ResolvedDownload, ResourceServiceError> {
        let fetcher = self.fetcher(source);

        // 资源类型（决定安装目标目录）；仅支持插件 / 模组。
        let info = fetcher
            .get_resource(project_id)
            .await
            .map_err(map_market_error)?;
        supported_kind(info.resource_type)?;

        // 定位目标版本与首选文件（无 primary 标记时取第一个）。
        let versions = fetcher
            .get_resource_versions(project_id)
            .await
            .map_err(map_market_error)?;
        let (version, file) =
            pick_version_file(&versions, version_id).ok_or(ResourceServiceError::NotFound)?;

        // 确保临时目录存在，前端下载落盘才能成功。
        let save_dir = get_app_data_dir().join(MARKET_TMP_DIR);
        tokio::fs::create_dir_all(&save_dir)
            .await
            .map_err(|_| ResourceServiceError::OperationFailed)?;
        let save_path = save_dir.join(unique_download_name(project_id, version_id, &file.filename));

        Ok(ResolvedDownload {
            url: file.url.clone(),
            filename: file.filename.clone(),
            save_path: save_path.to_string_lossy().to_string(),
            kind: info.resource_type,
            provenance: ResourceProvenance {
                source: source.into(),
                project_id: Some(project_id.to_string()),
                version_id: Some(version_id.to_string()),
                version_number: Some(version.version_number.clone()),
                installed_at_unix_secs: now_unix_secs(),
            },
        })
    }
}

/// 从资源目标中按种类选出安装目标。
fn select_target(
    targets: &[ResourceTarget],
    kind: ResourceType,
) -> Result<&ResourceTarget, ResourceServiceError> {
    let kind = extension_kind(kind)?;
    targets
        .iter()
        .find(|target| target.kind == kind)
        .ok_or(ResourceServiceError::NoResourceDirs)
}

/// 资源类型 → 实例扩展种类；仅插件 / 模组可作为实例资源管理。
fn extension_kind(kind: ResourceType) -> Result<InstanceExtensionKind, ResourceServiceError> {
    match kind {
        ResourceType::Plugin => Ok(InstanceExtensionKind::Plugin),
        ResourceType::Mod => Ok(InstanceExtensionKind::Mod),
        _ => Err(ResourceServiceError::Unsupported),
    }
}

/// 资源类型是否可作为实例资源管理（仅插件 / 模组；数据包等暂不支持）。
fn supported_kind(kind: ResourceType) -> Result<(), ResourceServiceError> {
    if matches!(kind, ResourceType::Plugin | ResourceType::Mod) {
        Ok(())
    } else {
        Err(ResourceServiceError::Unsupported)
    }
}

/// 从版本列表定位目标版本并挑选下载文件：优先 `primary`，否则取第一个。
fn pick_version_file<'v>(
    versions: &'v [Version],
    version_id: &str,
) -> Option<(&'v Version, &'v VersionFile)> {
    let version = versions.iter().find(|version| version.id == version_id)?;
    let file = version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())?;
    Some((version, file))
}

/// 生成互不冲突的临时下载文件名，避免不同项目的同名文件互相覆盖。
///
/// 各输入中的路径分隔符统一替换为 `_`，保证结果为单一普通路径组件。
fn unique_download_name(project_id: &str, version_id: &str, filename: &str) -> String {
    let safe = |value: &str| {
        value
            .chars()
            .map(|ch| if matches!(ch, '/' | '\\') { '_' } else { ch })
            .collect::<String>()
    };
    format!("{}-{}-{}", safe(project_id), safe(version_id), safe(filename))
}

/// 清理市场下载留下的临时文件；非 `market-tmp/` 路径不动。
async fn cleanup_downloaded_tmp(source_path: &str) {
    let source = Path::new(source_path);
    let tmp_root = get_app_data_dir().join(MARKET_TMP_DIR);
    if !source.starts_with(&tmp_root) {
        return;
    }
    if let Err(error) = tokio::fs::remove_file(source).await {
        tracing::warn!(
            target: "sealantern.application.resource",
            path = %source.display(),
            error = %error,
            "failed to clean downloaded temporary file"
        );
    }
}

fn map_manager_error(error: ResourceManagerError) -> ResourceServiceError {
    match error {
        ResourceManagerError::InstanceDirNotFound(_) => ResourceServiceError::InstanceNotFound,
        ResourceManagerError::InvalidFileName(_)
        | ResourceManagerError::UnsupportedExtension(_)
        | ResourceManagerError::AlreadyExists(_) => ResourceServiceError::InvalidInput,
        ResourceManagerError::NotFound(_) => ResourceServiceError::NotFound,
        ResourceManagerError::Manifest { .. }
        | ResourceManagerError::Io(_)
        | ResourceManagerError::Infra(_)
        | ResourceManagerError::InvalidExtension(_) => ResourceServiceError::OperationFailed,
    }
}

fn map_market_error(error: MarketError) -> ResourceServiceError {
    match error {
        MarketError::Config(_) => ResourceServiceError::InvalidInput,
        MarketError::NotFound { .. } => ResourceServiceError::NotFound,
        MarketError::Http { .. } | MarketError::Json { .. } => ResourceServiceError::Market,
        MarketError::Download(_) => ResourceServiceError::OperationFailed,
    }
}

fn map_instance_error(error: InstanceServiceError) -> ResourceServiceError {
    match error {
        InstanceServiceError::InvalidInput => ResourceServiceError::InvalidInput,
        _ => ResourceServiceError::OperationFailed,
    }
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use sealantern_core::instance::server_metadata::{
        ServerMetadataIdentity, ServerMetadataJava, ServerMetadataSnapshot, ServerMetadataSubject,
        ServerMetadataSubjectKind,
    };
    use sealantern_core::instance::{
        Instance, InstanceError, InstanceSpec, LocalLaunch, StartupMode,
    };
    use sealantern_feature::resource::manager::InstanceExtensionError;

    use super::*;

    // ── 纯逻辑：select_target ──────────────────────────────

    #[test]
    fn select_target_maps_kind_to_existing_directory() {
        let targets = vec![
            ResourceTarget::new(InstanceExtensionKind::Mod, "mods"),
            ResourceTarget::new(InstanceExtensionKind::Plugin, "plugins"),
        ];

        let mod_target = select_target(&targets, ResourceType::Mod).expect("mod target exists");
        assert_eq!(mod_target.kind, InstanceExtensionKind::Mod);

        let plugin_target =
            select_target(&targets, ResourceType::Plugin).expect("plugin target exists");
        assert_eq!(plugin_target.kind, InstanceExtensionKind::Plugin);
    }

    #[test]
    fn select_target_rejects_unsupported_kinds() {
        let targets = vec![ResourceTarget::new(InstanceExtensionKind::Mod, "mods")];
        assert!(matches!(
            select_target(&targets, ResourceType::Datapack),
            Err(ResourceServiceError::Unsupported)
        ));
        assert!(matches!(
            select_target(&targets, ResourceType::Shader),
            Err(ResourceServiceError::Unsupported)
        ));
    }

    #[test]
    fn select_target_reports_missing_directory() {
        let targets = vec![ResourceTarget::new(InstanceExtensionKind::Plugin, "plugins")];
        assert!(matches!(
            select_target(&targets, ResourceType::Mod),
            Err(ResourceServiceError::NoResourceDirs)
        ));
    }

    // ── 纯逻辑：supported_kind / pick_version_file ─────────

    #[test]
    fn supported_kinds_are_plugins_and_mods() {
        assert!(supported_kind(ResourceType::Plugin).is_ok());
        assert!(supported_kind(ResourceType::Mod).is_ok());
        assert!(matches!(
            supported_kind(ResourceType::Datapack),
            Err(ResourceServiceError::Unsupported)
        ));
        assert!(matches!(
            supported_kind(ResourceType::Shader),
            Err(ResourceServiceError::Unsupported)
        ));
        assert!(matches!(
            supported_kind(ResourceType::ResourcePack),
            Err(ResourceServiceError::Unsupported)
        ));
    }

    #[test]
    fn pick_version_prefers_primary_file_and_falls_back_to_first() {
        let versions = vec![Version {
            id: "v1".into(),
            name: "1.0".into(),
            version_number: "1.0".into(),
            game_versions: vec![],
            loaders: vec![],
            downloads: 0,
            files: vec![
                VersionFile {
                    url: "first".into(),
                    filename: "a.jar".into(),
                    size: 1,
                    primary: false,
                },
                VersionFile {
                    url: "main".into(),
                    filename: "b.jar".into(),
                    size: 2,
                    primary: true,
                },
            ],
        }];

        let (version, file) = pick_version_file(&versions, "v1").expect("version should be found");
        assert_eq!(version.id, "v1");
        assert_eq!(file.url, "main");
    }

    #[test]
    fn pick_version_falls_back_to_first_file_without_primary() {
        let versions = vec![Version {
            id: "v1".into(),
            name: "1.0".into(),
            version_number: "1.0".into(),
            game_versions: vec![],
            loaders: vec![],
            downloads: 0,
            files: vec![VersionFile {
                url: "only".into(),
                filename: "a.jar".into(),
                size: 1,
                primary: false,
            }],
        }];

        let (_version, file) = pick_version_file(&versions, "v1").expect("version should be found");
        assert_eq!(file.url, "only");
    }

    #[test]
    fn pick_version_returns_none_for_missing_version() {
        let versions = vec![Version {
            id: "v1".into(),
            name: "1.0".into(),
            version_number: "1.0".into(),
            game_versions: vec![],
            loaders: vec![],
            downloads: 0,
            files: vec![],
        }];

        assert!(pick_version_file(&versions, "v2").is_none());
    }

    #[test]
    fn unique_download_name_keeps_distinct_identities_and_sanitizes_separators() {
        let first = unique_download_name("proj-a", "v1", "sodium.jar");
        let second = unique_download_name("proj-b", "v1", "sodium.jar");
        assert_ne!(first, second, "same filename from different projects must not collide");

        let sanitized = unique_download_name("a/b", "v1", "x.jar");
        assert_eq!(sanitized, "a_b-v1-x.jar");
        assert!(!sanitized.contains('/'));
        assert!(!sanitized.contains('\\'));
    }

    // ── 错误映射表驱动 ─────────────────────────────────────

    #[test]
    fn maps_manager_errors_by_category() {
        let cases = [
            (
                ResourceManagerError::InstanceDirNotFound(PathBuf::from("x")),
                ResourceServiceError::InstanceNotFound,
            ),
            (
                ResourceManagerError::InvalidFileName("x".into()),
                ResourceServiceError::InvalidInput,
            ),
            (
                ResourceManagerError::UnsupportedExtension("x".into()),
                ResourceServiceError::InvalidInput,
            ),
            (
                ResourceManagerError::AlreadyExists("x".into()),
                ResourceServiceError::InvalidInput,
            ),
            (ResourceManagerError::NotFound("x".into()), ResourceServiceError::NotFound),
            (
                ResourceManagerError::Manifest {
                    path: PathBuf::from("x"),
                    message: "m".into(),
                },
                ResourceServiceError::OperationFailed,
            ),
            (
                ResourceManagerError::Io(std::io::Error::other("io")),
                ResourceServiceError::OperationFailed,
            ),
            (
                ResourceManagerError::InvalidExtension(InstanceExtensionError::EmptyFileName),
                ResourceServiceError::OperationFailed,
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(map_manager_error(input), expected, "manager error mapping mismatch");
        }
    }

    #[test]
    fn maps_market_errors_by_category() {
        let cases = [
            (MarketError::Config("x".into()), ResourceServiceError::InvalidInput),
            (
                MarketError::Http { operation: "op", source: "s".into() },
                ResourceServiceError::Market,
            ),
            (
                MarketError::Json { operation: "op", message: "m".into() },
                ResourceServiceError::Market,
            ),
            (MarketError::NotFound { resource: "r".into() }, ResourceServiceError::NotFound),
            (MarketError::Download("x".into()), ResourceServiceError::OperationFailed),
        ];
        for (input, expected) in cases {
            assert_eq!(map_market_error(input), expected, "market error mapping mismatch");
        }
    }

    // ── resolve_targets（fake InstanceService） ─────────────

    struct FakeInstanceService {
        instance: Option<Instance>,
    }

    #[async_trait]
    impl InstanceService for FakeInstanceService {
        async fn list(&self) -> Result<Vec<Instance>, InstanceServiceError> {
            Ok(vec![])
        }

        async fn find(&self, _id: &InstanceId) -> Result<Option<Instance>, InstanceServiceError> {
            Ok(self.instance.clone())
        }

        async fn create(&self, _spec: InstanceSpec) -> Result<Instance, InstanceServiceError> {
            unimplemented!("fake 不用于创建")
        }

        async fn delete(&self, _id: &InstanceId) -> Result<(), InstanceServiceError> {
            Ok(())
        }

        async fn rename(&self, _id: &InstanceId, _name: &str) -> Result<(), InstanceServiceError> {
            Ok(())
        }

        async fn update_path(
            &self,
            _id: &InstanceId,
            _path: &str,
        ) -> Result<(), InstanceServiceError> {
            Ok(())
        }

        async fn import_existing_server(
            &self,
            _request: sealantern_core::provisioning::ImportExistingServerRequest,
        ) -> Result<Instance, InstanceServiceError> {
            unimplemented!("fake 不用于导入")
        }

        async fn import_modpack(
            &self,
            _request: sealantern_core::provisioning::ImportModpackRequest,
        ) -> Result<Instance, InstanceServiceError> {
            unimplemented!("fake 不用于导入")
        }
    }

    fn sample_spec(directory: PathBuf) -> InstanceSpec {
        let startup_target = directory.join("server.jar");
        InstanceSpec {
            id: InstanceId::new("server-1").expect("valid id"),
            name: "测试服".into(),
            aliases: Vec::new(),
            core_type: "paper".into(),
            core_version: "1.20.4".into(),
            game_version: "1.20.4".into(),
            directory,
            port: 25565,
            max_memory_mib: 1024,
            min_memory_mib: 512,
            created_at_unix_secs: 0,
            last_started_at_unix_secs: None,
            server_metadata: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(startup_target),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        }
    }

    fn sample_instance(directory: PathBuf) -> Result<Instance, InstanceError> {
        Instance::new(sample_spec(directory))
    }

    /// 构造带指定生态标记的实例（用于通过 resolve_targets 的资源目录推导）。
    fn instance_with_ecosystems(
        directory: PathBuf,
        ecosystems: &[&str],
    ) -> Result<Instance, InstanceError> {
        let mut spec = sample_spec(directory);
        spec.server_metadata = Some(ServerMetadataSnapshot {
            schema_version: 1,
            inspected_at_unix_secs: 0,
            subject: ServerMetadataSubject {
                kind: ServerMetadataSubjectKind::Directory,
                size_bytes: None,
                modified_at_unix_secs: None,
                fingerprint: None,
            },
            identity: Some(ServerMetadataIdentity {
                category: "implementation".into(),
                implementation_key: "paper".into(),
                implementation_name: "Paper".into(),
                implementation_confidence: 100,
                version: None,
                version_confidence: 100,
                release_channel: None,
                ecosystems: ecosystems.iter().map(|value| value.to_string()).collect(),
            }),
            minecraft: None,
            java: ServerMetadataJava::default(),
            components: Vec::new(),
            launches: Vec::new(),
            diagnostics: Vec::new(),
        });
        Instance::new(spec)
    }

    #[tokio::test]
    async fn resolve_targets_rejects_missing_instance() {
        let service = CoreResourceService::new(Arc::new(FakeInstanceService { instance: None }));
        let error = service.resolve_targets("server-1").await.unwrap_err();
        assert!(matches!(error, ResourceServiceError::InstanceNotFound));
    }

    #[tokio::test]
    async fn resolve_targets_rejects_instances_without_ecosystems() {
        let instance = sample_instance(PathBuf::from("/tmp/server-1")).expect("valid instance");
        let service =
            CoreResourceService::new(Arc::new(FakeInstanceService { instance: Some(instance) }));
        let error = service.resolve_targets("server-1").await.unwrap_err();
        assert!(matches!(error, ResourceServiceError::NoResourceDirs));
    }

    // ── install 清理 market-tmp ─────────────────────────────

    #[tokio::test]
    async fn install_cleans_up_market_tmp_source_after_success() {
        // 构造 market-tmp 下载文件 + 实例目录。
        let tmp_root = get_app_data_dir().join(MARKET_TMP_DIR);
        tokio::fs::create_dir_all(&tmp_root)
            .await
            .expect("tmp dir should be created");
        let source = tmp_root.join("proj-v1-sodium.jar");
        tokio::fs::write(&source, b"jar")
            .await
            .expect("fixture file should be written");

        let root = std::env::temp_dir()
            .join(format!("sealantern-resource-service-{}", uuid::Uuid::new_v4()));
        let instance_dir = root.join("instance");
        tokio::fs::create_dir_all(instance_dir.join("mods"))
            .await
            .unwrap();

        let instance =
            instance_with_ecosystems(instance_dir.clone(), &["fabric"]).expect("valid instance");
        let service =
            CoreResourceService::new(Arc::new(FakeInstanceService { instance: Some(instance) }));

        let managed = service
            .install("server-1", source.to_str().expect("utf8 path"), ResourceType::Mod, None)
            .await
            .expect("install should succeed");
        assert_eq!(managed.file_name, "proj-v1-sodium.jar");
        assert!(instance_dir.join("mods/proj-v1-sodium.jar").exists());
        assert!(!source.exists(), "market-tmp source should be cleaned after install");

        std::fs::remove_dir_all(&tmp_root).ok();
        std::fs::remove_dir_all(&root).ok();
    }
}
