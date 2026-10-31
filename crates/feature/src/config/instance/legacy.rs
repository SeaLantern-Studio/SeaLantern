//! 旧版实例配置（`sea_lantern_servers.json`）读取与迁移映射。
//!
//! 1.2.x 把全部服务器实例集中存放在主配置目录的 JSON 数组里；1.3.x 起实例
//! 权威配置改为各服务器目录内的 `sl.json`。本模块只负责**只读解析**旧文件
//! 并把条目映射为 [`InstanceDocument`]——写 `sl.json`、登记信任名单等有副作用
//! 的步骤由应用层编排，这样检测阶段可以反复调用而不触碰磁盘。
//!
//! 旧格式字段全部按「缺失即默认」解析：旧版本可能在不同补丁中增删字段，
//! 严格模式会让一个可选字段缺失就丢掉整份列表。

use std::path::{Path, PathBuf};

use sealantern_core::instance::{InstanceId, StartupMode};
use sealantern_infra::fs::{DataLimit, FsError, read_limited};
use serde::Deserialize;
use uuid::Uuid;

use super::{CoreSpec, DOCUMENT_FILE_NAME, InstanceDocument, MemorySpec, StartupSpec};

/// 旧版实例列表文件名（位于主配置目录）。
pub const LEGACY_SERVERS_FILE_NAME: &str = "sea_lantern_servers.json";

/// 旧文件读取上限：最大 10 MiB（与 `sl.json` 保持一致）。
const LEGACY_READ_LIMIT: DataLimit = DataLimit::new(10 * 1024 * 1024);

/// 旧配置缺省启动方式。
fn default_startup_mode() -> String {
    "jar".to_string()
}

/// 旧版 `ServerInstance`（1.2.x）的只读镜像。
///
/// 字段名与旧序列化格式一致；`Option`/`default` 覆盖旧补丁的字段差异。
#[derive(Debug, Clone, Deserialize)]
pub struct LegacyServerInstance {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub core_type: String,
    #[serde(default)]
    pub core_version: String,
    #[serde(default)]
    pub mc_version: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub jar_path: String,
    #[serde(default = "default_startup_mode")]
    pub startup_mode: String,
    #[serde(default)]
    pub custom_command: Option<String>,
    #[serde(default)]
    pub java_path: String,
    #[serde(default)]
    pub max_memory: u32,
    #[serde(default)]
    pub min_memory: u32,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub last_started_at: Option<u64>,
}

/// 旧配置条目读取失败的原因。
#[derive(Debug)]
pub enum LegacyConfigError {
    /// 旧文件不存在（没有可迁移数据）。
    NotFound {
        /// 预期位置。
        path: PathBuf,
    },
    /// 文件读取失败。
    Storage {
        /// 底层文件系统错误。
        source: FsError,
    },
    /// 内容不是旧格式的实例数组。
    Invalid {
        /// 旧文件路径。
        path: PathBuf,
        /// 稳定的失败描述。
        message: String,
    },
}

impl std::fmt::Display for LegacyConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { path } => {
                write!(formatter, "legacy server list not found: {}", path.display())
            }
            Self::Storage { source } => {
                write!(formatter, "failed to read legacy server list: {source}")
            }
            Self::Invalid { path, message } => {
                write!(formatter, "invalid legacy server list '{}': {message}", path.display())
            }
        }
    }
}

impl std::error::Error for LegacyConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage { source } => Some(source),
            _ => None,
        }
    }
}

/// 旧配置文件在主配置目录下的约定位置。
pub fn legacy_servers_path(config_dir: &Path) -> PathBuf {
    config_dir.join(LEGACY_SERVERS_FILE_NAME)
}

