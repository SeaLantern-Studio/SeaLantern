//! 实例信任层。
//!
//! 在发现结果之上叠加注册表名单，给出每个实例/问题的处置状态，并提供
//! 名单维护原语。信任按实例 id 记录（[`trusted_instances`]），忽略按目录
//! 记录（[`ignored_dirs`]）——两个键域刻意不同：被忽略的目录可能没有
//! 有效 `sl.json`、没有 id 可记；同一目录与同一 id 交叠时**忽略优先**
//! （用户明确说了"别再提示这个目录"）。
//!
//! 信任状态存于 `settings.json`（机器级），绝不写进 `sl.json`——否则目录
//! 被拷贝到其他机器会携带伪造的信任状态。

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use sealantern_contract::settings::InstanceRegistrySection;

use super::discovery::{
    DiscoveredInstance, DiscoveryProblem, DiscoveryReport, dir_key, normalize_dir,
};
use crate::config::settings::{SettingsError, SettingsManager};

/// 已发现实例的信任处置状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustState {
    /// 实例 id 已登记在 `trusted_instances`。
    Trusted,
    /// 既未登记也未被忽略——需要提示用户处理。
    Unmarked,
    /// 所在目录在 `ignored_dirs`——用户明确不再提示（忽略优先于信任）。
    Ignored,
}

/// 叠加信任状态后的实例。
#[derive(Debug, Clone)]
pub struct ClassifiedInstance {
    pub instance: DiscoveredInstance,
    pub state: TrustState,
}

/// 叠加忽略标记后的发现问题。
#[derive(Debug, Clone)]
pub struct ClassifiedProblem {
    pub problem: DiscoveryProblem,
    /// 该问题涉及的目录**全部**在忽略名单里（可默认折叠不提示）。
    pub ignored: bool,
}

/// 发现结果与名单合成后的完整报告。
#[derive(Debug, Default)]
pub struct TrustReport {
    pub instances: Vec<ClassifiedInstance>,
    pub problems: Vec<ClassifiedProblem>,
}

impl TrustReport {
    /// 需要提示用户处理的实例（未标记）。
    pub fn pending_instances(&self) -> impl Iterator<Item = &ClassifiedInstance> {
        self.instances
            .iter()
            .filter(|i| i.state == TrustState::Unmarked)
    }
}

/// 把发现报告与注册表名单合成分类报告。
///
/// 分类规则：
/// - 实例：目录被忽略 → `Ignored`；否则 id 已登记 → `Trusted`；否则 `Unmarked`
/// - 问题：其涉及的所有目录都在忽略名单里 → `ignored = true`
pub fn classify(report: DiscoveryReport, registry: &InstanceRegistrySection) -> TrustReport {
    let ignored: HashSet<String> = registry
        .ignored_dirs
        .iter()
        .map(|dir| dir_key(dir))
        .collect();
    let trusted: HashSet<&str> = registry
        .trusted_instances
        .iter()
        .map(String::as_str)
        .collect();

    let is_ignored = |dir: &Path| ignored.contains(&dir_key(dir));

    let instances = report
        .instances
        .into_iter()
        .map(|instance| {
            let state = if is_ignored(&instance.dir) {
                TrustState::Ignored
            } else if trusted.contains(instance.document.id.as_str()) {
                TrustState::Trusted
            } else {
                TrustState::Unmarked
            };
            ClassifiedInstance { instance, state }
        })
        .collect();

    let problems = report
        .problems
        .into_iter()
        .map(|problem| {
            let ignored = problem_dirs(&problem).all(&is_ignored);
            ClassifiedProblem { problem, ignored }
        })
        .collect();

    TrustReport { instances, problems }
}

/// 问题涉及的目录列表（`DuplicateId` 携带多个目录）。
fn problem_dirs(problem: &DiscoveryProblem) -> impl Iterator<Item = &Path> {
    let single = match problem {
        DiscoveryProblem::UnreadableDir { dir, .. }
        | DiscoveryProblem::MissingDocument { dir }
        | DiscoveryProblem::InvalidDocument { dir, .. }
        | DiscoveryProblem::UnsupportedSchemaVersion { dir, .. }
        | DiscoveryProblem::DuplicateDirAlias { dir, .. } => Some(dir.as_path()),
        DiscoveryProblem::DuplicateId { .. } => None,
    };
    single.into_iter().chain(match problem {
        DiscoveryProblem::DuplicateId { dirs, .. } => {
            dirs.iter().map(PathBuf::as_path).collect::<Vec<_>>()
        }
        _ => Vec::new(),
    })
}

