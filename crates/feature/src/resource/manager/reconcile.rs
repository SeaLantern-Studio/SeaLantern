//! 扫描事实与资源清单的对账（纯逻辑，零 IO）。
//!
//! 扫描是"事实来源"：文件系统中存在而清单缺失的条目视为未知来源，
//! 清单存在而文件已消失的条目视为缺失。
//!
//! # 账目键
//!
//! 对账以 **`(资源种类, 文件名)`** 为复合键。`mods/` 与 `plugins/` 下允许
//! 出现同名文件（同名 jar 分别作为模组与插件），仅以文件名对账会让二者互相顶替。

use std::collections::HashMap;

use crate::observability;

use super::models::{
    InstanceExtension, InstanceExtensionKind, ManagedResource, ReconcileReport, ReconciledResource,
    ResourceManifest, ResourceState,
};

/// 对账用的复合账目键：资源种类 + 文件名。
pub(crate) type LedgerKey<'a> = (InstanceExtensionKind, &'a str);

/// 构造账目键（借用文件名字符串）。
pub(crate) fn ledger_key<'a>(kind: InstanceExtensionKind, file_name: &'a str) -> LedgerKey<'a> {
    (kind, file_name)
}

/// 对账扫描事实与持久化清单。
///
/// 返回的报告按资源种类、再按文件名排序，保证输出稳定可测。
///
/// 清单中若存在重复的 `(种类, 文件名)` 键，只保留最后一条账目，并把重复项
/// 记入 [`ReconcileReport::duplicate_entries`] 并发出告警（不静默丢弃）。
pub fn reconcile(scanned: &[InstanceExtension], manifest: &ResourceManifest) -> ReconcileReport {
    let mut ledger: HashMap<LedgerKey<'_>, &ManagedResource> = HashMap::new();
    let mut duplicate_entries = Vec::new();

    for resource in &manifest.resources {
        let key = ledger_key(resource.kind, resource.file_name.as_str());
        if ledger.insert(key, resource).is_some() {
            duplicate_entries.push(resource.file_name.clone());
        }
    }

    if !duplicate_entries.is_empty() {
        observability::resource_manifest_duplicates(&duplicate_entries);
    }

    let mut items = Vec::with_capacity(scanned.len().max(manifest.resources.len()));

    for extension in scanned {
        let managed = ledger.remove(&ledger_key(extension.kind, extension.file_name.as_str()));
        let state = if managed.is_some() {
            ResourceState::Normal
        } else {
            ResourceState::UnknownSource
        };
        items.push(ReconciledResource {
            kind: extension.kind,
            file_name: extension.file_name.clone(),
            extension: Some(extension.clone()),
            managed: managed.cloned(),
            state,
        });
    }

    for resource in ledger.into_values() {
        items.push(ReconciledResource {
            kind: resource.kind,
            file_name: resource.file_name.clone(),
            extension: None,
            managed: Some(resource.clone()),
            state: ResourceState::Missing,
        });
    }

    items.sort_by(|left, right| {
        left.kind
            .as_str()
            .cmp(right.kind.as_str())
            .then_with(|| left.file_name.cmp(&right.file_name))
    });

    ReconcileReport { items, duplicate_entries }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::resource::manager::models::{
        InstanceExtensionKind, ResourceProvenance, ResourceSource,
    };

    fn extension(file_name: &str, enabled: bool) -> InstanceExtension {
        extension_of(InstanceExtensionKind::Mod, file_name, enabled)
    }

    fn extension_of(
        kind: InstanceExtensionKind,
        file_name: &str,
        enabled: bool,
    ) -> InstanceExtension {
        InstanceExtension::new(
            kind,
            file_name,
            PathBuf::from(kind.as_str()).join(file_name),
            enabled,
            1024,
        )
        .expect("fixture extension should be valid")
    }

    fn managed(file_name: &str) -> ManagedResource {
        managed_of(InstanceExtensionKind::Mod, file_name)
    }

    fn managed_of(kind: InstanceExtensionKind, file_name: &str) -> ManagedResource {
        ManagedResource {
            file_name: file_name.to_string(),
            kind,
            enabled: true,
            hash: None,
            size_bytes: Some(1024),
            provenance: Some(ResourceProvenance {
                source: ResourceSource::Modrinth,
                project_id: Some("sodium".into()),
                version_id: None,
                version_number: Some("0.5.8".into()),
                installed_at_unix_secs: 1,
            }),
        }
    }

    #[test]
    fn matched_file_and_ledger_is_normal() {
        let report = reconcile(
            &[extension("sodium.jar", true)],
            &ResourceManifest {
                schema_version: 1,
                resources: vec![managed("sodium.jar")],
            },
        );
        assert_eq!(report.total(), 1);
        assert_eq!(report.normal_count(), 1);
        assert_eq!(report.items[0].state, ResourceState::Normal);
        assert!(report.duplicate_entries.is_empty());
    }

    #[test]
    fn file_without_ledger_is_unknown_source() {
        let report = reconcile(&[extension("mystery.jar", true)], &ResourceManifest::default());
        assert_eq!(report.unknown_count(), 1);
        assert!(report.items[0].managed.is_none());
        assert!(report.items[0].extension.is_some());
    }

    #[test]
    fn ledger_without_file_is_missing() {
        let report = reconcile(
            &[],
            &ResourceManifest {
                schema_version: 1,
                resources: vec![managed("removed.jar")],
            },
        );
        assert_eq!(report.missing_count(), 1);
        assert!(report.items[0].extension.is_none());
        assert!(report.items[0].managed.is_some());
    }

    #[test]
    fn duplicate_ledger_entries_are_reported_instead_of_dropped() {
        let report = reconcile(
            &[extension("sodium.jar", true)],
            &ResourceManifest {
                schema_version: 1,
                resources: vec![managed("sodium.jar"), managed("sodium.jar")],
            },
        );

        assert_eq!(report.duplicate_entries, vec!["sodium.jar".to_string()]);
        assert_eq!(report.normal_count(), 1);
    }

    #[test]
    fn report_is_sorted_by_name_within_kind() {
        let report = reconcile(
            &[extension("b.jar", true), extension("a.jar", true)],
            &ResourceManifest::default(),
        );
        let names: Vec<&str> = report
            .items
            .iter()
            .map(|item| item.file_name.as_str())
            .collect();
        assert_eq!(names, vec!["a.jar", "b.jar"]);
    }

    #[test]
    fn same_name_across_kinds_are_distinct_entries() {
        // 同名文件分别作为模组与插件：两条独立账目，均对账正常。
        let scanned = vec![
            extension_of(InstanceExtensionKind::Mod, "shared.jar", true),
            extension_of(InstanceExtensionKind::Plugin, "shared.jar", true),
        ];
        let manifest = ResourceManifest {
            schema_version: 1,
            resources: vec![
                managed_of(InstanceExtensionKind::Mod, "shared.jar"),
                managed_of(InstanceExtensionKind::Plugin, "shared.jar"),
            ],
        };

        let report = reconcile(&scanned, &manifest);

        assert_eq!(report.total(), 2);
        assert_eq!(report.normal_count(), 2);
        assert!(report.duplicate_entries.is_empty(), "跨种类的同名文件不是重复账目");
    }

    #[test]
    fn kind_mismatch_is_reported_as_unknown_source() {
        // 文件是插件、账目却是模组：不能按文件名误匹配，应判未知来源。
        let scanned = vec![extension_of(InstanceExtensionKind::Plugin, "shared.jar", true)];
        let manifest = ResourceManifest {
            schema_version: 1,
            resources: vec![managed_of(InstanceExtensionKind::Mod, "shared.jar")],
        };

        let report = reconcile(&scanned, &manifest);

        assert_eq!(report.unknown_count(), 1);
        assert_eq!(report.missing_count(), 1);
    }
}
