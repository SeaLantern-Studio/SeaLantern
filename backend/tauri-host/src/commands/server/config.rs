use crate::utils::logger;
use sea_lantern_server_config_core::properties::{
    parse_server_properties, parse_server_properties_from_source, preview_properties_write,
    preview_properties_write_from_source, read_properties, read_raw_text, write_properties,
    write_raw_text,
};
use sea_lantern_server_config_core::startup::write_server_startup_config;
use sea_lantern_server_config_core::startup::{
    build_server_properties_path, read_server_startup_config, validate_config_path,
    validate_path_within_server,
};
use sea_lantern_server_config_core::types::{SLStartupConfig, ServerProperties};
use std::collections::HashMap;

fn trace_missing_server_properties(server_path: &str, action: &str) {
    logger::log_trace_ctx(
        "server.config",
        action,
        &format!("missing server.properties path={}", server_path),
    );
}

#[tauri::command]
pub fn read_config(server_path: String, path: String) -> Result<HashMap<String, String>, String> {
    validate_config_path(&path)?;
    validate_path_within_server(&server_path, &path)?;
    read_properties(&path)
}

#[tauri::command]
pub fn write_config(
    server_path: String,
    path: String,
    values: HashMap<String, String>,
) -> Result<(), String> {
    validate_config_path(&path)?;
    validate_path_within_server(&server_path, &path)?;
    write_properties(&path, &values)
}

#[tauri::command]
pub fn read_server_properties(server_path: String) -> Result<ServerProperties, String> {
    let props_path = build_server_properties_path(&server_path)?;
    match parse_server_properties(&props_path) {
        Ok(properties) => Ok(properties),
        Err(error) if sea_lantern_server_config_core::startup::is_missing_file_error(&error) => {
            trace_missing_server_properties(&server_path, "read_server_properties");
            Ok(sea_lantern_server_config_core::startup::empty_server_properties())
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
    write_properties(&props_path, &values)
}

#[tauri::command]
pub fn read_server_properties_source(server_path: String) -> Result<String, String> {
    let props_path = build_server_properties_path(&server_path)?;
    match read_raw_text(&props_path) {
        Ok(source) => Ok(source),
        Err(error) if sea_lantern_server_config_core::startup::is_missing_file_error(&error) => {
            trace_missing_server_properties(&server_path, "read_server_properties_source");
            Ok(String::new())
        }
        Err(error) => Err(error),
    }
}

#[tauri::command]
pub fn write_server_properties_source(server_path: String, source: String) -> Result<(), String> {
    let props_path = build_server_properties_path(&server_path)?;
    write_raw_text(&props_path, &source)
}

#[tauri::command]
pub fn parse_server_properties_source(source: String) -> Result<ServerProperties, String> {
    parse_server_properties_from_source(&source)
}

#[tauri::command]
pub fn preview_server_properties_write(
    server_path: String,
    values: HashMap<String, String>,
) -> Result<String, String> {
    let props_path = build_server_properties_path(&server_path)?;
    preview_properties_write(&props_path, &values)
}

#[tauri::command]
pub fn preview_server_properties_write_from_source(
    source: String,
    values: HashMap<String, String>,
) -> Result<String, String> {
    preview_properties_write_from_source(&source, &values)
}

/// 读取服务器目录下的实例级启动配置。
/// 优先读取 `SeaLantern/config.toml`，兼容回退到旧的 `SL.json`。
#[tauri::command]
pub fn read_sl_config(server_path: String) -> Result<SLStartupConfig, String> {
    read_server_startup_config(&server_path)
}

/// 写入服务器目录下的实例级启动配置。
/// 写回统一落到 `SeaLantern/config.toml`。
#[tauri::command]
pub fn write_sl_config(server_path: String, config: SLStartupConfig) -> Result<(), String> {
    write_server_startup_config(&server_path, &config)
}

#[cfg(test)]
mod tests {
    use super::{
        read_server_properties, read_server_properties_source, read_sl_config, write_sl_config,
    };
    use crate::models::server::{CpuPolicyConfig, CpuPolicyMode, JvmPresetConfig, JvmPresetId};
    use sea_lantern_server_config_core::types::SLStartupConfig;
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

    #[test]
    fn write_sl_config_writes_to_sealantern_config_toml() {
        let dir = tempdir().unwrap();
        let server_path = dir.path().to_string_lossy().to_string();
        let config = SLStartupConfig {
            max_memory: Some(4096),
            min_memory: Some(2048),
            jvm_args: vec!["-Dfoo=bar".to_string()],
            cpu_policy: CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(4),
                explicit_set: None,
                sync_active_processor_count: true,
            },
            jvm_preset: JvmPresetConfig { preset: JvmPresetId::AikarG1 },
        };

        write_sl_config(server_path.clone(), config.clone()).unwrap();

        let new_path = dir.path().join("SeaLantern").join("config.toml");
        assert!(new_path.exists());
        assert!(!dir.path().join("SL.json").exists());

        let reloaded = read_sl_config(server_path).unwrap();
        assert_eq!(reloaded.max_memory, Some(4096));
        assert_eq!(reloaded.min_memory, Some(2048));
        assert_eq!(reloaded.jvm_args, vec!["-Dfoo=bar"]);
        assert_eq!(reloaded.cpu_policy.mode, CpuPolicyMode::Count);
        assert_eq!(reloaded.cpu_policy.count, Some(4));
        assert_eq!(reloaded.jvm_preset.preset, JvmPresetId::AikarG1);
    }

    #[test]
    fn read_sl_config_falls_back_to_legacy_sl_json() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("SL.json"),
            r#"{
  "max_memory": 6144,
  "min_memory": 1536,
  "jvm_args": ["-XX:+UseG1GC"],
  "cpu_policy": {
    "mode": "count",
    "count": 2,
    "explicit_set": null,
    "sync_active_processor_count": true
  },
  "jvm_preset": {
    "preset": "g1_basic"
  }
}"#,
        )
        .unwrap();

        let config = read_sl_config(dir.path().to_string_lossy().to_string()).unwrap();

        assert_eq!(config.max_memory, Some(6144));
        assert_eq!(config.min_memory, Some(1536));
        assert_eq!(config.jvm_args, vec!["-XX:+UseG1GC"]);
        assert_eq!(config.cpu_policy.mode, CpuPolicyMode::Count);
        assert_eq!(config.cpu_policy.count, Some(2));
        assert_eq!(config.jvm_preset.preset, JvmPresetId::G1Basic);
    }
}
