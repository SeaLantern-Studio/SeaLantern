use std::collections::HashMap;
use std::path::{Path, PathBuf};
use toml_edit::{value, DocumentMut};

use crate::properties;
use crate::types::{
    CpuPolicyConfig, JvmPresetConfig, SLStartupConfig, ServerStartupConfigDocument,
    StartupConfigPresence,
};

const INSTANCE_CONFIG_DIR_NAME: &str = "SeaLantern";
const INSTANCE_CONFIG_FILE_NAME: &str = "config.toml";
const SERVER_PATH_PERMISSION_TEST_FILE_NAME: &str = ".sl_permission_test";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupRuntimeDefaults {
    pub max_memory: u32,
    pub min_memory: u32,
    pub jvm_args: Vec<String>,
    pub cpu_policy: CpuPolicyConfig,
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupResolutionDefaults {
    pub default_max_memory: u32,
    pub default_min_memory: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveStartupConfig {
    pub max_memory: u32,
    pub min_memory: u32,
    pub jvm_args: Vec<String>,
    pub cpu_policy: CpuPolicyConfig,
    pub jvm_preset: JvmPresetConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedJvmBuildInput {
    pub effective: EffectiveStartupConfig,
    pub java_encoding: String,
    pub default_jvm_args: Vec<String>,
    pub active_processor_count_arg: Option<String>,
}

pub fn resolve_effective_startup_config_from_document(
    startup: &ServerStartupConfigDocument,
    runtime: &StartupRuntimeDefaults,
    defaults: &StartupResolutionDefaults,
) -> EffectiveStartupConfig {
    EffectiveStartupConfig {
        max_memory: if startup.presence.max_memory {
            startup
                .config
                .max_memory
                .unwrap_or(defaults.default_max_memory)
        } else {
            runtime.max_memory
        },
        min_memory: if startup.presence.min_memory {
            startup
                .config
                .min_memory
                .unwrap_or(defaults.default_min_memory)
        } else {
            runtime.min_memory
        },
        jvm_args: if startup.presence.jvm_args {
            startup.config.jvm_args.clone()
        } else {
            runtime.jvm_args.clone()
        },
        cpu_policy: if startup.presence.cpu_policy {
            startup.config.cpu_policy.clone()
        } else {
            runtime.cpu_policy.clone()
        },
        jvm_preset: if startup.presence.jvm_preset {
            startup.config.jvm_preset.clone()
        } else {
            runtime.jvm_preset.clone()
        },
    }
}

pub fn build_managed_jvm_args_from_input(input: ManagedJvmBuildInput) -> Vec<String> {
    let ManagedJvmBuildInput {
        effective,
        java_encoding,
        default_jvm_args,
        active_processor_count_arg,
    } = input;

    let mut args = vec![
        format!("-Xmx{}M", effective.max_memory),
        format!("-Xms{}M", effective.min_memory),
        format!("-Dfile.encoding={}", java_encoding),
        format!("-Dsun.stdout.encoding={}", java_encoding),
        format!("-Dsun.stderr.encoding={}", java_encoding),
    ];

    let mut preset_args = effective
        .jvm_preset
        .preset
        .preset_args()
        .iter()
        .map(|arg| (*arg).to_string())
        .collect::<Vec<_>>();
    let user_args = effective.jvm_args;

    let user_already_set_apc = jvm_args_contain_active_processor_count(&default_jvm_args)
        || jvm_args_contain_active_processor_count(&preset_args)
        || jvm_args_contain_active_processor_count(&user_args);

    if !user_already_set_apc {
        if let Some(arg) = active_processor_count_arg {
            args.push(arg);
        }
    }

    args.append(&mut preset_args);
    args.extend(default_jvm_args);
    args.extend(user_args);
    args
}

fn jvm_args_contain_active_processor_count(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg.starts_with("-XX:ActiveProcessorCount="))
}

pub fn validate_config_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);

    for component in path.components() {
        if let std::path::Component::ParentDir = component {
            return Err("Path traversal not allowed".to_string());
        }
    }

    Ok(())
}

