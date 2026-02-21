use crate::models::plugin::PluginManifest;
use std::fs;
use std::path::{Path, PathBuf};

pub struct PluginLoader;

impl PluginLoader {
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

            if path.is_dir() && path.join("manifest.json").exists() {
                plugin_dirs.push(path);
            }
        }

        Ok(plugin_dirs)
    }

    pub fn load_manifest(plugin_dir: &Path) -> Result<PluginManifest, String> {
        let manifest_path = plugin_dir.join("manifest.json");

        let content = fs::read_to_string(&manifest_path).map_err(|e| {
            format!("Failed to read manifest at {}: {}", manifest_path.display(), e)
        })?;

        let manifest: PluginManifest = serde_json::from_str(&content).map_err(|e| {
            format!("Failed to parse manifest at {}: {}", manifest_path.display(), e)
        })?;

        Ok(manifest)
    }

    pub fn validate_manifest(manifest: &PluginManifest) -> Result<(), String> {
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

        let valid_permissions = [
            "log",
            "fs",
            "api",
            "storage",
            "network",
            "system",
            "server",
            "console",
            "ui",
            "element",
            "execute_program",
            "plugin_folder_access",
            "plugins",
            "ui.component.read",
            "ui.component.write",
            "ui.component.proxy",
            "ui.component.create",
        ];
        for perm in &manifest.permissions {
            if !valid_permissions.contains(&perm.as_str()) {
                return Err(format!(
                    "Plugin '{}': unknown permission '{}' is not allowed. Valid permissions are: {:?}",
                    manifest.id, perm, valid_permissions
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::plugin::PluginAuthor;
    use std::fs;

    fn make_temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("sealantern_test_{}_{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_manifest_json() -> &'static str {
        r#"{
            "id": "com.example.test",
            "name": "Test Plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "author": { "name": "Tester" },
            "main": "main.lua"
        }"#
    }

    #[test]
    fn test_discover_plugins_finds_valid_dirs() {
        let tmp = make_temp_dir("discover");
        let plugin_a = tmp.join("plugin-a");
        fs::create_dir(&plugin_a).unwrap();
        fs::write(plugin_a.join("manifest.json"), "{}").unwrap();

        let no_manifest = tmp.join("no-manifest");
        fs::create_dir(&no_manifest).unwrap();

        let result = PluginLoader::discover_plugins(&tmp).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], plugin_a);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_discover_plugins_empty_on_missing_dir() {
        let result =
            PluginLoader::discover_plugins(Path::new("/nonexistent/sealantern_test_path")).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_load_manifest_success() {
        let tmp = make_temp_dir("load");
        fs::write(tmp.join("manifest.json"), sample_manifest_json()).unwrap();

        let manifest = PluginLoader::load_manifest(&tmp).unwrap();
        assert_eq!(manifest.id, "com.example.test");
        assert_eq!(manifest.name, "Test Plugin");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.main, "main.lua");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_load_manifest_file_not_found() {
        let tmp = make_temp_dir("load_missing");
        let result = PluginLoader::load_manifest(&tmp);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_validate_manifest_ok() {
        let manifest = PluginManifest {
            id: "com.example.test".into(),
            name: "Test".into(),
            version: "1.0.0".into(),
            description: "desc".into(),
            author: PluginAuthor {
                name: "Dev".into(),
                email: None,
                url: None,
            },
            main: "main.lua".into(),
            license: None,
            homepage: None,
            repository: None,
            engines: None,
            permissions: vec![],
            ui: None,
            events: vec![],
            commands: vec![],
            dependencies: Default::default(),
            optional_dependencies: Default::default(),
            icon: None,
            settings: None,
            sidebar: None,
            locales: None,
            include: vec![],
        };
        assert!(PluginLoader::validate_manifest(&manifest).is_ok());
    }

    #[test]
    fn test_validate_manifest_empty_id() {
        let manifest = PluginManifest {
            id: "".into(),
            name: "Test".into(),
            version: "1.0.0".into(),
            description: "desc".into(),
            author: PluginAuthor {
                name: "Dev".into(),
                email: None,
                url: None,
            },
            main: "main.lua".into(),
            license: None,
            homepage: None,
            repository: None,
            engines: None,
            permissions: vec![],
            ui: None,
            events: vec![],
            commands: vec![],
            dependencies: Default::default(),
            optional_dependencies: Default::default(),
            icon: None,
            settings: None,
            sidebar: None,
            locales: None,
            include: vec![],
        };
        let err = PluginLoader::validate_manifest(&manifest).unwrap_err();
        assert!(err.contains("id"));
    }
}