/// 读取并解析旧配置；文件不存在时返回 [`LegacyConfigError::NotFound`]。
///
/// 兼容两种历史形态：裸数组（1.2.x 实际格式）与 `{ "servers": [...] }`
/// 包装（个别补丁/第三方分支），避免用户恰好是后者时看不到迁移提示。
pub async fn read_legacy_servers(
    path: &Path,
) -> Result<Vec<LegacyServerInstance>, LegacyConfigError> {
    let bytes = match read_limited(path, LEGACY_READ_LIMIT).await {
        Ok(bytes) => bytes,
        Err(FsError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
            return Err(LegacyConfigError::NotFound { path: path.to_path_buf() });
        }
        Err(source) => return Err(LegacyConfigError::Storage { source }),
    };
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Ok(Vec::new());
    }
    if let Ok(servers) = serde_json::from_slice::<Vec<LegacyServerInstance>>(&bytes) {
        return Ok(servers);
    }
    #[derive(Deserialize)]
    struct Wrapped {
        #[serde(default)]
        servers: Vec<LegacyServerInstance>,
    }
    serde_json::from_slice::<Wrapped>(&bytes)
        .map(|wrapped| wrapped.servers)
        .map_err(|error| LegacyConfigError::Invalid {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
}

/// 一个已映射到新文档模型的旧条目。
#[derive(Debug, Clone)]
pub struct MappedLegacyServer {
    /// 服务器目录（旧配置的绝对路径）。
    pub directory: PathBuf,
    /// 迁移后的实例文档（`schema_version` 已就位，未落盘）。
    pub document: InstanceDocument,
}

/// 旧条目无法映射为新实例文档。
#[derive(Debug)]
pub enum LegacyMappingError {
    /// 旧配置没有记录目录（无法定位 `sl.json`）。
    MissingDirectory,
    /// 目录不是有效的 UTF-8 路径。
    InvalidDirectory,
    /// 实例字段未通过领域校验。
    Instance {
        /// 底层领域错误。
        source: sealantern_core::instance::InstanceError,
    },
}

impl std::fmt::Display for LegacyMappingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingDirectory => write!(formatter, "legacy server has no directory"),
            Self::InvalidDirectory => {
                write!(formatter, "legacy server directory is not valid UTF-8")
            }
            Self::Instance { source } => write!(formatter, "legacy instance is invalid: {source}"),
        }
    }
}

impl std::error::Error for LegacyMappingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Instance { source } => Some(source),
            _ => None,
        }
    }
}

impl LegacyServerInstance {
    /// 旧条目的服务器目录；空白视为缺失。
    pub fn directory(&self) -> Result<PathBuf, LegacyMappingError> {
        if self.path.trim().is_empty() {
            return Err(LegacyMappingError::MissingDirectory);
        }
        Ok(PathBuf::from(&self.path))
    }

    /// 映射为新的实例文档。
    ///
    /// 既定规则：
    /// - id 沿用旧值（保持信任/外部引用稳定）；为空则生成 v4 UUID；
    /// - 启动目标把旧的绝对 `jar_path` 换算为相对服务器目录的路径，
    ///   超出目录的（旧版本允许）回落到文件名；
    /// - `custom` 模式不带启动目标，命令沿用 `custom_command`；
    /// - 端口为 0（旧配置未设置）时回落到默认 25565，因为新文档要求显式端口；
    /// - 核心类型/版本缺失时写空串（新模型不做非空约束）。
    pub fn to_document(&self) -> Result<MappedLegacyServer, LegacyMappingError> {
        let directory = self.directory()?;
        let id = match InstanceId::new(self.id.clone()) {
            Ok(id) if !id.as_str().is_empty() => id,
            _ => InstanceId::new(Uuid::new_v4().to_string())
                .map_err(|source| LegacyMappingError::Instance { source })?,
        };
        let name = if self.name.trim().is_empty() {
            directory
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| id.as_str().to_string())
        } else {
            self.name.trim().to_string()
        };

        let mode = StartupMode::parse(&self.startup_mode).unwrap_or(StartupMode::Jar);
        let target = if mode == StartupMode::Custom {
            None
        } else {
            relative_startup_target(&directory, &self.jar_path)
        };
        let custom_command = self
            .custom_command
            .as_deref()
            .map(str::trim)
            .filter(|command| !command.is_empty())
            .map(str::to_string);
        let java_path = {
            let trimmed = self.java_path.trim();
            (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
        };

        let document = InstanceDocument {
            schema_version: super::CURRENT_SCHEMA_VERSION,
            id,
            name,
            aliases: Vec::new(),
            core: CoreSpec {
                core_type: self.core_type.trim().to_string(),
                version: self.core_version.trim().to_string(),
            },
            mc_version: {
                let trimmed = self.mc_version.trim();
                (!trimmed.is_empty()).then(|| trimmed.to_string())
            },
            required_java: None,
            startup: StartupSpec {
                mode,
                target,
                custom_command,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_path,
                jvm_args: self
                    .jvm_args
                    .iter()
                    .map(|arg| arg.trim().to_string())
                    .filter(|arg| !arg.is_empty())
                    .collect(),
                memory_mib: MemorySpec { min: self.min_memory, max: self.max_memory },
                port: if self.port == 0 { 25565 } else { self.port },
            },
            cron: Vec::new(),
            server_metadata: None,
            created_at_unix_secs: (self.created_at != 0).then_some(self.created_at),
            last_started_at_unix_secs: self.last_started_at,
        };
        Ok(MappedLegacyServer { directory, document })
    }
}

