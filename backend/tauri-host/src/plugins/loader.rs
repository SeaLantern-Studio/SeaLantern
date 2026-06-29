use crate::hardcode_data::app_files::PLUGIN_MANIFEST_FILE_NAME;
use crate::models::plugin::{PluginInstallIssue, PluginManifest, SemVer};
use crate::plugins::runtime::permissions::{
    is_valid_declared_permission, valid_permission_summary,
};
use std::fs;
use std::path::{Path, PathBuf};

const INCOMPATIBLE_SEALANTERN_VERSION_PREFIX: &str = "Plugin '";
const INCOMPATIBLE_SEALANTERN_VERSION_MIDDLE: &str = "' requires SeaLantern version '";
const INCOMPATIBLE_SEALANTERN_VERSION_SUFFIX: &str = "' but current version is '";

fn validate_safe_relative_file_path(path: &str, field: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(format!("Manifest field '{}' must not be empty", field));
    }

    let candidate = Path::new(trimmed);
    if candidate.is_absolute() {
        return Err(format!("Manifest field '{}' must be a relative path", field));
    }

    if trimmed.contains("..") {
        return Err(format!("Manifest field '{}' must not contain '..' segments", field));
    }

    if candidate
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!(
            "Manifest field '{}' must not contain parent directory segments",
            field
        ));
    }

    if trimmed.ends_with('/') || trimmed.ends_with('\\') {
        return Err(format!("Manifest field '{}' must point to a file", field));
    }

    Ok(())
}

fn normalize_manifest_program_path(path: &str) -> String {
    path.trim().replace('\\', "/")
}

/// 插件加载器
///
/// 负责发现插件目录、读取清单和做基础校验
pub struct PluginLoader;

impl PluginLoader {
    pub fn classify_install_error(error: &str) -> Option<PluginInstallIssue> {
        let plugin_start = error.strip_prefix(INCOMPATIBLE_SEALANTERN_VERSION_PREFIX)?;
        let (plugin_id, remaining) =
            plugin_start.split_once(INCOMPATIBLE_SEALANTERN_VERSION_MIDDLE)?;
        let (required_version, current_with_quote) =
            remaining.split_once(INCOMPATIBLE_SEALANTERN_VERSION_SUFFIX)?;
        let current_version = current_with_quote.strip_suffix('\'')?;

        Some(PluginInstallIssue::incompatible_sealantern_version(
            plugin_id,
            required_version,
            current_version,
        ))
    }

    /// 扫描插件目录下的可用插件文件夹
    ///
    /// # Parameters
    ///
    /// - `plugins_dir`: 插件根目录
    ///
    /// # Returns
    ///
    /// 返回所有包含 `manifest.json` 的插件目录
    pub fn discover_plugins(plugins_dir: &Path) -> Result<Vec<PathBuf>, String> {
        if !plugins_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(plugins_dir)
            .map_err(|e| format!("Failed to read plugins directory: {}", e))?;

        let mut plugin_dirs = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() && path.join(PLUGIN_MANIFEST_FILE_NAME).exists() {
                plugin_dirs.push(path);
            }
        }

        Ok(plugin_dirs)
    }

    /// 读取单个插件目录里的清单文件
    ///
    /// # Parameters
    ///
    /// - `plugin_dir`: 插件目录
    ///
    /// # Returns
    ///
    /// 读取成功时返回解析后的插件清单
    pub fn load_manifest(plugin_dir: &Path) -> Result<PluginManifest, String> {
        let manifest_path = plugin_dir.join(PLUGIN_MANIFEST_FILE_NAME);

        let content = fs::read_to_string(&manifest_path).map_err(|e| {
            format!("Failed to read manifest at {}: {}", manifest_path.display(), e)
        })?;

        let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
            format!("Failed to parse manifest at {}: {}", manifest_path.display(), e)
        })?;

        Ok(manifest)
    }

    /// 校验插件清单的基础合法性
    ///
    /// # Parameters
    ///
    /// - `manifest`: 待校验的插件清单
    ///
    /// # Returns
    ///
    /// 校验通过时返回 `Ok(())`
    pub fn validate_manifest(manifest: &PluginManifest) -> Result<(), String> {
        Self::validate_manifest_against_version(manifest, crate::utils::app_version::base_version())
    }

    pub fn validate_manifest_against_version(
        manifest: &PluginManifest,
        current_sealantern_version: &str,
    ) -> Result<(), String> {
        if manifest.id.trim().is_empty() {
            return Err("Manifest field 'id' is required".into());
        }

        if !manifest
            .id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(format!("Plugin ID '{}' contains invalid characters. Only alphanumeric, '-', '_' and '.' are allowed.", manifest.id));
        }
        if manifest.name.trim().is_empty() {
            return Err("Manifest field 'name' is required".into());
        }
        if manifest.version.trim().is_empty() {
            return Err("Manifest field 'version' is required".into());
        }
        if manifest.description.trim().is_empty() {
            return Err("Manifest field 'description' is required".into());
        }
        if manifest.author.name.trim().is_empty() {
            return Err("Manifest field 'author.name' is required".into());
        }
        if manifest.main.trim().is_empty() {
            return Err("Manifest field 'main' is required".into());
        }

        if manifest.main.contains("..") || std::path::Path::new(&manifest.main).is_absolute() {
            return Err(format!(
                "Plugin main file '{}' must be a safe relative path without '..'",
                manifest.main
            ));
        }

        let mut seen_programs = std::collections::HashSet::new();
        for (index, program) in manifest.programs.iter().enumerate() {
            let field = format!("programs[{}].path", index);
            validate_safe_relative_file_path(&program.path, &field)?;

            let normalized = normalize_manifest_program_path(&program.path);
            if !seen_programs.insert(normalized.clone()) {
                return Err(format!(
                    "Manifest field '{}' duplicates declared program path '{}'",
                    field, normalized
                ));
            }
        }

        for perm in &manifest.permissions {
            if !is_valid_declared_permission(perm) {
                return Err(format!(
                    "Plugin '{}': unknown permission '{}' is not allowed. Valid permissions include: {}",
                    manifest.id,
                    perm,
                    valid_permission_summary()
                ));
            }
        }

        if let Some(requirement) = manifest
            .engines
            .as_ref()
            .and_then(|engines| engines.sealantern.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let current = SemVer::parse(current_sealantern_version).ok_or_else(|| {
                format!(
                    "Current SeaLantern version '{}' is not a valid semantic version for engines.sealantern compatibility check",
                    current_sealantern_version
                )
            })?;

            if !current.satisfies(requirement) {
                return Err(format!(
                    "Plugin '{}' requires SeaLantern version '{}' but current version is '{}'",
                    manifest.id, requirement, current_sealantern_version
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/plugins_loader_tests.rs"]
mod tests;
