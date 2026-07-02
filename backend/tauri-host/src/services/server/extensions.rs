use crate::models::server::{ServerInstance, ServerRuntimeConfig};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const PLUGIN_DIR_CANDIDATES: [&str; 2] = ["plugins", "plugin"];
const MOD_DIR_CANDIDATES: [&str; 2] = ["mods", "mod"];
const ROOT_OTHER_FILE_EXTENSIONS: [&str; 4] = ["jar", "zip", "mrpack", "litemod"];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerExtensionEntryKind {
    Plugin,
    Mod,
    Other,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerExtensionEntrySummary {
    pub name: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub modified_at: Option<String>,
    pub kind: ServerExtensionEntryKind,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerExtensionsSummary {
    pub has_plugins_dir: bool,
    pub has_mods_dir: bool,
    pub plugin_entries: Vec<ServerExtensionEntrySummary>,
    pub mod_entries: Vec<ServerExtensionEntrySummary>,
    pub other_entries: Vec<ServerExtensionEntrySummary>,
}

pub fn read_server_extensions_summary(
    server_path: &str,
    server: Option<&ServerInstance>,
) -> Result<ServerExtensionsSummary, String> {
    let root = Path::new(server_path);
    if !root.exists() {
        return Err(format!("Server path does not exist: {server_path}"));
    }

    if !root.is_dir() {
        return Err(format!("Server path is not a directory: {server_path}"));
    }

    let plugin_dirs = resolve_existing_top_level_dirs(root, &PLUGIN_DIR_CANDIDATES)?;
    let mod_dirs = resolve_existing_top_level_dirs(root, &MOD_DIR_CANDIDATES)?;
    let excluded_root_files = resolve_excluded_root_files(root, server);

    let mut plugin_entries =
        collect_direct_child_files(root, &plugin_dirs, ServerExtensionEntryKind::Plugin)?;
    let mut mod_entries =
        collect_direct_child_files(root, &mod_dirs, ServerExtensionEntryKind::Mod)?;
    let mut other_entries = collect_other_root_entries(root, &excluded_root_files)?;

    sort_entries(&mut plugin_entries);
    sort_entries(&mut mod_entries);
    sort_entries(&mut other_entries);

    Ok(ServerExtensionsSummary {
        has_plugins_dir: !plugin_dirs.is_empty(),
        has_mods_dir: !mod_dirs.is_empty(),
        plugin_entries,
        mod_entries,
        other_entries,
    })
}

fn resolve_existing_top_level_dirs(
    root: &Path,
    candidates: &[&str],
) -> Result<Vec<PathBuf>, String> {
    let mut directories = Vec::new();

    for candidate in candidates {
        let path = root.join(candidate);
        match fs::symlink_metadata(&path) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    continue;
                }

                if metadata.is_dir() {
                    directories.push(path);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!("Failed to inspect directory {}: {}", path.display(), error));
            }
        }
    }

    Ok(directories)
}

fn collect_direct_child_files(
    root: &Path,
    directories: &[PathBuf],
    kind: ServerExtensionEntryKind,
) -> Result<Vec<ServerExtensionEntrySummary>, String> {
    let mut entries = Vec::new();

    for directory in directories {
        let read_dir = fs::read_dir(directory).map_err(|error| {
            format!("Failed to read directory {}: {}", directory.display(), error)
        })?;

        for child in read_dir {
            let child = child.map_err(|error| {
                format!("Failed to read directory entry in {}: {}", directory.display(), error)
            })?;
            let child_path = child.path();
            let metadata = fs::symlink_metadata(&child_path).map_err(|error| {
                format!("Failed to read metadata for {}: {}", child_path.display(), error)
            })?;

            if metadata.file_type().is_symlink() || !metadata.is_file() {
                continue;
            }

            entries.push(build_entry_summary(root, &child_path, metadata, kind.clone())?);
        }
    }

    Ok(entries)
}

fn collect_other_root_entries(
    root: &Path,
    excluded_root_files: &HashSet<String>,
) -> Result<Vec<ServerExtensionEntrySummary>, String> {
    let mut entries = Vec::new();
    let read_dir = fs::read_dir(root)
        .map_err(|error| format!("Failed to read directory {}: {}", root.display(), error))?;

    for child in read_dir {
        let child = child.map_err(|error| {
            format!("Failed to read directory entry in {}: {}", root.display(), error)
        })?;
        let child_path = child.path();
        let metadata = fs::symlink_metadata(&child_path).map_err(|error| {
            format!("Failed to read metadata for {}: {}", child_path.display(), error)
        })?;

        if metadata.file_type().is_symlink() || !metadata.is_file() {
            continue;
        }

        let relative_path = normalize_relative_path(root, &child_path)?;
        if excluded_root_files.contains(&relative_path) || !is_root_other_candidate(&child_path) {
            continue;
        }

        entries.push(build_entry_summary(
            root,
            &child_path,
            metadata,
            ServerExtensionEntryKind::Other,
        )?);
    }

    Ok(entries)
}

fn build_entry_summary(
    root: &Path,
    path: &Path,
    metadata: fs::Metadata,
    kind: ServerExtensionEntryKind,
) -> Result<ServerExtensionEntrySummary, String> {
    let name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .ok_or_else(|| format!("Failed to resolve file name for {}", path.display()))?;

    Ok(ServerExtensionEntrySummary {
        name,
        relative_path: normalize_relative_path(root, path)?,
        size_bytes: metadata.len(),
        modified_at: metadata.modified().ok().map(format_system_time),
        kind,
    })
}

