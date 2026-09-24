//! 实例列表持久化存储。
//!
//! 基于 `infra::persistence::config::ConfigFile` 实现，
//! 提供实例元数据的原子读写和备份能力。
//!
//! `update` 使用 `ConfigFile::update_persisted` 在单个文件锁内完成
//! "加载 → 修改 → 保存"，避免多个 `InstanceStore` 实例并发修改时
//! 基于过期快照互相覆盖。

use std::path::{Path, PathBuf};

use sealantern_infra::fs::{FileLock, FsError, write_atomic};
use sealantern_infra::persistence::config::ConfigFile;

use super::types::InstanceList;
use crate::models::CURRENT_INSTANCE_SCHEMA_VERSION;
use crate::observability;

/// 实例列表管理器的持久化句柄
pub struct InstanceStore {
    inner: ConfigFile<InstanceList>,
    path: PathBuf,
}

impl InstanceStore {
    /// 加载或创建实例列表文件。
    ///
    /// 处理顺序：
    /// 1. 若文件是 1.2.0 旧版裸数组格式，先迁移为带版本号的对象形式并写回；
    /// 2. 校验持久化格式版本，来自更新版本的数据被明确拒绝，而不是在后续读取时
    ///    抛出难以理解的缺字段错误；
    /// 3. 版本落后时逐级升级；升级属于结构性变更，写回前保留一份备份。
    pub async fn load(path: impl Into<std::path::PathBuf>) -> Result<Self, FsError> {
        let path = path.into();
        migrate_legacy_servers_file(&path).await?;

        let mut inner = ConfigFile::load_or_create(&path, InstanceList::default()).await?;

        // 必须先拒绝来自更新版本的数据：按旧结构回写会丢掉新版本才有的字段。
        ensure_supported_schema_version(&path, inner.get().version)?;

        let mut upgraded_from = None;
        inner.update(|list| {
            upgraded_from = upgrade_instance_list(list);
        });
        if let Some(from_version) = upgraded_from {
            inner.save(true).await?;
            observability::config_registry_schema_version_upgraded(
                &path,
                from_version,
                CURRENT_INSTANCE_SCHEMA_VERSION,
            );
        }

        Ok(Self { inner, path })
    }

    /// 获取当前实例列表（只读快照）
    pub fn get(&self) -> &InstanceList {
        self.inner.get()
    }

    /// 获取可变引用并持久化
    pub async fn save(&self) -> Result<(), sealantern_infra::fs::FsError> {
        self.inner.save(false).await
    }

    /// 更新实例列表并持久化。
    ///
    /// 使用底层锁内"读-改-写"原语 `ConfigFile::update_persisted`，
    /// 在单个文件锁内重新加载最新状态、应用修改并保存，
    /// 防止基于过期快照覆盖其它实例的修改。
    /// 更新完成后同步内存快照，保持与磁盘一致。
    pub async fn update(&mut self, f: impl FnOnce(&mut InstanceList)) -> Result<(), FsError> {
        let updated =
            ConfigFile::update_persisted(&self.path, InstanceList::default(), false, f).await?;
        self.inner.set(updated);
        Ok(())
    }

    /// 创建备份
    pub async fn backup(&self) -> Result<std::path::PathBuf, sealantern_infra::fs::FsError> {
        self.inner.backup().await
    }
}

/// 校验持久化格式版本不超过当前实现支持的版本。
///
/// 超过时明确拒绝，避免按旧结构误解或回写来自更新版本的数据。
fn ensure_supported_schema_version(path: &Path, version: u32) -> Result<(), FsError> {
    if version <= CURRENT_INSTANCE_SCHEMA_VERSION {
        return Ok(());
    }

    observability::config_registry_schema_version_unsupported(
        path,
        version,
        CURRENT_INSTANCE_SCHEMA_VERSION,
    );
    Err(FsError::Serialization {
        format: "json",
        operation: "check instance registry schema version",
        path: path.to_path_buf(),
        message: format!(
            "unsupported future instance registry schema version {version}; current version is {CURRENT_INSTANCE_SCHEMA_VERSION}"
        ),
    })
}

/// 按持久化格式版本分步升级实例列表，返回升级前的版本号；无需升级时为 `None`。
///
/// 新增格式版本时在此处追加一段 `if list.version < N { ... }` 并把 `list.version`
/// 推进到对应值，使任意历史数据都能逐级升到当前版本。每一步升级都应可单独测试。
fn upgrade_instance_list(list: &mut InstanceList) -> Option<u32> {
    let from = list.version;
    if from >= CURRENT_INSTANCE_SCHEMA_VERSION {
        return None;
    }

    // 版本 1 之前没有额外字段迁移：`version` 缺失的历史数据在此对齐版本号即可
    // （1.2.0 的裸数组已在 `migrate_legacy_servers_file` 中完成转换）。
    // 后续版本示例：`if list.version < 2 { ...; list.version = 2; }`
    list.version = CURRENT_INSTANCE_SCHEMA_VERSION;
    Some(from)
}

