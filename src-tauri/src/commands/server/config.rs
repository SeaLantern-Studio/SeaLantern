use crate::models::config::ServerProperties;
use crate::models::server::{CpuPolicyConfig, JvmPresetConfig};
use crate::services::server::config as config_parser;
use crate::utils::logger;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// SL.json 启动配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SLStartupConfig {
    #[serde(default)]
    pub max_memory: Option<u32>,
    #[serde(default)]
    pub min_memory: Option<u32>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub cpu_policy: CpuPolicyConfig,
    #[serde(default)]
    pub jvm_preset: JvmPresetConfig,
}

fn validate_config_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);

    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err("Path traversal not allowed".to_string());
        }
    }

    if path.to_string_lossy().contains("..") {
        return Err("Path traversal not allowed".to_string());
    }

    Ok(())
}

fn validate_path_within_server(server_path: &str, file_path: &str) -> Result<(), String> {
    let canonical_server =
        std::fs::canonicalize(server_path).map_err(|e| format!("无效的服务器目录: {}", e))?;

    let fp = Path::new(file_path);
    let parent = fp.parent().unwrap_or(fp);
    let canonical_parent =
        std::fs::canonicalize(parent).map_err(|e| format!("无效的配置路径: {}", e))?;

    if !canonical_parent.starts_with(&canonical_server) {
        return Err("配置路径必须在服务器目录内".to_string());
    }

    Ok(())
}

fn build_server_properties_path(server_path: &str) -> Result<String, String> {
    validate_config_path(server_path)?;
    let props_path = format!("{}/server.properties", server_path);
    validate_path_within_server(server_path, &props_path)?;
    Ok(props_path)
}

fn trace_missing_server_properties(server_path: &str, action: &str) {
    logger::log_trace(&format!(
        "[server.config] action={} missing_server_properties path={}",
        action, server_path
    ));
}

fn is_missing_server_properties_error(error: &str) -> bool {
    error.contains("os error 2")
        || error.contains("系统找不到指定的文件")
        || error.contains("No such file or directory")
}

#[tauri::command]
pub fn read_config(server_path: String, path: String) -> Result<HashMap<String, String>, String> {
    validate_config_path(&path)?;
    validate_path_within_server(&server_path, &path)?;
    config_parser::read_properties(&path)
}

#[tauri::command]
pub fn write_config(
    server_path: String,
    path: String,
    values: HashMap<String, String>,
) -> Result<(), String> {
    validate_config_path(&path)?;
    validate_path_within_server(&server_path, &path)?;
    config_parser::write_properties(&path, &values)
}

#[tauri::command]
pub fn read_server_properties(server_path: String) -> Result<ServerProperties, String> {
    let props_path = build_server_properties_path(&server_path)?;
    match config_parser::parse_server_properties(&props_path) {
        Ok(properties) => Ok(properties),
        Err(error) if is_missing_server_properties_error(&error) => {
            trace_missing_server_properties(&server_path, "read_server_properties");
            Ok(ServerProperties { entries: Vec::new(), raw: HashMap::new() })
        }
        Err(error) => Err(error),
    }
}

#[tauri::command]
pub fn write_server_properties(
    server_path: String,
    values: HashMap<String, String>,
) -> Result<(), String> {
    let props_path = build_server_properties_path(&server_path)?;
    config_parser::write_properties(&props_path, &values)
}

#[tauri::command]
pub fn read_server_properties_source(server_path: String) -> Result<String, String> {
    let props_path = build_server_properties_path(&server_path)?;
    match config_parser::read_raw_text(&props_path) {
        Ok(source) => Ok(source),
        Err(error) if is_missing_server_properties_error(&error) => {
            trace_missing_server_properties(&server_path, "read_server_properties_source");
            Ok(String::new())
        }
        Err(error) => Err(error),
    }
}

#[tauri::command]
pub fn write_server_properties_source(server_path: String, source: String) -> Result<(), String> {
    let props_path = build_server_properties_path(&server_path)?;
    config_parser::write_raw_text(&props_path, &source)
}

#[tauri::command]
pub fn parse_server_properties_source(source: String) -> Result<ServerProperties, String> {
    config_parser::parse_server_properties_from_source(&source)
}

#[tauri::command]
pub fn preview_server_properties_write(
    server_path: String,
    values: HashMap<String, String>,
) -> Result<String, String> {
    let props_path = build_server_properties_path(&server_path)?;
    config_parser::preview_properties_write(&props_path, &values)
}

#[tauri::command]
pub fn preview_server_properties_write_from_source(
    source: String,
    values: HashMap<String, String>,
) -> Result<String, String> {
    config_parser::preview_properties_write_from_source(&source, &values)
}

/// 读取服务器目录下的 SL.json 启动配置
#[tauri::command]
pub fn read_sl_config(server_path: String) -> Result<SLStartupConfig, String> {
    validate_config_path(&server_path)?;
    let path = Path::new(&server_path).join("SL.json");
    if !path.exists() {
        return Ok(SLStartupConfig::default());
    }

    let content = std::fs::read_to_string(path).map_err(|e| format!("读取 SL.json 失败: {}", e))?;

    let config: SLStartupConfig =
        serde_json::from_str(&content).map_err(|e| format!("解析 SL.json 失败: {}", e))?;

    Ok(config)
}

/// 写入服务器目录下的 SL.json 启动配置
#[tauri::command]
pub fn write_sl_config(server_path: String, config: SLStartupConfig) -> Result<(), String> {
    validate_config_path(&server_path)?;
    let sl_path = format!("{}/SL.json", server_path);

    let content =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化 SL.json 失败: {}", e))?;

    std::fs::write(&sl_path, content).map_err(|e| format!("写入 SL.json 失败: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{read_server_properties, read_server_properties_source};
    use tempfile::tempdir;

    #[test]
    fn read_server_properties_returns_empty_when_server_properties_missing() {
        let dir = tempdir().unwrap();
        let result = read_server_properties(dir.path().to_string_lossy().to_string()).unwrap();

        assert!(result.entries.is_empty());
        assert!(result.raw.is_empty());
    }

    #[test]
    fn read_server_properties_source_returns_empty_string_when_file_missing() {
        let dir = tempdir().unwrap();
        let result =
            read_server_properties_source(dir.path().to_string_lossy().to_string()).unwrap();

        assert!(result.is_empty());
    }
}
