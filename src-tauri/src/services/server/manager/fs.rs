use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::server::ServerInstance;
use crate::utils::constants::{DATA_FILE, RUN_PATH_MAP_FILE};
use serde::{Deserialize, Serialize};

use super::common::detect_startup_mode_from_path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct RunPathServerMapping {
    pub(super) run_path: String,
    pub(super) server_id: String,
    pub(super) server_name: String,
    pub(super) startup_mode: String,
    pub(super) startup_file_path: Option<String>,
    pub(super) custom_command: Option<String>,
    pub(super) source_modpack_path: String,
    pub(super) updated_at: u64,
}

pub(super) fn normalize_path_for_compare(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string()
}

pub(super) fn paths_equal(left: &Path, right: &Path) -> bool {
    normalize_path_for_compare(left) == normalize_path_for_compare(right)
}

pub(super) fn normalize_absolute_path_for_compare(path: &Path) -> Option<String> {
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };

    let mut normalized = PathBuf::new();
    for component in absolute_path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }

    let normalized = normalize_path_for_compare(&normalized);

    #[cfg(target_os = "windows")]
    {
        Some(normalized.to_ascii_lowercase())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Some(normalized)
    }
}

pub(super) fn path_is_child_of(candidate: &Path, parent: &Path) -> bool {
    let Some(candidate_norm) = normalize_absolute_path_for_compare(candidate) else {
        return false;
    };
    let Some(parent_norm) = normalize_absolute_path_for_compare(parent) else {
        return false;
    };

    candidate_norm.starts_with(&(parent_norm + "/"))
}

pub(super) fn find_server_executable(server_path: &Path) -> Result<(String, String), String> {
    let preferred_scripts = [
        "start.bat",
        "run.bat",
        "launch.bat",
        "start.sh",
        "run.sh",
        "launch.sh",
        "start.ps1",
        "run.ps1",
        "launch.ps1",
    ];

    for script in preferred_scripts {
        let script_path = server_path.join(script);
        if script_path.exists() {
            let mode = detect_startup_mode_from_path(&script_path);
            return Ok((script_path.to_string_lossy().to_string(), mode));
        }
    }

    if let Ok(jar_path) = crate::services::server::installer::find_server_jar(server_path) {
        return Ok((jar_path, "jar".to_string()));
    }

    let entries =
        std::fs::read_dir(server_path).map_err(|e| format!("无法读取服务器目录: {}", e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .unwrap_or_default();
        if extension != "jar" && extension != "bat" && extension != "sh" && extension != "ps1" {
            continue;
        }

        let mode = detect_startup_mode_from_path(&path);
        return Ok((path.to_string_lossy().to_string(), mode));
    }

    Err("未找到可用的启动文件（.jar/.bat/.sh/.ps1）".to_string())
}

pub(super) fn resolve_startup_file_path(
    source_path: &Path,
    run_dir: &Path,
    startup_file_path: &str,
) -> Result<String, String> {
    let startup_path = PathBuf::from(startup_file_path);
    if startup_path.is_relative() {
        return Ok(run_dir.join(&startup_path).to_string_lossy().to_string());
    }

    if source_path.is_file() {
        let source_file_name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("server.jar");
        return Ok(run_dir.join(source_file_name).to_string_lossy().to_string());
    }

    if source_path.is_dir() {
        let source_norm = normalize_path_for_compare(source_path);
        let startup_norm = normalize_path_for_compare(&startup_path);
        if startup_norm.starts_with(&(source_norm.clone() + "/")) {
            if let Ok(relative) = startup_path.strip_prefix(source_path) {
                return Ok(run_dir.join(relative).to_string_lossy().to_string());
            }
        }
    }

    Err(format!("无法安全映射启动文件路径，请重新扫描后重试: {}", startup_file_path))
}

pub(super) fn load_run_path_mappings(dir: &str) -> Vec<RunPathServerMapping> {
    let path = Path::new(dir).join(RUN_PATH_MAP_FILE);
    if !path.exists() {
        return Vec::new();
    }

    std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str::<Vec<RunPathServerMapping>>(&content).ok())
        .unwrap_or_default()
}

pub(super) fn save_run_path_mappings(
    dir: &str,
    mappings: &[RunPathServerMapping],
) -> Result<(), String> {
    let path = Path::new(dir).join(RUN_PATH_MAP_FILE);
    let json = serde_json::to_string_pretty(mappings)
        .map_err(|e| format!("序列化运行路径映射失败: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("写入运行路径映射失败: {}", e))
}

pub(super) fn upsert_run_path_mapping(
    dir: &str,
    mapping: RunPathServerMapping,
) -> Result<(), String> {
    let mut mappings = load_run_path_mappings(dir);
    mappings
        .retain(|item| item.server_id != mapping.server_id && item.run_path != mapping.run_path);
    mappings.push(mapping);
    save_run_path_mappings(dir, &mappings)
}

pub(super) fn update_run_path_mapping(dir: &str, server_id: &str, new_path: &str) {
    let mut mappings = load_run_path_mappings(dir);
    let mut found = false;

    for mapping in mappings.iter_mut() {
        if mapping.server_id == server_id {
            mapping.run_path = new_path.to_string();
            mapping.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            found = true;
            break;
        }
    }

    if found {
        let _ = save_run_path_mappings(dir, &mappings);
    }
}

pub(super) fn remove_run_path_mapping(dir: &str, server_id: &str) {
    let mut mappings = load_run_path_mappings(dir);
    let before = mappings.len();
    mappings.retain(|item| item.server_id != server_id);
    if mappings.len() == before {
        return;
    }

    let _ = save_run_path_mappings(dir, &mappings);
}

pub(super) fn load_servers(dir: &str) -> Vec<ServerInstance> {
    let path = Path::new(dir).join(DATA_FILE);
    if !path.exists() {
        return Vec::new();
    }

    std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub(super) fn save_servers(dir: &str, servers: &[ServerInstance]) {
    let path = Path::new(dir).join(DATA_FILE);
    if let Ok(json) = serde_json::to_string_pretty(servers) {
        let _ = std::fs::write(&path, json);
    }
}

pub(super) fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            if paths_equal(&src_path, dst) {
                continue;
            }
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
