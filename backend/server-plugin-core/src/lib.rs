//! Shared server-extension and plugin directory helpers.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use sea_lantern_server_installer_core::CoreType;
use serde::{Deserialize, Serialize};
use server_flavor_core::{resolve_profile_from_parts, ServerExtensionKind};
use zip::ZipArchive;

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone)]
/// Serializable plugin summary returned by server plugin management APIs.
pub struct m_PluginInfo {
    pub m_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub file_name: String,
    pub file_size: u64,
    pub enabled: bool,
    pub main_class: String,
    pub has_config_folder: bool,
    pub config_files: Vec<m_PluginConfigFile>,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone)]
/// Serializable plugin config file payload returned by config inspection APIs.
pub struct m_PluginConfigFile {
    pub file_name: String,
    pub content: String,
    pub file_type: String,
    pub file_path: String,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone)]
struct m_PluginConfig {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    author: Option<String>,
    authors: Option<Vec<String>>,
    main: Option<String>,
}

enum PluginJarParseError {
    MissingDescriptor,
    InvalidJar(String),
}

fn parse_plugin_jar(jar_path: &Path) -> Result<m_PluginConfig, PluginJarParseError> {
    let file = fs::File::open(jar_path).map_err(|e| {
        PluginJarParseError::InvalidJar(format!("Failed to open plugin jar: {}", e))
    })?;
    let mut zip = ZipArchive::new(file).map_err(|e| {
        PluginJarParseError::InvalidJar(format!("Failed to read plugin jar: {}", e))
    })?;

    for index in 0..zip.len() {
        let mut file = zip.by_index(index).map_err(|e| {
            PluginJarParseError::InvalidJar(format!("Failed to read zip entry: {}", e))
        })?;

        let entry_name = file.name().to_ascii_lowercase();
        if entry_name == "plugin.yml" || entry_name == "bungee.yml" {
            let mut content = String::new();
            file.read_to_string(&mut content).map_err(|e| {
                PluginJarParseError::InvalidJar(format!("Failed to read config file: {}", e))
            })?;

            let config: m_PluginConfig = serde_yaml::from_str(&content).map_err(|e| {
                PluginJarParseError::InvalidJar(format!("Failed to parse config file: {}", e))
            })?;
            return Ok(config);
        }
    }

    Err(PluginJarParseError::MissingDescriptor)
}

fn fallback_plugin_info(path: &Path, base_file_name: String, enabled: bool) -> m_PluginInfo {
    let fallback_plugin_name = plugin_name_from_file_name(&base_file_name);

    m_PluginInfo {
        m_id: format!("{}-unknown", fallback_plugin_name),
        name: fallback_plugin_name,
        version: "Unknown".to_string(),
        description: "No description".to_string(),
        author: "Unknown".to_string(),
        file_name: base_file_name,
        file_size: path.metadata().map(|meta| meta.len()).unwrap_or(0),
        enabled,
        main_class: "Unknown".to_string(),
        has_config_folder: false,
        config_files: Vec::new(),
    }
}

fn plugin_name_from_file_name(file_name: &str) -> String {
    file_name
        .strip_suffix(".jar")
        .or_else(|| file_name.strip_suffix(".JAR"))
        .unwrap_or(file_name)
        .to_string()
}

/// Resolves the relative extension directory for a server flavor and extension kind.
pub fn resolve_extension_relative_dir(
    core_type: &str,
    runtime_kind: &str,
    startup_mode: &str,
    kind: ServerExtensionKind,
) -> Option<&'static str> {
    let core_key = CoreType::normalize_to_api_core_key(core_type);
    let profile = resolve_profile_from_parts(
        core_key.as_deref(),
        Some(runtime_kind),
        Some(startup_mode),
        None,
        false,
    );
    if !profile.supports_extension_kind(kind) {
        return None;
    }

    match kind {
        ServerExtensionKind::Plugin | ServerExtensionKind::McdrPlugin => Some("plugins"),
        ServerExtensionKind::Mod => Some("mods"),
        ServerExtensionKind::Datapack => Some("world/datapacks"),
        ServerExtensionKind::Addon => Some("behavior_packs"),
    }
}

