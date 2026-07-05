//! 插件安装和删除流程

mod zip_ops;

use super::i18n::{plugin_t1, plugin_t2};
use super::{PluginInfo, PluginInstallResult, PluginManager, PluginState};
use crate::hardcode_data::app_files::PLUGIN_MANIFEST_FILE_NAME;
use crate::hardcode_data::plugin_manifest::{
    manifest_not_found_in_dir_message, parse_manifest_failed_message, read_manifest_failed_message,
    unsupported_plugin_source_message,
};
use crate::models::plugin::{PluginDistributionClass, PluginInstallIssue};
use crate::plugins::loader::PluginLoader;
use crate::services::plugin_trusted_catalog::PluginInstallMetadata;
use std::fs::{self};
use std::path::{Path, PathBuf};

fn build_install_notices(plugin_info: &PluginInfo) -> Vec<PluginInstallIssue> {
    let mut notices = Vec::new();
    let normalized_permissions = crate::plugins::runtime::permissions::normalize_permissions(
        plugin_info.manifest.permissions.clone(),
    );

    if matches!(
        plugin_info.permission_profile,
        crate::models::plugin::PluginPermissionProfile::Unreviewed
    ) && crate::hardcode_data::plugin_permissions::requests_trusted_capabilities(
        &normalized_permissions,
    ) {
        notices.push(PluginInstallIssue::requests_trusted_capabilities(
            plugin_info.manifest.id.clone(),
            normalized_permissions.clone(),
        ));
    }

    if plugin_info.exceeds_standard_sandbox {
        notices.push(PluginInstallIssue::exceeds_standard_sandbox(
            plugin_info.manifest.id.clone(),
            normalized_permissions,
        ));
    }

    notices
}

/// 安装本地插件
///
/// 支持目录、`manifest.json` 和 ZIP 压缩包
pub(super) fn install_local_plugin(
    manager: &mut PluginManager,
    path: &Path,
    metadata: &PluginInstallMetadata,
) -> Result<PluginInstallResult, String> {
    let (plugin_info, distribution_class) = if path.extension().is_some_and(|ext| ext == "zip") {
        let distribution_class = metadata
            .distribution_class
            .clone()
            .unwrap_or(PluginDistributionClass::StandardPackage);
        (zip_ops::install_plugin_from_zip(manager, path, metadata)?, distribution_class)
    } else if path
        .file_name()
        .is_some_and(|name| name == PLUGIN_MANIFEST_FILE_NAME)
    {
        let plugin_dir = path.parent().ok_or("Invalid manifest path")?;
        let distribution_class = metadata
            .distribution_class
            .clone()
            .unwrap_or(PluginDistributionClass::ManualImport);
        (
            install_plugin_from_dir(manager, plugin_dir, distribution_class.clone(), metadata)?,
            distribution_class,
        )
    } else if path.is_dir() {
        let distribution_class = metadata
            .distribution_class
            .clone()
            .unwrap_or(PluginDistributionClass::LocalDirectory);
        (
            install_plugin_from_dir(manager, path, distribution_class.clone(), metadata)?,
            distribution_class,
        )
    } else {
        return Err(unsupported_plugin_source_message());
    };

    let missing_dependencies = manager.get_missing_dependencies(&plugin_info.manifest);
    let install_notices = build_install_notices(&plugin_info);
    let exceeds_standard_sandbox = plugin_info.exceeds_standard_sandbox;

    Ok(PluginInstallResult {
        plugin: plugin_info,
        missing_dependencies,
        untrusted_url: false,
        suggested_trust_level: None,
        integrity_status: None,
        review_status: None,
        distribution_class: Some(distribution_class),
        permission_profile: None,
        trusted_catalog_matched: false,
        hash_matched: false,
        exceeds_standard_sandbox,
        install_notices,
    })
}

