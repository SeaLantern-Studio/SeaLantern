use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::server::{
    CpuPolicyConfig, JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
};
use crate::utils::constants::{DATA_FILE, RUN_PATH_MAP_FILE};
use crate::utils::logger;
use sea_lantern_server_local_setup_core::{
    canonical_core_type, paths_equal, resolve_local_startup_entry_checked,
};
use serde::{Deserialize, Serialize};

use super::i18n::{manager_t, manager_t1};

fn log_fs_trace(function: &str, message: &str) {
    logger::log_trace_ctx("server.manager.fs", function, message);
}

fn log_fs_warn(function: &str, message: &str) {
    logger::log_warn_ctx("server.manager.fs", function, message);
}

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
            core_type: canonical_core_type(&value.core_type),
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

fn normalize_loaded_server_instance(mut server: ServerInstance) -> ServerInstance {
    server.core_type = canonical_core_type(&server.core_type);
    server
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

pub(super) fn find_server_executable(server_path: &Path) -> Result<(String, String), String> {
    if let Some((path, mode)) = resolve_local_startup_entry_checked(server_path)? {
        return Ok((path, mode));
    }

    Err(manager_t("server.manager.startup_executable_not_found"))
}

pub(super) fn load_run_path_mappings_checked(
    dir: &str,
) -> Result<Vec<RunPathServerMapping>, String> {
    let path = Path::new(dir).join(RUN_PATH_MAP_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| manager_t1("server.manager.run_path_map_read_failed", e.to_string()))?;
    serde_json::from_str::<Vec<RunPathServerMapping>>(&content)
        .map_err(|e| manager_t1("server.manager.run_path_map_parse_failed", e.to_string()))
}

#[allow(dead_code)]
pub(super) fn load_run_path_mappings(dir: &str) -> Vec<RunPathServerMapping> {
    load_run_path_mappings_checked(dir).unwrap_or_default()
}

pub(super) fn save_run_path_mappings(
    dir: &str,
    mappings: &[RunPathServerMapping],
) -> Result<(), String> {
    let path = Path::new(dir).join(RUN_PATH_MAP_FILE);
    let json = serde_json::to_string_pretty(mappings)
        .map_err(|e| manager_t1("server.manager.run_path_map_serialize_failed", e.to_string()))?;
    std::fs::write(path, json)
        .map_err(|e| manager_t1("server.manager.run_path_map_write_failed", e.to_string()))
}

pub(super) fn upsert_run_path_mapping(
    dir: &str,
    mapping: RunPathServerMapping,
) -> Result<(), String> {
    let mut mappings = load_run_path_mappings_checked(dir)?;
    mappings
        .retain(|item| item.server_id != mapping.server_id && item.run_path != mapping.run_path);
    mappings.push(mapping);
    save_run_path_mappings(dir, &mappings)
}

pub(super) fn update_run_path_mapping(
    dir: &str,
    server_id: &str,
    new_path: &str,
) -> Result<(), String> {
    let mut mappings = load_run_path_mappings_checked(dir)?;
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
        save_run_path_mappings(dir, &mappings)?;
    }

    Ok(())
}

pub(super) fn remove_run_path_mapping(dir: &str, server_id: &str) -> Result<(), String> {
    let mut mappings = load_run_path_mappings_checked(dir)?;
    let before = mappings.len();
    mappings.retain(|item| item.server_id != server_id);
    if mappings.len() == before {
        return Ok(());
    }

    save_run_path_mappings(dir, &mappings)
}

#[allow(dead_code)]
pub(super) fn load_servers(dir: &str) -> Vec<ServerInstance> {
    load_servers_checked(dir).unwrap_or_default()
}

pub(super) fn load_servers_checked(dir: &str) -> Result<Vec<ServerInstance>, String> {
    let path = Path::new(dir).join(DATA_FILE);
    log_fs_trace("load_servers_checked", &format!("begin dir={} file={}", dir, path.display()));
    if !path.exists() {
        log_fs_trace("load_servers_checked", &format!("missing file={}", path.display()));
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&path).map_err(|err| {
        log_fs_warn(
            "load_servers_checked",
            &format!("read failed file={} error={}", path.display(), err),
        );
        manager_t1("server.manager.server_list_read_failed", err.to_string())
    })?;

    let raw_servers = parse_server_registry_entries(&path, &content)?;

    let total = raw_servers.len();
    let mut loaded = Vec::with_capacity(total);
    for (index, server) in raw_servers.into_iter().enumerate() {
        let has_runtime_fields = server
            .as_object()
            .map(|value| value.contains_key("runtime") || value.contains_key("runtime_kind"))
            .unwrap_or(false);

        let loaded_server = if has_runtime_fields {
            match serde_json::from_value::<ServerInstance>(server) {
                Ok(server) => Some(normalize_loaded_server_instance(server)),
                Err(err) => {
                    log_fs_warn(
                        "load_servers_checked",
                        &format!("entry failed index={} shape=new error={}", index, err),
                    );
                    None
                }
            }
        } else {
            match serde_json::from_value::<LegacyServerInstance>(server) {
                Ok(server) => Some(normalize_loaded_server_instance(ServerInstance::from(server))),
                Err(err) => {
                    log_fs_warn(
                        "load_servers_checked",
                        &format!("entry failed index={} shape=legacy error={}", index, err),
                    );
                    None
                }
            }
        };

        if let Some(server) = loaded_server {
            loaded.push(server);
        }
    }

    log_fs_trace(
        "load_servers_checked",
        &format!("end file={} total={} loaded={}", path.display(), total, loaded.len()),
    );

    Ok(loaded)
}