/// 注册表里登记为受信任、但本次扫描中不存在对应实例的 id（孤儿记录）。
///
/// 产生场景：实例目录被手工删除 / 移除但名单未清理。发现层只报告；
/// 清理动作由调用方经 [`untrust_instance`] 决定，不在此处自动执行。
pub fn orphan_trusted_ids(
    report: &DiscoveryReport,
    registry: &InstanceRegistrySection,
) -> Vec<String> {
    let live: HashSet<&str> = report
        .instances
        .iter()
        .map(|i| i.document.id.as_str())
        .collect();
    registry
        .trusted_instances
        .iter()
        .filter(|id| !live.contains(id.as_str()))
        .cloned()
        .collect()
}

/// 把实例登记为受信任。幂等：已登记时不写回。
///
/// 由 `create_instance` / `import_modpack` 成功后调用（登记动作本身即导入），
/// 也供用户对未标记实例做显式信任。
pub async fn trust_instance(
    settings: &mut SettingsManager,
    id: &str,
) -> Result<InstanceRegistrySection, SettingsError> {
    settings
        .update_registry(|registry| {
            if registry
                .trusted_instances
                .iter()
                .any(|existing| existing == id)
            {
                return false;
            }
            registry.trusted_instances.push(id.to_string());
            true
        })
        .await
}

/// 撤销实例的信任登记。幂等。
///
/// 删除实例与「移除但保留目录」的调用方都应经此清理名单——名单中的孤儿
/// 条目会让后来拷贝来的同 id 目录静默继承信任（见决策文档 D15）。
pub async fn untrust_instance(
    settings: &mut SettingsManager,
    id: &str,
) -> Result<InstanceRegistrySection, SettingsError> {
    settings
        .update_registry(|registry| {
            let len = registry.trusted_instances.len();
            registry.trusted_instances.retain(|existing| existing != id);
            registry.trusted_instances.len() != len
        })
        .await
}

/// 把目录加入忽略名单（词法规范化后存储）。幂等。
pub async fn ignore_dir(
    settings: &mut SettingsManager,
    dir: &Path,
) -> Result<InstanceRegistrySection, SettingsError> {
    let normalized = normalize_dir(dir);
    settings
        .update_registry(|registry| {
            if registry
                .ignored_dirs
                .iter()
                .any(|existing| existing == &normalized)
            {
                return false;
            }
            registry.ignored_dirs.push(normalized.clone());
            true
        })
        .await
}

/// 把目录移出忽略名单（按规范化路径匹配）。幂等。
pub async fn unignore_dir(
    settings: &mut SettingsManager,
    dir: &Path,
) -> Result<InstanceRegistrySection, SettingsError> {
    let normalized = normalize_dir(dir);
    settings
        .update_registry(|registry| {
            let len = registry.ignored_dirs.len();
            registry
                .ignored_dirs
                .retain(|existing| existing != &normalized);
            registry.ignored_dirs.len() != len
        })
        .await
}

/// 把目录登记为附加服务器目录（词法规范化后存储）。幂等。
///
/// 当实例位于主资源目录 `instances/` 之外时，由 `create`/`import` 的
/// 调用方在信任登记前调用，使该目录在后续发现中被扫描。已在名单内
/// （含不同写法）时不写回；不做文件系统 IO，纯名单维护。
pub async fn register_extra_dir(
    settings: &mut SettingsManager,
    dir: &Path,
) -> Result<InstanceRegistrySection, SettingsError> {
    let normalized = normalize_dir(dir);
    let key = dir_key(&normalized);
    settings
        .update_registry(|registry| {
            if registry
                .extra_server_dirs
                .iter()
                .any(|existing| dir_key(existing) == key)
            {
                return false;
            }
            registry.extra_server_dirs.push(normalized.clone());
            true
        })
        .await
}

