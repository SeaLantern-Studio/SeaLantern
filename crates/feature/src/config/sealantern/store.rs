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
use sealantern_infra::persistence::config::{ConfigFile, UpdatePersistedError};

use super::types::InstanceList;
use crate::models::{CURRENT_INSTANCE_SCHEMA_VERSION, legacy_schema_version};
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
    /// 2. 从原始 JSON 预读版本号并校验：来自更新版本的数据被明确拒绝，且这一步
    ///    发生在类型化反序列化之前，因此未来版本即便改了 `instances` 的结构，
    ///    得到的也是版本错误而不是通用的解码失败；
    /// 3. 版本落后时逐级升级；升级在单次持锁期间完成「读最新值 → 升级 → 备份 →
    ///    写回」，避免覆盖同一时间窗内其它写入者的修改，并在锁内再次校验版本。
    pub async fn load(path: impl Into<std::path::PathBuf>) -> Result<Self, FsError> {
        let path = path.into();
        migrate_legacy_servers_file(&path).await?;

        // 预检：只解析顶层 `version`，不触碰 `instances` 的结构。
        ensure_supported_schema_version(&path, read_persisted_schema_version(&path).await?)?;

        let mut upgraded_from = None;
        let (list, _) = ConfigFile::try_update_persisted_if_changed_with_backup(
            &path,
            InstanceList::default(),
            true,
            |list| {
                // 锁内二次校验：预检与本次写入之间可能有其它写入者把数据换成更新版本，
                // 那时按旧结构写回会丢掉新字段。
                ensure_supported_schema_version(&path, list.version)?;
                upgraded_from = upgrade_instance_list(list);
                Ok(upgraded_from.is_some())
            },
        )
        .await
        .map_err(|error| match error {
            UpdatePersistedError::Storage(error) => error,
            UpdatePersistedError::Update(error) => error,
        })?;

        if let Some(from_version) = upgraded_from {
            observability::config_registry_schema_version_upgraded(
                &path,
                from_version,
                CURRENT_INSTANCE_SCHEMA_VERSION,
            );
        }

        // 升级已把结果写回磁盘；重建句柄后直接注入锁内读到的最新值，
        // 避免再读一次盘导致内存快照与磁盘进度不一致。
        let mut inner = ConfigFile::load_or_create(&path, InstanceList::default()).await?;
        inner.set(list);

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

/// 从原始 JSON 文档中预读持久化格式版本号。
///
/// 只解析顶层 `version` 字段，不触碰 `instances` 的结构：`serde` 默认忽略未知字段，
/// 因此即便未来版本把 `instances` 改成了完全不同的形状，这里也能读出版本号。
/// 这正是「版本检查必须先于类型化反序列化」所依赖的前提——否则未来版本会先让
/// `InstanceList` 的反序列化失败，错误退化成通用的解码失败。
///
/// 文件不存在或 `version` 缺失时返回版本 0：前者等价于全新数据，后者是版本化改造
/// 之前的历史数据，两者都应交给后续升级流程处理。
async fn read_persisted_schema_version(path: &Path) -> Result<u32, FsError> {
    /// 只含版本字段的宽松视图，用于在类型化反序列化之前读出格式版本。
    #[derive(serde::Deserialize)]
    struct RegistryHeader {
        #[serde(default = "legacy_schema_version")]
        version: u32,
    }

    let raw = match tokio::fs::read(path).await {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => {
            return Err(FsError::Io {
                operation: "read instance registry for schema version",
                path: path.to_path_buf(),
                source: error,
            });
        }
    };

    let header: RegistryHeader =
        serde_json::from_slice(&raw).map_err(|error| FsError::Serialization {
            format: "json",
            operation: "decode instance registry header",
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;

    Ok(header.version)
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
    use sealantern_infra::persistence::config::ConfigFile;

    use super::{CURRENT_INSTANCE_SCHEMA_VERSION, InstanceList, InstanceStore};

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

        // 断言错误的**性质**而非仅仅包含版本数字：版本号可能恰好出现在路径或
        // 列号里，只做 contains 会假阳性。
        let message = error.to_string();
        assert!(
            message.contains("unsupported future instance registry schema version"),
            "应报出版本不兼容，而不是退化成通用解码失败: {message}"
        );
        assert!(
            message.contains(&format!("version {future_version}")),
            "错误应携带触发拒绝的版本号: {message}"
        );
    }

    /// 未来版本即使改了 `instances` 的结构，也必须在类型化反序列化之前被拒绝。
    ///
    /// 这条覆盖"版本检查晚于反序列化"的回归：若先做类型化反序列化，`instances`
    /// 从数组变成对象会让 `load` 抛通用的解码错误（`invalid type: map, expected
    /// a sequence`），既拿不到版本号，也不会触发版本不兼容事件。
    #[tokio::test]
    async fn registry_with_a_reshaped_instances_field_is_still_rejected_by_version() {
        let root = tempfile::tempdir().expect("temporary config directory should be created");
        let path = root.path().join("servers.json");
        let future_version = CURRENT_INSTANCE_SCHEMA_VERSION + 1;
        // 未来版本把 instances 从数组改成了对象，当前结构无法反序列化。
        tokio::fs::write(
            &path,
            format!(r#"{{ "version": {future_version}, "instances": {{ "by_id": {{}} }} }}"#),
        )
        .await
        .expect("reshaped future fixture should be written");

        let error = InstanceStore::load(&path)
            .await
            .map(drop)
            .expect_err("reshaped future registry must be rejected before typed decoding");

        let message = error.to_string();
        assert!(
            message.contains("unsupported future instance registry schema version"),
            "应报出版本不兼容；若报 'invalid type: map, expected a sequence' 就说明\
             版本检查仍晚于类型化反序列化: {message}"
        );
        assert!(
            message.contains(&format!("version {future_version}")),
            "错误应携带触发拒绝的版本号: {message}"
        );
    }

    /// 升级必须基于锁内的磁盘最新值，不能拿加载时的快照回写。
    ///
    /// 分两步精确复现 sourcery 描述的时序：
    /// 1. 先按旧写法取到"升级前快照"（此时磁盘上只有空的版本 0 数据）；
    /// 2. 在升级保存之前，让另一个写入者落盘一条实例；
    /// 3. 再执行升级。
    ///
    /// 若升级沿用第 1 步的快照回写，第 2 步落盘的实例就会被抹掉；锁内读-改-写则会
    /// 读到它并保留下来。因为不涉及并发抢锁，结果是确定的。
    #[tokio::test]
    async fn upgrade_does_not_overwrite_writes_that_land_before_it() {
        let root = tempfile::tempdir().expect("temporary config directory should be created");
        let path = root.path().join("servers.json");
        tokio::fs::write(&path, r#"{ "instances": [] }"#)
            .await
            .expect("unversioned fixture should be written");

        // 第 1 步：模拟旧实现先加载出的快照（版本 0、空实例）。
        let stale = ConfigFile::load_or_create(&path, InstanceList::default())
            .await
            .expect("stale snapshot should load");
        assert_eq!(stale.get().version, 0, "夹具应停留在版本 0 以触发升级");

        // 第 2 步：另一个写入者在升级保存之前落盘一条实例（仍保持版本 0）。
        let legacy_record = serde_json::json!([{
            "id": "srv-concurrent",
            "name": "并发写入",
            "core_type": "paper",
            "core_version": "1.20.4",
            "mc_version": "1.20.4",
            "path": "D:\\MCServers\\Concurrent",
            "jar_path": "D:\\MCServers\\Concurrent\\server.jar",
            "startup_mode": "jar",
            "custom_command": null,
            "java_path": "D:\\Java\\jdk-21\\bin\\java.exe",
            "max_memory": 2048,
            "min_memory": 512,
            "jvm_args": [],
            "port": 25565,
            "created_at": 1786865648,
            "last_started_at": null
        }]);
        let records: Vec<crate::models::LegacyServerInstance> =
            serde_json::from_value(legacy_record).expect("legacy fixture should parse");
        let concurrent = InstanceList {
            version: 0,
            instances: InstanceList::migrate_legacy(records).instances,
        };
        let content = serde_json::to_string_pretty(&concurrent).expect("fixture should serialize");
        tokio::fs::write(&path, content)
            .await
            .expect("concurrent write should land");

        // 第 3 步：执行升级。锁内读-改-写会读到第 2 步写入的实例。
        let store = InstanceStore::load(&path)
            .await
            .expect("registry should load and upgrade");

        assert_eq!(
            store.get().version,
            CURRENT_INSTANCE_SCHEMA_VERSION,
            "升级应把版本推进到当前值"
        );
        assert_eq!(store.get().instances.len(), 1, "升级后的内存快照应包含并发写入的实例");
        let persisted = tokio::fs::read_to_string(&path)
            .await
            .expect("upgraded registry should be readable");
        assert!(persisted.contains("srv-concurrent"), "升级不得覆盖此前写入的实例: {persisted}");
    }
}
