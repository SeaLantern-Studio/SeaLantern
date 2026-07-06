use std::fs;

use super::common::{ensure_plugin_target_dir_for_server, validate_plugin_file_name};
use crate::models::server::ServerInstance;
use tokio::io::AsyncWriteExt;

pub(crate) fn toggle_plugin(
    server: &ServerInstance,
    file_name: &str,
    enabled: bool,
) -> Result<(), String> {
    let plugins_dir = ensure_plugin_target_dir_for_server(server)?;
    let base_file_name = validate_plugin_file_name(file_name)?;

    let current_path = if enabled {
        plugins_dir.join(format!("{}.disabled", base_file_name))
    } else {
        plugins_dir.join(&base_file_name)
    };

    let new_path = if enabled {
        plugins_dir.join(&base_file_name)
    } else {
        plugins_dir.join(format!("{}.disabled", base_file_name))
    };

    if current_path.exists() {
        fs::rename(&current_path, &new_path)
            .map_err(|e| format!("Failed to toggle plugin: {}", e))?;
    }

    Ok(())
}

pub(crate) fn delete_plugin(server: &ServerInstance, file_name: &str) -> Result<(), String> {
    let plugins_dir = ensure_plugin_target_dir_for_server(server)?;
    let base_file_name = validate_plugin_file_name(file_name)?;

    let enabled_path = plugins_dir.join(&base_file_name);
    let disabled_path = plugins_dir.join(format!("{}.disabled", base_file_name));

    if enabled_path.exists() {
        trash::delete(&enabled_path).map_err(|e| format!("Failed to delete plugin: {}", e))?;
    }

    if disabled_path.exists() {
        trash::delete(&disabled_path).map_err(|e| format!("Failed to delete plugin: {}", e))?;
    }

    Ok(())
}

pub(crate) async fn install_plugin(
    server: &ServerInstance,
    file_data: Vec<u8>,
    file_name: &str,
) -> Result<(), String> {
    let plugins_dir = ensure_plugin_target_dir_for_server(server)?;
    let base_file_name = validate_plugin_file_name(file_name)?;
    let plugin_path = plugins_dir.join(base_file_name);

    let mut file = tokio::fs::File::create(&plugin_path)
        .await
        .map_err(|e| format!("Failed to create plugin file: {}", e))?;

    file.write_all(&file_data)
        .await
        .map_err(|e| format!("Failed to write plugin file: {}", e))?;

    Ok(())
}
