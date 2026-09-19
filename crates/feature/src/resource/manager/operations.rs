//! 实例资源的增删与启用禁用（IO）。
//!
//! 所有写操作遵循同一模式：定位目标文件 → 变更文件系统 → 更新清单。
//!
//! - 文件与目录操作统一走 `tokio::fs`，避免大文件复制阻塞异步运行时；
//! - 清单的"读 → 改 → 写"通过 [`manifest::update`] 在跨进程锁内完成，
//!   避免并发写互相覆盖；
//! - 外部传入的文件名一律经 [`validate_file_name`] 校验，拒绝路径穿越。

use std::path::{Path, PathBuf};

use sealantern_infra::fs::sha256_file;

use super::ResourceManagerError;
use super::layout::ResourceTarget;
use super::models::{
    InstanceExtension, InstanceExtensionKind, MANIFEST_SCHEMA_VERSION, ManagedResource,
    ReconcileReport, ResourceProvenance, ResourceState,
};
use super::{manifest, naming, path, reconcile, scan};

/// 列出实例资源：扫描目录并与清单对账（只读，不写盘）。
pub async fn list(
    instance_dir: &Path,
    targets: &[ResourceTarget],
) -> Result<ReconcileReport, ResourceManagerError> {
    ensure_instance_dir(instance_dir)?;
    let scanned = scan::scan_instance(instance_dir, targets).await?;
    let stored = manifest::load(instance_dir).await?;
    Ok(reconcile::reconcile(&scanned, &stored))
}

/// 安装资源：把 `source_path` 复制到目标资源目录并写入清单。
pub async fn install(
    instance_dir: &Path,
    target: &ResourceTarget,
    source_path: &Path,
    provenance: Option<ResourceProvenance>,
) -> Result<ManagedResource, ResourceManagerError> {
    ensure_instance_dir(instance_dir)?;

    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| ResourceManagerError::InvalidFileName(source_path.display().to_string()))?
        .to_string();

    validate_file_name(&file_name)?;
    if !naming::is_resource_file(&file_name) {
        return Err(ResourceManagerError::UnsupportedExtension(file_name));
    }
    // 禁用态文件不能作为安装来源：`.disabled` 是运行时状态而非资源名，
    // 直接落盘会让账目 file_name 带上后缀、enabled 记成 true，账实错位。
    if naming::is_disabled(&file_name) {
        return Err(ResourceManagerError::DisabledFileName(file_name));
    }

    let target_dir = instance_dir.join(&target.relative);
    tokio::fs::create_dir_all(&target_dir).await?;

    let destination = target_dir.join(&file_name);
    if tokio::fs::try_exists(&destination).await.unwrap_or(false) {
        return Err(ResourceManagerError::AlreadyExists(file_name));
    }

    tokio::fs::copy(source_path, &destination).await?;

    let managed = ManagedResource {
        file_name: file_name.clone(),
        kind: target.kind,
        enabled: true,
        hash: file_hash(&destination).await,
        size_bytes: tokio::fs::metadata(&destination)
            .await
            .map(|metadata| metadata.len())
            .ok(),
        provenance,
    };

    manifest::update(instance_dir, |stored| {
        stored.resources.retain(|resource| {
            !(resource.kind == managed.kind && resource.file_name == managed.file_name)
        });
        stored.resources.push(managed.clone());
        Ok(None::<()>)
    })
    .await?;

    Ok(managed)
}

/// 卸载资源：删除目标文件（若仍存在）并从清单移除对应账目。
///
/// `kind` 与 `file_name` 共同定位账目：`mods/` 与 `plugins/` 下的同名文件
/// 是两条独立账目，互不影响。
///
/// 文件已丢失时仍然清理账目，以支持修复"账目在、文件缺"的缺失状态。
pub async fn remove(
    instance_dir: &Path,
    targets: &[ResourceTarget],
    kind: InstanceExtensionKind,
    file_name: &str,
) -> Result<(), ResourceManagerError> {
    ensure_instance_dir(instance_dir)?;

    match locate(instance_dir, targets, kind, file_name).await {
        Ok((located, _target)) => {
            if tokio::fs::try_exists(&located).await.unwrap_or(false) {
                tokio::fs::remove_file(&located).await?;
            }
        }
        // 文件已不存在：只清理账目，让"缺失"状态可被修复。
        Err(ResourceManagerError::NotFound(_)) => {}
        Err(error) => return Err(error),
    }

    manifest::update(instance_dir, |stored| {
        stored
            .resources
            .retain(|resource| !(resource.kind == kind && resource.file_name == file_name));
        Ok(None::<()>)
    })
    .await?;

    Ok(())
}

