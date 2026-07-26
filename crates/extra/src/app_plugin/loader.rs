use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::{Map, Value};

use super::error::AppPluginError;
use super::manifest::{PluginManifest, PluginPermission, PLUGIN_API_VERSION};

/// Locates and validates plugin manifests without evaluating plugin code.
pub struct PluginLoader;

impl PluginLoader {
    pub const MANIFEST_FILE_NAME: &'static str = "manifest.json";

    /// Returns immediate child directories that contain a regular manifest file.
    ///
    /// A missing plugin root is treated as an empty installation. No directories,
    /// data files, or Lua state are created during discovery.
    pub fn discover_plugins(plugins_dir: &Path) -> Result<Vec<PathBuf>, AppPluginError> {
        if !plugins_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(plugins_dir).map_err(|error| AppPluginError::Io {
            path: plugins_dir.to_path_buf(),
            message: error.to_string(),
        })?;

        let mut plugin_dirs = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| AppPluginError::Io {
                path: plugins_dir.to_path_buf(),
                message: error.to_string(),
            })?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|error| AppPluginError::Io {
                path: path.clone(),
                message: error.to_string(),
            })?;
            if !file_type.is_dir() {
                continue;
            }

            let manifest_path = path.join(Self::MANIFEST_FILE_NAME);
            match fs::metadata(&manifest_path) {
                Ok(metadata) if metadata.is_file() => plugin_dirs.push(path),
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(AppPluginError::Io {
                        path: manifest_path,
                        message: error.to_string(),
                    });
                }
            }
        }

        plugin_dirs.sort();
        Ok(plugin_dirs)
    }

    /// Reads and validates a manifest before the caller can initialize a plugin.
    pub fn load_manifest(plugin_dir: &Path) -> Result<PluginManifest, AppPluginError> {
        let manifest_path = plugin_dir.join(Self::MANIFEST_FILE_NAME);
        let content = fs::read_to_string(&manifest_path).map_err(|error| AppPluginError::Io {
            path: manifest_path.clone(),
            message: error.to_string(),
        })?;

        let value: Value =
            serde_json::from_str(&content).map_err(|error| AppPluginError::MalformedManifest {
                path: manifest_path.clone(),
                message: error.to_string(),
            })?;
        let object = value
            .as_object()
            .ok_or_else(|| AppPluginError::MalformedManifest {
                path: manifest_path.clone(),
                message: "manifest root must be a JSON object".to_string(),
            })?;

        Self::validate_api_version(object, &manifest_path)?;
        Self::validate_permission_names(object, &manifest_path)?;

        let manifest: PluginManifest =
            serde_json::from_value(value).map_err(|error| AppPluginError::MalformedManifest {
                path: manifest_path,
                message: error.to_string(),
            })?;
        Self::validate_manifest(&manifest)?;
        Ok(manifest)
    }

    /// Validates metadata and filesystem constraints after successful parsing.
    pub fn validate_manifest(manifest: &PluginManifest) -> Result<(), AppPluginError> {
        if manifest.api_version < PLUGIN_API_VERSION {
            return Err(AppPluginError::ApiVersionTooOld);
        }
        if manifest.api_version > PLUGIN_API_VERSION {
            return Err(AppPluginError::UnsupportedApiVersion {
                found: manifest.api_version,
                supported: PLUGIN_API_VERSION,
            });
        }

        Self::validate_required_field("id", &manifest.id)?;
        Self::validate_required_field("name", &manifest.name)?;
        Self::validate_required_field("version", &manifest.version)?;
        Self::validate_required_field("main", &manifest.main)?;

        if !is_safe_plugin_id(&manifest.id) {
            return Err(AppPluginError::InvalidManifest {
                message: "field 'id' must start with an ASCII lowercase letter or digit and contain only lowercase letters, digits, '.', '-', or '_'".to_string(),
            });
        }

        validate_main_path(&manifest.main)?;
        Ok(())
    }

    fn validate_api_version(
        object: &Map<String, Value>,
        manifest_path: &Path,
    ) -> Result<(), AppPluginError> {
        let version = match object.get("apiVersion") {
            None => return Err(AppPluginError::ApiVersionTooOld),
            Some(Value::Number(number)) => number
                .as_u64()
                .and_then(|version| u32::try_from(version).ok())
                .ok_or_else(|| AppPluginError::MalformedManifest {
                    path: manifest_path.to_path_buf(),
                    message: "field 'apiVersion' must be an unsigned 32-bit integer".to_string(),
                })?,
            Some(_) => {
                return Err(AppPluginError::MalformedManifest {
                    path: manifest_path.to_path_buf(),
                    message: "field 'apiVersion' must be an unsigned 32-bit integer".to_string(),
                });
            }
        };

        if version < PLUGIN_API_VERSION {
            Err(AppPluginError::ApiVersionTooOld)
        } else if version > PLUGIN_API_VERSION {
            Err(AppPluginError::UnsupportedApiVersion {
                found: version,
                supported: PLUGIN_API_VERSION,
            })
        } else {
            Ok(())
        }
    }

    fn validate_permission_names(
        object: &Map<String, Value>,
        manifest_path: &Path,
    ) -> Result<(), AppPluginError> {
        let Some(value) = object.get("permissions") else {
            return Ok(());
        };
        let permissions = value
            .as_array()
            .ok_or_else(|| AppPluginError::MalformedManifest {
                path: manifest_path.to_path_buf(),
                message: "field 'permissions' must be an array of strings".to_string(),
            })?;

        for permission in permissions {
            let permission =
                permission
                    .as_str()
                    .ok_or_else(|| AppPluginError::MalformedManifest {
                        path: manifest_path.to_path_buf(),
                        message: "field 'permissions' must be an array of strings".to_string(),
                    })?;
            if PluginPermission::parse(permission).is_none() {
                return Err(AppPluginError::UnsupportedCapability {
                    capability: permission.to_string(),
                });
            }
        }
        Ok(())
    }

    fn validate_required_field(field: &str, value: &str) -> Result<(), AppPluginError> {
        if value.trim().is_empty() {
            return Err(AppPluginError::InvalidManifest {
                message: format!("field '{field}' must not be empty"),
            });
        }
        Ok(())
    }
}