/// 从目录安装插件
///
/// # Parameters
///
/// - `manager`: 插件管理器
/// - `source_dir`: 插件目录
pub(super) fn install_plugin_from_dir(
    manager: &mut PluginManager,
    source_dir: &Path,
    distribution_class: PluginDistributionClass,
    metadata: &PluginInstallMetadata,
) -> Result<PluginInfo, String> {
    let manifest_path = source_dir.join(PLUGIN_MANIFEST_FILE_NAME);
    if !manifest_path.exists() {
        return Err(manifest_not_found_in_dir_message(source_dir));
    }

    let manifest_content =
        fs::read_to_string(&manifest_path).map_err(|e| read_manifest_failed_message(&e))?;

    let manifest: crate::models::plugin::PluginManifest =
        serde_json::from_str(&manifest_content).map_err(|e| parse_manifest_failed_message(&e))?;

    PluginLoader::validate_manifest(&manifest)?;

    let plugin_id = manifest.id.clone();
    super::source::validate_replace_target(
        manager,
        crate::models::plugin::PluginSource::Local,
        &plugin_id,
    )?;

    let target_dir = manager.plugins_dir.join(&plugin_id);

    let source_canonical = source_dir
        .canonicalize()
        .map_err(|e| plugin_t1("plugin.install.source_path_resolve_failed", e.to_string()))?;
    let target_canonical =
        if target_dir.exists() {
            Some(target_dir.canonicalize().map_err(|e| {
                plugin_t1("plugin.install.target_path_resolve_failed", e.to_string())
            })?)
        } else {
            None
        };

    if target_canonical.as_ref() == Some(&source_canonical) {
        let loaded_manifest = PluginLoader::load_manifest(&target_dir)?;
        PluginLoader::validate_manifest(&loaded_manifest)?;

        let mut persisted_metadata = metadata.clone();
        persisted_metadata.installed_tree_sha256 = Some(
            crate::services::plugin_trusted_catalog::compute_plugin_tree_sha256(&target_dir)?,
        );

        let missing_deps = manager.get_missing_dependencies(&loaded_manifest);

        let plugin_info = manager.make_local_plugin_info(
            loaded_manifest,
            PluginState::Loaded,
            target_dir.to_string_lossy().to_string(),
            distribution_class.clone(),
            metadata.archive_sha256.as_deref(),
            missing_deps,
        );

        crate::services::plugin_trusted_catalog::write_install_metadata(
            &target_dir,
            &persisted_metadata,
        )?;

        manager.plugins.insert(plugin_id, plugin_info.clone());
        return Ok(plugin_info);
    }

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)
            .map_err(|e| plugin_t1("plugin.install.remove_existing_dir_failed", e.to_string()))?;
    }

    PluginManager::copy_dir_recursive(source_dir, &target_dir)?;

    let mut persisted_metadata = metadata.clone();
    persisted_metadata.installed_tree_sha256 = Some(
        crate::services::plugin_trusted_catalog::compute_plugin_tree_sha256(&target_dir)?,
    );
    crate::services::plugin_trusted_catalog::write_install_metadata(
        &target_dir,
        &persisted_metadata,
    )?;

    let loaded_manifest = PluginLoader::load_manifest(&target_dir)?;
    PluginLoader::validate_manifest(&loaded_manifest)?;

    let missing_deps = manager.get_missing_dependencies(&loaded_manifest);

    let plugin_info = manager.make_local_plugin_info(
        loaded_manifest,
        PluginState::Loaded,
        target_dir.to_string_lossy().to_string(),
        distribution_class,
        metadata.archive_sha256.as_deref(),
        missing_deps,
    );

    manager.plugins.insert(plugin_id, plugin_info.clone());

    Ok(plugin_info)
}

/// 删除插件和可选数据目录
///
/// # Parameters
///
/// - `manager`: 插件管理器
/// - `plugin_id`: 插件 ID
/// - `delete_data`: 是否一并删除数据目录
pub(super) fn delete_plugin(
    manager: &mut PluginManager,
    plugin_id: &str,
    delete_data: bool,
) -> Result<(), String> {
    if let Some(plugin_info) = manager.plugins.get(plugin_id) {
        if matches!(plugin_info.source, crate::models::plugin::PluginSource::Builtin) {
            return Err(format!("Builtin plugin '{}' cannot be deleted", plugin_id));
        }
        if matches!(plugin_info.state, PluginState::Enabled) {
            return Err(plugin_t1(
                "plugin.delete.already_running",
                plugin_info.manifest.name.clone(),
            ));
        }
    }

    let _dropped_runtime = {
        let mut runtimes = manager.runtimes.write().unwrap_or_else(|e| {
            eprintln!("[WARN] RwLock poisoned, recovering: {}", e);
            e.into_inner()
        });
        runtimes.remove(plugin_id)
    };

    drop(_dropped_runtime);

    let plugin_info = manager
        .plugins
        .remove(plugin_id)
        .ok_or_else(|| plugin_t1("plugin.common.not_found", plugin_id.to_string()))?;

    let plugin_path = PathBuf::from(&plugin_info.path);
    if plugin_path.exists() {
        remove_dir_all_with_retry(
            &plugin_path,
            &plugin_t1("plugin.delete.path_label_plugin_dir", plugin_info.manifest.name.clone()),
        )?;
    }

    let data_dir = manager.data_dir.join(plugin_id);
    if data_dir.exists() {
        let should_delete = delete_data || {
            fs::read_dir(&data_dir)
                .map(|mut e| e.next().is_none())
                .unwrap_or(false)
        };
        if should_delete {
            remove_dir_all_with_retry(
                &data_dir,
                &plugin_t1("plugin.delete.path_label_plugin_data_dir", plugin_id.to_string()),
            )?;
        }
    }

    Ok(())
}