/// Resolves the absolute extension directory path for a server.
pub fn resolve_extension_target_dir(
    server_path: &str,
    core_type: &str,
    runtime_kind: &str,
    startup_mode: &str,
    kind: ServerExtensionKind,
) -> Option<PathBuf> {
    let relative_dir = resolve_extension_relative_dir(core_type, runtime_kind, startup_mode, kind)?;
    Some(Path::new(server_path).join(relative_dir))
}

/// Resolves and creates the extension directory when the selected flavor supports it.
pub fn ensure_extension_target_dir(
    server_path: &str,
    core_type: &str,
    runtime_kind: &str,
    startup_mode: &str,
    kind: ServerExtensionKind,
) -> Result<PathBuf, String> {
    let target_dir =
        resolve_extension_target_dir(server_path, core_type, runtime_kind, startup_mode, kind)
            .ok_or_else(|| {
                format!("Extension kind '{:?}' is not supported for core '{}'.", kind, core_type)
            })?;

    if !target_dir.exists() {
        fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create extension directory: {}", e))?;
    }

    Ok(target_dir)
}

/// Lists plugins from the default `plugins` directory with fallback metadata for unreadable jars.
pub fn get_plugins(server_path: &str) -> Result<Vec<m_PluginInfo>, String> {
    get_plugins_in_dir(server_path, "plugins")
}

/// Lists plugins from the default `plugins` directory and surfaces invalid jars as errors.
pub fn get_plugins_checked(server_path: &str) -> Result<Vec<m_PluginInfo>, String> {
    get_plugins_checked_in_dir(server_path, "plugins")
}

/// Lists plugins from a caller-selected extension directory with fallback metadata.
pub fn get_plugins_in_dir(
    server_path: &str,
    relative_dir: &str,
) -> Result<Vec<m_PluginInfo>, String> {
    collect_plugins(server_path, relative_dir, false)
}

/// Lists plugins from a caller-selected extension directory and surfaces invalid jars as errors.
pub fn get_plugins_checked_in_dir(
    server_path: &str,
    relative_dir: &str,
) -> Result<Vec<m_PluginInfo>, String> {
    collect_plugins(server_path, relative_dir, true)
}

