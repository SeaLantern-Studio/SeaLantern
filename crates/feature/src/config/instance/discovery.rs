//! 实例发现层（只读）。
//!
//! 扫描主资源目录的 `instances/` 容器与注册表里的附加服务器目录，产出
//! 「身份有效」的实例列表与问题清单。本层绝不写盘、不修复、不自动信任：
//! - `sl.json` 缺失 → [`DiscoveryProblem::MissingDocument`]
//! - `sl.json` 不可解析 / 缺 `schema_version` / 结构校验失败 →
//!   [`DiscoveryProblem::InvalidDocument`]
//! - `schema_version` 超前 → [`DiscoveryProblem::UnsupportedSchemaVersion`]
//! - 同一 id 出现在多个目录 → [`DiscoveryProblem::DuplicateId`]（整组移出
//!   实例列表，交上层决策）
//! - 同一目录经不同写法出现多次 → [`DiscoveryProblem::DuplicateDirAlias`]
//!
//! 目录去重与名单匹配使用纯词法规范化（[`normalize_dir`]/[`dir_key`]），
//! 不做 `canonicalize`：符号链接别名会以「重复 id」的形式如实浮现，
//! 而不是被静默合并；忽略名单对已删除目录的比较也保持字符串语义。

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

use sealantern_infra::platform::AppLayout;
use serde_json::Value;

use crate::models::InstanceRegistrySection;

use super::document::read_document_bytes;
use super::{CURRENT_SCHEMA_VERSION, DocumentStore, InstanceDocument};

/// 实例来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryOrigin {
    /// 主资源目录 `instances/` 容器下的目录。
    Managed,
    /// 注册表里登记的附加服务器目录。
    Extra,
}

/// 一个通过身份检查的实例：目录 + 已校验的 `sl.json` 内容。
#[derive(Debug, Clone)]
pub struct DiscoveredInstance {
    /// 词法规范化后的目录路径（保留原大小写）。
    pub dir: PathBuf,
    /// 发现来源。
    pub origin: DiscoveryOrigin,
    /// 解析并校验后的实例文档。
    pub document: InstanceDocument,
}

/// 发现过程中记录的问题；每条都携带目录，供上层逐条展示或决策。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryProblem {
    /// 候选目录或其 `sl.json` 无法读取（权限、损坏的文件系统等）。
    UnreadableDir { dir: PathBuf, message: String },
    /// 目录里没有 `sl.json`（可能是半成品、外来目录或手工放置的文件）。
    MissingDocument { dir: PathBuf },
    /// `sl.json` 存在但不可用：读取失败、JSON 损坏、缺 `schema_version`
    /// 或未过结构校验——统称为「无有效配置」。
    InvalidDocument { dir: PathBuf, message: String },
    /// `sl.json` 由更新版本的程序写入。
    UnsupportedSchemaVersion { dir: PathBuf, found: u32 },
    /// 同一实例 id 出现在多个目录；`dirs` 按规范化路径排序。
    DuplicateId { id: String, dirs: Vec<PathBuf> },
    /// 同一目录经不同写法被扫描了多次；仅保留首次出现。
    DuplicateDirAlias { dir: PathBuf, alias_of: PathBuf },
}

/// 一次发现的结果。
#[derive(Debug, Default)]
pub struct DiscoveryReport {
    /// 身份检查通过的实例（目录唯一、id 本机唯一），按目录排序。
    pub instances: Vec<DiscoveredInstance>,
    /// 未通过身份检查或需要提示的条目。
    pub problems: Vec<DiscoveryProblem>,
}

impl DiscoveryReport {
    /// 是否存在需要人工介入的问题。
    pub fn has_problems(&self) -> bool {
        !self.problems.is_empty()
    }
}

/// 词法规范化目录路径：消去 `.` 与可消解的 `..`、合并分隔符、去尾部
/// 分隔符。不做任何 IO，不解析符号链接；越过根目录的 `..` 被丢弃，
/// 相对路径开头的 `..` 保留以维持指向。
pub(crate) fn normalize_dir(path: &Path) -> PathBuf {
    let mut stack: Vec<Component<'_>> = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => match stack.last() {
                Some(Component::Normal(_)) => {
                    stack.pop();
                }
                // 根目录/前缀之上没有可退的层级
                Some(Component::Prefix(_)) | Some(Component::RootDir) => {}
                // 相对路径连续的 `..` 保留
                Some(Component::ParentDir) | Some(Component::CurDir) | None => {
                    stack.push(component);
                }
            },
            other => stack.push(other),
        }
    }
    stack.iter().collect()
}