/// 启用或禁用资源：重命名文件并同步清单中的文件名与状态。
///
/// `kind` 与 `file_name` 共同定位账目，避免同名文件跨目录互相顶替。
pub async fn set_enabled(
    instance_dir: &Path,
    targets: &[ResourceTarget],
    kind: InstanceExtensionKind,
    file_name: &str,
    enabled: bool,
) -> Result<InstanceExtension, ResourceManagerError> {
    ensure_instance_dir(instance_dir)?;

    let (located, target) = locate(instance_dir, targets, kind, file_name).await?;
    let desired_name = if enabled {
        naming::enabled_name(file_name).to_string()
    } else {
        naming::disabled_name(file_name)
    };

    let renamed = located.with_file_name(&desired_name);
    if renamed != located {
        if tokio::fs::try_exists(&renamed).await.unwrap_or(false) {
            return Err(ResourceManagerError::AlreadyExists(desired_name));
        }
        tokio::fs::rename(&located, &renamed).await?;
    }

    manifest::update(instance_dir, |stored| {
        for resource in &mut stored.resources {
            if resource.kind == kind && resource.file_name == file_name {
                resource.file_name = desired_name.clone();
                resource.enabled = enabled;
            }
        }
        Ok(None::<()>)
    })
    .await?;

    let size_bytes = tokio::fs::metadata(&renamed)
        .await
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    Ok(InstanceExtension::new(target.kind, desired_name, renamed, enabled, size_bytes)?)
}

/// 对账并将结果回写清单。
///
/// 接受文件系统中新出现的资源（记为未知来源），移除已消失的账目，并刷新
/// 启用状态与文件大小。返回基于回写前快照的对账报告。
pub async fn sync(
    instance_dir: &Path,
    targets: &[ResourceTarget],
) -> Result<ReconcileReport, ResourceManagerError> {
    ensure_instance_dir(instance_dir)?;

    // 扫描是"事实来源"：目录只读，基于锁外的扫描结果可以接受。
    let scanned = scan::scan_instance(instance_dir, targets).await?;

    // 对账必须基于锁内最新清单：若先基于锁外快照对账再回写，会覆盖对账期间
    // 并发写入的新账目。这里把对账移入 update 闭包，以锁内清单为准重建，
    // 并把报告通过返回值带出闭包。
    let (_, report) = manifest::update(instance_dir, |manifest| {
        let current = reconcile::reconcile(&scanned, manifest);

        let mut resources = Vec::with_capacity(current.items.len());
        for item in &current.items {
            if item.state == ResourceState::Missing {
                continue;
            }
            let Some(extension) = item.extension.as_ref() else {
                continue;
            };
            let mut managed = item
                .managed
                .clone()
                .unwrap_or_else(|| ManagedResource::from_extension(extension));
            managed.kind = extension.kind;
            managed.enabled = extension.enabled;
            managed.size_bytes = Some(extension.size_bytes);
            resources.push(managed);
        }

        manifest.schema_version = MANIFEST_SCHEMA_VERSION;
        manifest.resources = resources;
        Ok(Some(current))
    })
    .await?;

    // 闭包必然执行且返回 `Some`；`unwrap_or_default` 仅为防御 future 误改
    // 返回值时不再 panic。
    Ok(report.unwrap_or_default())
}

