//! 实例配置文档 `sl.json`。
//!
//! `sl.json` 是实例的权威配置，跟随服务器目录存放。结构与读写策略：
//! - `schema_version` 缺失即视为「无有效配置」，由加载方分类处理；
//! - 版本超前于当前程序时拒绝加载并提示升级；
//! - 版本落后时在文件锁内分步迁移，迁移前自动备份并原子写回；
//! - 服务器级运行参数（`java_path`/`memory`/`port`）单层存于 `startup`，
//!   不存在中心覆盖层；
//! - 目录归属（`directory`）与实例锁（`sl.json.lock`）由位置隐含，不写入文档。

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use sealantern_contract::cron::{CronTask, CronTaskAction};
use sealantern_core::instance::{
    Instance, InstanceId, InstanceSpec, LocalLaunch, ServerMetadataSnapshot, StartupMode,
};
use sealantern_infra::fs::{
    DataLimit, FileLock, FsError, SafeRelativePath, ensure_parent, read_limited, write_atomic,
};
use sealantern_infra::persistence::config::{ConfigFile, UpdatePersistedError};
use sealantern_infra::persistence::process_lock_registry;
use serde::{Deserialize, Serialize};
use tokio::sync::OwnedRwLockWriteGuard;

use super::DocumentError;

/// 实例配置文件名。
pub const DOCUMENT_FILE_NAME: &str = "sl.json";

/// 当前程序支持的实例文档 schema 版本。
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// 文档读取上限：最大 10 MiB。
const DOCUMENT_READ_LIMIT: DataLimit = DataLimit::new(10 * 1024 * 1024);

/// 实例文档。
///
/// 必填字段（`schema_version`/`id`/`name`/`core`）缺失时反序列化直接失败，
/// 由加载方分类为「无有效配置」；其余字段缺失时回落默认值。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceDocument {
    /// 文档格式版本；写入方必须填写，缺失即判无有效配置。
    pub schema_version: u32,

    /// 实例标识（本机唯一）。
    pub id: InstanceId,

    /// 实例显示名。
    pub name: String,

    /// 搜索别名。
    #[serde(default)]
    pub aliases: Vec<String>,

    /// 服务端核心描述。
    pub core: CoreSpec,

    /// Minecraft 版本。
    #[serde(default)]
    pub mc_version: Option<String>,

    /// 最低 Java 主版本要求；启动前检查读它。
    #[serde(default)]
    pub required_java: Option<u32>,

    /// 启动配置。
    #[serde(default)]
    pub startup: StartupSpec,

    /// 实例拥有的定时任务（状态与定义同存一处）。
    #[serde(default)]
    pub cron: Vec<InstanceCronEntry>,

    /// 最近一次运行时采集的服务器元数据快照。
    #[serde(default)]
    pub server_metadata: Option<ServerMetadataSnapshot>,

    /// 创建时间（Unix 秒）。
    #[serde(default)]
    pub created_at_unix_secs: Option<u64>,

    /// 最近启动时间（Unix 秒）。
    #[serde(default)]
    pub last_started_at_unix_secs: Option<u64>,
}

