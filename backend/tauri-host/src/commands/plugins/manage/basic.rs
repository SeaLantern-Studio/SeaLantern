use super::common::{lock_manager, validate_plugin_id};
use crate::hardcode_data::app_files::PLUGIN_MANIFEST_FILE_NAME;
use crate::hardcode_data::plugin_manifest::{
    invalid_manifest_path_message, missing_manifest_in_folder_message,
    unsupported_plugin_source_message,
};
use crate::models::plugin::{
    BatchInstallError, BatchInstallResult, PluginInfo, PluginInstallResult,
};
use crate::plugins::manager::i18n::plugin_t1;
use crate::plugins::manager::PluginManager;
use std::sync::{Arc, Mutex};

/// 读取插件列表
pub(super) fn list_plugins(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<PluginInfo>, String> {
    let manager = lock_manager(&manager);
    Ok(manager.get_plugin_list())
}

/// 重新扫描插件目录
pub(super) fn scan_plugins(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<PluginInfo>, String> {
    let mut manager = lock_manager(&manager);
    manager.scan_plugins()
}

/// 启用插件
pub(super) fn enable_plugin(
    plugin_id: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    validate_plugin_id(&plugin_id)?;
    let mut manager = lock_manager(&manager);
    manager.enable_plugin(&plugin_id)
}

/// 禁用插件
pub(super) fn disable_plugin(
    plugin_id: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<String>, String> {
    validate_plugin_id(&plugin_id)?;
    let mut manager = lock_manager(&manager);
    manager.disable_plugin(&plugin_id)
}

/// 读取插件导航项
pub(super) fn get_plugin_nav_items(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<serde_json::Value>, String> {
    let manager = lock_manager(&manager);
    Ok(manager.get_nav_items())
}

/// 从本地路径安装插件
pub(super) fn install_plugin(
    path: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<PluginInstallResult, String> {
    let file_path = std::path::PathBuf::from(path);
    let is_zip = file_path.extension().and_then(|e| e.to_str()) == Some("zip");
    let is_manifest = file_path
        .file_name()
        .is_some_and(|name| name == PLUGIN_MANIFEST_FILE_NAME);
    let is_dir = file_path.is_dir();

    if !is_zip && !is_manifest && !is_dir {
        return Err(unsupported_plugin_source_message());
    }
    if (is_zip || is_manifest) && !file_path.is_file() {
        return Err(plugin_t1("plugin.install.path_not_file", file_path.display().to_string()));
    }

    let mut manager = lock_manager(&manager);
    manager.install_plugin(&file_path)
}

/// 读取插件图标
pub(super) fn get_plugin_icon(
    plugin_id: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<String, String> {
    validate_plugin_id(&plugin_id)?;
    let manager = lock_manager(&manager);
    manager.get_plugin_icon(&plugin_id)
}

/// 读取插件设置
pub(super) fn get_plugin_settings(
    plugin_id: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<serde_json::Value, String> {
    validate_plugin_id(&plugin_id)?;
    let manager = lock_manager(&manager);
    manager.get_plugin_settings(&plugin_id)
}

/// 写入插件设置
pub(super) fn set_plugin_settings(
    plugin_id: String,
    settings: serde_json::Value,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<(), String> {
    validate_plugin_id(&plugin_id)?;
    let manager = lock_manager(&manager);
    manager.set_plugin_settings(&plugin_id, settings)
}

/// 读取插件样式
pub(super) fn get_plugin_css(
    plugin_id: String,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<String, String> {
    validate_plugin_id(&plugin_id)?;
    let manager = lock_manager(&manager);
    manager.get_plugin_css(&plugin_id)
}

/// 读取全部插件样式
pub(super) fn get_all_plugin_css(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<Vec<(String, String)>, String> {
    let manager = lock_manager(&manager);
    manager.get_all_plugin_css()
}

/// 删除单个插件
pub(super) async fn delete_plugin(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
    plugin_id: String,
    delete_data: Option<bool>,
) -> Result<(), String> {
    validate_plugin_id(&plugin_id)?;
    let mut manager = lock_manager(&manager);
    manager.delete_plugin(&plugin_id, delete_data.unwrap_or(false))
}

/// 批量删除插件
pub(super) async fn delete_plugins(
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
    plugin_ids: Vec<String>,
    delete_data: Option<bool>,
) -> Result<(), String> {
    let delete_data = delete_data.unwrap_or(false);
    let mut manager = lock_manager(&manager);

    for plugin_id in &plugin_ids {
        validate_plugin_id(plugin_id)?;
    }

    for plugin_id in plugin_ids {
        manager.delete_plugin(&plugin_id, delete_data)?;
    }

    Ok(())
}

/// 批量安装插件
pub(super) fn install_plugins_batch(
    paths: Vec<String>,
    manager: tauri::State<'_, Arc<Mutex<PluginManager>>>,
) -> Result<BatchInstallResult, String> {
    let mut success = Vec::new();
    let mut failed = Vec::new();
    let mut manager = lock_manager(&manager);

    for path_str in paths {
        let path = std::path::PathBuf::from(&path_str);

        let result = if path.is_file() {
            if path_str.ends_with(".zip") {
                manager.install_plugin(&path)
            } else if path_str.ends_with(PLUGIN_MANIFEST_FILE_NAME) {
                if let Some(parent) = path.parent() {
                    manager.install_plugin(parent)
                } else {
                    Err(invalid_manifest_path_message())
                }
            } else {
                Err(unsupported_plugin_source_message())
            }
        } else if path.is_dir() {
            let manifest_path = path.join(PLUGIN_MANIFEST_FILE_NAME);
            if manifest_path.exists() {
                manager.install_plugin(&path)
            } else {
                Err(missing_manifest_in_folder_message())
            }
        } else {
            Err(plugin_t1("plugin.install.path_not_exist", path_str.clone()))
        };

        match result {
            Ok(install_result) => success.push(install_result),
            Err(error) => failed.push(BatchInstallError { path: path_str, error }),
        }
    }

    Ok(BatchInstallResult { success, failed })
}
