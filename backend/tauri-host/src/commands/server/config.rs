use crate::utils::logger;
use server_config::discovery::{
    discover_server_config_files, discover_server_config_files_with_options,
    resolve_discovered_config_path, resolve_discovered_config_path_with_options,
    search_server_config_files as search_server_config_files_core,
    search_server_config_files_with_options,
};
use server_config::properties::{
    parse_server_properties, parse_server_properties_from_source, preview_properties_write,
    preview_properties_write_from_source, read_properties, read_raw_text, write_properties,
    write_raw_text,
};
use server_config::startup::write_server_startup_config;
use server_config::startup::{
    build_server_properties_path, read_server_startup_config, validate_config_path,
    validate_path_within_server,
};
use server_config::types::{
    DiscoveredServerConfigFile, SLStartupConfig, ServerConfigDiscoveryOptions,
    ServerConfigDocument, ServerConfigFileKind, ServerConfigSearchHit, ServerConfigSearchMode,
    ServerConfigSearchScope, ServerProperties,
};
use std::collections::HashMap;

fn map_permission_error(error: String, path: &str, action: &str) -> String {
    if error.contains("Permission denied")
        || error.contains("os error 13")
        || error.contains("拒绝访问")
    {
        return format!(
            "{} 失败: 权限不足 path={}. 在 Linux/macOS 上这通常表示当前用户没有该文件或目录的读写权限；如果这是受系统权限保护的目录，请让用户检查属主/权限，必要时提权后再运行。原始错误: {}",
            action, path, error
        );
    }
    error
}

fn read_document_by_kind(
    path: &str,
    kind: &ServerConfigFileKind,
) -> Result<serde_json::Value, String> {
    let source =
        read_raw_text(path).map_err(|error| map_permission_error(error, path, "读取配置文件"))?;
    match kind {
        ServerConfigFileKind::Properties => {
            let parsed = read_properties(path)
                .map_err(|error| map_permission_error(error, path, "解析 properties 配置"))?;
            serde_json::to_value(parsed).map_err(|e| e.to_string())
        }
        ServerConfigFileKind::Toml => {
            let parsed: toml::Value =
                toml::from_str(&source).map_err(|e| format!("解析 TOML 配置失败: {}", e))?;
            serde_json::to_value(parsed).map_err(|e| e.to_string())
        }
        ServerConfigFileKind::Yaml => {
            let parsed: serde_yaml::Value =
                serde_yaml::from_str(&source).map_err(|e| format!("解析 YAML 配置失败: {}", e))?;
            serde_json::to_value(parsed).map_err(|e| e.to_string())
        }
        ServerConfigFileKind::Json => {
            serde_json::from_str(&source).map_err(|e| format!("解析 JSON 配置失败: {}", e))
        }
    }
}

fn write_document_by_kind(
    path: &str,
    kind: &ServerConfigFileKind,
    content: serde_json::Value,
) -> Result<(), String> {
    let source = match kind {
        ServerConfigFileKind::Properties => {
            let map = serde_json::from_value::<HashMap<String, String>>(content)
                .map_err(|e| format!("properties 配置内容必须是字符串键值表: {}", e))?;
            preview_properties_write(path, &map)
                .map_err(|error| map_permission_error(error, path, "预览 properties 配置写入"))?
        }
        ServerConfigFileKind::Toml => {
            toml::to_string_pretty(&content).map_err(|e| format!("序列化 TOML 配置失败: {}", e))?
        }
        ServerConfigFileKind::Yaml => {
            serde_yaml::to_string(&content).map_err(|e| format!("序列化 YAML 配置失败: {}", e))?
        }
        ServerConfigFileKind::Json => serde_json::to_string_pretty(&content)
            .map_err(|e| format!("序列化 JSON 配置失败: {}", e))?,
    };

    write_raw_text(path, &source).map_err(|error| map_permission_error(error, path, "写入配置文件"))
}

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
pub fn read_server_config_source(
    server_path: String,
    relative_path: String,
    locator: Option<String>,
    discovery_options: Option<ServerConfigDiscoveryOptions>,
) -> Result<String, String> {
    let path = match (locator.as_deref(), discovery_options.as_ref()) {
        (Some(locator), Some(options)) => resolve_discovered_config_path_with_options(
            &server_path,
            &relative_path,
            Some(locator),
            options,
        )?,
        (Some(locator), None) => resolve_discovered_config_path_with_options(
            &server_path,
            &relative_path,
            Some(locator),
            &ServerConfigDiscoveryOptions::default(),
        )?,
        (None, Some(options)) => resolve_discovered_config_path_with_options(
            &server_path,
            &relative_path,
            None,
            options,
        )?,
        (None, None) => resolve_discovered_config_path(&server_path, &relative_path)?,
    };
    read_raw_text(&path.to_string_lossy()).map_err(|error| {
        map_permission_error(error, path.to_string_lossy().as_ref(), "读取配置文件源码")
    })
}