impl InstanceDocument {
    /// 由已验证实例复制出等价文档；`schema_version` 固定为当前版本。
    ///
    /// `launch.startup_target` 在领域层是完整路径，写回文档前换算为
    /// 相对 `instance.directory` 的路径；目标不在实例目录内时报错。
    pub fn try_from_instance(instance: &Instance) -> Result<Self, DocumentError> {
        Ok(Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            id: instance.id.clone(),
            name: instance.name.clone(),
            aliases: instance.aliases.clone(),
            core: CoreSpec {
                core_type: instance.core_type.clone(),
                version: instance.core_version.clone(),
            },
            mc_version: Some(instance.game_version.clone()),
            required_java: instance.required_java,
            startup: StartupSpec::try_from_instance(instance)?,
            cron: Vec::new(),
            server_metadata: instance.server_metadata.clone(),
            created_at_unix_secs: Some(instance.created_at_unix_secs),
            last_started_at_unix_secs: instance.last_started_at_unix_secs,
        })
    }

    /// 由文档复制出实例输入规格；`directory` 由位置隐含，须由调用方传入，
    /// 相对启动目标会被换算回实例目录内的完整路径。
    pub fn to_spec(&self, directory: PathBuf) -> InstanceSpec {
        InstanceSpec {
            id: self.id.clone(),
            name: self.name.clone(),
            aliases: self.aliases.clone(),
            core_type: self.core.core_type.clone(),
            core_version: self.core.version.clone(),
            game_version: self.mc_version.clone().unwrap_or_default(),
            required_java: self.required_java,
            directory: directory.clone(),
            port: self.startup.port,
            max_memory_mib: self.startup.memory_mib.max,
            min_memory_mib: self.startup.memory_mib.min,
            created_at_unix_secs: self.created_at_unix_secs.unwrap_or(0),
            last_started_at_unix_secs: self.last_started_at_unix_secs,
            server_metadata: self.server_metadata.clone(),
            launch: self.startup.to_launch(&directory),
        }
    }

    /// 文档级校验（写回前的最后一道关卡）。
    ///
    /// 检查顺序：版本号 → 标识/名称 → 启动配置。
    pub fn validate(&self) -> Result<(), DocumentError> {
        match self.schema_version {
            0 => return Err(DocumentError::invalid("schema_version must be at least 1")),
            version if version > CURRENT_SCHEMA_VERSION => {
                return Err(DocumentError::UnsupportedSchemaVersion {
                    found: version,
                    current: CURRENT_SCHEMA_VERSION,
                });
            }
            _ => {}
        }
        if self.id.as_str().trim().is_empty() {
            return Err(DocumentError::invalid("id must not be empty"));
        }
        if self.name.trim().is_empty() {
            return Err(DocumentError::invalid("name must not be empty"));
        }
        self.startup.validate()?;
        Ok(())
    }
}

/// 服务端核心描述。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreSpec {
    /// 核心类型（如 `paper`、`vanilla`）。
    #[serde(rename = "type")]
    pub core_type: String,
    /// 核心构建版本。
    pub version: String,
}

/// 内存区间（MiB）。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemorySpec {
    /// 最小堆内存；`0` 表示未指定。
    #[serde(default)]
    pub min: u32,
    /// 最大堆内存；`0` 表示未指定。
    #[serde(default)]
    pub max: u32,
}

/// 实例启动配置。
///
/// 字段名保持 `sl.json` 的稳定 schema；与 `core::instance::LocalLaunch`
/// 之间通过 [`Self::to_launch`] / [`Self::from_launch`] 双向转换，校验规则
/// 复用 `LocalLaunch::normalize_and_validate`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartupSpec {
    /// 启动方式。
    pub mode: StartupMode,
    /// 启动目标（服务器目录内的相对路径）。
    #[serde(default)]
    pub target: Option<PathBuf>,
    /// 传统 shell 文本命令（`custom` 模式，与 `custom_executable` 互斥）。
    #[serde(default)]
    pub custom_command: Option<String>,
    /// 直接可执行文件（`custom` 模式，与 `custom_command` 互斥）。
    #[serde(default)]
    pub custom_executable: Option<PathBuf>,
    /// 传给 `custom_executable` 的参数。
    #[serde(default)]
    pub custom_arguments: Vec<String>,
    /// 覆盖的 Java 可执行文件；`None` 表示跟随全局检测。
    #[serde(default)]
    pub java_path: Option<PathBuf>,
    /// JVM 附加参数。
    #[serde(default)]
    pub jvm_args: Vec<String>,
    /// 堆内存区间。
    #[serde(default)]
    pub memory_mib: MemorySpec,
    /// 服务器端口；`0` 非法（受管实例必须显式端口）。
    #[serde(default)]
    pub port: u16,
}

impl Default for StartupSpec {
    fn default() -> Self {
        Self {
            mode: StartupMode::Jar,
            target: None,
            custom_command: None,
            custom_executable: None,
            custom_arguments: Vec::new(),
            java_path: None,
            jvm_args: Vec::new(),
            memory_mib: MemorySpec::default(),
            port: 0,
        }
    }
}

