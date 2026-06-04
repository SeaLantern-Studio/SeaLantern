use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::server::{
    CpuPolicyConfig, JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
};
use crate::utils::constants::{DATA_FILE, RUN_PATH_MAP_FILE};
use crate::utils::logger;
use crate::utils::path::find_root_startup_file;
use serde::{Deserialize, Serialize};

use super::common::detect_startup_mode_from_path;

fn default_startup_mode() -> String {
    "jar".to_string()
}

#[derive(Debug, Clone, Deserialize)]
struct LegacyServerInstance {
    id: String,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    core_type: String,
    #[serde(default)]
    core_version: String,
    mc_version: String,
    path: String,
    jar_path: String,
    #[serde(default = "default_startup_mode")]
    startup_mode: String,
    #[serde(default)]
    custom_command: Option<String>,
    java_path: String,
    #[serde(default)]
    jvm_args: Vec<String>,
    port: u16,
    created_at: u64,
    #[serde(default)]
    last_started_at: Option<u64>,
    #[serde(default)]
    max_memory: Option<u32>,
    #[serde(default)]
    min_memory: Option<u32>,
}

impl From<LegacyServerInstance> for ServerInstance {
    fn from(value: LegacyServerInstance) -> Self {
        let _ = (value.max_memory, value.min_memory);
        ServerInstance {
            id: value.id,
            name: value.name,
            aliases: value.aliases,
            core_type: value.core_type,
            core_version: value.core_version,
            mc_version: value.mc_version,
            path: value.path,
            port: value.port,
            max_memory: value.max_memory.unwrap_or(2048),
            min_memory: value.min_memory.unwrap_or(512),
            created_at: value.created_at,
            last_started_at: value.last_started_at,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: value.jar_path,
                startup_mode: value.startup_mode,
                custom_command: value.custom_command,
                java_path: value.java_path,
                jvm_args: value.jvm_args,
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }
}

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

    if let Some(path) = find_root_startup_file(server_path) {
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
    logger::log_trace(&format!(
        "[server.manager.fs] action=load_servers_begin dir={} file={}",
        dir,
        path.display()
    ));
    if !path.exists() {
        logger::log_trace(&format!(
            "[server.manager.fs] action=load_servers_missing file={}",
            path.display()
        ));
        return Vec::new();
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            logger::log_warn(&format!(
                "[server.manager.fs] action=load_servers_read_failed file={} error={}",
                path.display(),
                err
            ));
            return Vec::new();
        }
    };

    let raw_servers = match serde_json::from_str::<Vec<serde_json::Value>>(&content) {
        Ok(servers) => servers,
        Err(err) => {
            logger::log_warn(&format!(
                "[server.manager.fs] action=load_servers_parse_failed file={} error={}",
                path.display(),
                err
            ));
            return Vec::new();
        }
    };

    let total = raw_servers.len();
    let mut loaded = Vec::with_capacity(total);
    for (index, server) in raw_servers.into_iter().enumerate() {
        let has_runtime_fields = server
            .as_object()
            .map(|value| value.contains_key("runtime") || value.contains_key("runtime_kind"))
            .unwrap_or(false);

        let loaded_server = if has_runtime_fields {
            match serde_json::from_value::<ServerInstance>(server) {
                Ok(server) => Some(server),
                Err(err) => {
                    logger::log_warn(&format!(
                        "[server.manager.fs] action=load_servers_entry_failed index={} shape=new error={}",
                        index,
                        err
                    ));
                    None
                }
            }
        } else {
            match serde_json::from_value::<LegacyServerInstance>(server) {
                Ok(server) => Some(ServerInstance::from(server)),
                Err(err) => {
                    logger::log_warn(&format!(
                        "[server.manager.fs] action=load_servers_entry_failed index={} shape=legacy error={}",
                        index,
                        err
                    ));
                    None
                }
            }
        };

        if let Some(server) = loaded_server {
            loaded.push(server);
        }
    }

    logger::log_trace(&format!(
        "[server.manager.fs] action=load_servers_end file={} total={} loaded={}",
        path.display(),
        total,
        loaded.len()
    ));

    loaded
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

#[cfg(test)]
mod tests {
    use super::{load_servers, save_servers};
    use crate::models::server::{
        CpuPolicyConfig, JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    };
    use crate::utils::constants::DATA_FILE;
    use serde_json::Value;
    use tempfile::tempdir;

