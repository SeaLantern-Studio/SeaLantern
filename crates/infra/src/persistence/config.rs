use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::fs::{ensure_parent, read_limited, write_atomic, DataLimit, FsError};

const CONFIG_READ_LIMIT: DataLimit = DataLimit::new(10 * 1024 * 1024);

/// Errors returned by configuration operations.
#[derive(Debug)]
pub enum ConfigError {
    /// Serialization failed.
    Serialize {
        format: &'static str,
        message: String,
    },
    /// Deserialization failed.
    Deserialize {
        format: &'static str,
        message: String,
    },
    /// File extension does not match any supported format.
    UnsupportedFormat {
        path: PathBuf,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Serialize { format, message } => {
                write!(f, "failed to serialize {format}: {message}")
            }
            ConfigError::Deserialize { format, message } => {
                write!(f, "failed to deserialize {format}: {message}")
            }
            ConfigError::UnsupportedFormat { path } => {
                write!(f, "unsupported config format: {}", path.display())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Supported configuration file format.
///
/// Can be used directly for string-level serialization and deserialization
/// without touching the file system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Json,
    Toml,
    Yaml,
}

impl ConfigFormat {
    fn from_extension(path: &Path) -> Result<Self, ConfigError> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("json") => Ok(ConfigFormat::Json),
            Some("toml") => Ok(ConfigFormat::Toml),
            Some("yaml") | Some("yml") => Ok(ConfigFormat::Yaml),
            _ => Err(ConfigError::UnsupportedFormat { path: path.to_path_buf() }),
        }
    }

    pub fn serialize<T: Serialize>(&self, value: &T) -> Result<String, ConfigError> {
        match self {
            ConfigFormat::Json => serde_json::to_string_pretty(value)
                .map_err(|e| ConfigError::Serialize { format: "JSON", message: e.to_string() }),
            ConfigFormat::Toml => toml::to_string_pretty(value)
                .map_err(|e| ConfigError::Serialize { format: "TOML", message: e.to_string() }),
            ConfigFormat::Yaml => serde_yaml::to_string(value)
                .map_err(|e| ConfigError::Serialize { format: "YAML", message: e.to_string() }),
        }
    }

    pub fn deserialize<T: DeserializeOwned>(&self, data: &str) -> Result<T, ConfigError> {
        match self {
            ConfigFormat::Json => serde_json::from_str(data)
                .map_err(|e| ConfigError::Deserialize { format: "JSON", message: e.to_string() }),
            ConfigFormat::Toml => toml::from_str(data)
                .map_err(|e| ConfigError::Deserialize { format: "TOML", message: e.to_string() }),
            ConfigFormat::Yaml => serde_yaml::from_str(data)
                .map_err(|e| ConfigError::Deserialize { format: "YAML", message: e.to_string() }),
        }
    }
}

impl From<ConfigError> for FsError {
    fn from(err: ConfigError) -> Self {
        match err {
            ConfigError::Serialize { format, message } => {
                FsError::serialization(format, "encode", "", message)
            }
            ConfigError::Deserialize { format, message } => {
                FsError::serialization(format, "decode", "", message)
            }
            ConfigError::UnsupportedFormat { path } => FsError::InvalidPath {
                path,
                reason: "unsupported config format (expected .json, .toml, .yaml, or .yml)",
            },
        }
    }
}

/// Generic configuration file manager.
///
/// Provides load, save, backup and restore operations for any serializable type.
/// The config format is inferred from the file extension.
///
/// File I/O methods are asynchronous and reuse `fs` primitives
/// (`write_atomic`, `read_limited`, `ensure_parent`).
pub struct ConfigFile<T> {
    path: PathBuf,
    data: T,
    format: ConfigFormat,
}

impl<T: Serialize + DeserializeOwned> ConfigFile<T> {
    pub async fn load_or_create(path: impl Into<PathBuf>, default: T) -> Result<Self, FsError> {
        let path = path.into();
        let format = ConfigFormat::from_extension(&path)?;

        let data = if path.exists() {
            let content = read_limited(&path, CONFIG_READ_LIMIT).await?;
            let text = String::from_utf8(content)
                .map_err(|e| FsError::serialization("config", "decode UTF-8", &path, e.to_string()))?;
            format.deserialize(&text).map_err(|e| {
                FsError::serialization("config", "decode", &path, e.to_string())
            })?
        } else {
            ensure_parent(&path).await?;
            let content =
                format.serialize(&default).map_err(|e| {
                    FsError::serialization("config", "encode", &path, e.to_string())
                })?;
            write_atomic(&path, content.as_bytes()).await?;
            default
        };

        Ok(Self { path, data, format })
    }

    pub async fn load(path: impl Into<PathBuf>) -> Result<Self, FsError> {
        let path = path.into();
        let format = ConfigFormat::from_extension(&path)?;
        let content = read_limited(&path, CONFIG_READ_LIMIT).await?;
        let text = String::from_utf8(content)
            .map_err(|e| FsError::serialization("config", "decode UTF-8", &path, e.to_string()))?;
        let data = format.deserialize(&text).map_err(|e| {
            FsError::serialization("config", "decode", &path, e.to_string())
        })?;
        Ok(Self { path, data, format })
    }

    /// Save current configuration to file using atomic write.
    ///
    /// When `auto_backup` is true, the previous version is backed up before
    /// the new content is written, enabling recovery if the new file is corrupt.
    pub async fn save(&self, auto_backup: bool) -> Result<(), FsError> {
        if auto_backup && self.path.exists() {
            self.backup().await?;
        }
        let content = self.format.serialize(&self.data).map_err(|e| {
            FsError::serialization("config", "encode", &self.path, e.to_string())
        })?;
        write_atomic(&self.path, content.as_bytes()).await
    }

