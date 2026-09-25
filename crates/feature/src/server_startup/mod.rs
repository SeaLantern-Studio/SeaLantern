//! 实例启动配置（`SeaLantern/config.toml`）管理。
//!
//! 权威文件是服务器目录下的 `SeaLantern/config.toml`；历史版本曾把同样的
//! 覆盖项写在服务器根目录的 `SL.json`，该文件仅作为兼容读取的回退来源，
//! 不会再被写入。
//!
//! 写入采用原子替换并配合文件锁，避免自动保存与外部编辑互相截断。

use std::fs;
use std::path::{Path, PathBuf};

use sealantern_contract::server_startup::SLStartupConfig;
use sealantern_infra::fs::{FileLock, FsError, write_atomic_blocking};

/// 实例配置所在目录名。
const INSTANCE_CONFIG_DIR_NAME: &str = "SeaLantern";
/// 权威配置文件名。
const INSTANCE_CONFIG_FILE_NAME: &str = "config.toml";
/// 历史配置文件（仅读取，用于兼容迁移前写入的数据）。
const LEGACY_CONFIG_FILE_NAME: &str = "SL.json";
/// 启动配置读取上限：该文件只承载少量覆盖项，超出视为异常输入。
const MAX_CONFIG_BYTES: u64 = 64 * 1024;

/// 启动配置处理错误。
#[derive(Debug, thiserror::Error)]
pub enum ServerStartupError {
    /// 客户端提供的输入不合法（如最小内存大于最大内存）。
    #[error("无效的启动配置: {0}")]
    InvalidInput(String),

    /// 配置文件读写失败。
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

/// 实例启动配置管理器。
pub struct ServerStartupManager {
    server_dir: PathBuf,
}

impl ServerStartupManager {
    /// 以服务器目录构造；调用时不会创建任何目录。
    pub fn new(server_dir: impl AsRef<Path>) -> Self {
        Self {
            server_dir: server_dir.as_ref().to_path_buf(),
        }
    }

    /// 权威配置文件路径。
    fn config_file(&self) -> PathBuf {
        self.server_dir
            .join(INSTANCE_CONFIG_DIR_NAME)
            .join(INSTANCE_CONFIG_FILE_NAME)
    }

    /// 历史配置文件路径。
    fn legacy_config_file(&self) -> PathBuf {
        self.server_dir.join(LEGACY_CONFIG_FILE_NAME)
    }

    /// 读取启动配置覆盖。
    ///
    /// 权威文件不存在时返回全 `None`（表示无覆盖），仅在此时才尝试历史文件。
    /// 文件存在但内容无法解析时返回 [`ServerStartupError::InvalidInput`]：静默
    /// 回落会让用户看到「配置凭空消失」，不利于定位问题。
    pub fn read(&self) -> Result<SLStartupConfig, ServerStartupError> {
        let config_file = self.config_file();
        if config_file.is_file() {
            let content = read_text_bounded(&config_file, MAX_CONFIG_BYTES)?;
            return parse_toml(&content).map_err(ServerStartupError::InvalidInput);
        }

        let legacy_file = self.legacy_config_file();
        if legacy_file.is_file() {
            let content = read_text_bounded(&legacy_file, MAX_CONFIG_BYTES)?;
            return parse_legacy_json(&content).map_err(ServerStartupError::InvalidInput);
        }

        Ok(SLStartupConfig::default())
    }