/// 删除目录，失败时做几次短暂重试
fn remove_dir_all_with_retry(path: &Path, label: &str) -> Result<(), String> {
    let mut last_error = None;
    for attempt in 0..3 {
        match fs::remove_dir_all(path) {
            Ok(_) => {
                last_error = None;
                break;
            }
            Err(e) => {
                last_error = Some(e);
                if attempt < 2 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }
    if let Some(e) = last_error {
        return Err(plugin_t2("plugin.delete.remove_failed", label.to_string(), e.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build_install_notices;
    use crate::models::plugin::{
        PluginActions, PluginAuthor, PluginDistributionClass, PluginExecutionClass, PluginInfo,
        PluginIntegrityStatus, PluginManifest, PluginPermissionProfile, PluginReviewStatus,
        PluginRuntimeKind, PluginSource, PluginState, PluginTrustLevelDisplay,
        PluginTrustedPolicySource,
    };
    use std::collections::HashMap;

    fn sample_plugin_info(permissions: Vec<&str>) -> PluginInfo {
        PluginInfo {
            manifest: PluginManifest {
                id: "demo.plugin".to_string(),
                name: "Demo".to_string(),
                version: "1.0.0".to_string(),
                description: "demo".to_string(),
                author: PluginAuthor {
                    name: "SeaLantern".to_string(),
                    email: None,
                    url: None,
                },
                main: "main.lua".to_string(),
                license: None,
                homepage: None,
                repository: None,
                engines: None,
                permissions: permissions.into_iter().map(str::to_string).collect(),
                ui: None,
                events: Vec::new(),
                commands: Vec::new(),
                programs: Vec::new(),
                dependencies: Vec::new(),
                optional_dependencies: Vec::new(),
                icon: None,
                settings: None,
                sidebar: None,
                locales: None,
                include: Vec::new(),
                capabilities: Vec::new(),
                theme_var_map: HashMap::new(),
                presets: HashMap::new(),
                server_events: HashMap::new(),
            },
            state: PluginState::Loaded,
            source: PluginSource::Local,
            runtime: PluginRuntimeKind::Lua,
            path: "demo".to_string(),
            actions: PluginActions {
                can_toggle: true,
                can_delete: true,
                can_check_update: true,
            },
            missing_dependencies: Vec::new(),
            trust_level_display: PluginTrustLevelDisplay::Unreviewed,
            execution_class: PluginExecutionClass::Sandboxed,
            review_status: PluginReviewStatus::Unreviewed,
            integrity_status: PluginIntegrityStatus::Unknown,
            trusted_policy_source: PluginTrustedPolicySource::None,
            permission_profile: PluginPermissionProfile::Unreviewed,
            publisher_id: None,
            distribution_class: PluginDistributionClass::LocalDirectory,
            trusted_catalog_matched: false,
            hash_matched: false,
            verified_hash: None,
            verified_signature: false,
            reviewed_at: None,
            revoked: false,
            exceeds_standard_sandbox: true,
            requires_explicit_consent: true,
        }
    }

    #[test]
    fn install_notices_include_trusted_and_ceiling_signals() {
        let plugin = sample_plugin_info(vec!["execute_program", "network"]);

        let notices = build_install_notices(&plugin);
        let codes = notices
            .into_iter()
            .map(|notice| notice.code)
            .collect::<Vec<_>>();

        assert!(codes.contains(&"plugins.install.issue.requests_trusted_capabilities".to_string()));
        assert!(codes.contains(&"plugins.install.issue.exceeds_standard_sandbox".to_string()));
    }
}