pub fn canonical_server_dir(server_path: &str) -> Result<PathBuf, String> {
    validate_config_path(server_path)?;
    let canonical_server =
        std::fs::canonicalize(server_path).map_err(|e| format!("无效的服务器目录: {}", e))?;
    if !canonical_server.is_dir() {
        return Err("服务器目录无效".to_string());
    }
    Ok(canonical_server)
}

pub fn build_instance_config_path(server_path: &str) -> Result<PathBuf, String> {
    let canonical_server = canonical_server_dir(server_path)?;
    let instance_config_path = canonical_server
        .join(INSTANCE_CONFIG_DIR_NAME)
        .join(INSTANCE_CONFIG_FILE_NAME);

    if !instance_config_path.starts_with(&canonical_server) {
        return Err("实例配置路径必须在服务器目录内".to_string());
    }

    Ok(instance_config_path)
}

pub fn build_legacy_sl_config_path(server_path: &str) -> Result<PathBuf, String> {
    let canonical_server = canonical_server_dir(server_path)?;
    let legacy_path = canonical_server.join("SL.json");

    if !legacy_path.starts_with(&canonical_server) {
        return Err("实例配置路径必须在服务器目录内".to_string());
    }

    Ok(legacy_path)
}

pub fn validate_path_within_server(server_path: &str, file_path: &str) -> Result<(), String> {
    let canonical_server = canonical_server_dir(server_path)?;
    let target_path = resolve_config_target_path(file_path)?;

    if !target_path.starts_with(&canonical_server) {
        return Err("配置路径必须在服务器目录内".to_string());
    }

    Ok(())
}

fn resolve_config_target_path(file_path: &str) -> Result<PathBuf, String> {
    let target = Path::new(file_path);

    match std::fs::canonicalize(target) {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent = target.parent().unwrap_or(target);
            std::fs::canonicalize(parent).map_err(|e| format!("无效的配置路径: {}", e))
        }
        Err(error) => Err(format!("无效的配置路径: {}", error)),
    }
}

pub fn build_server_properties_path(server_path: &str) -> Result<String, String> {
    validate_config_path(server_path)?;
    let props_path = format!("{}/server.properties", server_path);
    validate_path_within_server(server_path, &props_path)?;
    Ok(props_path)
}

fn read_startup_document_from_toml(path: &Path) -> Result<ServerStartupConfigDocument, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("读取实例配置失败: {}", e))?;
    let config = toml::from_str(&content).map_err(|e| format!("解析实例配置失败: {}", e))?;
    let value: toml::Value =
        toml::from_str(&content).map_err(|e| format!("解析实例配置失败: {}", e))?;
    Ok(ServerStartupConfigDocument {
        config,
        presence: startup_config_presence_from_toml(&value),
    })
}

fn read_startup_document_from_legacy_json(
    path: &Path,
) -> Result<ServerStartupConfigDocument, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("读取 SL.json 失败: {}", e))?;
    let config = serde_json::from_str(&content).map_err(|e| format!("解析 SL.json 失败: {}", e))?;
    let value: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析 SL.json 失败: {}", e))?;
    Ok(ServerStartupConfigDocument {
        config,
        presence: startup_config_presence_from_json(&value),
    })
}

fn startup_config_presence_from_toml(value: &toml::Value) -> StartupConfigPresence {
    let table = value.as_table();
    StartupConfigPresence {
        max_memory: table.and_then(|table| table.get("max_memory")).is_some(),
        min_memory: table.and_then(|table| table.get("min_memory")).is_some(),
        jvm_args: table.and_then(|table| table.get("jvm_args")).is_some(),
        cpu_policy: table.and_then(|table| table.get("cpu_policy")).is_some(),
        jvm_preset: table.and_then(|table| table.get("jvm_preset")).is_some(),
    }
}