fn collect_plugins(
    server_path: &str,
    relative_dir: &str,
    strict_jar_parse: bool,
) -> Result<Vec<m_PluginInfo>, String> {
    let plugins_dir = Path::new(server_path).join(relative_dir);
    let mut plugins = Vec::new();

    let entries = fs::read_dir(&plugins_dir)
        .map_err(|e| format!("Failed to read plugins directory: {}", e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let file_name_lower = file_name.to_ascii_lowercase();
        if !file_name_lower.ends_with(".jar") && !file_name_lower.ends_with(".jar.disabled") {
            continue;
        }

        let enabled = !file_name_lower.ends_with(".jar.disabled");
        let base_file_name = if enabled {
            file_name.clone()
        } else {
            file_name[..file_name.len() - ".disabled".len()].to_string()
        };
        let fallback_plugin_name = plugin_name_from_file_name(&base_file_name);

        match parse_plugin_jar(&path) {
            Ok(plugin_config) => {
                let plugin_name = plugin_config
                    .name
                    .clone()
                    .unwrap_or_else(|| fallback_plugin_name.clone());
                let config_folder_path = plugins_dir.join(&plugin_name);
                let has_config_folder = config_folder_path.exists();
                let file_size = path.metadata().map(|meta| meta.len()).unwrap_or(0);
                let author = resolve_author(&plugin_config);

                plugins.push(m_PluginInfo {
                    m_id: format!(
                        "{}-{}",
                        plugin_name,
                        plugin_config.version.as_deref().unwrap_or("unknown")
                    ),
                    name: plugin_name,
                    version: plugin_config
                        .version
                        .unwrap_or_else(|| "Unknown".to_string()),
                    description: plugin_config
                        .description
                        .unwrap_or_else(|| "No description".to_string()),
                    author,
                    file_name: base_file_name,
                    file_size,
                    enabled,
                    main_class: plugin_config.main.unwrap_or_else(|| "Unknown".to_string()),
                    has_config_folder,
                    config_files: Vec::new(),
                });
            }
            Err(PluginJarParseError::MissingDescriptor) => {
                plugins.push(fallback_plugin_info(&path, base_file_name, enabled));
            }
            Err(PluginJarParseError::InvalidJar(error)) => {
                if strict_jar_parse {
                    return Err(format!(
                        "Failed to inspect plugin jar {}: {}",
                        path.display(),
                        error
                    ));
                }
                plugins.push(fallback_plugin_info(&path, base_file_name, enabled));
            }
        }
    }

    Ok(plugins)
}

/// Reads config files for one plugin from the default `plugins` directory.
pub fn get_plugin_config_files(
    server_path: &str,
    plugin_name: &str,
) -> Result<Vec<m_PluginConfigFile>, String> {
    get_plugin_config_files_in_dir(server_path, "plugins", plugin_name)
}

/// Reads config files for one plugin from a caller-selected extension directory.
pub fn get_plugin_config_files_in_dir(
    server_path: &str,
    relative_dir: &str,
    plugin_name: &str,
) -> Result<Vec<m_PluginConfigFile>, String> {
    let config_folder_path = Path::new(server_path).join(relative_dir).join(plugin_name);

    if config_folder_path.exists() {
        scan_plugin_config_files(&config_folder_path)
    } else {
        Ok(Vec::new())
    }
}

fn resolve_author(plugin_config: &m_PluginConfig) -> String {
    if let Some(author) = &plugin_config.author {
        author.clone()
    } else if let Some(authors) = &plugin_config.authors {
        authors.first().unwrap_or(&"Unknown".to_string()).clone()
    } else {
        "Unknown".to_string()
    }
}

fn scan_plugin_config_files(plugin_dir: &Path) -> Result<Vec<m_PluginConfigFile>, String> {
    let mut config_files = Vec::new();

    if !plugin_dir.exists() {
        return Ok(config_files);
    }

    let entries = fs::read_dir(plugin_dir).map_err(|e| {
        format!("Failed to read plugin config directory {}: {}", plugin_dir.display(), e)
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            format!("Failed to read plugin config entry in {}: {}", plugin_dir.display(), e)
        })?;
        let path = entry.path();
        let file_type = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();

        if path.is_file() {
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            if is_supported_config_extension(&file_type) {
                let content = fs::read_to_string(&path).map_err(|e| {
                    format!("Failed to read plugin config file {}: {}", path.display(), e)
                })?;
                config_files.push(m_PluginConfigFile {
                    file_name,
                    content,
                    file_type,
                    file_path: path.to_string_lossy().to_string(),
                });
            }
        } else if path.is_dir() {
            if is_supported_config_extension(&file_type) {
                return Err(format!(
                    "Expected plugin config file but found directory {}",
                    path.display()
                ));
            }
            config_files.extend(scan_plugin_config_files(&path)?);
        }
    }

    Ok(config_files)
}

