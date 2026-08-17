//! server.properties 读写命令，按前端 `configApi` 契约补回 #847 移除的最小子集。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub description: String,
    pub value_type: String,
    pub default_value: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProperties {
    pub entries: Vec<ConfigEntry>,
    pub raw: HashMap<String, String>,
}

/// 解析 server.properties 的绝对路径，并约束其落在服务器目录内。
fn props_path(server_path: &str) -> Result<PathBuf, String> {
    let dir = Path::new(server_path);
    let props = dir.join("server.properties");

    let canonical_dir = std::fs::canonicalize(dir).map_err(|e| format!("无效的服务器目录: {e}"))?;
    let canonical_parent = std::fs::canonicalize(props.parent().unwrap_or(dir))
        .map_err(|e| format!("无效的配置路径: {e}"))?;
    if !canonical_parent.starts_with(&canonical_dir) {
        return Err("配置路径必须在服务器目录内".to_string());
    }

    Ok(props)
}

fn parse_properties(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            continue;
        }
        if let Some(idx) = line.find('=').or_else(|| line.find(':')) {
            let key = line[..idx].trim();
            if !key.is_empty() {
                map.insert(key.to_string(), line[idx + 1..].trim_start().to_string());
            }
        }
    }
    map
}

#[tauri::command(rename_all = "snake_case")]
pub fn read_server_properties(server_path: String) -> Result<ServerProperties, String> {
    let props = props_path(&server_path)?;
    let text =
        std::fs::read_to_string(&props).map_err(|e| format!("读取 server.properties 失败: {e}"))?;
    let raw = parse_properties(&text);
    let entries = raw
        .iter()
        .map(|(k, v)| ConfigEntry {
            key: k.clone(),
            value: v.clone(),
            description: String::new(),
            value_type: String::new(),
            default_value: String::new(),
            category: String::new(),
        })
        .collect();
    Ok(ServerProperties { entries, raw })
}

/// 将给定键值合并写回 server.properties：保留原文件顺序与注释，仅更新/追加指定键。
#[tauri::command(rename_all = "snake_case")]
pub fn write_server_properties(
    server_path: String,
    values: HashMap<String, String>,
) -> Result<(), String> {
    let props = props_path(&server_path)?;
    let text =
        std::fs::read_to_string(&props).map_err(|e| format!("读取 server.properties 失败: {e}"))?;

    let mut remaining: HashSet<String> = values.keys().cloned().collect();
    let mut out: Vec<String> = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            out.push(line.to_string());
            continue;
        }
        if let Some(idx) = line.find('=').or_else(|| line.find(':')) {
            let key = line[..idx].trim();
            if let Some(value) = values.get(key) {
                out.push(format!("{key}={value}"));
                remaining.remove(key);
                continue;
            }
        }
        out.push(line.to_string());
    }

    for key in &remaining {
        out.push(format!("{key}={}", values[key]));
    }

    let mut content = out.join("\n");
    if !content.is_empty() {
        content.push('\n');
    }
    std::fs::write(&props, content).map_err(|e| format!("写入 server.properties 失败: {e}"))?;
    Ok(())
}