impl StartupSpec {
    /// 由实例复制出启动配置；启动目标换算为相对 `instance.directory` 的路径。
    pub fn try_from_instance(instance: &Instance) -> Result<Self, DocumentError> {
        let launch = &instance.launch;
        let target = launch
            .startup_target
            .as_ref()
            .map(|target| {
                target
                    .strip_prefix(&instance.directory)
                    .map_err(|_| {
                        DocumentError::invalid("startup target is outside the instance directory")
                    })
                    .and_then(|relative| {
                        // 落盘的相对目标同样须通过可移植相对路径校验，避免写出
                        // 含 `..`/反斜杠的条目，使 to_launch 还原时逃逸目录。
                        SafeRelativePath::parse(relative)
                            .map(|safe| safe.as_path().to_path_buf())
                            .map_err(|error| {
                                DocumentError::invalid(format!(
                                    "startup target must stay inside the instance directory: {error}"
                                ))
                            })
                    })
            })
            .transpose()?;
        Ok(Self {
            mode: launch.startup_mode,
            target,
            custom_command: launch.custom_command.clone(),
            custom_executable: launch.custom_executable.clone(),
            custom_arguments: launch.custom_arguments.clone(),
            java_path: launch.java_executable.clone(),
            jvm_args: launch.jvm_arguments.clone(),
            memory_mib: MemorySpec {
                min: instance.min_memory_mib,
                max: instance.max_memory_mib,
            },
            port: instance.port,
        })
    }

    /// 转换为领域 `LocalLaunch`；相对启动目标按 `instance_dir` 换算回
    /// 完整路径。不做规范化与校验。
    pub fn to_launch(&self, instance_dir: &Path) -> LocalLaunch {
        LocalLaunch {
            startup_mode: self.mode,
            startup_target: self.target.as_ref().map(|target| instance_dir.join(target)),
            custom_command: self.custom_command.clone(),
            custom_executable: self.custom_executable.clone(),
            custom_arguments: self.custom_arguments.clone(),
            java_executable: self.java_path.clone(),
            jvm_arguments: self.jvm_args.clone(),
        }
    }

    /// 校验启动配置。
    ///
    /// `target` 必须是**可移植的相对路径**（文档随目录走）：经
    /// [`SafeRelativePath`] 拒绝绝对路径、`..` 遍历组件与反斜杠——否则
    /// `to_launch` 用 `instance_dir.join(target)` 还原时会把启动目标解析到
    /// 实例目录之外。其余模式一致性规则复用 `LocalLaunch::normalize_and_validate`。
    fn validate(&self) -> Result<(), DocumentError> {
        if let Some(target) = &self.target {
            SafeRelativePath::parse(target).map_err(|error| {
                DocumentError::invalid(format!(
                    "startup target must stay inside the instance directory: {error}"
                ))
            })?;
        }
        if self.port == 0 {
            return Err(DocumentError::invalid(
                "port 0 is not supported; managed instances require an explicit port",
            ));
        }
        let MemorySpec { min, max } = self.memory_mib;
        if min != 0 && max != 0 && min > max {
            return Err(DocumentError::invalid(format!(
                "minimum memory {min} MiB exceeds maximum memory {max} MiB"
            )));
        }
        // 模式一致性校验与目标位置无关，空基目录即可。
        self.to_launch(Path::new(""))
            .normalize_and_validate()
            .map_err(|error| DocumentError::invalid(error.to_string()))?;
        Ok(())
    }
}

/// 实例拥有的定时任务（等价于 `CronTask`，归属由所在文档隐含）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceCronEntry {
    /// 任务标识。
    pub id: String,
    /// 任务显示名。
    pub name: String,
    /// cron 表达式。
    pub cron_expression: String,
    /// 要执行的动作。
    pub action: CronTaskAction,
    /// 是否启用。
    pub enabled: bool,
    /// 最近一次执行时间。
    #[serde(default)]
    pub last_run_at: Option<DateTime<Utc>>,
    /// 下一次预计执行时间。
    #[serde(default)]
    pub next_run_at: Option<DateTime<Utc>>,
    /// 最近一次执行失败原因。
    #[serde(default)]
    pub last_error: Option<String>,
}

impl InstanceCronEntry {
    /// 转换为契约层 `CronTask`；`server_id` 由所属实例补充。
    pub fn to_cron_task(&self, server_id: &str) -> CronTask {
        CronTask {
            id: self.id.clone(),
            name: self.name.clone(),
            server_id: server_id.to_string(),
            cron_expression: self.cron_expression.clone(),
            action: self.action.clone(),
            enabled: self.enabled,
            last_run_at: self.last_run_at,
            next_run_at: self.next_run_at,
            last_error: self.last_error.clone(),
        }
    }