#[tauri::command]
pub fn write_server_config_source(
    server_path: String,
    relative_path: String,
    locator: Option<String>,
    discovery_options: Option<ServerConfigDiscoveryOptions>,
    source: String,
) -> Result<(), String> {
    let path = match (locator.as_deref(), discovery_options.as_ref()) {
        (Some(locator), Some(options)) => resolve_discovered_config_path_with_options(
            &server_path,
            &relative_path,
            Some(locator),
            options,
        )?,
        (Some(locator), None) => resolve_discovered_config_path_with_options(
            &server_path,
            &relative_path,
            Some(locator),
            &ServerConfigDiscoveryOptions::default(),
        )?,
        (None, Some(options)) => resolve_discovered_config_path_with_options(
            &server_path,
            &relative_path,
            None,
            options,
        )?,
        (None, None) => resolve_discovered_config_path(&server_path, &relative_path)?,
    };
    write_raw_text(&path.to_string_lossy(), &source).map_err(|error| {
        map_permission_error(error, path.to_string_lossy().as_ref(), "写入配置文件源码")
    })
}

#[tauri::command]
pub fn read_server_config_document(
    server_path: String,
    relative_path: String,
    locator: Option<String>,
    discovery_options: Option<ServerConfigDiscoveryOptions>,
) -> Result<ServerConfigDocument, String> {
    let discovered = match discovery_options.as_ref() {
        Some(options) => discover_server_config_files_with_options(&server_path, options)?,
        None => discover_server_config_files(&server_path)?,
    };
    let entry = discovered
        .into_iter()
        .find(|entry| match locator.as_deref() {
            Some(locator) => entry.locator == locator,
            None => entry.relative_path == relative_path,
        })
        .ok_or_else(|| format!("未找到配置文件: {}", relative_path))?;
    let path = match discovery_options.as_ref() {
        Some(options) => resolve_discovered_config_path_with_options(
            &server_path,
            &entry.relative_path,
            Some(&entry.locator),
            options,
        )?,
        None => resolve_discovered_config_path_with_options(
            &server_path,
            &entry.relative_path,
            Some(&entry.locator),
            &ServerConfigDiscoveryOptions::default(),
        )?,
    };
    let content = read_document_by_kind(path.to_string_lossy().as_ref(), &entry.kind)?;

    Ok(ServerConfigDocument {
        relative_path: entry.relative_path,
        kind: entry.kind,
        content,
    })
}

#[tauri::command]
pub fn write_server_config_document(
    server_path: String,
    relative_path: String,
    locator: Option<String>,
    discovery_options: Option<ServerConfigDiscoveryOptions>,
    content: serde_json::Value,
) -> Result<(), String> {
    let discovered = match discovery_options.as_ref() {
        Some(options) => discover_server_config_files_with_options(&server_path, options)?,
        None => discover_server_config_files(&server_path)?,
    };
    let entry = discovered
        .into_iter()
        .find(|entry| match locator.as_deref() {
            Some(locator) => entry.locator == locator,
            None => entry.relative_path == relative_path,
        })
        .ok_or_else(|| format!("未找到配置文件: {}", relative_path))?;
    let path = match discovery_options.as_ref() {
        Some(options) => resolve_discovered_config_path_with_options(
            &server_path,
            &entry.relative_path,
            Some(&entry.locator),
            options,
        )?,
        None => resolve_discovered_config_path_with_options(
            &server_path,
            &entry.relative_path,
            Some(&entry.locator),
            &ServerConfigDiscoveryOptions::default(),
        )?,
    };
    write_document_by_kind(path.to_string_lossy().as_ref(), &entry.kind, content)
}

#[tauri::command]
pub fn list_server_config_files(
    server_path: String,
    discovery_options: Option<ServerConfigDiscoveryOptions>,
) -> Result<Vec<DiscoveredServerConfigFile>, String> {
    match discovery_options.as_ref() {
        Some(options) => discover_server_config_files_with_options(&server_path, options),
        None => discover_server_config_files(&server_path),
    }
}