/// 定位资源文件并返回其所在目标目录。
///
/// `kind` 限定在对应种类的目标目录内查找（`mods/` 与 `plugins/` 下的同名
/// 文件互不干扰）；`file_name` 必须为单一普通路径组件，否则返回
/// [`ResourceManagerError::InvalidFileName`] 且不触碰文件系统。
async fn locate(
    instance_dir: &Path,
    targets: &[ResourceTarget],
    kind: InstanceExtensionKind,
    file_name: &str,
) -> Result<(PathBuf, ResourceTarget), ResourceManagerError> {
    validate_file_name(file_name)?;

    for target in targets {
        if target.kind != kind {
            continue;
        }
        let candidate = instance_dir.join(&target.relative).join(file_name);
        // 只认普通文件：同名目录顶替文件时不得命中，否则 remove 会误删目录、
        // set_enabled 会把目录改名成 `.disabled`。
        let metadata = match tokio::fs::metadata(&candidate).await {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        if metadata.is_file() {
            return Ok((candidate, target.clone()));
        }
    }

    Err(ResourceManagerError::NotFound(file_name.to_string()))
}

/// 校验外部传入的文件名，拒绝绝对路径、`..` 与路径分隔符。
fn validate_file_name(file_name: &str) -> Result<(), ResourceManagerError> {
    if path::is_single_normal_component(file_name) {
        Ok(())
    } else {
        Err(ResourceManagerError::InvalidFileName(file_name.to_string()))
    }
}

fn ensure_instance_dir(instance_dir: &Path) -> Result<(), ResourceManagerError> {
    if instance_dir.is_dir() {
        Ok(())
    } else {
        Err(ResourceManagerError::InstanceDirNotFound(instance_dir.to_path_buf()))
    }
}

async fn file_hash(path: &Path) -> Option<String> {
    let digest = sha256_file(path).await.ok()?;
    Some(format!("sha256:{}", hex_encode(&digest)))
}

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;
    use crate::resource::manager::models::{InstanceExtensionKind, ManagedResource};

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir()
            .join(format!("sealantern-resource-ops-{label}-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&path).expect("temp dir should be created");
        path
    }

    fn mod_target() -> ResourceTarget {
        ResourceTarget::new(InstanceExtensionKind::Mod, "mods")
    }

    #[tokio::test]
    async fn install_copies_file_and_records_manifest() {
        let root = temp_dir("install");
        let source = root.join("sodium.jar");
        fs::write(&source, b"jar-bytes").unwrap();
        let instance = root.join("instance");
        fs::create_dir_all(&instance).unwrap();

        let managed = install(&instance, &mod_target(), &source, None)
            .await
            .unwrap();
        assert_eq!(managed.file_name, "sodium.jar");
        assert!(
            managed
                .hash
                .as_deref()
                .is_some_and(|hash| hash.starts_with("sha256:"))
        );
        assert!(instance.join("mods/sodium.jar").exists());

        let stored = manifest::load(&instance).await.unwrap();
        assert_eq!(stored.resources.len(), 1);

        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn install_rejects_duplicate_and_unsupported_files() {
        let root = temp_dir("install-guard");
        let instance = root.join("instance");
        fs::create_dir_all(&instance).unwrap();
        let source = root.join("sodium.jar");
        fs::write(&source, b"jar").unwrap();

        install(&instance, &mod_target(), &source, None)
            .await
            .unwrap();
        let error = install(&instance, &mod_target(), &source, None)
            .await
            .unwrap_err();
        assert!(matches!(error, ResourceManagerError::AlreadyExists(_)));

        let unsupported = root.join("readme.txt");
        fs::write(&unsupported, b"text").unwrap();
        let error = install(&instance, &mod_target(), &unsupported, None)
            .await
            .unwrap_err();
        assert!(matches!(error, ResourceManagerError::UnsupportedExtension(_)));

        // 禁用态文件不能作为安装来源：避免账目 file_name 带 .disabled 后缀
        // 且 enabled 记成 true 的账实错位。
        let disabled = root.join("paused.jar.disabled");
        fs::write(&disabled, b"jar").unwrap();
        let error = install(&instance, &mod_target(), &disabled, None)
            .await
            .unwrap_err();
        assert!(matches!(error, ResourceManagerError::DisabledFileName(_)));

        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn set_enabled_renames_file_and_updates_manifest() {
        let root = temp_dir("toggle");
        let instance = root.join("instance");
        fs::create_dir_all(instance.join("mods")).unwrap();
        fs::write(instance.join("mods/sodium.jar"), b"jar").unwrap();

        let disabled = set_enabled(
            &instance,
            &[mod_target()],
            InstanceExtensionKind::Mod,
            "sodium.jar",
            false,
        )
        .await
        .unwrap();
        assert_eq!(disabled.file_name, "sodium.jar.disabled");
        assert!(!disabled.enabled);
        assert!(instance.join("mods/sodium.jar.disabled").exists());

        let enabled = set_enabled(
            &instance,
            &[mod_target()],
            InstanceExtensionKind::Mod,
            "sodium.jar.disabled",
            true,
        )
        .await
        .unwrap();
        assert_eq!(enabled.file_name, "sodium.jar");
        assert!(enabled.enabled);

        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn remove_deletes_file_and_ledger_entry() {
        let root = temp_dir("remove");
        let instance = root.join("instance");
        fs::create_dir_all(instance.join("mods")).unwrap();
        fs::write(instance.join("mods/sodium.jar"), b"jar").unwrap();

        remove(&instance, &[mod_target()], InstanceExtensionKind::Mod, "sodium.jar")
            .await
            .unwrap();
        assert!(!instance.join("mods/sodium.jar").exists());

        let stored = manifest::load(&instance).await.unwrap();
        assert!(stored.resources.is_empty());

        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn remove_clears_ledger_entry_when_file_is_already_missing() {
        let root = temp_dir("remove-missing");
        let instance = root.join("instance");
        fs::create_dir_all(instance.join("mods")).unwrap();
        fs::write(instance.join("mods/gone.jar"), b"jar").unwrap();
        sync(&instance, &[mod_target()]).await.unwrap();

        fs::remove_file(instance.join("mods/gone.jar")).unwrap();
        assert_eq!(
            list(&instance, &[mod_target()])
                .await
                .unwrap()
                .missing_count(),
            1
        );

        remove(&instance, &[mod_target()], InstanceExtensionKind::Mod, "gone.jar")
            .await
            .unwrap();
        assert_eq!(
            list(&instance, &[mod_target()])
                .await
                .unwrap()
                .missing_count(),
            0
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn traversal_file_names_are_rejected_without_touching_the_filesystem() {
        let root = temp_dir("traversal");
        let instance = root.join("instance");
        fs::create_dir_all(instance.join("mods")).unwrap();
        let outside = root.join("outside.jar");
        fs::write(&outside, b"must stay").unwrap();

        for file_name in ["..", "../outside.jar", "/etc/passwd", "a/b.jar", ""] {
            let error = remove(&instance, &[mod_target()], InstanceExtensionKind::Mod, file_name)
                .await
                .unwrap_err();
            assert!(
                matches!(error, ResourceManagerError::InvalidFileName(_)),
                "remove must reject `{file_name}`"
            );

            let error = set_enabled(
                &instance,
                &[mod_target()],
                InstanceExtensionKind::Mod,
                file_name,
                false,
            )
            .await
            .unwrap_err();
            assert!(
                matches!(error, ResourceManagerError::InvalidFileName(_)),
                "set_enabled must reject `{file_name}`"
            );
        }

        assert!(outside.exists(), "traversal must not delete files outside the instance");

        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn directory_shadowing_a_resource_is_ignored_by_locate() {
        let root = temp_dir("dir-shadow");
        let instance = root.join("instance");

        // 创建同名目录顶替资源文件，并在清单中登记该文件名的账目。
        fs::create_dir_all(instance.join("mods/foo.jar")).unwrap();
        manifest::update(&instance, |stored| {
            stored.resources.push(ManagedResource {
                file_name: "foo.jar".into(),
                kind: InstanceExtensionKind::Mod,
                enabled: true,
                hash: None,
                size_bytes: None,
                provenance: None,
            });
            Ok(None::<()>)
        })
        .await
        .unwrap();

        // 目录不是资源文件：对账应报 Missing，set_enabled 不应把目录改名。
        let report = list(&instance, &[mod_target()]).await.unwrap();
        assert_eq!(report.missing_count(), 1);

        let error =
            set_enabled(&instance, &[mod_target()], InstanceExtensionKind::Mod, "foo.jar", false)
                .await
                .unwrap_err();
        assert!(matches!(error, ResourceManagerError::NotFound(_)));

        // remove 只清理账目，不得删除同名目录。
        remove(&instance, &[mod_target()], InstanceExtensionKind::Mod, "foo.jar")
            .await
            .unwrap();
        assert!(instance.join("mods/foo.jar").is_dir(), "directory must not be deleted");
        assert_eq!(
            list(&instance, &[mod_target()])
                .await
                .unwrap()
                .missing_count(),
            0
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn sync_accepts_unknown_files_and_drops_missing_entries() {
        let root = temp_dir("sync");
        let instance = root.join("instance");
        fs::create_dir_all(instance.join("mods")).unwrap();
        fs::write(instance.join("mods/manual.jar"), b"jar").unwrap();

        let report = sync(&instance, &[mod_target()]).await.unwrap();
        assert_eq!(report.unknown_count(), 1);

        let stored = manifest::load(&instance).await.unwrap();
        assert_eq!(stored.resources.len(), 1);
        assert_eq!(stored.resources[0].file_name, "manual.jar");

        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn list_reports_missing_ledger_entries() {
        let root = temp_dir("list");
        let instance = root.join("instance");
        fs::create_dir_all(instance.join("mods")).unwrap();
        fs::write(instance.join("mods/gone.jar"), b"jar").unwrap();
        sync(&instance, &[mod_target()]).await.unwrap();
        fs::remove_file(instance.join("mods/gone.jar")).unwrap();

        let report = list(&instance, &[mod_target()]).await.unwrap();
        assert_eq!(report.missing_count(), 1);

        fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test]
    async fn same_file_name_in_mods_and_plugins_are_independent_entries() {
        let root = temp_dir("same-name");
        let instance = root.join("instance");
        fs::create_dir_all(instance.join("mods")).unwrap();
        fs::create_dir_all(instance.join("plugins")).unwrap();

        // 同名 jar 分别作为模组与插件安装。
        let source = root.join("test-mod.jar");
        fs::write(&source, b"jar").unwrap();
        let mod_target = ResourceTarget::new(InstanceExtensionKind::Mod, "mods");
        let plugin_target = ResourceTarget::new(InstanceExtensionKind::Plugin, "plugins");
        install(&instance, &mod_target, &source, None)
            .await
            .unwrap();
        install(&instance, &plugin_target, &source, None)
            .await
            .unwrap();

        let targets = vec![mod_target, plugin_target];

        // 两条独立账目，均对账正常。
        let report = list(&instance, &targets).await.unwrap();
        assert_eq!(report.total(), 2);
        assert_eq!(report.normal_count(), 2);

        // 卸载模组侧，插件侧账目与文件不受影响。
        remove(&instance, &targets, InstanceExtensionKind::Mod, "test-mod.jar")
            .await
            .unwrap();
        assert!(!instance.join("mods/test-mod.jar").exists());
        assert!(instance.join("plugins/test-mod.jar").exists());

        let stored = manifest::load(&instance).await.unwrap();
        assert_eq!(stored.resources.len(), 1);
        assert_eq!(stored.resources[0].kind, InstanceExtensionKind::Plugin);

        // 禁用插件侧，同样只影响自身。
        let disabled =
            set_enabled(&instance, &targets, InstanceExtensionKind::Plugin, "test-mod.jar", false)
                .await
                .unwrap();
        assert_eq!(disabled.file_name, "test-mod.jar.disabled");
        assert!(instance.join("plugins/test-mod.jar.disabled").exists());

        fs::remove_dir_all(root).unwrap();
    }
}