    /// 由契约层 `CronTask` 提取（丢弃 `server_id`）。
    pub fn from_cron_task(task: &CronTask) -> Self {
        Self {
            id: task.id.clone(),
            name: task.name.clone(),
            cron_expression: task.cron_expression.clone(),
            action: task.action.clone(),
            enabled: task.enabled,
            last_run_at: task.last_run_at,
            next_run_at: task.next_run_at,
            last_error: task.last_error.clone(),
        }
    }
}

/// 实例文档的持久化句柄。
///
/// 与 `SettingsManager` 使用同一套 `ConfigFile` 原语（进程内锁 + 跨进程
/// 文件锁落在 `<dir>/sl.json.lock`、原子写、`.bak-<ts>-<uuid>` 备份）。
pub struct DocumentStore {
    inner: ConfigFile<InstanceDocument>,
    path: PathBuf,
}

impl DocumentStore {
    /// 服务器目录下 `sl.json` 的约定路径。
    pub fn path_for(dir: &Path) -> PathBuf {
        dir.join(DOCUMENT_FILE_NAME)
    }

    /// 打开并校验文档。
    ///
    /// 处理顺序：文件不存在 → `Storage(NotFound)` 上抛；内容损坏 →
    /// `Storage` 上抛（由发现层分类为「无有效配置」）；版本超前 →
    /// `UnsupportedSchemaVersion`；版本落后 → 锁内迁移+备份+原子写回。
    /// 成功后对文档做结构校验。
    ///
    /// 版本门禁在读原始字节后、类型化反序列化**之前**判断：未来 schema 若
    /// 改动必需字段/枚举值，超前文档仍能稳定报 `UnsupportedSchemaVersion`，
    /// 而不会被当前类型的反序列化错误掩盖成 `Storage`。迁移在锁内基于重读
    /// 的版本重新判断，避免用锁外旧快照覆盖并发写入的更新版本。
    pub async fn load(path: impl Into<PathBuf>) -> Result<Self, DocumentError> {
        let path = path.into();
        // 先读字节 → 提取 schema_version：超前版本即使字段兼容也一律拒绝。
        let bytes = read_document_bytes(&path)
            .await
            .map_err(DocumentError::from)?;
        let version = extract_schema_version(&path, &bytes)?;
        if version > CURRENT_SCHEMA_VERSION {
            return Err(DocumentError::UnsupportedSchemaVersion {
                found: version,
                current: CURRENT_SCHEMA_VERSION,
            });
        }

        let document: InstanceDocument =
            serde_json::from_slice(&bytes).map_err(|error| DocumentError::Storage {
                source: FsError::Serialization {
                    format: "json",
                    operation: "decode",
                    path: path.clone(),
                    message: error.to_string(),
                },
            })?;
        let mut store = Self {
            inner: ConfigFile::from_parts(path.clone(), document).map_err(DocumentError::from)?,
            path: path.clone(),
        };

        if version < CURRENT_SCHEMA_VERSION {
            // 版本落后：锁内迁移（迁移前自动备份）。回调基于锁内重读的
            // `document.schema_version` 再判断，防止过时加载器覆盖并发升级。
            let (migrated, _backup) =
                ConfigFile::<InstanceDocument>::try_update_persisted_if_changed_with_backup(
                    &path,
                    store.inner.get().clone(),
                    true,
                    |document| {
                        let found = document.schema_version;
                        if found >= CURRENT_SCHEMA_VERSION {
                            // 并发进程已迁移到当前或更高版本：不再改写。
                            return Ok(false);
                        }
                        migrate_document(document, found).map(|()| true)
                    },
                )
                .await
                .map_err(|error| match error {
                    UpdatePersistedError::Storage(source) => DocumentError::Storage { source },
                    UpdatePersistedError::Update(error) => error,
                })?;
            store.inner.set(migrated);
        }

        store.inner.get().validate()?;
        Ok(store)
    }

