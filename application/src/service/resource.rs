//! 资源管理服务实现。
//!
//! 组合 `feature` 的资源管理器（实例资源管理）与市场获取器（市场查询），
//! 并向宿主提供统一端口。下载动作由前端驱动：后端只负责"解析下载目标"
//! 与"安装已下载文件"，进度显示复用前端 TaskPill 机制。
//!
//! # 市场下载临时目录
//!
//! [`market_resolve_download`](ResourceService::market_resolve_download) 解析出的
//! `save_path` 位于应用数据根目录的 `market-tmp/<项目>-<版本>-<随机后缀>/`
//! 子目录下，**文件名保持市场原名**（唯一化只作用于目录，避免污染实例内
//! 文件名）；随机后缀让同一项目与版本的并发解析互不干扰。
//!
//! 前端把文件下载到该路径后调 [`install`](ResourceService::install)：安装
//! 成功后本服务**尽力**清理整个子目录（清理失败仅记日志，不影响安装结果）；
//! 下载失败或取消留下的残留由 `prune_stale_market_tmp` 按年龄回收。
//!
//! 错误分层：`ResourceManagerError` / `MarketError` 经映射收敛为契约错误
//! [`ResourceServiceError`]，不向宿主泄露底层细节。

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use sealantern_contract::{InstanceServiceError, ResourceServiceError};
use sealantern_core::instance::InstanceId;
use sealantern_feature::resource::manager::{
    InstanceExtension, InstanceExtensionKind, ManagedResource, ReconcileReport,
    ResourceManagerError, ResourceProvenance, ResourceTarget, ResourceTargets, install,
    is_single_normal_component, list, remove, resource_targets, set_enabled,
    sync as sync_resources,
};
use sealantern_feature::resource::market::{
    Fetcher, MarketError, MarketSource, ModrinthFetcher, ResourceInfo, ResourceType, SearchResult,
    SpigetFetcher, Version, VersionFile,
};
use sealantern_infra::platform::get_app_data_dir;

use crate::port::{InstanceService, ResolvedDownload, ResourceService};

/// 市场下载临时目录名（位于应用数据根目录下）。
const MARKET_TMP_DIR: &str = "market-tmp";