/// 目录比较键：词法规范化 + 统一分隔符，Windows 下再折叠大小写。
/// 用于目录去重与 `ignored_dirs`/`extra_server_dirs` 匹配——比较必须在
/// 两侧使用同一函数，否则同一目录的不同写法会被当成不同条目。
pub(crate) fn dir_key(path: &Path) -> String {
    let normalized = normalize_dir(path).to_string_lossy().replace('\\', "/");
    if cfg!(windows) {
        normalized.to_lowercase()
    } else {
        normalized
    }
}

/// 扫描 `layout.instances_dir()` 的子目录与 `registry.extra_server_dirs`，
/// 产出实例列表与问题清单。永远不返回错误：单个候选的失败只会变成一条
/// [`DiscoveryProblem`]。
pub async fn discover(layout: &AppLayout, registry: &InstanceRegistrySection) -> DiscoveryReport {
    let mut report = DiscoveryReport::default();
    // dir_key → 首次出现的目录；后续同键条目转为 DuplicateDirAlias
    let mut seen_dirs: HashMap<String, PathBuf> = HashMap::new();

    // instances/ 容器下的子目录（只认目录；desktop.ini / *.lock / *.bak-* 等
    // 文件天然被 file_type 过滤）。
    let instances_dir = layout.instances_dir();
    match tokio::fs::read_dir(&instances_dir).await {
        Ok(mut entries) => loop {
            match entries.next_entry().await {
                Ok(Some(entry)) => {
                    let dir = normalize_dir(&entry.path());
                    match entry.file_type().await {
                        Ok(file_type) if file_type.is_dir() => {
                            collect_candidate(
                                dir,
                                DiscoveryOrigin::Managed,
                                &mut seen_dirs,
                                &mut report,
                            )
                            .await;
                        }
                        Ok(_) => {}
                        Err(error) => report.problems.push(DiscoveryProblem::UnreadableDir {
                            dir,
                            message: error.to_string(),
                        }),
                    }
                }
                Ok(None) => break,
                Err(error) => {
                    report.problems.push(DiscoveryProblem::UnreadableDir {
                        dir: instances_dir.clone(),
                        message: error.to_string(),
                    });
                    break;
                }
            }
        },
        // 容器不存在 = 还没有实例，属于正常情况
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => report.problems.push(DiscoveryProblem::UnreadableDir {
            dir: instances_dir,
            message: error.to_string(),
        }),
    }

    // 附加服务器目录：每个目录本身就是一个服务器
    for extra in &registry.extra_server_dirs {
        let dir = normalize_dir(extra);
        collect_candidate(dir, DiscoveryOrigin::Extra, &mut seen_dirs, &mut report).await;
    }

    // 重复 id：同一 id 出现在多个目录时整组移出实例列表
    let mut by_id: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, instance) in report.instances.iter().enumerate() {
        by_id
            .entry(instance.document.id.as_str().to_string())
            .or_default()
            .push(index);
    }
    let mut duplicate_indices: HashSet<usize> = HashSet::new();
    let mut duplicate_groups: Vec<(String, Vec<PathBuf>)> = Vec::new();
    for (id, indices) in by_id {
        if indices.len() > 1 {
            let mut dirs: Vec<PathBuf> = indices
                .iter()
                .map(|&i| report.instances[i].dir.clone())
                .collect();
            dirs.sort();
            duplicate_groups.push((id, dirs));
            duplicate_indices.extend(indices);
        }
    }
    if !duplicate_groups.is_empty() {
        report.instances = report
            .instances
            .into_iter()
            .enumerate()
            .filter(|(index, _)| !duplicate_indices.contains(index))
            .map(|(_, instance)| instance)
            .collect();
        duplicate_groups.sort_by(|a, b| a.0.cmp(&b.0));
        for (id, dirs) in duplicate_groups {
            report
                .problems
                .push(DiscoveryProblem::DuplicateId { id, dirs });
        }
    }

    report.instances.sort_by(|a, b| a.dir.cmp(&b.dir));
    report
}

/// 处理单个候选目录：去重 → 探查 → 归入实例列表或问题清单。
async fn collect_candidate(
    dir: PathBuf,
    origin: DiscoveryOrigin,
    seen_dirs: &mut HashMap<String, PathBuf>,
    report: &mut DiscoveryReport,
) {
    let key = dir_key(&dir);
    if let Some(first) = seen_dirs.get(&key) {
        report
            .problems
            .push(DiscoveryProblem::DuplicateDirAlias { dir, alias_of: first.clone() });
        return;
    }
    seen_dirs.insert(key, dir.clone());

    match probe_dir(dir, origin).await {
        Ok(instance) => report.instances.push(instance),
        Err(problem) => report.problems.push(problem),
    }
}

