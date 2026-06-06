//! 插件安装和删除流程

mod zip_ops;

use super::i18n::{plugin_t1, plugin_t2};
use super::{PluginInfo, PluginInstallResult, PluginManager, PluginState};
use crate::hardcode_data::app_files::PLUGIN_MANIFEST_FILE_NAME;
use crate::hardcode_data::plugin_manifest::{
    manifest_not_found_in_dir_message, parse_manifest_failed_message, read_manifest_failed_message,
    unsupported_plugin_source_message,
};
use crate::plugins::loader::PluginLoader;
use std::fs::{self};
use std::path::{Path, PathBuf};

/// 按输入来源安装插件
///
/// 支持目录、`manifest.json` 和 ZIP 压缩包
pub(super) fn install_plugin(
    manager: &mut PluginManager,
    path: &Path,
) -> Result<PluginInstallResult, String> {
    let plugin_info = if path.extension().is_some_and(|ext| ext == "zip") {
        zip_ops::install_plugin_from_zip(manager, path)?
    } else if path
        .file_name()
        .is_some_and(|name| name == PLUGIN_MANIFEST_FILE_NAME)
    {
        let plugin_dir = path.parent().ok_or("Invalid manifest path")?;
        install_plugin_from_dir(manager, plugin_dir)?
    } else if path.is_dir() {
        install_plugin_from_dir(manager, path)?
    } else {
        return Err(unsupported_plugin_source_message());
    };

    let missing_dependencies = manager.get_missing_dependencies(&plugin_info.manifest);

    Ok(PluginInstallResult {
        plugin: plugin_info,
        missing_dependencies,
        untrusted_url: false,
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

    if let Some(existing) = manager.plugins.get(&plugin_id) {
        if matches!(existing.state, PluginState::Enabled) {
            return Err(plugin_t1(
                "plugin.install.already_running_replace",
                existing.manifest.name.clone(),
            ));
        }
    }

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

        let missing_deps = manager.get_missing_dependencies(&loaded_manifest);

        let plugin_info = PluginInfo {
            manifest: loaded_manifest,
            state: PluginState::Loaded,
            path: target_dir.to_string_lossy().to_string(),
            missing_dependencies: missing_deps,
        };

        manager.plugins.insert(plugin_id, plugin_info.clone());
        return Ok(plugin_info);
    }

    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)
            .map_err(|e| plugin_t1("plugin.install.remove_existing_dir_failed", e.to_string()))?;
    }

    PluginManager::copy_dir_recursive(source_dir, &target_dir)?;

    let loaded_manifest = PluginLoader::load_manifest(&target_dir)?;
    PluginLoader::validate_manifest(&loaded_manifest)?;

    let missing_deps = manager.get_missing_dependencies(&loaded_manifest);

    let plugin_info = PluginInfo {
        manifest: loaded_manifest,
        state: PluginState::Loaded,
        path: target_dir.to_string_lossy().to_string(),
        missing_dependencies: missing_deps,
    };

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