fn startup_config_presence_from_json(value: &serde_json::Value) -> StartupConfigPresence {
    let object = value.as_object();
    StartupConfigPresence {
        max_memory: object.and_then(|object| object.get("max_memory")).is_some(),
        min_memory: object.and_then(|object| object.get("min_memory")).is_some(),
        jvm_args: object.and_then(|object| object.get("jvm_args")).is_some(),
        cpu_policy: object.and_then(|object| object.get("cpu_policy")).is_some(),
        jvm_preset: object.and_then(|object| object.get("jvm_preset")).is_some(),
    }
}

pub fn read_server_startup_config(server_path: &str) -> Result<SLStartupConfig, String> {
    Ok(read_server_startup_config_document(server_path)?.config)
}

pub fn read_server_startup_config_document(
    server_path: &str,
) -> Result<ServerStartupConfigDocument, String> {
    let instance_config_path = build_instance_config_path(server_path)?;
    if instance_config_path.exists() {
        return read_startup_document_from_toml(&instance_config_path);
    }

    let legacy_path = build_legacy_sl_config_path(server_path)?;
    if legacy_path.exists() {
        return read_startup_document_from_legacy_json(&legacy_path);
    }

    Ok(ServerStartupConfigDocument::default())
}

pub fn write_server_startup_config(
    server_path: &str,
    config: &SLStartupConfig,
) -> Result<(), String> {
    let instance_config_path = build_instance_config_path(server_path)?;
    if let Some(parent) = instance_config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建实例配置目录失败: {}", e))?;
    }

    let content =
        toml::to_string_pretty(config).map_err(|e| format!("序列化实例配置失败: {}", e))?;

    std::fs::write(&instance_config_path, content).map_err(|e| format!("写入实例配置失败: {}", e))
}

pub fn write_server_startup_config_for_dir(
    server_dir: &Path,
    max_memory: u32,
    min_memory: u32,
    jvm_args: Vec<String>,
    cpu_policy: crate::types::CpuPolicyConfig,
    jvm_preset: crate::types::JvmPresetConfig,
) -> Result<(), String> {
    let config = SLStartupConfig {
        max_memory: Some(max_memory),
        min_memory: Some(min_memory),
        jvm_args,
        cpu_policy,
        jvm_preset,
    };

    let server_dir = server_dir
        .to_str()
        .ok_or_else(|| "服务器目录路径不是有效的 UTF-8".to_string())?;

    write_server_startup_config(server_dir, &config)
}

pub fn read_server_port(server_dir: &Path, fallback: u16) -> u16 {
    match read_server_port_checked(server_dir) {
        Ok(Some(port)) => port,
        Ok(None) | Err(_) => fallback,
    }
}

pub fn read_server_port_checked(server_dir: &Path) -> Result<Option<u16>, String> {
    let pumpkin_path = server_dir.join("pumpkin.toml");
    if pumpkin_path.exists() {
        return read_pumpkin_port_checked(&pumpkin_path);
    }

    let server_properties_path = server_dir.join("server.properties");
    if !server_properties_path.exists() {
        return Ok(None);
    }

    let props = properties::read_properties(&server_properties_path.to_string_lossy())?;
    let Some(port_str) = props.get("server-port") else {
        return Ok(None);
    };

    port_str
        .parse::<u16>()
        .map(Some)
        .map_err(|e| format!("server.properties 中的 server-port 无效: {}", e))
}

fn read_pumpkin_port_checked(pumpkin_path: &Path) -> Result<Option<u16>, String> {
    let content = std::fs::read_to_string(pumpkin_path)
        .map_err(|e| format!("读取 pumpkin.toml 失败: {}", e))?;
    let document = content
        .parse::<DocumentMut>()
        .map_err(|e| format!("解析 pumpkin.toml 失败: {}", e))?;
    let Some(address) = document
        .get("java_edition_address")
        .and_then(|item| item.as_str())
    else {
        return Ok(None);
    };

    let Some(port_text) = address.rsplit(':').next() else {
        return Ok(None);
    };

    port_text
        .parse::<u16>()
        .map(Some)
        .map_err(|e| format!("pumpkin.toml 中的 java_edition_address 端口无效: {}", e))
}