/// 探查单个目录是否为身份有效的实例。
///
/// 与发现层共用同一条管线：读 `sl.json` → 版本门禁 → 反序列化 → 结构校验。
/// 不做目录去重、不看名单——调用方（`import_existing_server`、发现层）自行
/// 决定 origin 与去重语义。每一步失败映射为对应的 [`DiscoveryProblem`]。
pub async fn probe_dir(
    dir: PathBuf,
    origin: DiscoveryOrigin,
) -> Result<DiscoveredInstance, DiscoveryProblem> {
    let dir = normalize_dir(&dir);
    let document_path = DocumentStore::path_for(&dir);
    let bytes = read_document_bytes(&document_path).await.map_err(|error| {
        if is_not_found(&error) {
            DiscoveryProblem::MissingDocument { dir: dir.clone() }
        } else {
            DiscoveryProblem::InvalidDocument {
                dir: dir.clone(),
                message: error.to_string(),
            }
        }
    })?;

    // 先读 schema_version：超前版本即使字段兼容也一律拒绝（D3）。
    let raw: Value =
        serde_json::from_slice(&bytes).map_err(|error| DiscoveryProblem::InvalidDocument {
            dir: dir.clone(),
            message: error.to_string(),
        })?;
    match raw.get("schema_version").and_then(Value::as_u64) {
        None => {
            return Err(DiscoveryProblem::InvalidDocument {
                dir,
                message: "missing or invalid schema_version".to_string(),
            });
        }
        Some(found) if found > u64::from(CURRENT_SCHEMA_VERSION) => {
            return Err(DiscoveryProblem::UnsupportedSchemaVersion {
                dir,
                found: found.min(u64::from(u32::MAX)) as u32,
            });
        }
        Some(_) => {}
    }

    let document: InstanceDocument =
        serde_json::from_slice(&bytes).map_err(|error| DiscoveryProblem::InvalidDocument {
            dir: dir.clone(),
            message: error.to_string(),
        })?;
    document
        .validate()
        .map_err(|error| DiscoveryProblem::InvalidDocument {
            dir: dir.clone(),
            message: error.to_string(),
        })?;

    Ok(DiscoveredInstance { dir, origin, document })
}

/// 判断底层错误是否为「文件不存在」。
fn is_not_found(error: &sealantern_infra::fs::FsError) -> bool {
    matches!(
        error,
        sealantern_infra::fs::FsError::Io { source, .. }
            if source.kind() == std::io::ErrorKind::NotFound
    )
}

#[cfg(test)]
mod tests {
    use sealantern_contract::cron::CronTaskAction;
    use sealantern_core::instance::StartupMode;

    use super::*;
    use crate::config::instance::{CoreSpec, InstanceCronEntry, MemorySpec, StartupSpec};
    use sealantern_core::instance::InstanceId;

    fn fixture_document(id: &str) -> InstanceDocument {
        InstanceDocument {
            schema_version: CURRENT_SCHEMA_VERSION,
            id: InstanceId::new(id).expect("instance ID should be valid"),
            name: format!("Server {id}"),
            aliases: Vec::new(),
            core: CoreSpec {
                core_type: "paper".to_string(),
                version: "1.21.1".to_string(),
            },
            mc_version: Some("1.21.1".to_string()),
            required_java: None,
            startup: StartupSpec {
                mode: StartupMode::Jar,
                target: Some(PathBuf::from("server.jar")),
                port: 25565,
                ..StartupSpec::default()
            },
            cron: Vec::new(),
            server_metadata: None,
            created_at_unix_secs: Some(1_700_000_000),
            last_started_at_unix_secs: None,
        }
    }

    async fn write_document(dir: &Path, document: &InstanceDocument) {
        tokio::fs::create_dir_all(dir)
            .await
            .expect("instance dir should be created");
        let json = serde_json::to_string_pretty(document).expect("document should serialize");
        tokio::fs::write(DocumentStore::path_for(dir), json)
            .await
            .expect("document fixture should be written");
    }

    async fn write_raw(dir: &Path, content: &str) {
        tokio::fs::create_dir_all(dir)
            .await
            .expect("instance dir should be created");
        tokio::fs::write(DocumentStore::path_for(dir), content)
            .await
            .expect("document fixture should be written");
    }

    fn layout_with(root: &Path) -> AppLayout {
        AppLayout::new(root.join("config"), None)
    }