fn is_safe_plugin_id(id: &str) -> bool {
    let mut chars = id.bytes();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() || first.is_ascii_digit() => {}
        _ => return false,
    }
    chars.all(|character| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, b'.' | b'-' | b'_')
    })
}

fn validate_main_path(main: &str) -> Result<(), AppPluginError> {
    let path = Path::new(main);
    let invalid = path.is_absolute()
        || path.components().any(|component| {
            matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        });

    if invalid {
        return Err(AppPluginError::InvalidPath {
            path: PathBuf::from(main),
            message: "main script must be a relative path without '..'".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::PluginLoader;
    use crate::app_plugin::error::AppPluginError;
    use crate::app_plugin::manifest::{PluginPermission, PLUGIN_API_VERSION};

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("sealantern-app-plugin-loader-{}-{sequence}", std::process::id()));
            fs::create_dir_all(&path).expect("test directory should be created");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_manifest(root: &Path, plugin_name: &str, manifest: &str) -> PathBuf {
        let plugin_dir = root.join(plugin_name);
        fs::create_dir_all(&plugin_dir).expect("plugin directory should be created");
        fs::write(plugin_dir.join(PluginLoader::MANIFEST_FILE_NAME), manifest)
            .expect("manifest should be written");
        plugin_dir
    }

    fn valid_manifest() -> String {
        format!(
            r#"{{
                "apiVersion": {PLUGIN_API_VERSION},
                "id": "example.plugin",
                "name": "Example Plugin",
                "version": "1.0.0",
                "main": "main.lua",
                "permissions": ["log", "storage"]
            }}"#
        )
    }

    #[test]
    fn discovery_returns_only_directories_with_manifest_files() {
        let root = TestDirectory::new();
        let expected = write_manifest(root.path(), "plugin-a", &valid_manifest());
        fs::create_dir_all(root.path().join("empty")).expect("empty directory should be created");
        fs::write(root.path().join("file.txt"), "not a plugin").expect("file should be written");

        let discovered =
            PluginLoader::discover_plugins(root.path()).expect("discovery should work");

        assert_eq!(discovered, vec![expected]);
    }

    #[test]
    fn missing_or_old_api_is_rejected_before_other_manifest_validation() {
        let root = TestDirectory::new();
        let plugin_dir = write_manifest(
            root.path(),
            "legacy",
            r#"{
                "id": "",
                "name": "",
                "version": "",
                "main": "../old.lua",
                "permissions": ["network"]
            }"#,
        );

        let error =
            PluginLoader::load_manifest(&plugin_dir).expect_err("legacy API must be rejected");

        assert!(matches!(error, AppPluginError::ApiVersionTooOld));
        assert_eq!(error.to_string(), "版本过旧");
    }

    #[test]
    fn future_api_is_rejected_before_script_evaluation() {
        let root = TestDirectory::new();
        let plugin_dir = write_manifest(
            root.path(),
            "future",
            r#"{
                "apiVersion": 3,
                "id": "future.plugin",
                "name": "Future Plugin",
                "version": "1.0.0",
                "main": "main.lua"
            }"#,
        );

        let error =
            PluginLoader::load_manifest(&plugin_dir).expect_err("future API must be rejected");

        assert!(matches!(
            error,
            AppPluginError::UnsupportedApiVersion { found: 3, supported: PLUGIN_API_VERSION }
        ));
    }

    #[test]
    fn unknown_permission_has_a_capability_error() {
        let root = TestDirectory::new();
        let plugin_dir = write_manifest(
            root.path(),
            "unsupported-capability",
            r#"{
                "apiVersion": 2,
                "id": "example.plugin",
                "name": "Example Plugin",
                "version": "1.0.0",
                "main": "main.lua",
                "permissions": ["network"]
            }"#,
        );

        let error =
            PluginLoader::load_manifest(&plugin_dir).expect_err("unsupported permission must fail");

        assert!(matches!(
            error,
            AppPluginError::UnsupportedCapability { ref capability } if capability == "network"
        ));
    }

    #[test]
    fn main_path_cannot_escape_plugin_directory() {
        let root = TestDirectory::new();
        let plugin_dir = write_manifest(
            root.path(),
            "unsafe-main",
            r#"{
                "apiVersion": 2,
                "id": "example.plugin",
                "name": "Example Plugin",
                "version": "1.0.0",
                "main": "../main.lua"
            }"#,
        );

        let error = PluginLoader::load_manifest(&plugin_dir).expect_err("unsafe main must fail");

        assert!(matches!(error, AppPluginError::InvalidPath { .. }));
    }

    #[test]
    fn valid_manifest_returns_typed_permissions() {
        let root = TestDirectory::new();
        let plugin_dir = write_manifest(root.path(), "valid", &valid_manifest());

        let manifest = PluginLoader::load_manifest(&plugin_dir).expect("manifest should load");

        assert_eq!(manifest.permissions, vec![PluginPermission::Log, PluginPermission::Storage]);
    }
}