    fn sample_server() -> ServerInstance {
        ServerInstance {
            id: "server-1".to_string(),
            name: "Test Server".to_string(),
            aliases: vec!["cache_server".to_string(), "test_server".to_string()],
            core_type: "paper".to_string(),
            core_version: "1.0.0".to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/servers/test".to_string(),
            port: 25565,
            max_memory: 2048,
            min_memory: 1024,
            created_at: 1,
            last_started_at: Some(2),
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "E:/servers/test/server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: vec!["-Xmx2G".to_string()],
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn load_servers_upgrades_legacy_instances_to_local_runtime() {
        let dir = tempdir().unwrap();
        let data_path = dir.path().join(DATA_FILE);
        std::fs::write(
            &data_path,
            r#"[
  {
    "id": "legacy-1",
    "name": "Legacy Server",
    "core_type": "paper",
    "core_version": "",
    "mc_version": "1.20.6",
    "path": "E:/legacy/server",
    "jar_path": "E:/legacy/server/server.jar",
    "startup_mode": "jar",
    "custom_command": null,
    "java_path": "C:/Java/bin/java.exe",
    "jvm_args": ["-Xmx2G"],
    "port": 25565,
    "created_at": 100,
    "last_started_at": 200
  }
]"#,
        )
        .unwrap();

        let servers = load_servers(dir.path().to_str().unwrap());
        assert_eq!(servers.len(), 1);

        let server = &servers[0];
        assert_eq!(server.runtime_kind, "local");

        let runtime = server
            .local_runtime()
            .expect("legacy server should upgrade to local runtime");
        assert_eq!(runtime.jar_path, "E:/legacy/server/server.jar");
        assert_eq!(runtime.startup_mode, "jar");
        assert_eq!(runtime.java_path, "C:/Java/bin/java.exe");
        assert_eq!(runtime.jvm_args, vec!["-Xmx2G"]);
    }

    #[test]
    fn save_servers_writes_new_runtime_shape_and_round_trips() {
        let dir = tempdir().unwrap();
        let original = vec![sample_server()];

        save_servers(dir.path().to_str().unwrap(), &original);

        let data_path = dir.path().join(DATA_FILE);
        let saved = std::fs::read_to_string(&data_path).unwrap();
        let saved_json: Value = serde_json::from_str(&saved).unwrap();

        assert_eq!(saved_json[0]["runtime_kind"], "local");
        assert_eq!(saved_json[0]["aliases"][0], "cache_server");
        assert_eq!(saved_json[0]["runtime"]["kind"], "local");
        assert_eq!(saved_json[0]["runtime"]["jar_path"], "E:/servers/test/server.jar");
        assert!(saved_json[0].get("jar_path").is_none());
        assert!(saved_json[0].get("java_path").is_none());

        let reloaded = load_servers(dir.path().to_str().unwrap());
        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded[0].runtime_kind, "local");
        assert_eq!(reloaded[0].aliases, vec!["cache_server", "test_server"]);
        assert_eq!(reloaded[0].jar_path(), Some("E:/servers/test/server.jar"));
        assert_eq!(reloaded[0].java_path(), Some("C:/Java/bin/java.exe"));
    }

    #[test]
    fn load_servers_supports_mixed_legacy_and_new_runtime_records() {
        let dir = tempdir().unwrap();
        let data_path = dir.path().join(DATA_FILE);
        std::fs::write(
            &data_path,
            r#"[
  {
    "id": "legacy-1",
    "name": "Legacy Server",
    "core_type": "paper",
    "core_version": "",
    "mc_version": "1.20.6",
    "path": "E:/legacy/server",
    "jar_path": "E:/legacy/server/server.jar",
    "startup_mode": "jar",
    "custom_command": null,
    "java_path": "C:/Java/bin/java.exe",
    "jvm_args": ["-Xmx2G"],
    "port": 25565,
    "created_at": 100,
    "last_started_at": 200
  },
  {
    "id": "new-1",
    "name": "New Server",
    "core_type": "paper",
    "core_version": "",
    "mc_version": "1.21.1",
    "path": "E:/new/server",
    "port": 25566,
    "created_at": 300,
    "last_started_at": null,
    "runtime_kind": "local",
    "runtime": {
      "kind": "local",
      "jar_path": "E:/new/server/server.jar",
      "startup_mode": "custom",
      "custom_command": "java -jar server.jar nogui",
      "java_path": "C:/Java21/bin/java.exe",
      "jvm_args": ["-Xmx4G"]
    }
  }
]"#,
        )
        .unwrap();

        let servers = load_servers(dir.path().to_str().unwrap());
        assert_eq!(servers.len(), 2);

        let legacy = servers
            .iter()
            .find(|server| server.id == "legacy-1")
            .unwrap();
        assert_eq!(legacy.runtime_kind, "local");
        assert_eq!(legacy.jar_path(), Some("E:/legacy/server/server.jar"));
        assert_eq!(legacy.java_path(), Some("C:/Java/bin/java.exe"));

        let new_server = servers.iter().find(|server| server.id == "new-1").unwrap();
        assert_eq!(new_server.runtime_kind, "local");
        assert_eq!(new_server.jar_path(), Some("E:/new/server/server.jar"));
        assert_eq!(new_server.startup_mode_str(), "custom");
        assert_eq!(new_server.custom_command(), Some("java -jar server.jar nogui"));
        assert_eq!(new_server.java_path(), Some("C:/Java21/bin/java.exe"));
        assert_eq!(new_server.jvm_args(), ["-Xmx4G".to_string()]);
    }

    #[test]
    fn load_servers_prefers_new_shape_when_runtime_kind_exists_without_runtime() {
        let dir = tempdir().unwrap();
        let data_path = dir.path().join(DATA_FILE);
        std::fs::write(
            &data_path,
            r#"[
  {
    "id": "defaulted-new",
    "name": "Defaulted New",
    "core_type": "paper",
    "core_version": "",
    "mc_version": "1.21.1",
    "path": "E:/defaulted/new",
    "port": 25565,
    "created_at": 1,
    "last_started_at": null,
    "runtime_kind": "local"
  }
]"#,
        )
        .unwrap();

        let servers = load_servers(dir.path().to_str().unwrap());
        assert_eq!(servers.len(), 1);

        let server = &servers[0];
        assert_eq!(server.runtime_kind, "local");
        let runtime = server
            .local_runtime()
            .expect("runtime default should deserialize for new shape");
        assert_eq!(runtime.startup_mode, "jar");
        assert_eq!(runtime.jar_path, "");
        assert_eq!(runtime.java_path, "");
        assert!(runtime.jvm_args.is_empty());
    }
}
