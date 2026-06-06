use super::{PluginInfo, PluginManager, PluginState};
use crate::hardcode_data::app_files::PLUGIN_MANIFEST_FILE_NAME;
use crate::hardcode_data::plugin_manifest::{
    manifest_not_found_in_zip_message, parse_manifest_failed_message, read_manifest_failed_message,
};
use crate::plugins::loader::PluginLoader;
use crate::plugins::manager::i18n::{plugin_t1, plugin_t2};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;

/// 从 ZIP 压缩包安装插件
///
/// # Parameters
///
/// - `manager`: 插件管理器
/// - `zip_path`: ZIP 文件路径
pub(super) fn install_plugin_from_zip(
    manager: &mut PluginManager,
    zip_path: &Path,
) -> Result<PluginInfo, String> {
    let file = File::open(zip_path)
        .map_err(|e| plugin_t1("plugin.install.open_zip_failed", e.to_string()))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| plugin_t1("plugin.install.read_zip_failed", e.to_string()))?;

    let (manifest_content, prefix) = find_manifest_in_zip(&mut archive)?;

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
    if target_dir.exists() {
        fs::remove_dir_all(&target_dir)
            .map_err(|e| plugin_t1("plugin.install.remove_existing_dir_failed", e.to_string()))?;
    }
    fs::create_dir_all(&target_dir)
        .map_err(|e| plugin_t1("plugin.install.create_plugin_dir_failed", e.to_string()))?;

    extract_zip_to_dir(&mut archive, &prefix, &target_dir)?;

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

/// 在 ZIP 包里查找插件清单
fn find_manifest_in_zip(archive: &mut zip::ZipArchive<File>) -> Result<(String, String), String> {
    if let Ok(mut file) = archive.by_name(PLUGIN_MANIFEST_FILE_NAME) {
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| read_manifest_failed_message(&e))?;
        return Ok((content, String::new()));
    }

    let mut found_prefix: Option<String> = None;
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| plugin_t1("plugin.install.read_zip_entry_failed", e.to_string()))?;
        let name = file.name();

        if name.ends_with(&format!("/{}", PLUGIN_MANIFEST_FILE_NAME)) {
            let parts: Vec<&str> = name.split('/').collect();
            if parts.len() == 2 {
                found_prefix = Some(format!("{}/", parts[0]));
                break;
            }
        }
    }

    if let Some(prefix) = found_prefix {
        let manifest_path = format!("{}{}", prefix, PLUGIN_MANIFEST_FILE_NAME);
        let mut file = archive.by_name(&manifest_path).map_err(|e| {
            plugin_t2("plugin.install.open_zip_entry_failed", manifest_path.clone(), e.to_string())
        })?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| {
            plugin_t2(
                "plugin.install.read_zip_entry_content_failed",
                manifest_path.clone(),
                e.to_string(),
            )
        })?;
        return Ok((content, prefix));
    }

    Err(manifest_not_found_in_zip_message())
}

/// 把 ZIP 内容解压到目标目录
///
/// 这里会顺手拦截路径穿越和超大文件
fn extract_zip_to_dir(
    archive: &mut zip::ZipArchive<File>,
    prefix: &str,
    target_dir: &Path,
) -> Result<(), String> {
    const MAX_SINGLE_FILE_SIZE: u64 = 50 * 1024 * 1024;
    const MAX_TOTAL_SIZE: u64 = 200 * 1024 * 1024;
    let mut total_extracted_size: u64 = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            plugin_t2("plugin.install.read_zip_entry_index_failed", i.to_string(), e.to_string())
        })?;

        let name = file.name().to_string();

        if !prefix.is_empty() && !name.starts_with(prefix) {
            continue;
        }

        let relative_path = if prefix.is_empty() {
            name.clone()
        } else {
            name.strip_prefix(prefix).unwrap_or(&name).to_string()
        };

        if relative_path.is_empty() {
            continue;
        }

        let target_path = target_dir.join(&relative_path);

        let canonical_target = if target_path.exists() {
            target_path
                .canonicalize()
                .unwrap_or_else(|_| target_path.clone())
        } else if let Some(parent) = target_path.parent() {
            if parent.exists() {
                let canonical_parent = parent
                    .canonicalize()
                    .unwrap_or_else(|_| parent.to_path_buf());
                canonical_parent.join(target_path.file_name().unwrap_or_default())
            } else {
                target_path.clone()
            }
        } else {
            target_path.clone()
        };
        let canonical_base = target_dir
            .canonicalize()
            .unwrap_or_else(|_| target_dir.to_path_buf());
        if !canonical_target.starts_with(&canonical_base) {
            return Err(plugin_t1("plugin.install.zip_path_traversal", relative_path));
        }

        let file_size = file.size();
        if file_size > MAX_SINGLE_FILE_SIZE {
            return Err(plugin_t2(
                "plugin.install.zip_file_too_large",
                file.name().to_string(),
                (file_size / 1024 / 1024).to_string(),
            ));
        }
        total_extracted_size += file_size;
        if total_extracted_size > MAX_TOTAL_SIZE {
            return Err(plugin_t2(
                "plugin.install.zip_total_too_large",
                (total_extracted_size / 1024 / 1024).to_string(),
                (MAX_TOTAL_SIZE / 1024 / 1024).to_string(),
            ));
        }

        if file.is_dir() {
            fs::create_dir_all(&target_path).map_err(|e| {
                plugin_t2(
                    "plugin.install.create_directory_failed",
                    target_path.display().to_string(),
                    e.to_string(),
                )
            })?;
        } else {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    plugin_t2(
                        "plugin.install.create_parent_dir_failed",
                        parent.display().to_string(),
                        e.to_string(),
                    )
                })?;
            }

            let mut outfile = File::create(&target_path).map_err(|e| {
                plugin_t2(
                    "plugin.install.create_file_failed",
                    target_path.display().to_string(),
                    e.to_string(),
                )
            })?;
            io::copy(&mut file, &mut outfile).map_err(|e| {
                plugin_t2(
                    "plugin.install.write_file_failed",
                    target_path.display().to_string(),
                    e.to_string(),
                )
            })?;
        }
    }

    Ok(())
}