fn parse_server_registry_entries(
    path: &Path,
    content: &str,
) -> Result<Vec<serde_json::Value>, String> {
    match serde_json::from_str::<Vec<serde_json::Value>>(content) {
        Ok(entries) => Ok(entries),
        Err(array_error) => {
            let fallback = serde_json::from_str::<serde_json::Value>(content)
                .ok()
                .and_then(extract_server_registry_entries_from_value);

            if let Some(entries) = fallback {
                log_fs_warn(
                    "parse_server_registry_entries",
                    &format!(
                        "compat parse used file={} detail=wrapped_registry extracted_entries={}",
                        path.display(),
                        entries.len()
                    ),
                );
                Ok(entries)
            } else {
                log_fs_warn(
                    "parse_server_registry_entries",
                    &format!("parse failed file={} error={}", path.display(), array_error),
                );
                Err(manager_t1("server.manager.server_list_parse_failed", array_error.to_string()))
            }
        }
    }
}

fn extract_server_registry_entries_from_value(
    value: serde_json::Value,
) -> Option<Vec<serde_json::Value>> {
    match value {
        serde_json::Value::Array(entries) => Some(entries),
        serde_json::Value::Object(mut object) => match object.remove("servers") {
            Some(serde_json::Value::Array(entries)) => Some(entries),
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn load_servers_for_bootstrap(dir: &str) -> Result<Vec<ServerInstance>, String> {
    match load_servers_checked(dir) {
        Ok(servers) => Ok(servers),
        Err(error) if is_corrupt_server_registry_error(&error) => {
            log_fs_warn(
                "load_servers_for_bootstrap",
                &format!("recovering corrupt registry dir={} error={}", dir, error),
            );
            backup_corrupt_server_registry_file(Path::new(dir).join(DATA_FILE).as_path());
            save_servers(dir, &[])?;
            Ok(Vec::new())
        }
        Err(error) => Err(error),
    }
}

fn is_corrupt_server_registry_error(error: &str) -> bool {
    error.contains("server.manager.server_list_parse_failed")
        || error.contains("解析服务器列表失败")
}

fn backup_corrupt_server_registry_file(path: &Path) {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let backup_path = path.with_extension(format!("json.bak-corrupt-{}", timestamp));

    match std::fs::copy(path, &backup_path) {
        Ok(_) => log_fs_warn(
            "backup_corrupt_server_registry_file",
            &format!(
                "backed up corrupt registry file={} backup={}",
                path.display(),
                backup_path.display()
            ),
        ),
        Err(error) => log_fs_warn(
            "backup_corrupt_server_registry_file",
            &format!("failed to back up corrupt registry file={} error={}", path.display(), error),
        ),
    }
}

pub(super) fn save_servers(dir: &str, servers: &[ServerInstance]) -> Result<(), String> {
    let path = Path::new(dir).join(DATA_FILE);
    let json = serde_json::to_string_pretty(servers)
        .map_err(|e| manager_t1("server.manager.server_list_serialize_failed", e.to_string()))?;
    std::fs::write(&path, json)
        .map_err(|e| manager_t1("server.manager.server_list_write_failed", e.to_string()))
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
    use super::{
        find_server_executable, load_run_path_mappings_checked, load_servers, load_servers_checked,
        load_servers_for_bootstrap, remove_run_path_mapping, save_run_path_mappings, save_servers,
        update_run_path_mapping, RunPathServerMapping,
    };
    use crate::models::server::{
        CpuPolicyConfig, JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    };
    use crate::utils::constants::{DATA_FILE, RUN_PATH_MAP_FILE};
    use serde_json::Value;
    use tempfile::tempdir;

    fn restore_writable_permissions(path: &std::path::Path) {
        let mut permissions = std::fs::metadata(path)
            .expect("metadata should exist")
            .permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            permissions.set_mode(0o644);
        }
        #[cfg(windows)]
        {
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
        }
        std::fs::set_permissions(path, permissions).expect("path should become writable again");
    }

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
    fn load_servers_canonicalizes_legacy_core_type_aliases() {
        let dir = tempdir().unwrap();
        let data_path = dir.path().join(DATA_FILE);
        std::fs::write(
            &data_path,
            r#"[
  {
    "id": "legacy-paper",
    "name": "Legacy Paper",
    "core_type": "Paper",
    "core_version": "",
    "mc_version": "1.20.6",
    "path": "E:/legacy/paper",
    "jar_path": "E:/legacy/paper/server.jar",
    "startup_mode": "jar",
    "custom_command": null,
    "java_path": "C:/Java/bin/java.exe",
    "jvm_args": [],
    "port": 25565,
    "created_at": 100,
    "last_started_at": 200
  },
  {
    "id": "legacy-bedrock",
    "name": "Legacy Bedrock",
    "core_type": "bedrock",
    "core_version": "",
    "mc_version": "latest",
    "path": "E:/legacy/bedrock",
    "jar_path": "E:/legacy/bedrock/bedrock_server.exe",
    "startup_mode": "custom",
    "custom_command": "./bedrock_server.exe",
    "java_path": "",
    "jvm_args": [],
    "port": 19132,
    "created_at": 101,
    "last_started_at": null
  }
]"#,
        )
        .unwrap();

        let servers =
            load_servers_checked(dir.path().to_str().unwrap()).expect("legacy records should load");

        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].core_type, "paper");
        assert_eq!(servers[1].core_type, "bds");
    }

    #[test]
    fn save_servers_writes_new_runtime_shape_and_round_trips() {
        let dir = tempdir().unwrap();
        let original = vec![sample_server()];

        save_servers(dir.path().to_str().unwrap(), &original).expect("server list should save");

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
        assert_eq!(new_server.local_runtime().unwrap().jvm_args, ["-Xmx4G".to_string()]);
    }

    #[test]
    fn load_servers_canonicalizes_new_runtime_core_type_aliases() {
        let dir = tempdir().unwrap();
        let data_path = dir.path().join(DATA_FILE);
        std::fs::write(
            &data_path,
            r#"[
  {
    "id": "new-bungee",
    "name": "New Bungee",
    "core_type": "BungeeCord",
    "core_version": "",
    "mc_version": "1.21.1",
    "path": "E:/new/bungee",
    "port": 25577,
    "created_at": 300,
    "last_started_at": null,
    "runtime_kind": "local",
    "runtime": {
      "kind": "local",
      "jar_path": "E:/new/bungee/proxy.jar",
      "startup_mode": "jar",
      "custom_command": null,
      "java_path": "C:/Java21/bin/java.exe",
      "jvm_args": []
    }
  }
]"#,
        )
        .unwrap();

        let servers = load_servers_checked(dir.path().to_str().unwrap())
            .expect("new runtime records should load");

        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].core_type, "bungeecord");
    }

    #[test]
    fn find_server_executable_prefers_root_script_over_jar() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("start.sh"), "#!/bin/sh\n").unwrap();
        std::fs::write(dir.path().join("server.jar"), "jar").unwrap();

        let (path, mode) =
            find_server_executable(dir.path()).expect("startup file should be found");

        assert!(path.ends_with("start.sh"));
        assert_eq!(mode, "sh");
    }

    #[test]
    fn find_server_executable_falls_back_to_detected_jar() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("fabric-server-launch.jar"), "jar").unwrap();

        let (path, mode) = find_server_executable(dir.path()).expect("jar should be found");

        assert!(path.ends_with("fabric-server-launch.jar"));
        assert_eq!(mode, "jar");
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

    #[test]
    fn load_servers_checked_surfaces_top_level_parse_failures() {
        let dir = tempdir().unwrap();
        let data_path = dir.path().join(DATA_FILE);
        std::fs::write(&data_path, "{").expect("broken server registry should be written");

        let error = load_servers_checked(dir.path().to_str().unwrap())
            .expect_err("invalid registry should not be silently downgraded to an empty list");

        assert!(error.contains("解析服务器列表失败"), "unexpected error: {}", error);
    }

    #[test]
    fn load_servers_for_bootstrap_recovers_top_level_parse_failures() {
        let dir = tempdir().unwrap();
        let data_path = dir.path().join(DATA_FILE);
        std::fs::write(&data_path, "{").expect("broken server registry should be written");

        let servers = load_servers_for_bootstrap(dir.path().to_str().unwrap())
            .expect("bootstrap load should recover corrupted registry");

        assert!(servers.is_empty(), "recovered registry should start empty");

        let backup_count = std::fs::read_dir(dir.path())
            .expect("read temp dir")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains(&format!("{}.bak-corrupt-", DATA_FILE))
            })
            .count();

        assert_eq!(backup_count, 1);
    }

    #[test]
    fn load_servers_checked_accepts_wrapped_registry_and_skips_bad_entries() {
        let dir = tempdir().unwrap();
        let data_path = dir.path().join(DATA_FILE);
        std::fs::write(
            &data_path,
            format!(
                r#"{{
  "servers": [
    {},
    {{"id":"broken-entry"}}
  ]
}}"#,
                serde_json::to_string(&sample_server()).unwrap()
            ),
        )
        .expect("wrapped registry should be written");

        let servers = load_servers_checked(dir.path().to_str().unwrap())
            .expect("wrapped registry should be accepted");

        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id, sample_server().id);
    }

    #[test]
    fn save_servers_surfaces_write_failures() {
        let dir = tempdir().unwrap();
        let blocked_root = dir.path().join("blocked-root");
        std::fs::write(&blocked_root, b"not a directory").expect("blocked root should write");

        let error = save_servers(blocked_root.to_string_lossy().as_ref(), &[sample_server()])
            .expect_err("save failure should not be silently downgraded");

        assert!(error.contains("写入服务器列表失败"), "unexpected error: {}", error);
    }

    #[test]
    fn load_run_path_mappings_checked_returns_empty_for_missing_file() {
        let dir = tempdir().unwrap();

        let mappings = load_run_path_mappings_checked(dir.path().to_str().unwrap())
            .expect("missing mapping file should be treated as empty");

        assert!(mappings.is_empty());
    }

    #[test]
    fn load_run_path_mappings_checked_surfaces_parse_failures() {
        let dir = tempdir().unwrap();
        let mapping_path = dir.path().join(RUN_PATH_MAP_FILE);
        std::fs::write(&mapping_path, "{not json}").expect("invalid mapping should write");

        let error = load_run_path_mappings_checked(dir.path().to_str().unwrap())
            .expect_err("invalid mapping json should not be silently downgraded");

        assert!(error.contains("解析运行路径映射失败"), "unexpected error: {}", error);
    }

    #[test]
    fn update_run_path_mapping_surfaces_write_failures() {
        let dir = tempdir().unwrap();
        let real_dir = dir.path().join("real-dir");
        std::fs::create_dir_all(&real_dir).expect("real dir should create");
        save_run_path_mappings(
            real_dir.to_string_lossy().as_ref(),
            &[RunPathServerMapping {
                run_path: "E:/servers/original".to_string(),
                server_id: "server-1".to_string(),
                server_name: "Test Server".to_string(),
                startup_mode: "jar".to_string(),
                startup_file_path: None,
                custom_command: None,
                source_modpack_path: String::new(),
                updated_at: 1,
            }],
        )
        .expect("mapping should save");

        let mapping_path = real_dir.join(RUN_PATH_MAP_FILE);
        let mut permissions = std::fs::metadata(&mapping_path)
            .expect("mapping metadata should exist")
            .permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&mapping_path, permissions)
            .expect("mapping should become readonly");

        let error = update_run_path_mapping(
            real_dir.to_string_lossy().as_ref(),
            "server-1",
            "E:/servers/updated",
        )
        .expect_err("mapping save failure should not be silently downgraded");

        assert!(error.contains("写入运行路径映射失败"), "unexpected error: {}", error);
        restore_writable_permissions(&mapping_path);
    }

    #[test]
    fn remove_run_path_mapping_surfaces_write_failures() {
        let dir = tempdir().unwrap();
        let real_dir = dir.path().join("real-dir");
        std::fs::create_dir_all(&real_dir).expect("real dir should create");
        save_run_path_mappings(
            real_dir.to_string_lossy().as_ref(),
            &[RunPathServerMapping {
                run_path: "E:/servers/original".to_string(),
                server_id: "server-1".to_string(),
                server_name: "Test Server".to_string(),
                startup_mode: "jar".to_string(),
                startup_file_path: None,
                custom_command: None,
                source_modpack_path: String::new(),
                updated_at: 1,
            }],
        )
        .expect("mapping should save");

        let mapping_path = real_dir.join(RUN_PATH_MAP_FILE);
        let mut permissions = std::fs::metadata(&mapping_path)
            .expect("mapping metadata should exist")
            .permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&mapping_path, permissions)
            .expect("mapping should become readonly");

        let error = remove_run_path_mapping(real_dir.to_string_lossy().as_ref(), "server-1")
            .expect_err("mapping remove save failure should not be silently downgraded");

        assert!(error.contains("写入运行路径映射失败"), "unexpected error: {}", error);
        restore_writable_permissions(&mapping_path);
    }
}