    #[tokio::test]
    async fn empty_roots_produce_an_empty_report() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let report = discover(&layout_with(root.path()), &InstanceRegistrySection::default()).await;

        assert!(report.instances.is_empty());
        assert!(report.problems.is_empty());
        assert!(!report.has_problems());
    }

    #[tokio::test]
    async fn managed_instance_is_discovered() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let dir = layout_with(root.path()).instances_dir().join("alpha");
        write_document(&dir, &fixture_document("alpha")).await;

        let report = discover(&layout_with(root.path()), &InstanceRegistrySection::default()).await;

        assert_eq!(report.instances.len(), 1);
        assert_eq!(report.instances[0].origin, DiscoveryOrigin::Managed);
        assert_eq!(report.instances[0].document.id.as_str(), "alpha");
        assert!(report.problems.is_empty());
    }

    #[tokio::test]
    async fn extra_server_dirs_are_discovered_with_their_origin() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let extra = root.path().join("elsewhere").join("beta");
        write_document(&extra, &fixture_document("beta")).await;
        let registry = InstanceRegistrySection {
            extra_server_dirs: vec![extra.clone()],
            ..InstanceRegistrySection::default()
        };

        let report = discover(&layout_with(root.path()), &registry).await;

        assert_eq!(report.instances.len(), 1);
        assert_eq!(report.instances[0].origin, DiscoveryOrigin::Extra);
        assert_eq!(report.instances[0].dir, extra);
    }

    #[tokio::test]
    async fn directory_without_a_document_is_reported_as_missing() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let dir = layout_with(root.path()).instances_dir().join("unfinished");
        tokio::fs::create_dir_all(&dir)
            .await
            .expect("instance dir should be created");

        let report = discover(&layout_with(root.path()), &InstanceRegistrySection::default()).await;

        assert!(report.instances.is_empty());
        assert_eq!(report.problems, vec![DiscoveryProblem::MissingDocument { dir }]);
    }

    #[tokio::test]
    async fn unparseable_document_is_reported_as_invalid() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let dir = layout_with(root.path()).instances_dir().join("broken");
        write_raw(&dir, "not-json").await;

        let report = discover(&layout_with(root.path()), &InstanceRegistrySection::default()).await;

        assert!(report.instances.is_empty());
        assert!(matches!(
            report.problems.as_slice(),
            [DiscoveryProblem::InvalidDocument { dir: d, .. }] if *d == dir
        ));
    }

    #[tokio::test]
    async fn document_without_schema_version_is_reported_as_invalid() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let dir = layout_with(root.path()).instances_dir().join("legacy");
        write_raw(&dir, r#"{"id":"x","name":"x","core":{"type":"paper","version":"1"}}"#).await;

        let report = discover(&layout_with(root.path()), &InstanceRegistrySection::default()).await;

        assert!(matches!(
            report.problems.as_slice(),
            [DiscoveryProblem::InvalidDocument { dir: d, message }]
                if *d == dir && message.contains("schema_version")
        ));
    }

    #[tokio::test]
    async fn future_schema_version_is_reported_without_parsing() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let dir = layout_with(root.path()).instances_dir().join("future");
        let mut doc = serde_json::to_value(fixture_document("future")).expect("fixture serializes");
        doc["schema_version"] = (CURRENT_SCHEMA_VERSION + 7).into();
        write_raw(&dir, &serde_json::to_string(&doc).expect("fixture encodes")).await;

        let report = discover(&layout_with(root.path()), &InstanceRegistrySection::default()).await;

        assert_eq!(
            report.problems,
            vec![DiscoveryProblem::UnsupportedSchemaVersion {
                dir,
                found: CURRENT_SCHEMA_VERSION + 7,
            }]
        );
    }

    #[tokio::test]
    async fn duplicate_ids_are_grouped_and_removed_from_instances() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let dir_a = layout_with(root.path()).instances_dir().join("a");
        let dir_b = layout_with(root.path()).instances_dir().join("b");
        let dir_c = layout_with(root.path()).instances_dir().join("c");
        write_document(&dir_a, &fixture_document("same-id")).await;
        write_document(&dir_b, &fixture_document("same-id")).await;
        write_document(&dir_c, &fixture_document("unique")).await;

        let report = discover(&layout_with(root.path()), &InstanceRegistrySection::default()).await;

        assert_eq!(report.instances.len(), 1);
        assert_eq!(report.instances[0].document.id.as_str(), "unique");
        assert_eq!(
            report.problems,
            vec![DiscoveryProblem::DuplicateId {
                id: "same-id".to_string(),
                dirs: vec![dir_a, dir_b],
            }]
        );
    }

    #[tokio::test]
    async fn the_same_directory_via_different_spellings_is_an_alias() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let extra = root.path().join("shared");
        write_document(&extra, &fixture_document("shared")).await;
        let registry = InstanceRegistrySection {
            extra_server_dirs: vec![
                extra.clone(),
                // 同一目录的另一种写法：多一层 `..` 且带尾部分隔符语义
                extra.join("nested").join(".."),
            ],
            ..InstanceRegistrySection::default()
        };

        let report = discover(&layout_with(root.path()), &registry).await;

        assert_eq!(report.instances.len(), 1);
        assert!(matches!(
            report.problems.as_slice(),
            [DiscoveryProblem::DuplicateDirAlias { alias_of, .. }] if *alias_of == extra
        ));
    }

    #[tokio::test]
    async fn stray_files_in_the_container_are_skipped() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let container = layout_with(root.path()).instances_dir();
        tokio::fs::create_dir_all(&container)
            .await
            .expect("container should be created");
        tokio::fs::write(container.join("sl.json.lock"), "lock")
            .await
            .expect("stray file should be written");
        tokio::fs::write(container.join("sl.json.bak-1-uuid"), "bak")
            .await
            .expect("stray file should be written");
        write_document(&container.join("alpha"), &fixture_document("alpha")).await;

        let report = discover(&layout_with(root.path()), &InstanceRegistrySection::default()).await;

        assert_eq!(report.instances.len(), 1);
        assert!(report.problems.is_empty());
    }

    #[tokio::test]
    async fn cron_entries_inside_a_document_round_trip() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let dir = layout_with(root.path()).instances_dir().join("crony");
        let mut document = fixture_document("crony");
        document.cron.push(InstanceCronEntry {
            id: "t1".to_string(),
            name: "Restart".to_string(),
            cron_expression: "0 0 4 * * *".to_string(),
            action: CronTaskAction::Restart,
            enabled: true,
            last_run_at: None,
            next_run_at: None,
            last_error: None,
        });
        write_document(&dir, &document).await;

        let report = discover(&layout_with(root.path()), &InstanceRegistrySection::default()).await;

        assert_eq!(report.instances.len(), 1);
        assert_eq!(report.instances[0].document.cron.len(), 1);
        assert_eq!(report.instances[0].document.cron[0].id, "t1");
    }

    #[test]
    fn normalize_dir_collapses_redundant_segments() {
        // `/` 在两个平台上都是合法分隔符，作为跨平台断言基准。
        assert_eq!(normalize_dir(Path::new("srv/./a/nested/../")), PathBuf::from("srv/a"));
        // 越过根目录的 .. 被丢弃
        assert_eq!(normalize_dir(Path::new("/../a")), PathBuf::from("/a"));
        // 相对路径开头的 .. 保留
        assert_eq!(normalize_dir(Path::new("../a")), PathBuf::from("../a"));

        // Windows 驱动器前缀 + 反斜杠写法的等价断言。
        #[cfg(windows)]
        {
            assert_eq!(
                normalize_dir(Path::new("D:\\servers\\.\\a\\nested\\..\\")),
                PathBuf::from("D:\\servers\\a")
            );
            assert_eq!(normalize_dir(Path::new("D:\\..\\a")), PathBuf::from("D:\\a"));
            assert_eq!(normalize_dir(Path::new("..\\a")), PathBuf::from("..\\a"));
        }
    }

    #[test]
    fn dir_key_matches_different_spellings_of_the_same_dir() {
        // `/` 写法跨平台等价：分隔符统一 + `..` 消解 + 尾部分隔符去除。
        assert_eq!(dir_key(Path::new("srv/a")), dir_key(Path::new("srv//a/")));
        assert_eq!(dir_key(Path::new("srv/x/../a")), dir_key(Path::new("srv/a")));

        // 盘符前缀与大小写折叠是 Windows 专属语义。
        #[cfg(windows)]
        {
            assert_eq!(dir_key(Path::new("D:\\srv\\a")), dir_key(Path::new("D:/srv/a/")));
            assert_eq!(dir_key(Path::new("D:\\Srv\\A")), dir_key(Path::new("d:\\srv\\a")));
        }

        // 非 Windows 文件系统大小写敏感：不同大小写必须得到不同键。
        #[cfg(not(windows))]
        assert_ne!(dir_key(Path::new("srv/A")), dir_key(Path::new("srv/a")));
    }

    #[test]
    fn memory_spec_defaults_are_zero() {
        assert_eq!(MemorySpec::default(), MemorySpec { min: 0, max: 0 });
    }
}