fn is_supported_config_extension(file_type: &str) -> bool {
    ["yml", "yaml", "json", "properties"].contains(&file_type)
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_extension_target_dir, get_plugin_config_files, get_plugin_config_files_in_dir,
        get_plugins, get_plugins_checked, get_plugins_checked_in_dir, m_PluginConfig,
        resolve_author, resolve_extension_relative_dir, resolve_extension_target_dir,
    };
    use server_flavor_core::ServerExtensionKind;
    use std::fs;
    use std::io::Write;
    use zip::write::FileOptions;

    fn write_plugin_jar(path: &std::path::Path, yaml_body: &str) {
        let file = fs::File::create(path).expect("plugin jar should create");
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("plugin.yml", FileOptions::<()>::default())
            .expect("plugin.yml entry should start");
        zip.write_all(yaml_body.as_bytes())
            .expect("plugin.yml should write");
        zip.finish().expect("plugin jar should finish");
    }

    fn write_plugin_jar_with_entry(path: &std::path::Path, entry_name: &str, yaml_body: &str) {
        let file = fs::File::create(path).expect("plugin jar should create");
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file(entry_name, FileOptions::<()>::default())
            .expect("plugin config entry should start");
        zip.write_all(yaml_body.as_bytes())
            .expect("plugin config should write");
        zip.finish().expect("plugin jar should finish");
    }

    #[test]
    fn resolve_author_prefers_primary_author_then_authors_list() {
        let with_author = m_PluginConfig {
            name: None,
            version: None,
            description: None,
            author: Some("Solo".to_string()),
            authors: Some(vec!["Group".to_string()]),
            main: None,
        };
        let with_authors = m_PluginConfig {
            name: None,
            version: None,
            description: None,
            author: None,
            authors: Some(vec!["First".to_string(), "Second".to_string()]),
            main: None,
        };
        let unknown = m_PluginConfig {
            name: None,
            version: None,
            description: None,
            author: None,
            authors: None,
            main: None,
        };

        assert_eq!(resolve_author(&with_author), "Solo");
        assert_eq!(resolve_author(&with_authors), "First");
        assert_eq!(resolve_author(&unknown), "Unknown");
    }

    #[test]
    fn get_plugin_config_files_filters_supported_extensions_recursively() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let server_path = dir.path();
        let plugin_dir = server_path.join("plugins").join("ExamplePlugin");
        let nested_dir = plugin_dir.join("nested");
        fs::create_dir_all(&nested_dir).expect("plugin config dir should exist");

        fs::write(plugin_dir.join("config.yml"), "enabled: true\n")
            .expect("yaml config should write");
        fs::write(nested_dir.join("extra.json"), "{\"level\":2}")
            .expect("json config should write");
        fs::write(plugin_dir.join("readme.txt"), "ignored").expect("unsupported file should write");

        let files =
            get_plugin_config_files(server_path.to_string_lossy().as_ref(), "ExamplePlugin")
                .expect("config scan should succeed");

        assert_eq!(files.len(), 2);
        assert!(files
            .iter()
            .any(|file| file.file_name == "config.yml" && file.file_type == "yml"));
        assert!(files
            .iter()
            .any(|file| file.file_name == "extra.json" && file.file_type == "json"));
        assert!(files.iter().all(|file| !file.file_name.ends_with(".txt")));
    }

    #[test]
    fn get_plugins_checked_in_dir_reads_non_default_extension_directory() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let extensions_dir = dir.path().join("behavior_packs");
        fs::create_dir_all(&extensions_dir).expect("extension dir should exist");

        write_plugin_jar(
            &extensions_dir.join("BedrockBridge.jar"),
            "name: BedrockBridge\nversion: 1.0.0\nmain: bedrock.Bridge\n",
        );

        let plugins =
            get_plugins_checked_in_dir(dir.path().to_string_lossy().as_ref(), "behavior_packs")
                .expect("plugin listing should succeed in custom extension dir");

        assert!(plugins.iter().any(|plugin| plugin.name == "BedrockBridge"));
    }

    #[test]
    fn resolve_extension_relative_dir_follows_shared_flavor_contracts() {
        assert_eq!(
            resolve_extension_relative_dir("Paper", "local", "jar", ServerExtensionKind::Plugin),
            Some("plugins")
        );
        assert_eq!(
            resolve_extension_relative_dir("Fabric", "local", "jar", ServerExtensionKind::Mod),
            Some("mods")
        );
        assert_eq!(
            resolve_extension_relative_dir(
                "bedrock",
                "local",
                "custom",
                ServerExtensionKind::Addon
            ),
            Some("behavior_packs")
        );
        assert_eq!(
            resolve_extension_relative_dir("Paper", "local", "jar", ServerExtensionKind::Mod),
            None
        );
    }

    #[test]
    fn resolve_extension_target_dir_builds_expected_path() {
        let target = resolve_extension_target_dir(
            "E:/servers/fabric-main",
            "Fabric",
            "local",
            "jar",
            ServerExtensionKind::Mod,
        )
        .expect("fabric mod dir should resolve");

        assert_eq!(target.to_string_lossy().replace('\\', "/"), "E:/servers/fabric-main/mods");
    }

    #[test]
    fn ensure_extension_target_dir_creates_missing_directory() {
        let dir = tempfile::tempdir().expect("temp dir should exist");

        let target = ensure_extension_target_dir(
            dir.path().to_string_lossy().as_ref(),
            "bedrock",
            "local",
            "custom",
            ServerExtensionKind::Addon,
        )
        .expect("bedrock addon dir should be created");

        assert!(target.exists());
        assert_eq!(target.file_name().and_then(|name| name.to_str()), Some("behavior_packs"));
    }

    #[test]
    fn get_plugin_config_files_in_dir_reads_non_default_extension_directory() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let plugin_dir = dir.path().join("behavior_packs").join("ExamplePlugin");
        fs::create_dir_all(&plugin_dir).expect("plugin config dir should exist");
        fs::write(plugin_dir.join("config.yml"), "enabled: true\n")
            .expect("yaml config should write");

        let files = get_plugin_config_files_in_dir(
            dir.path().to_string_lossy().as_ref(),
            "behavior_packs",
            "ExamplePlugin",
        )
        .expect("config scan should succeed");

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name, "config.yml");
    }

    #[test]
    fn get_plugin_config_files_accepts_uppercase_extensions() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let server_path = dir.path();
        let plugin_dir = server_path.join("plugins").join("ExamplePlugin");
        fs::create_dir_all(&plugin_dir).expect("plugin config dir should exist");

        fs::write(plugin_dir.join("CONFIG.YML"), "enabled: true\n")
            .expect("uppercase yaml config should write");
        fs::write(plugin_dir.join("extra.JSON"), "{\"level\":2}")
            .expect("uppercase json config should write");

        let files =
            get_plugin_config_files(server_path.to_string_lossy().as_ref(), "ExamplePlugin")
                .expect("config scan should succeed");

        assert!(files
            .iter()
            .any(|file| file.file_name == "CONFIG.YML" && file.file_type == "yml"));
        assert!(files
            .iter()
            .any(|file| file.file_name == "extra.JSON" && file.file_type == "json"));
    }

    #[test]
    fn get_plugin_config_files_propagates_supported_file_read_errors() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let server_path = dir.path();
        let plugin_dir = server_path.join("plugins").join("ExamplePlugin");
        fs::create_dir_all(plugin_dir.join("broken.yml"))
            .expect("directory-backed supported path should exist");

        let error =
            get_plugin_config_files(server_path.to_string_lossy().as_ref(), "ExamplePlugin")
                .expect_err("supported config path read failures should not be silently ignored");

        assert!(error.contains("Expected plugin config file but found directory"));
        assert!(error.contains("broken.yml"));
    }

    #[test]
    fn get_plugins_accepts_uppercase_jar_and_disabled_suffix() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = dir.path().join("plugins");
        fs::create_dir_all(&plugins_dir).expect("plugins dir should exist");

        write_plugin_jar(
            &plugins_dir.join("ExamplePlugin.JAR"),
            "name: ExamplePlugin\nversion: 1.0.0\nmain: example.Main\n",
        );
        write_plugin_jar(
            &plugins_dir.join("DisabledPlugin.JAR.DISABLED"),
            "name: DisabledPlugin\nversion: 2.0.0\nmain: disabled.Main\n",
        );

        let plugins = get_plugins(dir.path().to_string_lossy().as_ref())
            .expect("plugin listing should succeed");

        assert!(plugins.iter().any(|plugin| {
            plugin.name == "ExamplePlugin"
                && plugin.enabled
                && plugin.file_name == "ExamplePlugin.JAR"
        }));
        assert!(plugins.iter().any(|plugin| {
            plugin.name == "DisabledPlugin"
                && !plugin.enabled
                && plugin.file_name == "DisabledPlugin.JAR"
        }));
    }

    #[test]
    fn get_plugins_uses_case_insensitive_jar_suffix_for_fallback_name_and_config_dir() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = dir.path().join("plugins");
        let config_dir = plugins_dir.join("FallbackPlugin");
        fs::create_dir_all(&config_dir).expect("config dir should exist");

        write_plugin_jar(
            &plugins_dir.join("FallbackPlugin.JAR"),
            "version: 1.0.0\nmain: fallback.Main\n",
        );

        let plugins = get_plugins(dir.path().to_string_lossy().as_ref())
            .expect("plugin listing should succeed");

        let plugin = plugins
            .iter()
            .find(|plugin| plugin.file_name == "FallbackPlugin.JAR")
            .expect("fallback plugin should be listed");

        assert_eq!(plugin.name, "FallbackPlugin");
        assert_eq!(plugin.m_id, "FallbackPlugin-1.0.0");
        assert!(plugin.has_config_folder);
    }

    #[test]
    fn get_plugins_accepts_uppercase_plugin_yml_entry_name() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = dir.path().join("plugins");
        fs::create_dir_all(&plugins_dir).expect("plugins dir should exist");

        write_plugin_jar_with_entry(
            &plugins_dir.join("UppercaseEntry.jar"),
            "PLUGIN.YML",
            "name: UppercaseEntry\nversion: 1.0.0\nmain: uppercase.Main\n",
        );

        let plugins = get_plugins(dir.path().to_string_lossy().as_ref())
            .expect("plugin listing should succeed");

        assert!(plugins.iter().any(|plugin| {
            plugin.name == "UppercaseEntry" && plugin.main_class == "uppercase.Main"
        }));
    }

    #[test]
    fn get_plugins_keeps_broken_plugin_jars_visible_with_fallback_metadata() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = dir.path().join("plugins");
        fs::create_dir_all(&plugins_dir).expect("plugins dir should exist");

        fs::write(plugins_dir.join("BrokenPlugin.jar"), b"not a valid plugin jar")
            .expect("broken plugin jar should write");

        let plugins = get_plugins(dir.path().to_string_lossy().as_ref())
            .expect("plugin listing should succeed");

        let plugin = plugins
            .iter()
            .find(|plugin| plugin.file_name == "BrokenPlugin.jar")
            .expect("broken plugin jar should still be listed");

        assert_eq!(plugin.name, "BrokenPlugin");
        assert_eq!(plugin.m_id, "BrokenPlugin-unknown");
        assert_eq!(plugin.version, "Unknown");
        assert_eq!(plugin.main_class, "Unknown");
        assert!(plugin.enabled);
        assert!(!plugin.has_config_folder);
    }

    #[test]
    fn get_plugins_checked_rejects_broken_plugin_jars() {
        let dir = tempfile::tempdir().expect("temp dir should exist");
        let plugins_dir = dir.path().join("plugins");
        fs::create_dir_all(&plugins_dir).expect("plugins dir should exist");

        fs::write(plugins_dir.join("BrokenPlugin.jar"), b"not a valid plugin jar")
            .expect("broken plugin jar should write");

        let error = get_plugins_checked(dir.path().to_string_lossy().as_ref())
            .expect_err("checked plugin listing should not silently swallow invalid jars");

        assert!(error.contains("Failed to inspect plugin jar"), "unexpected error: {}", error);
        assert!(error.contains("BrokenPlugin.jar"), "unexpected error: {}", error);
        assert!(error.contains("Failed to read plugin jar"), "unexpected error: {}", error);
    }
}