/// 陈旧临时目录的保留时长：超过此时长的子目录视为无主残留。
///
/// 目录名带随机后缀后，下载失败或用户取消留下的目录不会再被后续解析复用
/// （每次解析都生成新目录），因此需要按年龄主动回收。
const STALE_MARKET_TMP_AGE: Duration = Duration::from_secs(6 * 60 * 60);

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

    async fn targets(&self, instance_id: &str) -> Result<ResourceTargets, ResourceServiceError> {
        let (_, targets) = self.resolve_targets(instance_id).await?;
        Ok(ResourceTargets::new(targets))
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

        // 市场下载的临时目录安装成功后清理；失败保留便于重试。
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

        // 资源类型（决定安装目标目录）；仅支持插件 / 模组，与安装目标选择
        // 共用同一个判定函数。
        let info = fetcher
            .get_resource(project_id)
            .await
            .map_err(map_market_error)?;
        extension_kind(info.resource_type)?;

        // 定位目标版本与首选文件（无 primary 标记时取第一个）。
        let versions = fetcher
            .get_resource_versions(project_id)
            .await
            .map_err(map_market_error)?;
        let (version, file) =
            pick_version_file(&versions, version_id).ok_or(ResourceServiceError::NotFound)?;

        // 市场返回的文件名不可信：必须是单一普通路径组件，含分隔符或 `..`
        // 会逃出临时目录。
        if !is_single_normal_component(&file.filename) {
            return Err(ResourceServiceError::InvalidInput);
        }

        // 顺带回收陈旧残留（尽力而为，不影响本次解析）。
        prune_stale_market_tmp().await;

        // 唯一化只作用于目录名：`market-tmp/<项目>-<版本>-<随机>/<原文件名>`。
        // 安装进实例后 `mods/` 里保持市场原名，不被前缀污染。
        let save_dir = get_app_data_dir()
            .join(MARKET_TMP_DIR)
            .join(unique_download_dir(project_id, version_id));
        tokio::fs::create_dir_all(&save_dir)
            .await
            .map_err(|_| ResourceServiceError::OperationFailed)?;
        let save_path = save_dir.join(&file.filename);

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
///
/// 该种类没有对应目录时返回 [`ResourceServiceError::NoTargetForKind`]
/// （实例有资源目录、只是不含这一种），与"整个实例都没有资源目录"区分。
fn select_target(
    targets: &[ResourceTarget],
    kind: ResourceType,
) -> Result<&ResourceTarget, ResourceServiceError> {
    let kind = extension_kind(kind)?;
    targets
        .iter()
        .find(|target| target.kind == kind)
        .ok_or(ResourceServiceError::NoTargetForKind)
}

/// 资源类型 → 实例扩展种类；仅插件 / 模组可作为实例资源管理。
///
/// 本函数是"哪些种类受支持"的**唯一**判定处：安装目标选择与市场下载解析
/// 都经它校验，避免两处各写一份支持列表而漂移。
fn extension_kind(kind: ResourceType) -> Result<InstanceExtensionKind, ResourceServiceError> {
    match kind {
        ResourceType::Plugin => Ok(InstanceExtensionKind::Plugin),
        ResourceType::Mod => Ok(InstanceExtensionKind::Mod),
        _ => Err(ResourceServiceError::Unsupported),
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

/// 生成互不冲突的临时下载子目录名（`<项目>-<版本>-<随机后缀>`）。
///
/// 唯一化只作用于目录名，市场原文件名保持不变——安装进实例后 `mods/` 里
/// 就是 `sodium.jar`，而不是被前缀污染的 `AANobbMI-xJNm0swY-sodium.jar`。
/// 各标识符中的路径分隔符替换为 `_`，保证结果仍是单一普通路径组件。
///
/// 追加随机后缀是为了让**同一项目与版本的并发解析**各自拿到独立目录：若
/// 只按 `<项目>-<版本>` 命名，两个并发下载会共用同一个 `save_path`，互相
/// 覆盖对方正在写入的文件，或抢删对方的临时目录。
fn unique_download_dir(project_id: &str, version_id: &str) -> String {
    // 取 UUID 的前 8 位十六进制，足够避免碰撞且不使目录名过长。
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    format!(
        "{}-{}-{}",
        sanitize_component(project_id),
        sanitize_component(version_id),
        &nonce[..8]
    )
}

/// 把标识符中的路径分隔符替换为 `_`，避免拼出多级路径。
fn sanitize_component(value: &str) -> String {
    value
        .chars()
        .map(|ch| if matches!(ch, '/' | '\\') { '_' } else { ch })
        .collect()
}

/// 清理市场下载留下的临时目录；非 `market-tmp/<唯一子目录>/` 形态不动。
///
/// 路径比较走 `canonicalize`：`Path::starts_with` 是纯语法的组件比较、不解析
/// `..`，`market-tmp/../x.jar` 能骗过前缀检查，删除时却被 OS 解析到
/// `market-tmp` 之外。规范化后只清理 `market-tmp` 的**直接子目录**。
///
/// # 尽力而为
///
/// 本函数在安装**已经成功**之后调用，因此清理失败（无法规范化、删除失败）
/// **不影响安装结果**，仅记录告警日志——若因清理失败而返回错误，调用方会
/// 误判为安装失败并重试，反而撞上 `AlreadyExists`。残留由
/// `prune_stale_market_tmp` 按年龄回收。
async fn cleanup_downloaded_tmp(source_path: &str) {
    let Some(sub_dir) = Path::new(source_path).parent() else {
        return;
    };
    let tmp_root = get_app_data_dir().join(MARKET_TMP_DIR);

    let (Ok(sub_dir), Ok(tmp_root)) =
        (tokio::fs::canonicalize(sub_dir).await, tokio::fs::canonicalize(&tmp_root).await)
    else {
        return;
    };

    if sub_dir.parent() != Some(tmp_root.as_path()) {
        return;
    }

    if let Err(error) = tokio::fs::remove_dir_all(&sub_dir).await {
        tracing::warn!(
            target: "sealantern.application.resource",
            path = %sub_dir.display(),
            error = %error,
            "failed to clean downloaded temporary directory"
        );
    }
}

/// 回收 `market-tmp` 下的陈旧子目录（尽力而为，失败只记日志）。
///
/// 只清理**目录**且只清理超过 `STALE_MARKET_TMP_AGE` 的条目，避免误删正在
/// 进行的下载。目录名带随机后缀后，失败/取消留下的目录不会再被后续解析
/// 复用，若不回收就会持续累积。
async fn prune_stale_market_tmp() {
    let tmp_root = get_app_data_dir().join(MARKET_TMP_DIR);
    let Ok(mut entries) = tokio::fs::read_dir(&tmp_root).await else {
        // 目录尚未创建（还没有过市场下载），无需清理。
        return;
    };

    let now = SystemTime::now();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let Ok(metadata) = entry.metadata().await else {
            continue;
        };
        if !metadata.is_dir() {
            continue;
        }
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        let Ok(age) = now.duration_since(modified) else {
            continue;
        };
        if age < STALE_MARKET_TMP_AGE {
            continue;
        }
        if let Err(error) = tokio::fs::remove_dir_all(entry.path()).await {
            tracing::warn!(
                target: "sealantern.application.resource",
                path = %entry.path().display(),
                error = %error,
                "failed to prune stale market tmp directory"
            );
        }
    }
}

fn map_manager_error(error: ResourceManagerError) -> ResourceServiceError {
    match error {
        ResourceManagerError::InstanceDirNotFound(_) => ResourceServiceError::InstanceNotFound,
        ResourceManagerError::InvalidFileName(_)
        | ResourceManagerError::UnsupportedExtension(_)
        | ResourceManagerError::DisabledFileName(_)
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
    fn select_target_reports_missing_directory_for_kind() {
        // 实例有 plugins/、但没有 mods/：这是"该种类无目录"，不是"整个实例
        // 都没有资源目录"，两者错误要区分。
        let targets = vec![ResourceTarget::new(InstanceExtensionKind::Plugin, "plugins")];
        assert!(matches!(
            select_target(&targets, ResourceType::Mod),
            Err(ResourceServiceError::NoTargetForKind)
        ));
    }

    // ── 纯逻辑：extension_kind / pick_version_file ─────────

    #[test]
    fn extension_kind_supports_only_plugins_and_mods() {
        assert_eq!(extension_kind(ResourceType::Plugin).unwrap(), InstanceExtensionKind::Plugin);
        assert_eq!(extension_kind(ResourceType::Mod).unwrap(), InstanceExtensionKind::Mod);

        for kind in [
            ResourceType::Datapack,
            ResourceType::Shader,
            ResourceType::ResourcePack,
            ResourceType::Unknown,
        ] {
            assert!(
                matches!(extension_kind(kind), Err(ResourceServiceError::Unsupported)),
                "{kind:?} 不应被支持"
            );
        }
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
    fn unique_download_dir_separates_projects_and_calls() {
        // 不同项目不冲突。
        let first = unique_download_dir("proj-a", "v1");
        let second = unique_download_dir("proj-b", "v1");
        assert_ne!(first, second, "同一文件名来自不同项目时目录必须不同");

        // 同一项目与版本的两次解析也必须不同目录，否则并发下载会互相覆盖。
        let a = unique_download_dir("proj-a", "v1");
        let b = unique_download_dir("proj-a", "v1");
        assert_ne!(a, b, "同一项目与版本的两次解析必须得到不同目录");

        // 保留可读前缀，且分隔符已被替换为单一普通路径组件。
        assert!(a.starts_with("proj-a-v1-"));
        let sanitized = unique_download_dir("a/b", "v1");
        assert!(sanitized.starts_with("a_b-v1-"));
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
                ResourceManagerError::DisabledFileName("x".into()),
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

    /// 在真实的 `market-tmp/<唯一子目录>/` 下写一个下载文件，返回其路径。
    ///
    /// 子目录名带随机后缀，避免并发测试互相干扰。
    async fn write_tmp_download(sub_dir: &str, file_name: &str) -> PathBuf {
        let dir = get_app_data_dir().join(MARKET_TMP_DIR).join(sub_dir);
        tokio::fs::create_dir_all(&dir)
            .await
            .expect("tmp sub dir should be created");
        let path = dir.join(file_name);
        tokio::fs::write(&path, b"jar")
            .await
            .expect("fixture file should be written");
        path
    }

    #[tokio::test]
    async fn install_keeps_original_name_and_cleans_tmp_dir() {
        let sub_dir = format!("proj-v1-{}", uuid::Uuid::new_v4());
        let source = write_tmp_download(&sub_dir, "sodium.jar").await;

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

        // 唯一化只作用于临时目录：实例内保持市场原名。
        assert_eq!(managed.file_name, "sodium.jar");
        assert!(instance_dir.join("mods/sodium.jar").exists());
        // 整个临时子目录被清理。
        assert!(!source.exists(), "下载文件应被清理");
        let tmp_sub_dir = source.parent().expect("下载路径有父目录");
        assert!(!tmp_sub_dir.exists(), "临时子目录应被整体清理");

        std::fs::remove_dir_all(&root).ok();
    }

    #[tokio::test]
    async fn cleanup_ignores_paths_outside_market_tmp() {
        // `Path::starts_with` 是纯语法组件比较、不解析 `..`，因此
        // `market-tmp/../x` 能骗过前缀检查；canonicalize 后必须识别为越界。
        let tmp_root = get_app_data_dir().join(MARKET_TMP_DIR);
        tokio::fs::create_dir_all(&tmp_root)
            .await
            .expect("tmp root should exist");

        let victim_dir =
            get_app_data_dir().join(format!("sealantern-victim-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&victim_dir).await.unwrap();
        let victim = victim_dir.join("victim.jar");
        tokio::fs::write(&victim, b"keep me").await.unwrap();

        // 构造 `<market-tmp>/../<victim_dir>/victim.jar`。
        let traversal = tmp_root
            .join("..")
            .join(victim_dir.file_name().expect("victim dir has a name"))
            .join("victim.jar");
        cleanup_downloaded_tmp(traversal.to_str().expect("utf8 path")).await;

        assert!(victim.exists(), "market-tmp 之外的路径不得被清理");
        assert!(victim_dir.exists(), "越界目录不得被删除");

        std::fs::remove_dir_all(&victim_dir).ok();
    }
}