/// 检测 1.2.0 旧版裸数组格式并迁移为新版 `InstanceList`，写回磁盘。
///
/// 旧版文件以 `[` 开头（数组）；新版以 `{` 开头（对象）。仅当文件存在且
/// 以数组形式开头时才触发迁移，避免对空文件或已迁移文件做多余写入。
async fn migrate_legacy_servers_file(path: &Path) -> Result<(), FsError> {
    let _lock = lock_legacy_file(path).await?;
    let raw = match tokio::fs::read(path).await {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(FsError::Io {
                operation: "read legacy servers",
                path: path.to_path_buf(),
                source: e,
            });
        }
    };
    let text = match String::from_utf8(raw) {
        Ok(text) => text,
        Err(e) => {
            return Err(FsError::Encoding {
                path: path.to_path_buf(),
                encoding: "UTF-8",
                message: e.to_string(),
            });
        }
    };
    let trimmed = text.trim_start();
    if !trimmed.starts_with('[') {
        return Ok(());
    }

    let records: Vec<crate::models::LegacyServerInstance> = match serde_json::from_str(&text) {
        Ok(records) => records,
        Err(e) => {
            return Err(FsError::Serialization {
                format: "json",
                operation: "decode legacy servers",
                path: path.to_path_buf(),
                message: e.to_string(),
            });
        }
    };
    let list = InstanceList::migrate_legacy(records);
    let content = match serde_json::to_string_pretty(&list) {
        Ok(content) => content,
        Err(e) => {
            return Err(FsError::Serialization {
                format: "json",
                operation: "encode migrated servers",
                path: path.to_path_buf(),
                message: e.to_string(),
            });
        }
    };
    write_atomic(path, content.as_bytes()).await?;
    Ok(())
}

async fn lock_legacy_file(path: &Path) -> Result<FileLock, FsError> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || FileLock::try_acquire(path))
        .await
        .map_err(|error| FsError::Task {
            operation: "acquire legacy servers file lock",
            message: error.to_string(),
        })?
}

#[cfg(test)]
mod tests {
    use super::{CURRENT_INSTANCE_SCHEMA_VERSION, InstanceStore};

    #[tokio::test]
    async fn legacy_server_array_is_migrated_to_an_object_atomically() {
        let root = tempfile::tempdir().expect("temporary config directory should be created");
        let path = root.path().join("servers.json");
        let legacy = r#"[
          {
            "id": "srv-1",
            "name": "My Server",
            "core_type": "paper",
            "core_version": "1.20.4",
            "mc_version": "1.20.4",
            "path": "D:\\MCServers\\A",
            "jar_path": "D:\\MCServers\\A\\server.jar",
            "startup_mode": "jar",
            "custom_command": null,
            "java_path": "D:\\Java\\jdk-21\\bin\\java.exe",
            "max_memory": 2048,
            "min_memory": 512,
            "jvm_args": [],
            "port": 25565,
            "created_at": 1786865648,
            "last_started_at": 1786865895
          }
        ]"#;
        tokio::fs::write(&path, legacy)
            .await
            .expect("legacy fixture should be written");

        let store = InstanceStore::load(&path)
            .await
            .expect("legacy server list should migrate");

        assert_eq!(store.get().version, CURRENT_INSTANCE_SCHEMA_VERSION);
        assert_eq!(store.get().instances.len(), 1);
        let persisted: serde_json::Value = serde_json::from_slice(
            &tokio::fs::read(&path)
                .await
                .expect("migrated server list should be readable"),
        )
        .expect("migrated server list should be valid JSON");
        assert!(persisted.is_object());
    }

    /// 历史数据缺少 `version` 字段时按版本 0 读出，加载后自动升级并写回当前版本号。
    #[tokio::test]
    async fn registry_without_version_field_is_upgraded_and_persisted() {
        let root = tempfile::tempdir().expect("temporary config directory should be created");
        let path = root.path().join("servers.json");
        tokio::fs::write(&path, r#"{ "instances": [] }"#)
            .await
            .expect("unversioned fixture should be written");

        let store = InstanceStore::load(&path)
            .await
            .expect("unversioned registry should load");

        assert_eq!(store.get().version, CURRENT_INSTANCE_SCHEMA_VERSION);

        let persisted: serde_json::Value = serde_json::from_str(
            &tokio::fs::read_to_string(&path)
                .await
                .expect("upgraded registry should be readable"),
        )
        .expect("upgraded registry should be valid JSON");
        assert_eq!(
            persisted["version"].as_u64(),
            Some(u64::from(CURRENT_INSTANCE_SCHEMA_VERSION)),
            "升级后的版本号必须写回磁盘，否则每次启动都会重复升级"
        );
    }

    /// 来自更新版本的数据必须被明确拒绝，避免按旧结构回写而丢失新字段。
    #[tokio::test]
    async fn registry_from_a_newer_schema_version_is_rejected() {
        let root = tempfile::tempdir().expect("temporary config directory should be created");
        let path = root.path().join("servers.json");
        let future_version = CURRENT_INSTANCE_SCHEMA_VERSION + 1;
        tokio::fs::write(&path, format!(r#"{{ "version": {future_version}, "instances": [] }}"#))
            .await
            .expect("future fixture should be written");

        // `InstanceStore` 未实现 `Debug`，先把 Ok 侧收敛为 `()` 才能用 `expect_err`。
        let error = InstanceStore::load(&path)
            .await
            .map(drop)
            .expect_err("newer schema version must be rejected");

        assert!(
            error.to_string().contains(&future_version.to_string()),
            "the error message should carry the offending schema version for diagnosis: {error}"
        );
    }
}