    /// 写入启动配置覆盖，必要时创建 `SeaLantern` 目录。
    pub fn write(&self, config: &SLStartupConfig) -> Result<(), ServerStartupError> {
        validate_memory_range(config).map_err(ServerStartupError::InvalidInput)?;

        let config_file = self.config_file();
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(config).map_err(|error| {
            ServerStartupError::InvalidInput(format!("序列化启动配置失败: {error}"))
        })?;

        let _lock = FileLock::try_acquire(&config_file).map_err(storage_error)?;
        write_atomic_blocking(&config_file, content.as_bytes()).map_err(storage_error)
    }
}

/// 校验内存覆盖项的区间关系。
///
/// `None` 与 `0` 都表示「该侧不参与比较」，与实例注册表的校验规则保持一致。
fn validate_memory_range(config: &SLStartupConfig) -> Result<(), String> {
    let (Some(min_memory), Some(max_memory)) = (config.min_memory, config.max_memory) else {
        return Ok(());
    };
    if min_memory != 0 && max_memory != 0 && min_memory > max_memory {
        return Err(format!("最小内存 {min_memory} MiB 不能大于最大内存 {max_memory} MiB"));
    }
    Ok(())
}

/// 解析权威 TOML 配置。
fn parse_toml(content: &str) -> Result<SLStartupConfig, String> {
    toml::from_str(content).map_err(|error| format!("解析 config.toml 失败: {error}"))
}

/// 解析历史 JSON 配置。
fn parse_legacy_json(content: &str) -> Result<SLStartupConfig, String> {
    serde_json::from_str(content).map_err(|error| format!("解析 SL.json 失败: {error}"))
}

/// 有界读取文本文件，超限或编码非法时返回错误。
fn read_text_bounded(path: &Path, max_bytes: u64) -> Result<String, ServerStartupError> {
    use std::io::Read;

    let file = fs::File::open(path)?;
    let mut content = String::new();
    file.take(max_bytes + 1).read_to_string(&mut content)?;
    if content.len() as u64 > max_bytes {
        return Err(ServerStartupError::InvalidInput(format!(
            "启动配置文件过大（上限 {max_bytes} 字节）"
        )));
    }
    Ok(content)
}

/// 把基础设施层的错误折叠为 IO 错误，保留文本供日志排查。
fn storage_error(error: FsError) -> ServerStartupError {
    ServerStartupError::Io(std::io::Error::other(error.to_string()))
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    fn manager(server: &Path) -> ServerStartupManager {
        ServerStartupManager::new(server)
    }

    #[test]
    fn returns_empty_override_when_no_config_exists() {
        let server = tempdir().unwrap();

        let config = manager(server.path()).read().unwrap();

        assert_eq!(config, SLStartupConfig::default());
        // 读操作不创建目录。
        assert!(!server.path().join(INSTANCE_CONFIG_DIR_NAME).exists());
    }

    #[test]
    fn writes_then_reads_back_sealantern_config_toml() {
        let server = tempdir().unwrap();
        let config = SLStartupConfig {
            max_memory: Some(4096),
            min_memory: Some(2048),
        };

        manager(server.path()).write(&config).unwrap();

        let config_file = server
            .path()
            .join(INSTANCE_CONFIG_DIR_NAME)
            .join(INSTANCE_CONFIG_FILE_NAME);
        assert!(config_file.is_file(), "应写入 SeaLantern/config.toml");
        assert_eq!(manager(server.path()).read().unwrap(), config);
    }

    #[test]
    fn partial_override_keeps_missing_fields_as_none() {
        let server = tempdir().unwrap();
        let config_file = server
            .path()
            .join(INSTANCE_CONFIG_DIR_NAME)
            .join(INSTANCE_CONFIG_FILE_NAME);
        fs::create_dir_all(config_file.parent().unwrap()).unwrap();
        fs::write(&config_file, "max_memory = 3072\n").unwrap();

        let config = manager(server.path()).read().unwrap();

        assert_eq!(config.max_memory, Some(3072));
        assert_eq!(config.min_memory, None, "缺省字段应保持为 None");
    }

    #[test]
    fn falls_back_to_legacy_sl_json_only_when_primary_is_absent() {
        let server = tempdir().unwrap();
        fs::write(
            server.path().join(LEGACY_CONFIG_FILE_NAME),
            r#"{"max_memory":1024,"min_memory":512}"#,
        )
        .unwrap();

        let config = manager(server.path()).read().unwrap();
        assert_eq!(config.max_memory, Some(1024));
        assert_eq!(config.min_memory, Some(512));

        // 权威文件出现后，历史文件不再参与读取。
        let primary = SLStartupConfig { max_memory: Some(8192), min_memory: None };
        manager(server.path()).write(&primary).unwrap();
        let config = manager(server.path()).read().unwrap();
        assert_eq!(config.max_memory, Some(8192));
        assert_eq!(config.min_memory, None);
    }

    #[test]
    fn write_preserves_legacy_file_untouched() {
        let server = tempdir().unwrap();
        let legacy = server.path().join(LEGACY_CONFIG_FILE_NAME);
        fs::write(&legacy, r#"{"max_memory":1024}"#).unwrap();

        manager(server.path())
            .write(&SLStartupConfig { max_memory: Some(2048), min_memory: None })
            .unwrap();

        assert_eq!(fs::read_to_string(&legacy).unwrap(), r#"{"max_memory":1024}"#);
    }

    #[test]
    fn rejects_unparsable_primary_instead_of_silently_falling_back() {
        let server = tempdir().unwrap();
        let config_file = server
            .path()
            .join(INSTANCE_CONFIG_DIR_NAME)
            .join(INSTANCE_CONFIG_FILE_NAME);
        fs::create_dir_all(config_file.parent().unwrap()).unwrap();
        fs::write(&config_file, "this is not = valid = toml").unwrap();
        // 同时放一份可用的历史文件，用来确认不会静默回落。
        fs::write(server.path().join(LEGACY_CONFIG_FILE_NAME), r#"{"max_memory":1024}"#).unwrap();

        assert!(matches!(
            manager(server.path()).read(),
            Err(ServerStartupError::InvalidInput(_))
        ));
    }

    #[test]
    fn rejects_min_memory_greater_than_max_memory() {
        let server = tempdir().unwrap();

        assert!(matches!(
            manager(server.path()).write(&SLStartupConfig {
                max_memory: Some(1024),
                min_memory: Some(2048)
            }),
            Err(ServerStartupError::InvalidInput(_))
        ));
        assert!(!server.path().join(INSTANCE_CONFIG_DIR_NAME).exists());
    }

    #[test]
    fn zero_memory_skips_range_validation_like_the_instance_registry() {
        let server = tempdir().unwrap();

        // 0 表示该侧不参与比较，与实例注册表的规则一致。
        manager(server.path())
            .write(&SLStartupConfig {
                max_memory: Some(0),
                min_memory: Some(2048),
            })
            .unwrap();
        manager(server.path())
            .write(&SLStartupConfig {
                max_memory: Some(1024),
                min_memory: Some(0),
            })
            .unwrap();
    }

    #[test]
    fn rejects_config_file_exceeding_the_read_limit() {
        let server = tempdir().unwrap();
        let config_file = server
            .path()
            .join(INSTANCE_CONFIG_DIR_NAME)
            .join(INSTANCE_CONFIG_FILE_NAME);
        fs::create_dir_all(config_file.parent().unwrap()).unwrap();
        fs::write(&config_file, vec![b'a'; (MAX_CONFIG_BYTES + 1) as usize]).unwrap();

        assert!(matches!(
            manager(server.path()).read(),
            Err(ServerStartupError::InvalidInput(_))
        ));
    }
}