    pub async fn reload(&mut self) -> Result<(), FsError> {
        let content = read_limited(&self.path, CONFIG_READ_LIMIT).await?;
        let text = String::from_utf8(content)
            .map_err(|e| FsError::serialization("config", "decode UTF-8", &self.path, e.to_string()))?;
        let data: T = self.format.deserialize(&text).map_err(|e| {
            FsError::serialization("config", "decode", &self.path, e.to_string())
        })?;
        self.data = data;
        Ok(())
    }

    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn set(&mut self, data: T) {
        self.data = data;
    }

    pub fn update(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.data);
    }

    pub async fn backup(&self) -> Result<PathBuf, FsError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let file_name = self.path.file_name().and_then(|n| n.to_str()).unwrap_or("config");
        let backup_name = format!("{}.bak-{}", file_name, timestamp);
        let backup_path = self.path.with_file_name(&backup_name);
        tokio::fs::copy(&self.path, &backup_path)
            .await
            .map_err(|e| FsError::io("backup config", &self.path, e))?;
        Ok(backup_path)
    }
}

impl<T: Serialize + DeserializeOwned + Default> ConfigFile<T> {
    pub async fn load_or_default(path: impl Into<PathBuf>) -> Result<Self, FsError> {
        Self::load_or_create(path, T::default()).await
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        port: u16,
        enabled: bool,
    }

    impl Default for TestConfig {
        fn default() -> Self {
            Self { name: "default".into(), port: 25565, enabled: true }
        }
    }

    fn test_dir(label: &str) -> PathBuf {
        crate::fs::test_dir(label)
    }

    #[test]
    fn config_format_round_trip_json() {
        let cfg = TestConfig { name: "test".into(), port: 8080, enabled: false };
        let json = ConfigFormat::Json.serialize(&cfg).unwrap();
        let restored: TestConfig = ConfigFormat::Json.deserialize(&json).unwrap();
        assert_eq!(cfg, restored);
    }

    #[test]
    fn config_format_round_trip_toml() {
        let cfg = TestConfig { name: "test".into(), port: 8080, enabled: false };
        let toml = ConfigFormat::Toml.serialize(&cfg).unwrap();
        let restored: TestConfig = ConfigFormat::Toml.deserialize(&toml).unwrap();
        assert_eq!(cfg, restored);
    }

    #[test]
    fn config_format_round_trip_yaml() {
        let cfg = TestConfig { name: "test".into(), port: 8080, enabled: false };
        let yaml = ConfigFormat::Yaml.serialize(&cfg).unwrap();
        let restored: TestConfig = ConfigFormat::Yaml.deserialize(&yaml).unwrap();
        assert_eq!(cfg, restored);
    }

    #[test]
    fn config_format_unsupported_extension() {
        let result = ConfigFormat::from_extension(Path::new("config.ini"));
        assert!(matches!(result, Err(ConfigError::UnsupportedFormat { .. })));
    }

    #[test]
    fn config_format_serialize_error() {
        let mut value = std::collections::HashMap::new();
        value.insert(vec![1u8], "value");
        let result = ConfigFormat::Json.serialize(&value);
        assert!(matches!(result, Err(ConfigError::Serialize { .. })));
    }

    #[tokio::test]
    async fn config_file_load_creates_default() {
        let dir = test_dir("load_default");
        let path = dir.join("settings.json");
        let cfg = ConfigFile::<TestConfig>::load_or_create(&path, TestConfig::default()).await.unwrap();
        assert_eq!(cfg.get().name, "default");
        assert!(path.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn config_file_save_atomic() {
        let dir = test_dir("save_atomic");
        let path = dir.join("settings.json");
        let mut cfg = ConfigFile::<TestConfig>::load_or_create(&path, TestConfig::default()).await.unwrap();

        cfg.set(TestConfig { name: "modified".into(), port: 9999, enabled: false });
        cfg.save(false).await.unwrap();

        let loaded = ConfigFile::<TestConfig>::load(&path).await.unwrap();
        assert_eq!(loaded.get().name, "modified");
        assert_eq!(loaded.get().port, 9999);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn config_file_reload() {
        let dir = test_dir("reload");
        let path = dir.join("settings.json");
        let mut cfg = ConfigFile::<TestConfig>::load_or_create(&path, TestConfig::default()).await.unwrap();

        let modified = TestConfig { name: "external".into(), port: 1234, enabled: true };
        let json = ConfigFormat::Json.serialize(&modified).unwrap();
        std::fs::write(&path, &json).unwrap();

        cfg.reload().await.unwrap();
        assert_eq!(cfg.get().name, "external");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn config_file_backup() {
        let dir = test_dir("backup");
        let path = dir.join("settings.json");
        let mut cfg = ConfigFile::<TestConfig>::load_or_create(&path, TestConfig::default()).await.unwrap();

        cfg.set(TestConfig { name: "before".into(), port: 1111, enabled: true });
        cfg.save(false).await.unwrap();

        let backup_path = cfg.backup().await.unwrap();
        assert!(backup_path.exists());

        cfg.set(TestConfig { name: "after".into(), port: 2222, enabled: false });
        cfg.save(false).await.unwrap();

        std::fs::copy(&backup_path, &path).unwrap();
        let restored = ConfigFile::<TestConfig>::load(&path).await.unwrap();
        assert_eq!(restored.get().name, "before");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn config_file_update_closure() {
        let dir = test_dir("update");
        let path = dir.join("settings.json");
        let mut cfg = ConfigFile::<TestConfig>::load_or_create(&path, TestConfig::default()).await.unwrap();

        cfg.update(|c| c.port = 8888);
        assert_eq!(cfg.get().port, 8888);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