/// 把目录移出附加服务器名单（按规范化路径匹配）。幂等。
///
/// 供 `update_path` 在实例目录迁移后收回旧位置的登记；主资源目录
/// `instances/` 下的实例从不登记，调用方无须为它们调用本原语。
pub async fn unregister_extra_dir(
    settings: &mut SettingsManager,
    dir: &Path,
) -> Result<InstanceRegistrySection, SettingsError> {
    let normalized = normalize_dir(dir);
    let key = dir_key(&normalized);
    settings
        .update_registry(|registry| {
            let len = registry.extra_server_dirs.len();
            registry
                .extra_server_dirs
                .retain(|existing| dir_key(existing) != key);
            registry.extra_server_dirs.len() != len
        })
        .await
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use sealantern_core::instance::{InstanceId, StartupMode};

    use super::*;
    use crate::config::instance::{
        CURRENT_SCHEMA_VERSION, CoreSpec, DiscoveryOrigin, InstanceDocument, StartupSpec,
    };

    fn instance(id: &str, dir: &Path, origin: DiscoveryOrigin) -> DiscoveredInstance {
        DiscoveredInstance {
            dir: dir.to_path_buf(),
            origin,
            document: InstanceDocument {
                schema_version: CURRENT_SCHEMA_VERSION,
                id: InstanceId::new(id).expect("instance ID should be valid"),
                name: id.to_string(),
                aliases: Vec::new(),
                core: CoreSpec {
                    core_type: "paper".to_string(),
                    version: "1".to_string(),
                },
                mc_version: None,
                required_java: None,
                startup: StartupSpec {
                    mode: StartupMode::Jar,
                    target: Some(PathBuf::from("server.jar")),
                    port: 25565,
                    ..StartupSpec::default()
                },
                cron: Vec::new(),
                server_metadata: None,
                created_at_unix_secs: None,
                last_started_at_unix_secs: None,
            },
        }
    }

    fn registry(trusted: &[&str], ignored: &[&str]) -> InstanceRegistrySection {
        InstanceRegistrySection {
            main_resource_dir: None,
            extra_server_dirs: Vec::new(),
            trusted_instances: trusted.iter().map(|s| s.to_string()).collect(),
            ignored_dirs: ignored.iter().map(PathBuf::from).collect(),
        }
    }

    #[test]
    fn classification_uses_id_for_trust_and_dir_for_ignore() {
        let report = DiscoveryReport {
            instances: vec![
                instance("a", Path::new("srv/a"), DiscoveryOrigin::Managed),
                instance("b", Path::new("srv/b"), DiscoveryOrigin::Managed),
                instance("c", Path::new("srv/c"), DiscoveryOrigin::Managed),
            ],
            problems: Vec::new(),
        };
        let registry = registry(&["a"], &["srv/c/"]);

        let report = classify(report, &registry);

        assert_eq!(report.instances[0].state, TrustState::Trusted);
        assert_eq!(report.instances[1].state, TrustState::Unmarked);
        assert_eq!(report.instances[2].state, TrustState::Ignored);
    }

    #[test]
    fn ignored_dir_wins_over_trusted_id() {
        let report = DiscoveryReport {
            instances: vec![instance("a", Path::new("srv/a"), DiscoveryOrigin::Managed)],
            problems: Vec::new(),
        };
        let registry = registry(&["a"], &["srv/a"]);

        let report = classify(report, &registry);

        assert_eq!(report.instances[0].state, TrustState::Ignored);
    }

    #[test]
    fn problems_carry_an_ignored_flag() {
        let report = DiscoveryReport {
            instances: Vec::new(),
            problems: vec![
                DiscoveryProblem::MissingDocument { dir: PathBuf::from("srv/skip") },
                DiscoveryProblem::MissingDocument { dir: PathBuf::from("srv/show") },
            ],
        };
        let registry = registry(&[], &["srv/skip/../skip"]);

        let report = classify(report, &registry);

        assert!(report.problems[0].ignored);
        assert!(!report.problems[1].ignored);
    }

    #[test]
    fn orphan_ids_are_reported_for_live_mismatch() {
        let report = DiscoveryReport {
            instances: vec![instance("alive", Path::new("srv/alive"), DiscoveryOrigin::Managed)],
            problems: Vec::new(),
        };
        let registry = registry(&["alive", "gone"], &[]);

        assert_eq!(orphan_trusted_ids(&report, &registry), vec!["gone".to_string()]);
    }

    #[tokio::test]
    async fn trust_and_untrust_are_idempotent() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let mut settings = SettingsManager::load(root.path().join("settings.json"))
            .await
            .expect("settings should load");

        trust_instance(&mut settings, "a")
            .await
            .expect("trust should succeed");
        trust_instance(&mut settings, "a")
            .await
            .expect("trust should be idempotent");
        assert_eq!(settings.get().registry.trusted_instances, vec!["a".to_string()]);

        untrust_instance(&mut settings, "a")
            .await
            .expect("untrust should succeed");
        untrust_instance(&mut settings, "a")
            .await
            .expect("untrust should be idempotent");
        assert!(settings.get().registry.trusted_instances.is_empty());
    }

    #[tokio::test]
    async fn ignore_dir_stores_the_normalized_path() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let mut settings = SettingsManager::load(root.path().join("settings.json"))
            .await
            .expect("settings should load");

        ignore_dir(&mut settings, Path::new("srv/a/nested/.."))
            .await
            .expect("ignore should succeed");
        ignore_dir(&mut settings, Path::new("srv/a"))
            .await
            .expect("ignore should be idempotent");

        assert_eq!(settings.get().registry.ignored_dirs, vec![PathBuf::from("srv/a")]);

        unignore_dir(&mut settings, Path::new("srv/a"))
            .await
            .expect("unignore should succeed");
        assert!(settings.get().registry.ignored_dirs.is_empty());
    }
}