    /// 在锁内独占创建文档；文件已存在时拒绝。
    pub async fn create(
        path: impl Into<PathBuf>,
        mut document: InstanceDocument,
    ) -> Result<Self, DocumentError> {
        let path = path.into();
        document.schema_version = CURRENT_SCHEMA_VERSION;
        document.validate()?;

        // 持锁期间检查存在性并写入，防止并发创建互相覆盖。
        let _guard = lock_document(&path).await?;
        if tokio::fs::metadata(&path).await.is_ok() {
            return Err(DocumentError::AlreadyExists { path });
        }
        ensure_parent(&path).await?;
        let content =
            serde_json::to_string_pretty(&document).map_err(|error| FsError::Serialization {
                format: "json",
                operation: "encode",
                path: path.clone(),
                message: error.to_string(),
            })?;
        write_atomic(&path, content.as_bytes()).await?;
        drop(_guard);

        Self::load(path).await
    }

    /// 删除文档文件（锁内）。文件不存在视为已删除（幂等）。
    ///
    /// 供创建流程在名单登记失败时回滚——避免留下一个未被信任、目录也未
    /// 登记的孤儿文档被后续发现层扫到。
    pub async fn remove(path: impl Into<PathBuf>) -> Result<(), DocumentError> {
        let path = path.into();
        let _guard = lock_document(&path).await?;
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(DocumentError::Storage {
                source: FsError::Io {
                    operation: "remove instance document",
                    path,
                    source: error,
                },
            }),
        }
    }

    /// 当前文档（只读）。
    pub fn get(&self) -> &InstanceDocument {
        self.inner.get()
    }

    /// 文档路径。
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 在单个文件锁内更新并持久化。
    ///
    /// `update` 在磁盘最新值上执行；返回 `true` 表示需要写回，写回前强制
    /// `schema_version = CURRENT_SCHEMA_VERSION` 并通过结构校验，校验失败
    /// 磁盘与内存均不变。即使无需写回，本方法也返回锁内最新文档并同步
    /// 内存快照——调用方据此避免覆盖他方（如调度器）同时写入的字段。
    pub async fn update(
        &mut self,
        update: impl FnOnce(&mut InstanceDocument) -> Result<bool, DocumentError>,
    ) -> Result<InstanceDocument, DocumentError> {
        let current = self.inner.get().clone();
        let updated: Result<_, UpdatePersistedError<DocumentError>> =
            ConfigFile::try_update_persisted_if_changed(&self.path, current, false, |document| {
                if !update(document)? {
                    return Ok(false);
                }
                document.schema_version = CURRENT_SCHEMA_VERSION;
                document.validate()?;
                Ok(true)
            })
            .await;
        let updated = match updated {
            Ok(document) => document,
            Err(UpdatePersistedError::Storage(error)) => return Err(error.into()),
            Err(UpdatePersistedError::Update(error)) => return Err(error),
        };
        self.inner.set(updated.clone());
        Ok(updated)
    }

    /// 备份当前文档，返回备份路径。
    pub async fn backup(&self) -> Result<PathBuf, DocumentError> {
        self.inner.backup().await.map_err(DocumentError::from)
    }
}

/// 文档锁守卫（进程内异步锁 + 跨进程文件锁），与 `ConfigFile` 内部一致。
struct DocumentLockGuard {
    _file_lock: FileLock,
    _process_guard: OwnedRwLockWriteGuard<()>,
}

/// 获取与 `ConfigFile` 一致的锁组合，供 `create` 的"检查存在 → 写"临界区使用。
async fn lock_document(path: &Path) -> Result<DocumentLockGuard, FsError> {
    let resource = process_lock_registry()
        .resource(path)
        .map_err(|error| FsError::Task {
            operation: "coordinate instance document access",
            message: error.to_string(),
        })?;
    let process_guard = resource.write().await;
    let lock_path = path.to_path_buf();
    let file_lock = tokio::task::spawn_blocking(move || FileLock::try_acquire(&lock_path))
        .await
        .map_err(|error| FsError::Task {
            operation: "acquire instance document lock",
            message: error.to_string(),
        })??;
    Ok(DocumentLockGuard {
        _file_lock: file_lock,
        _process_guard: process_guard,
    })
}

/// 将 `from_version` 文档分步迁移到当前版本。
///
/// 当前 schema 为 v1，没有实际迁移步骤；新增结构变更时在此按版本号追加。
fn migrate_document(
    document: &mut InstanceDocument,
    _from_version: u32,
) -> Result<(), DocumentError> {
    document.schema_version = CURRENT_SCHEMA_VERSION;
    document.validate()?;
    Ok(())
}