#[tauri::command]
pub fn search_server_config_files(
    server_path: String,
    query: String,
    mode: ServerConfigSearchMode,
    scope: Option<ServerConfigSearchScope>,
    discovery_options: Option<ServerConfigDiscoveryOptions>,
    limit: Option<usize>,
    case_sensitive: Option<bool>,
) -> Result<Vec<ServerConfigSearchHit>, String> {
    match discovery_options.as_ref() {
        Some(options) => search_server_config_files_with_options(
            &server_path,
            options,
            &query,
            mode,
            scope.unwrap_or(ServerConfigSearchScope::Path),
            limit,
            case_sensitive.unwrap_or(false),
        ),
        None => search_server_config_files_core(
            &server_path,
            &query,
            mode,
            scope.unwrap_or(ServerConfigSearchScope::Path),
            limit,
            case_sensitive.unwrap_or(false),
        ),
    }
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
        Err(error) if server_config::startup::is_missing_file_error(&error) => {
            trace_missing_server_properties(&server_path, "read_server_properties");
            Ok(server_config::startup::empty_server_properties())
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
        Err(error) if server_config::startup::is_missing_file_error(&error) => {
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
        list_server_config_files, read_server_config_document, read_server_config_source,
        read_server_properties, read_server_properties_source, read_sl_config,
        search_server_config_files, write_server_config_document, write_server_config_source,
        write_sl_config,
    };
    use crate::models::server::{CpuPolicyConfig, CpuPolicyMode, JvmPresetConfig, JvmPresetId};
    use server_config::types::{SLStartupConfig, ServerConfigSearchMode, ServerConfigSearchScope};
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
    fn list_server_config_files_discovers_supported_files() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("SeaLantern")).unwrap();
        std::fs::create_dir_all(dir.path().join("config")).unwrap();
        std::fs::write(dir.path().join("server.properties"), "server-port=25565\n").unwrap();
        std::fs::write(dir.path().join("SeaLantern").join("config.toml"), "max_memory = 2048\n")
            .unwrap();
        std::fs::write(dir.path().join("config").join("paper.yml"), "motd: test\n").unwrap();

        let files =
            list_server_config_files(dir.path().to_string_lossy().to_string(), None).unwrap();

        assert!(files
            .iter()
            .any(|file| file.relative_path == "server.properties"));
        assert!(files
            .iter()
            .any(|file| file.relative_path == "SeaLantern/config.toml"));
        assert!(files
            .iter()
            .any(|file| file.relative_path == "config/paper.yml"));
    }

    #[test]
    fn read_server_config_source_reads_discovered_yaml() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("config")).unwrap();
        std::fs::write(dir.path().join("config").join("paper.yml"), "motd: test\n").unwrap();

        let source = read_server_config_source(
            dir.path().to_string_lossy().to_string(),
            "config/paper.yml".to_string(),
            None,
            None,
        )
        .unwrap();

        assert_eq!(source, "motd: test\n");
    }

    #[test]
    fn read_server_config_document_parses_toml_to_json_value() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("SeaLantern")).unwrap();
        std::fs::write(dir.path().join("SeaLantern").join("config.toml"), "max_memory = 2048\n")
            .unwrap();

        let document = read_server_config_document(
            dir.path().to_string_lossy().to_string(),
            "SeaLantern/config.toml".to_string(),
            None,
            None,
        )
        .unwrap();

        assert_eq!(
            document
                .content
                .get("max_memory")
                .and_then(serde_json::Value::as_i64),
            Some(2048)
        );
    }

    #[test]
    fn write_server_config_document_writes_yaml_from_json_value() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("config")).unwrap();
        std::fs::write(dir.path().join("config").join("paper.yml"), "motd: test\n").unwrap();

        write_server_config_document(
            dir.path().to_string_lossy().to_string(),
            "config/paper.yml".to_string(),
            None,
            None,
            serde_json::json!({ "motd": "updated", "pvp": true }),
        )
        .unwrap();

        let written = std::fs::read_to_string(dir.path().join("config").join("paper.yml")).unwrap();
        assert!(written.contains("motd: updated"));
        assert!(written.contains("pvp: true"));
    }

    #[test]
    fn write_server_config_source_writes_discovered_file() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("server.properties"), "motd=old\n").unwrap();

        write_server_config_source(
            dir.path().to_string_lossy().to_string(),
            "server.properties".to_string(),
            None,
            None,
            "motd=new\n".to_string(),
        )
        .unwrap();

        let written = std::fs::read_to_string(dir.path().join("server.properties")).unwrap();
        assert_eq!(written, "motd=new\n");
    }

    #[test]
    fn search_server_config_files_returns_hits_for_third_party_configs() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("config").join("mods")).unwrap();
        std::fs::write(dir.path().join("config").join("mods").join("jei.toml"), "enabled=true\n")
            .unwrap();

        let hits = search_server_config_files(
            dir.path().to_string_lossy().to_string(),
            "jei".to_string(),
            ServerConfigSearchMode::Keyword,
            Some(ServerConfigSearchScope::Path),
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(hits[0].relative_path, "config/mods/jei.toml");
        assert_eq!(hits[0].reason, "file_name_contains");
    }

    #[test]
    fn search_server_config_files_returns_content_hits() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("config")).unwrap();
        std::fs::write(
            dir.path().join("config").join("paper-global.yml"),
            "motd: hello world\nview-distance: 12\n",
        )
        .unwrap();

        let hits = search_server_config_files(
            dir.path().to_string_lossy().to_string(),
            "hello world".to_string(),
            ServerConfigSearchMode::Keyword,
            Some(ServerConfigSearchScope::Content),
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(hits[0].relative_path, "config/paper-global.yml");
        assert_eq!(hits[0].reason, "content_line_contains");
        assert_eq!(hits[0].content_match.as_ref().map(|item| item.line_number), Some(1));
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