/// 把旧的绝对 `jar_path` 换算为相对服务器目录的启动目标。
///
/// 目录内路径保留完整相对层级（旧版本允许 `subdir/server.jar`）；目录外或
/// 空路径回落到文件名，后者仍无法确定时返回 `None`——此时文档校验会因
/// 非 `custom` 模式缺少启动目标而失败，由调用方分类为「不可迁移」。
fn relative_startup_target(directory: &Path, jar_path: &str) -> Option<PathBuf> {
    let jar_path = jar_path.trim();
    if jar_path.is_empty() {
        return None;
    }
    let jar_path = Path::new(jar_path);
    if let Ok(relative) = jar_path.strip_prefix(directory) {
        if !relative.as_os_str().is_empty() {
            return Some(relative.to_path_buf());
        }
    }
    jar_path.file_name().map(PathBuf::from)
}

/// 目录内是否已有实例文档（用于判定「已迁移」）。
pub fn has_document(directory: &Path) -> bool {
    directory.join(DOCUMENT_FILE_NAME).is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy(id: &str, path: &Path, jar: &Path) -> LegacyServerInstance {
        LegacyServerInstance {
            id: id.to_string(),
            name: "Paper Server".to_string(),
            core_type: "paper".to_string(),
            core_version: "1.20.4".to_string(),
            mc_version: "1.20.4".to_string(),
            path: path.to_string_lossy().into_owned(),
            jar_path: jar.to_string_lossy().into_owned(),
            startup_mode: "jar".to_string(),
            custom_command: None,
            java_path: String::new(),
            max_memory: 4096,
            min_memory: 1024,
            jvm_args: vec!["-XX:+UseG1GC".to_string()],
            port: 25565,
            created_at: 1_700_000_000,
            last_started_at: Some(1_700_000_100),
        }
    }

    #[test]
    fn maps_jar_instance_with_relative_target() {
        let dir = PathBuf::from("/srv/mc/paper");
        let entry = legacy("abc", &dir, &dir.join("server.jar"));

        let mapped = entry.to_document().expect("legacy entry should map");

        assert_eq!(mapped.directory, dir);
        assert_eq!(mapped.document.startup.target.as_deref(), Some(Path::new("server.jar")));
        assert_eq!(mapped.document.startup.mode, StartupMode::Jar);
        assert_eq!(mapped.document.startup.memory_mib.max, 4096);
        assert_eq!(mapped.document.created_at_unix_secs, Some(1_700_000_000));
        mapped.document.validate().expect("mapped document should validate");
    }

    #[test]
    fn maps_custom_instance_without_target() {
        let dir = PathBuf::from("/srv/mc/custom");
        let mut entry = legacy("abc", &dir, &dir.join("server.jar"));
        entry.startup_mode = "custom".to_string();
        entry.custom_command = Some(" ./start.sh ".to_string());
        entry.jar_path = String::new();

        let mapped = entry.to_document().expect("legacy entry should map");

        assert_eq!(mapped.document.startup.mode, StartupMode::Custom);
        assert_eq!(mapped.document.startup.target, None);
        assert_eq!(mapped.document.startup.custom_command.as_deref(), Some("./start.sh"));
        mapped.document.validate().expect("mapped document should validate");
    }

    #[test]
    fn zero_port_falls_back_to_default() {
        let dir = PathBuf::from("/srv/mc/paper");
        let mut entry = legacy("abc", &dir, &dir.join("server.jar"));
        entry.port = 0;

        let mapped = entry.to_document().expect("legacy entry should map");

        assert_eq!(mapped.document.startup.port, 25565);
    }

    #[test]
    fn blank_id_is_regenerated() {
        let dir = PathBuf::from("/srv/mc/paper");
        let entry = legacy("   ", &dir, &dir.join("server.jar"));

        let mapped = entry.to_document().expect("legacy entry should map");

        assert!(!mapped.document.id.as_str().trim().is_empty());
        assert_ne!(mapped.document.id.as_str(), "   ");
    }

    #[test]
    fn target_outside_directory_falls_back_to_file_name() {
        let dir = PathBuf::from("/srv/mc/paper");
        let entry = legacy("abc", &dir, Path::new("/opt/jars/server.jar"));

        let mapped = entry.to_document().expect("legacy entry should map");

        assert_eq!(mapped.document.startup.target.as_deref(), Some(Path::new("server.jar")));
    }

    #[test]
    fn missing_directory_is_rejected() {
        let mut entry = legacy(
            "abc",
            Path::new("/srv/mc/paper"),
            Path::new("/srv/mc/paper/server.jar"),
        );
        entry.path = "  ".to_string();

        assert!(matches!(
            entry.to_document(),
            Err(LegacyMappingError::MissingDirectory)
        ));
    }
}