/// 仅用于无锁读取探测（发现层先读字节判断是否为有效文档）；模块内部
/// 细节，不外泄到 `config::instance` 的公共 API。
pub(crate) async fn read_document_bytes(path: &Path) -> Result<Vec<u8>, FsError> {
    read_limited(path, DOCUMENT_READ_LIMIT).await
}

/// 从原始字节提取 `schema_version`，供类型化反序列化之前的版本门禁使用。
///
/// 缺失或非法的版本号按「损坏文档」上抛 `Storage`（与发现层「缺版本=无有效
/// 配置」一致），避免后续类型化反序列化把版本缺失误判为字段问题。
fn extract_schema_version(path: &Path, bytes: &[u8]) -> Result<u32, DocumentError> {
    let raw: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|error| DocumentError::Storage {
            source: FsError::Serialization {
                format: "json",
                operation: "decode",
                path: path.to_path_buf(),
                message: error.to_string(),
            },
        })?;
    match raw
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
    {
        Some(version) if version <= u64::from(u32::MAX) => Ok(version as u32),
        Some(version) => Ok(version.min(u64::from(u32::MAX)) as u32),
        None => Err(DocumentError::Storage {
            source: FsError::Serialization {
                format: "json",
                operation: "decode",
                path: path.to_path_buf(),
                message: "missing or invalid schema_version".to_string(),
            },
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_document() -> InstanceDocument {
        InstanceDocument {
            schema_version: CURRENT_SCHEMA_VERSION,
            id: InstanceId::new("instance-a").expect("instance ID should be valid"),
            name: "Primary".to_string(),
            aliases: Vec::new(),
            core: CoreSpec {
                core_type: "paper".to_string(),
                version: "1.21.1".to_string(),
            },
            mc_version: Some("1.21.1".to_string()),
            required_java: Some(21),
            startup: StartupSpec {
                mode: StartupMode::Jar,
                target: Some(PathBuf::from("server.jar")),
                ..StartupSpec::default()
            }
            .with_port(25565),
            cron: Vec::new(),
            server_metadata: None,
            created_at_unix_secs: Some(1_700_000_000),
            last_started_at_unix_secs: None,
        }
    }

    impl StartupSpec {
        fn with_port(mut self, port: u16) -> Self {
            self.port = port;
            self
        }
    }

    #[tokio::test]
    async fn create_writes_current_schema_version() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let path = DocumentStore::path_for(root.path());

        let store = DocumentStore::create(&path, base_document())
            .await
            .expect("document should be created");

        assert_eq!(store.get().schema_version, CURRENT_SCHEMA_VERSION);
        let persisted: serde_json::Value = serde_json::from_slice(
            &tokio::fs::read(&path)
                .await
                .expect("document should be readable"),
        )
        .expect("document should be JSON");
        assert_eq!(persisted["schema_version"], CURRENT_SCHEMA_VERSION);
        assert_eq!(persisted["core"]["type"], "paper");
        assert_eq!(persisted["startup"]["mode"], "jar");
        assert_eq!(persisted["startup"]["target"], "server.jar");
        assert_eq!(persisted["startup"]["port"], 25565);
    }

    #[tokio::test]
    async fn create_rejects_an_existing_document() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let path = DocumentStore::path_for(root.path());
        DocumentStore::create(&path, base_document())
            .await
            .expect("first create should succeed");

        let result = DocumentStore::create(&path, base_document()).await;

        assert!(matches!(result, Err(DocumentError::AlreadyExists { .. })));
    }

    #[tokio::test]
    async fn load_rejects_a_future_schema_version_without_rewriting() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let path = DocumentStore::path_for(root.path());
        let mut future = serde_json::to_value(base_document()).expect("document should serialize");
        future["schema_version"] = (CURRENT_SCHEMA_VERSION + 1).into();
        let original = serde_json::to_string_pretty(&future).expect("fixture should encode");
        tokio::fs::write(&path, &original)
            .await
            .expect("future document fixture should be written");

        let result = DocumentStore::load(&path).await;

        assert!(matches!(
            result,
            Err(DocumentError::UnsupportedSchemaVersion { found, .. }) if found == CURRENT_SCHEMA_VERSION + 1
        ));
        assert_eq!(
            tokio::fs::read_to_string(&path)
                .await
                .expect("document should remain readable"),
            original
        );
    }

    #[tokio::test]
    async fn load_reports_future_version_even_when_fields_are_incompatible() {
        // 版本门禁在读原始字节、类型化反序列化之前判断：即使未来版本新增了
        // 当前类型无法反序列化的字段，也应稳定报 UnsupportedSchemaVersion，
        // 而不是被反序列化错误掩盖成 Storage。
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let path = DocumentStore::path_for(root.path());
        let mut future = serde_json::to_value(base_document()).expect("document should serialize");
        future["schema_version"] = (CURRENT_SCHEMA_VERSION + 1).into();
        future["future_only_field"] = serde_json::json!({ "nested": [1, 2, 3] });
        tokio::fs::write(&path, serde_json::to_vec_pretty(&future).expect("encode"))
            .await
            .expect("fixture should be written");

        let result = DocumentStore::load(&path).await;

        assert!(matches!(
            result,
            Err(DocumentError::UnsupportedSchemaVersion { found, .. })
                if found == CURRENT_SCHEMA_VERSION + 1
        ));
    }

    #[tokio::test]
    async fn load_rejects_a_document_without_schema_version() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let path = DocumentStore::path_for(root.path());
        tokio::fs::write(&path, r#"{"id":"x","name":"x","core":{"type":"paper","version":"1"}}"#)
            .await
            .expect("fixture should be written");

        let result = DocumentStore::load(&path).await;

        assert!(matches!(result, Err(DocumentError::Storage { .. })));
    }

    #[tokio::test]
    async fn load_migrates_an_older_schema_version_with_backup() {
        // 版本落后的文档：load 锁内迁移到当前版本并先备份原文件。当前 v1
        // 没有实际迁移步骤（仅置版本号），但迁移链路的「备份 + 原子写回 +
        // 版本归位」行为必须可验证——否则新增真实迁移时没有回归锚点。
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let path = DocumentStore::path_for(root.path());
        let mut legacy = serde_json::to_value(base_document()).expect("document should serialize");
        legacy["schema_version"] = 0.into();
        tokio::fs::write(
            &path,
            serde_json::to_string_pretty(&legacy).expect("fixture should encode"),
        )
        .await
        .expect("legacy document fixture should be written");

        let store = DocumentStore::load(&path)
            .await
            .expect("load should migrate");

        // 内存中的版本归位且文档仍有效。
        assert_eq!(store.get().schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(store.get().id.as_str(), "instance-a");

        // 磁盘已写回当前版本。
        let persisted: serde_json::Value = serde_json::from_slice(
            &tokio::fs::read(&path)
                .await
                .expect("document should be readable"),
        )
        .expect("document should be JSON");
        assert_eq!(persisted["schema_version"], CURRENT_SCHEMA_VERSION);

        // 迁移前留下了 `.bak-*` 备份，内容为原始 v0 文件。
        let mut entries = tokio::fs::read_dir(root.path())
            .await
            .expect("directory should be readable");
        let mut backups = Vec::new();
        while let Some(entry) = entries.next_entry().await.expect("entry") {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("sl.json.bak-") {
                backups.push(entry.path());
            }
        }
        assert_eq!(backups.len(), 1, "exactly one backup should be produced");
        let backup: serde_json::Value = serde_json::from_slice(
            &tokio::fs::read(&backups[0])
                .await
                .expect("backup should be readable"),
        )
        .expect("backup should be JSON");
        assert_eq!(backup["schema_version"], 0);
    }

    #[tokio::test]
    async fn update_rejects_an_invalid_document_without_writing() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let path = DocumentStore::path_for(root.path());
        let mut store = DocumentStore::create(&path, base_document())
            .await
            .expect("document should be created");
        let before = tokio::fs::read_to_string(&path)
            .await
            .expect("document should be readable");

        let result = store
            .update(|document| {
                document.name = "  ".to_string();
                Ok(true)
            })
            .await;

        assert!(matches!(result, Err(DocumentError::Invalid { .. })));
        assert_eq!(store.get().name, "Primary");
        assert_eq!(
            tokio::fs::read_to_string(&path)
                .await
                .expect("document should remain readable"),
            before
        );
    }

    #[tokio::test]
    async fn update_enforces_the_current_schema_version() {
        let root = tempfile::tempdir().expect("temporary directory should be created");
        let path = DocumentStore::path_for(root.path());
        let mut store = DocumentStore::create(&path, base_document())
            .await
            .expect("document should be created");

        let updated = store
            .update(|document| {
                document.name = "Renamed".to_string();
                Ok(true)
            })
            .await
            .expect("update should succeed");

        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[tokio::test]
    async fn startup_target_must_be_relative() {
        // is_absolute 的语义是平台相关的：Windows 需盘符前缀 + 根，Unix 只需
        // 根——不存在单一字面对两平台都"绝对"，按编译期平台选字面量。
        let absolute = if cfg!(windows) {
            "C:\\server.jar"
        } else {
            "/server.jar"
        };
        let mut document = base_document();
        document.startup.target = Some(PathBuf::from(absolute));

        let result = document.validate();

        assert!(matches!(result, Err(DocumentError::Invalid { .. })));
    }

    #[tokio::test]
    async fn startup_target_must_not_traverse_out_of_the_instance_directory() {
        // `../outside/server.jar` 是相对路径（is_absolute 为 false），但
        // `instance_dir.join(target)` 会逃逸到实例目录之外——必须被拒绝。
        for escaping in ["../outside/server.jar", "sub/../../outside.jar", "a/../../b"] {
            let mut document = base_document();
            document.startup.target = Some(PathBuf::from(escaping));

            let result = document.validate();

            assert!(
                matches!(result, Err(DocumentError::Invalid { .. })),
                "{escaping} should be rejected"
            );
        }
    }

    #[tokio::test]
    async fn spec_round_trips_with_the_domain_instance() {
        let directory = PathBuf::from("servers/instance-a");
        let document = base_document();
        let instance = Instance::new(document.to_spec(directory.clone()))
            .expect("document should convert to a valid instance");

        let restored =
            InstanceDocument::try_from_instance(&instance).expect("instance should convert back");
        assert_eq!(restored.id, document.id);
        assert_eq!(restored.name, document.name);
        assert_eq!(restored.core, document.core);
        assert_eq!(restored.startup.port, document.startup.port);
        assert_eq!(restored.startup.target.as_deref(), Some(Path::new("server.jar")));
        assert_eq!(restored.startup.memory_mib, document.startup.memory_mib);
    }

    #[tokio::test]
    async fn required_java_round_trips_through_document_and_instance() {
        // `required_java` 必须在 文档 ↔ 领域实例 两个方向都保留，否则用户在
        // sl.json 里写的最低 Java 版本要求会在加载后丢失。
        let directory = PathBuf::from("servers/instance-a");
        let mut document = base_document();
        document.required_java = Some(21);

        let instance = Instance::new(document.to_spec(directory.clone()))
            .expect("document should convert to a valid instance");
        assert_eq!(instance.required_java, Some(21));

        let restored =
            InstanceDocument::try_from_instance(&instance).expect("instance should convert back");
        assert_eq!(restored.required_java, Some(21));
    }

    #[test]
    fn startup_target_outside_the_instance_directory_is_rejected_on_export() {
        let directory = PathBuf::from("servers/instance-a");
        let mut spec = base_document().to_spec(directory);
        spec.launch.startup_target = Some(PathBuf::from("elsewhere/server.jar"));
        let instance = Instance::new(spec).expect("instance should be valid");

        let result = InstanceDocument::try_from_instance(&instance);

        assert!(matches!(result, Err(DocumentError::Invalid { .. })));
    }

    #[test]
    fn cron_entry_converts_to_and_from_the_contract_task() {
        let entry = InstanceCronEntry {
            id: "task-1".to_string(),
            name: "Nightly restart".to_string(),
            cron_expression: "0 0 4 * * *".to_string(),
            action: CronTaskAction::Restart,
            enabled: true,
            last_run_at: None,
            next_run_at: None,
            last_error: Some("previous failure".to_string()),
        };

        let task = entry.to_cron_task("instance-a");
        assert_eq!(task.server_id, "instance-a");
        assert_eq!(task.action, CronTaskAction::Restart);

        let restored = InstanceCronEntry::from_cron_task(&task);
        assert_eq!(restored.id, entry.id);
        assert_eq!(restored.last_error, entry.last_error);
    }
}