pub fn update_pumpkin_config_if_present(
    server_dir: &Path,
    requested_port: u16,
    online_mode: bool,
) -> Result<bool, String> {
    let pumpkin_path = server_dir.join("pumpkin.toml");
    if !pumpkin_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&pumpkin_path)
        .map_err(|e| format!("读取 pumpkin.toml 失败: {}", e))?;
    let mut document = content
        .parse::<DocumentMut>()
        .map_err(|e| format!("解析 pumpkin.toml 失败: {}", e))?;

    document["java_edition_address"] = value(format!("0.0.0.0:{}", requested_port));
    document["online_mode"] = value(online_mode);
    if document["networking"]["query"]["address"].is_none() {
        document["networking"]["query"]["address"] = value(format!("0.0.0.0:{}", requested_port));
    } else {
        document["networking"]["query"]["address"] = value(format!("0.0.0.0:{}", requested_port));
    }

    std::fs::write(&pumpkin_path, document.to_string())
        .map_err(|e| format!("写入 pumpkin.toml 失败: {}", e))?;
    Ok(true)
}

pub fn create_server_properties_if_missing(
    server_dir: &Path,
    requested_port: u16,
    online_mode: bool,
) -> Result<(), String> {
    if update_pumpkin_config_if_present(server_dir, requested_port, online_mode)? {
        return Ok(());
    }

    let server_properties_path = server_dir.join("server.properties");
    if server_properties_path.exists() {
        return Ok(());
    }

    let content = format!(
        "# Minecraft server properties\n# Generated by SeaLantern\nserver-port={}\nonline-mode={}\n",
        requested_port, online_mode
    );
    std::fs::write(&server_properties_path, content)
        .map_err(|e| format!("创建 server.properties 失败: {}", e))?;

    Ok(())
}

pub fn ensure_server_path_writable(server_path: &Path) -> Result<(), String> {
    let test_file = server_path.join(SERVER_PATH_PERMISSION_TEST_FILE_NAME);
    if std::fs::write(&test_file, "").is_err() {
        return Err("无法写入服务器目录，请检查权限".to_string());
    }
    let _ = std::fs::remove_file(&test_file);
    Ok(())
}

pub fn is_missing_file_error(error: &str) -> bool {
    error.contains("os error 2")
        || error.contains("系统找不到指定的文件")
        || error.contains("No such file or directory")
}

pub fn empty_server_properties() -> crate::types::ServerProperties {
    crate::types::ServerProperties { entries: Vec::new(), raw: HashMap::new() }
}

#[cfg(test)]
mod tests {
    use super::{
        build_managed_jvm_args_from_input, build_server_properties_path,
        create_server_properties_if_missing, ensure_server_path_writable, read_server_port,
        read_server_port_checked, resolve_effective_startup_config_from_document,
        update_pumpkin_config_if_present, validate_config_path, write_server_startup_config_for_dir,
        EffectiveStartupConfig,
        ManagedJvmBuildInput, StartupResolutionDefaults, StartupRuntimeDefaults,
    };
    use crate::types::{
        CpuPolicyConfig, CpuPolicyMode, JvmPresetConfig, JvmPresetId, ServerStartupConfigDocument,
        StartupConfigPresence,
    };
    #[test]
    fn create_server_properties_if_missing_writes_default_content() {
        let dir = tempfile::tempdir().unwrap();

        create_server_properties_if_missing(dir.path(), 25570, true).unwrap();

        let content = std::fs::read_to_string(dir.path().join("server.properties")).unwrap();
        assert!(content.contains("server-port=25570"));
        assert!(content.contains("online-mode=true"));
    }