fn normalize_relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path.strip_prefix(root).map_err(|error| {
        format!("Failed to resolve relative path for {}: {}", path.display(), error)
    })?;
    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn format_system_time(time: std::time::SystemTime) -> String {
    let timestamp: DateTime<Utc> = time.into();
    timestamp.to_rfc3339()
}

fn sort_entries(entries: &mut [ServerExtensionEntrySummary]) {
    entries.sort_by(|left, right| {
        left.relative_path
            .cmp(&right.relative_path)
            .then_with(|| left.name.cmp(&right.name))
    });
}

fn is_root_other_candidate(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    if file_name.ends_with(".jar.disabled") || file_name.ends_with(".zip.disabled") {
        return true;
    }

    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    match extension.as_deref() {
        Some(ext) => ROOT_OTHER_FILE_EXTENSIONS.contains(&ext),
        None => false,
    }
}

fn resolve_excluded_root_files(root: &Path, server: Option<&ServerInstance>) -> HashSet<String> {
    let mut excluded = HashSet::new();

    let Some(server) = server else {
        return excluded;
    };

    if let ServerRuntimeConfig::Local(runtime) = &server.runtime {
        if runtime.jar_path.trim().is_empty() {
            return excluded;
        }

        let runtime_path = Path::new(&runtime.jar_path);
        if let Ok(relative) = runtime_path.strip_prefix(root) {
            if relative.components().count() == 1 {
                excluded.insert(relative.to_string_lossy().replace('\\', "/"));
            }
        }
    }

    excluded
}

#[cfg(test)]
mod tests {
    use super::{read_server_extensions_summary, ServerExtensionEntryKind};
    use crate::models::server::{
        CpuPolicyConfig, JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    };

    fn sample_server(path: &std::path::Path, jar_path: &str) -> ServerInstance {
        ServerInstance {
            id: "extensions-1".to_string(),
            name: "Extensions Test".to_string(),
            aliases: Vec::new(),
            core_type: "paper".to_string(),
            core_version: "1.0.0".to_string(),
            mc_version: "1.21.1".to_string(),
            path: path.to_string_lossy().to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: jar_path.to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "java".to_string(),
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn read_server_extensions_summary_collects_direct_child_files_by_directory() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        std::fs::create_dir_all(temp_dir.path().join("plugins")).expect("plugins dir should exist");
        std::fs::create_dir_all(temp_dir.path().join("mods")).expect("mods dir should exist");
        std::fs::write(temp_dir.path().join("plugins").join("essentials.jar"), b"jar")
            .expect("plugin file should write");
        std::fs::write(temp_dir.path().join("mods").join("jei.jar"), b"jar")
            .expect("mod file should write");
        std::fs::write(temp_dir.path().join("pack.zip"), b"zip").expect("other file should write");

        let server = sample_server(
            temp_dir.path(),
            temp_dir
                .path()
                .join("server.jar")
                .to_string_lossy()
                .as_ref(),
        );
        let summary = read_server_extensions_summary(&server.path, Some(&server))
            .expect("extensions summary should load");

        assert!(summary.has_plugins_dir);
        assert!(summary.has_mods_dir);
        assert_eq!(summary.plugin_entries.len(), 1);
        assert_eq!(summary.mod_entries.len(), 1);
        assert_eq!(summary.other_entries.len(), 1);
        assert_eq!(summary.plugin_entries[0].kind, ServerExtensionEntryKind::Plugin);
        assert_eq!(summary.mod_entries[0].kind, ServerExtensionEntryKind::Mod);
        assert_eq!(summary.other_entries[0].kind, ServerExtensionEntryKind::Other);
        assert_eq!(summary.plugin_entries[0].relative_path, "plugins/essentials.jar");
        assert_eq!(summary.mod_entries[0].relative_path, "mods/jei.jar");
        assert_eq!(summary.other_entries[0].relative_path, "pack.zip");
    }

    #[test]
    fn read_server_extensions_summary_excludes_root_runtime_jar_from_other_entries() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let runtime_jar = temp_dir.path().join("server.jar");
        std::fs::write(&runtime_jar, b"jar").expect("runtime jar should write");
        std::fs::write(temp_dir.path().join("addon.jar.disabled"), b"jar")
            .expect("disabled jar should write");

        let server = sample_server(temp_dir.path(), runtime_jar.to_string_lossy().as_ref());
        let summary = read_server_extensions_summary(&server.path, Some(&server))
            .expect("extensions summary should load");

        assert!(summary
            .other_entries
            .iter()
            .all(|entry| entry.relative_path != "server.jar"));
        assert!(summary
            .other_entries
            .iter()
            .any(|entry| entry.relative_path == "addon.jar.disabled"));
    }

    #[test]
    fn read_server_extensions_summary_ignores_nested_files_inside_plugin_subdirectories() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        std::fs::create_dir_all(temp_dir.path().join("plugins").join("ExamplePlugin"))
            .expect("nested plugin dir should exist");
        std::fs::write(
            temp_dir
                .path()
                .join("plugins")
                .join("ExamplePlugin")
                .join("config.yml"),
            b"content",
        )
        .expect("nested config should write");

        let summary =
            read_server_extensions_summary(temp_dir.path().to_string_lossy().as_ref(), None)
                .expect("extensions summary should load");

        assert!(summary.has_plugins_dir);
        assert!(summary.plugin_entries.is_empty());
    }
}