    #[test]
    fn create_server_properties_if_missing_updates_existing_pumpkin_config() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pumpkin.toml"),
            "java_edition_address = \"0.0.0.0:25565\"\nonline_mode = true\n[networking.query]\naddress = \"0.0.0.0:25565\"\n",
        )
        .unwrap();

        create_server_properties_if_missing(dir.path(), 25570, false).unwrap();

        let content = std::fs::read_to_string(dir.path().join("pumpkin.toml")).unwrap();
        assert!(content.contains("java_edition_address = \"0.0.0.0:25570\""));
        assert!(content.contains("online_mode = false"));
        assert!(content.contains("address = \"0.0.0.0:25570\""));
        assert!(!dir.path().join("server.properties").exists());
    }

    #[test]
    fn read_server_port_prefers_server_properties_value() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("server.properties"), "server-port=25580\n").unwrap();

        assert_eq!(read_server_port(dir.path(), 25565), 25580);
    }

    #[test]
    fn read_server_port_prefers_pumpkin_toml_when_present() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pumpkin.toml"),
            "java_edition_address = \"0.0.0.0:25581\"\n",
        )
        .unwrap();

        assert_eq!(read_server_port(dir.path(), 25565), 25581);
    }

    #[test]
    fn update_pumpkin_config_if_present_returns_false_when_missing() {
        let dir = tempfile::tempdir().unwrap();

        assert!(!update_pumpkin_config_if_present(dir.path(), 25570, true).unwrap());
    }

    #[test]
    fn read_server_port_checked_returns_none_when_server_properties_missing() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(read_server_port_checked(dir.path()).unwrap(), None);
    }

    #[test]
    fn read_server_port_checked_returns_none_when_server_port_missing() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("server.properties"), "motd=Hello\n").unwrap();

        assert_eq!(read_server_port_checked(dir.path()).unwrap(), None);
    }

    #[test]
    fn read_server_port_checked_rejects_invalid_port_value() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("server.properties"), "server-port=abc\n").unwrap();

        let error = read_server_port_checked(dir.path())
            .expect_err("invalid server-port should surface an explicit error");

        assert!(error.contains("server-port 无效"));
    }

    #[test]
    fn read_server_port_checked_propagates_read_errors() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("server.properties")).unwrap();

        let error = read_server_port_checked(dir.path())
            .expect_err("directory-backed server.properties should not be silently ignored");

        assert!(error.contains("Failed to read file:"));
    }

    #[test]
    fn read_server_port_keeps_compatibility_fallback_on_invalid_port() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("server.properties"), "server-port=abc\n").unwrap();

        assert_eq!(read_server_port(dir.path(), 25565), 25565);
    }

    #[test]
    fn ensure_server_path_writable_creates_and_cleans_probe_file() {
        let dir = tempfile::tempdir().unwrap();

        ensure_server_path_writable(dir.path()).unwrap();

        assert!(!dir.path().join(".sl_permission_test").exists());
    }

    #[test]
    fn write_server_startup_config_for_dir_writes_sealantern_config() {
        let dir = tempfile::tempdir().unwrap();

        write_server_startup_config_for_dir(
            dir.path(),
            4096,
            2048,
            vec!["-Dfoo=bar".to_string()],
            CpuPolicyConfig::default(),
            JvmPresetConfig::default(),
        )
        .unwrap();

        assert!(dir.path().join("SeaLantern").join("config.toml").exists());
    }

    #[cfg(unix)]
    #[test]
    fn write_server_startup_config_for_dir_rejects_non_utf8_server_dir() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        use std::path::PathBuf;

        let dir = tempfile::tempdir().unwrap();
        let non_utf8_dir = PathBuf::from(OsString::from_vec(vec![0x66, 0x6f, 0x80, 0x6f]));
        let path = dir.path().join(non_utf8_dir);
        std::fs::create_dir_all(&path).unwrap();

        let error = write_server_startup_config_for_dir(
            &path,
            4096,
            2048,
            vec!["-Dfoo=bar".to_string()],
            CpuPolicyConfig::default(),
            JvmPresetConfig::default(),
        )
        .expect_err("non-utf8 server path should not be silently downgraded to an empty string");

        assert!(error.contains("不是有效的 UTF-8"), "unexpected error: {}", error);
        assert!(!dir.path().join("SeaLantern").join("config.toml").exists());
    }

    #[test]
    fn resolve_effective_startup_config_prefers_instance_values_when_present() {
        let startup = ServerStartupConfigDocument {
            config: crate::types::SLStartupConfig {
                max_memory: Some(3072),
                min_memory: Some(1536),
                jvm_args: vec!["-Dinstance.flag=true".to_string()],
                cpu_policy: CpuPolicyConfig {
                    mode: CpuPolicyMode::Count,
                    count: Some(2),
                    explicit_set: None,
                    sync_active_processor_count: true,
                },
                jvm_preset: JvmPresetConfig { preset: JvmPresetId::AikarG1 },
            },
            presence: StartupConfigPresence {
                max_memory: true,
                min_memory: true,
                jvm_args: true,
                cpu_policy: true,
                jvm_preset: true,
            },
        };
        let runtime = StartupRuntimeDefaults {
            max_memory: 4096,
            min_memory: 2048,
            jvm_args: vec!["-Druntime.flag=true".to_string()],
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        };
        let defaults = StartupResolutionDefaults {
            default_max_memory: 8192,
            default_min_memory: 1024,
        };

        let effective =
            resolve_effective_startup_config_from_document(&startup, &runtime, &defaults);

        assert_eq!(
            effective,
            EffectiveStartupConfig {
                max_memory: 3072,
                min_memory: 1536,
                jvm_args: vec!["-Dinstance.flag=true".to_string()],
                cpu_policy: CpuPolicyConfig {
                    mode: CpuPolicyMode::Count,
                    count: Some(2),
                    explicit_set: None,
                    sync_active_processor_count: true,
                },
                jvm_preset: JvmPresetConfig { preset: JvmPresetId::AikarG1 },
            }
        );
    }

    #[test]
    fn resolve_effective_startup_config_falls_back_to_runtime_values() {
        let startup = ServerStartupConfigDocument::default();
        let runtime = StartupRuntimeDefaults {
            max_memory: 4096,
            min_memory: 2048,
            jvm_args: vec!["-Druntime.flag=true".to_string()],
            cpu_policy: CpuPolicyConfig {
                mode: CpuPolicyMode::Explicit,
                count: None,
                explicit_set: Some("4,6".to_string()),
                sync_active_processor_count: true,
            },
            jvm_preset: JvmPresetConfig {
                preset: JvmPresetId::PaperRecommendedLite,
            },
        };
        let defaults = StartupResolutionDefaults {
            default_max_memory: 8192,
            default_min_memory: 1024,
        };

        let effective =
            resolve_effective_startup_config_from_document(&startup, &runtime, &defaults);

        assert_eq!(effective.max_memory, 4096);
        assert_eq!(effective.min_memory, 2048);
        assert_eq!(effective.jvm_args, vec!["-Druntime.flag=true"]);
        assert_eq!(effective.cpu_policy.mode, CpuPolicyMode::Explicit);
        assert_eq!(effective.cpu_policy.explicit_set.as_deref(), Some("4,6"));
        assert_eq!(effective.jvm_preset.preset, JvmPresetId::PaperRecommendedLite);
    }

    #[test]
    fn resolve_effective_startup_config_uses_global_defaults_for_present_memory_none() {
        let startup = ServerStartupConfigDocument {
            config: crate::types::SLStartupConfig {
                max_memory: None,
                min_memory: None,
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            },
            presence: StartupConfigPresence {
                max_memory: true,
                min_memory: true,
                jvm_args: false,
                cpu_policy: false,
                jvm_preset: false,
            },
        };
        let runtime = StartupRuntimeDefaults {
            max_memory: 4096,
            min_memory: 2048,
            jvm_args: vec!["-Druntime.flag=true".to_string()],
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        };
        let defaults = StartupResolutionDefaults {
            default_max_memory: 6144,
            default_min_memory: 1536,
        };

        let effective =
            resolve_effective_startup_config_from_document(&startup, &runtime, &defaults);

        assert_eq!(effective.max_memory, 6144);
        assert_eq!(effective.min_memory, 1536);
    }

    #[test]
    fn build_managed_jvm_args_from_input_preserves_expected_order() {
        let args = build_managed_jvm_args_from_input(ManagedJvmBuildInput {
            effective: EffectiveStartupConfig {
                max_memory: 3072,
                min_memory: 1536,
                jvm_args: vec!["-Dserver.flag=true".to_string()],
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            },
            java_encoding: "UTF-8".to_string(),
            default_jvm_args: vec!["-Dglobal.flag=true".to_string(), "-XX:+UseG1GC".to_string()],
            active_processor_count_arg: None,
        });

        assert_eq!(
            args,
            vec![
                "-Xmx3072M".to_string(),
                "-Xms1536M".to_string(),
                "-Dfile.encoding=UTF-8".to_string(),
                "-Dsun.stdout.encoding=UTF-8".to_string(),
                "-Dsun.stderr.encoding=UTF-8".to_string(),
                "-Dglobal.flag=true".to_string(),
                "-XX:+UseG1GC".to_string(),
                "-Dserver.flag=true".to_string(),
            ]
        );
    }

    #[test]
    fn build_managed_jvm_args_from_input_injects_active_processor_count_only_when_missing() {
        let injected = build_managed_jvm_args_from_input(ManagedJvmBuildInput {
            effective: EffectiveStartupConfig {
                max_memory: 4096,
                min_memory: 2048,
                jvm_args: vec!["-Dserver.flag=true".to_string()],
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            },
            java_encoding: "UTF-8".to_string(),
            default_jvm_args: vec!["-Dglobal.flag=true".to_string()],
            active_processor_count_arg: Some("-XX:ActiveProcessorCount=2".to_string()),
        });
        assert!(injected
            .iter()
            .any(|arg| arg == "-XX:ActiveProcessorCount=2"));

        let skipped = build_managed_jvm_args_from_input(ManagedJvmBuildInput {
            effective: EffectiveStartupConfig {
                max_memory: 4096,
                min_memory: 2048,
                jvm_args: vec!["-XX:ActiveProcessorCount=6".to_string()],
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            },
            java_encoding: "UTF-8".to_string(),
            default_jvm_args: vec!["-Dglobal.flag=true".to_string()],
            active_processor_count_arg: Some("-XX:ActiveProcessorCount=2".to_string()),
        });
        assert_eq!(
            skipped
                .iter()
                .filter(|arg| arg.starts_with("-XX:ActiveProcessorCount="))
                .count(),
            1
        );
        assert!(skipped
            .iter()
            .any(|arg| arg == "-XX:ActiveProcessorCount=6"));
    }

    #[test]
    fn validate_config_path_allows_double_dots_in_normal_path_segments() {
        validate_config_path("plugins/example..backup/config.yml")
            .expect("normal path segments containing double dots should be allowed");
    }

    #[test]
    fn validate_config_path_rejects_parent_dir_components() {
        let error = validate_config_path("plugins/../config.yml")
            .expect_err("parent-dir traversal should be rejected");

        assert!(error.contains("Path traversal not allowed"));
    }

    #[test]
    fn build_server_properties_path_accepts_server_directory_name_with_double_dots() {
        let temp_dir = tempfile::tempdir().unwrap();
        let server_dir = temp_dir.path().join("server..prod");
        std::fs::create_dir_all(&server_dir).unwrap();

        let props_path = build_server_properties_path(server_dir.to_string_lossy().as_ref())
            .expect("server dir names containing double dots should be accepted");

        assert!(props_path
            .replace('\\', "/")
            .ends_with("server..prod/server.properties"));
    }
}
