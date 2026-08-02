//! 应用设置管理器。
//!
//! 管理 `AppSettings` 的加载、保存、分组 diff 和部分更新。
//! 底层复用 `infra::persistence::config::ConfigFile` 实现原子写入。
//!
//! 数据安全策略：
//! - 加载时自动检测旧版嵌套配置格式（v1：`{ version, preferences: {...} }`），
//!   在文件锁保护下迁移，迁移前备份旧文件；
//! - 配置文件损坏时备份隔离原文件，再以默认配置启动，不阻断应用；
//! - 版本升级前强制备份，并通过分步迁移链逐版本升级；
//! - `update`/`update_partial`/`reset` 持久化失败时回滚内存状态。

use std::path::{Path, PathBuf};

use sealantern_infra::fs::{read_string_limited, write_atomic, DataLimit, FileLock, FsError};
use sealantern_infra::persistence::config::ConfigFile;
use sealantern_infra::persistence::process_lock_registry;
use serde::Deserialize;
use tokio::sync::OwnedRwLockWriteGuard;

use super::types::{
    AppSettings, JavaInfo, PartialAppSettings, UpdateResult, CURRENT_CONFIG_VERSION,
};
use crate::observability;

/// 配置文件读取上限：最大 10 MiB。
const CONFIG_READ_LIMIT: DataLimit = DataLimit::new(10 * 1024 * 1024);

/// 旧版嵌套配置格式（v1：`{ version, preferences: {...} }`）。
///
/// 仅用于启动迁移检测，不属于新配置结构的一部分。
#[derive(Debug, Deserialize)]
struct LegacyAppConfig {
    #[serde(default)]
    preferences: LegacyPreferences,
}

#[derive(Debug, Default, Deserialize)]
struct LegacyPreferences {
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    theme: Option<String>,
    #[serde(default)]
    developer_mode: Option<bool>,
}

/// 配置文件访问锁守卫（进程内异步锁 + 跨进程文件锁）。
///
/// 与 `infra::persistence::config` 内部的锁保持一致，
/// 供 legacy 迁移等需要绕过 `ConfigFile` 直接读写的路径使用。
struct ConfigLockGuard {
    _file_lock: FileLock,
    _process_guard: OwnedRwLockWriteGuard<()>,
}

/// 应用设置管理器
pub struct SettingsManager {
    inner: ConfigFile<AppSettings>,
    path: PathBuf,
}

impl SettingsManager {
    /// 加载或创建设置文件，检测版本号并执行迁移。
    ///
    /// 处理顺序：
    /// 1. 旧版嵌套 `preferences` 格式迁移（文件锁保护，迁移前备份）；
    /// 2. 文件不存在 → 创建默认配置；
    /// 3. 文件损坏 → 备份隔离原文件，恢复默认配置并告警；
    /// 4. 版本落后 → 备份后分步升级。
    pub async fn load(path: impl Into<PathBuf>) -> Result<Self, sealantern_infra::fs::FsError> {
        let path = path.into();

        // 旧版嵌套格式检测与迁移（锁内执行，迁移前自动备份旧文件）
        migrate_legacy_format(&path).await?;

        let inner = match ConfigFile::load(&path).await {
            Ok(cf) => cf,
            Err(e) if is_not_found(&e) => {
                // 文件不存在，创建默认配置
                ConfigFile::load_or_create(&path, AppSettings::default()).await?
            }
            Err(e) if is_corrupt(&e) => {
                // 文件内容损坏（非法 JSON / 编码错误 / 超限）：备份隔离后恢复默认
                let backup = quarantine_corrupt_file(&path).await?;
                observability::config_settings_corrupt_recovered(&path, &backup);
                ConfigFile::load_or_create(&path, AppSettings::default()).await?
            }
            Err(e) => {
                // 锁冲突、IO 错误等：直接传播，不能把用户配置当损坏隔离
                return Err(e);
            }
        };

        let mut mgr = Self { inner, path: path.clone() };

        // 版本迁移：分步升级，迁移前备份
        let version = mgr.inner.get().config_version;
        if version < CURRENT_CONFIG_VERSION {
            let backup = backup_settings_file(&path).await?;
            let mut settings = mgr.inner.get().clone();
            upgrade_settings(&mut settings, version);
            mgr.inner.set(settings);
            mgr.inner.save(false).await?;
            observability::config_settings_version_upgraded(
                &path,
                version,
                CURRENT_CONFIG_VERSION,
                &backup,
            );
        }

        observability::config_settings_loaded(&path, mgr.inner.get().config_version);

        Ok(mgr)
    }

    /// 获取当前设置的只读引用
    pub fn get(&self) -> &AppSettings {
        self.inner.get()
    }

    /// 更新持久化的 Java 检测结果。
    ///
    /// Java 信息通过设置文件统一保存，旧缓存缺失置信度字段时由 serde
    /// 使用 0 补齐，并在配置版本升级时通过现有备份和原子写入流程保存。
    pub async fn update_java_cache(
        &mut self,
        installations: Vec<JavaInfo>,
    ) -> Result<UpdateResult, sealantern_infra::fs::FsError> {
        let partial = PartialAppSettings {
            cached_java_list: Some(installations),
            ..PartialAppSettings::default()
        };
        self.update_partial(partial).await
    }

    /// 全量替换设置并持久化
    /// 持久化失败时回滚内存状态
    pub async fn update(
        &mut self,
        new: AppSettings,
    ) -> Result<UpdateResult, sealantern_infra::fs::FsError> {
        let old = self.inner.get().clone();
        let changed_groups = old.changed_groups(&new);
        self.inner.set(new);
        match self.inner.save(false).await {
            Ok(()) => {
                observability::config_settings_updated(&self.path, &changed_groups);
                Ok(UpdateResult {
                    settings: self.inner.get().clone(),
                    changed_groups,
                })
            }
            Err(e) => {
                self.inner.set(old);
                observability::config_settings_persist_failed(&self.path, "update", &e);
                Err(e)
            }
        }
    }

    /// 部分更新（只传需要改的字段）
    /// 持久化失败时回滚内存状态
    pub async fn update_partial(
        &mut self,
        partial: PartialAppSettings,
    ) -> Result<UpdateResult, sealantern_infra::fs::FsError> {
        let old = self.inner.get().clone();
        self.inner.update(|s| partial.merge_into(s));
        let changed_groups = old.changed_groups(self.inner.get());
        if changed_groups.is_empty() {
            return Ok(UpdateResult {
                settings: self.inner.get().clone(),
                changed_groups,
            });
        }
        match self.inner.save(false).await {
            Ok(()) => {
                observability::config_settings_partial_updated(&self.path, &changed_groups);
                Ok(UpdateResult {
                    settings: self.inner.get().clone(),
                    changed_groups,
                })
            }
            Err(e) => {
                self.inner.set(old);
                observability::config_settings_persist_failed(&self.path, "update_partial", &e);
                Err(e)
            }
        }
    }

    /// 重置为默认设置
    /// 持久化失败时回滚内存状态，与 `update`/`update_partial` 语义一致
    pub async fn reset(&mut self) -> Result<AppSettings, sealantern_infra::fs::FsError> {
        let old = self.inner.get().clone();
        let default = AppSettings::default();
        self.inner.set(default.clone());
        match self.inner.save(false).await {
            Ok(()) => {
                observability::config_settings_reset(&self.path);
                Ok(default)
            }
            Err(e) => {
                self.inner.set(old);
                observability::config_settings_persist_failed(&self.path, "reset", &e);
                Err(e)
            }
        }
    }

    /// 导出设置为 JSON 字符串
    pub fn export_json(&self) -> Result<String, sealantern_infra::fs::FsError> {
        let json = serde_json::to_string_pretty(self.inner.get()).map_err(|e| {
            sealantern_infra::fs::FsError::Serialization {
                format: "json",
                operation: "serialize settings",
                path: self.path.clone(),
                message: e.to_string(),
            }
        })?;
        observability::config_settings_exported(&self.path);
        Ok(json)
    }

    /// 从 JSON 字符串导入设置
    pub async fn import_json(
        &mut self,
        json: &str,
    ) -> Result<UpdateResult, sealantern_infra::fs::FsError> {
        let imported: AppSettings = serde_json::from_str(json).map_err(|e| {
            sealantern_infra::fs::FsError::Serialization {
                format: "json",
                operation: "deserialize settings",
                path: self.path.clone(),
                message: e.to_string(),
            }
        })?;
        let result = self.update(imported).await?;
        observability::config_settings_imported(&self.path, &result.changed_groups);
        Ok(result)
    }
}

/// 判断配置加载错误是否为"文件不存在"。
fn is_not_found(err: &FsError) -> bool {
    matches!(
        err,
        FsError::Io { source, .. } if source.kind() == std::io::ErrorKind::NotFound
    )
}

/// 判断配置加载错误是否属于"内容损坏"（可安全隔离重建）。
///
/// 仅当文件内容确实无法解析时才隔离重建；锁冲突、IO 权限错误等
/// 必须向上传播，不能把用户的真实配置当损坏处理。
fn is_corrupt(err: &FsError) -> bool {
    matches!(
        err,
        FsError::Serialization { .. }
            | FsError::Encoding { .. }
            | FsError::DataLimitExceeded { .. }
    )
}

/// 将损坏的配置文件重命名隔离，返回隔离后的路径。
async fn quarantine_corrupt_file(path: &Path) -> Result<PathBuf, FsError> {
    let timestamp = timestamp_ms();
    let pid = std::process::id();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("settings");
    let backup = path.with_file_name(format!("{file_name}.corrupt-{timestamp}-{pid}"));
    tokio::fs::rename(path, &backup)
        .await
        .map_err(|e| FsError::Io {
            operation: "quarantine corrupt settings",
            path: backup.clone(),
            source: e,
        })?;
    Ok(backup)
}

/// 备份设置文件，返回备份路径。
async fn backup_settings_file(path: &Path) -> Result<PathBuf, FsError> {
    let timestamp = timestamp_ms();
    let pid = std::process::id();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("settings");
    let backup = path.with_file_name(format!("{file_name}.bak-{timestamp}-{pid}"));
    tokio::fs::copy(path, &backup)
        .await
        .map_err(|e| FsError::Io {
            operation: "backup settings",
            path: backup.clone(),
            source: e,
        })?;
    Ok(backup)
}

/// 当前时间戳（毫秒）。
fn timestamp_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// 获取与 `ConfigFile` 一致的配置文件锁（进程内异步锁 + 跨进程文件锁）。
async fn lock_config_file(path: &Path) -> Result<ConfigLockGuard, FsError> {
    let resource = process_lock_registry()
        .resource(path)
        .map_err(|error| FsError::Task {
            operation: "coordinate config access",
            message: error.to_string(),
        })?;
    let process_guard = resource.write().await;
    let lock_path = path.to_path_buf();
    let file_lock = tokio::task::spawn_blocking(move || FileLock::try_acquire(&lock_path))
        .await
        .map_err(|error| FsError::Task {
            operation: "acquire config file lock",
            message: error.to_string(),
        })??;
    Ok(ConfigLockGuard {
        _file_lock: file_lock,
        _process_guard: process_guard,
    })
}

/// 检测并迁移旧版嵌套配置格式（v1：`{ version, preferences: {...} }`）。
///
/// 新配置是扁平结构，serde 会静默忽略旧版的 `preferences` 对象并用默认值
/// 填充缺失字段——若不显式迁移，用户的语言/主题/开发者模式会被覆盖丢失。
/// 迁移在文件锁保护下执行，迁移前先备份原文件，成功后原子写入扁平格式。
async fn migrate_legacy_format(path: &Path) -> Result<bool, FsError> {
    // 锁外快速路径：文件不可读或不是旧版格式时直接返回。
    // 文件不存在（首次启动）属正常情况，不记录事件。
    let content = match read_string_limited(path, CONFIG_READ_LIMIT).await {
        Ok(c) => c,
        Err(_) => return Ok(false),
    };
    if !is_legacy_format(&content) {
        return Ok(false);
    }

    // 迁移全程持锁，防止与其它进程/任务的读写交错
    let _guard = lock_config_file(path).await?;

    // 锁内重读并重新校验，避免基于锁外快照覆盖并发写入：
    // 若另一进程已抢先完成迁移（内容已变为新格式），这里直接跳过
    let content = match read_string_limited(path, CONFIG_READ_LIMIT).await {
        Ok(c) => c,
        Err(e) => {
            // 锁内读取失败属异常，记录事件后跳过迁移
            observability::config_legacy_settings_migrate_failed(path, &e);
            return Ok(false);
        }
    };
    if !is_legacy_format(&content) {
        return Ok(false);
    }

    // 解析旧版格式
    let legacy: LegacyAppConfig = serde_json::from_str(&content).map_err(|e| {
        let error = FsError::Serialization {
            format: "json",
            operation: "decode legacy settings",
            path: path.to_path_buf(),
            message: e.to_string(),
        };
        observability::config_legacy_settings_migrate_failed(path, &error);
        error
    })?;

    // 迁移前备份旧文件（时间戳 + 进程 ID，避免跨进程同名冲突）
    let _backup = backup_settings_file(path)
        .await
        .inspect_err(|e| observability::config_legacy_settings_migrate_failed(path, e))?;

    // 旧值合并到新扁平结构，其余字段保持默认值
    let defaults = AppSettings::default();
    let settings = AppSettings {
        config_version: CURRENT_CONFIG_VERSION,
        language: legacy.preferences.language.unwrap_or(defaults.language),
        theme: legacy.preferences.theme.unwrap_or(defaults.theme),
        developer_mode: legacy
            .preferences
            .developer_mode
            .unwrap_or(defaults.developer_mode),
        ..defaults
    };

    // 原子写入新格式
    let json = serde_json::to_string_pretty(&settings).map_err(|e| {
        let error = FsError::Serialization {
            format: "json",
            operation: "encode migrated settings",
            path: path.to_path_buf(),
            message: e.to_string(),
        };
        observability::config_legacy_settings_migrate_failed(path, &error);
        error
    })?;
    write_atomic(path, json.as_bytes())
        .await
        .inspect_err(|e| observability::config_legacy_settings_migrate_failed(path, e))?;

    observability::config_legacy_settings_migrated(path);
    Ok(true)
}

/// 判断 JSON 内容是否为旧版嵌套格式（存在 `preferences` 对象）。
fn is_legacy_format(content: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(content)
        .map(|v| v.get("preferences").is_some_and(|p| p.is_object()))
        .unwrap_or(false)
}

/// 将设置从 `from_version` 分步升级到当前版本。
///
/// 未来新增结构变更时，在循环内按版本号补充分支迁移，
/// 版本号提升与字段迁移保持同步。
fn upgrade_settings(settings: &mut AppSettings, from_version: u32) {
    let mut version = from_version;
    while version < CURRENT_CONFIG_VERSION {
        // v0 → v1：扁平结构首次引入，此前旧版数据由 legacy 迁移处理。
        // v1 → v2：Java 缓存新增置信度字段，缺失值由 serde 默认补齐。
        version += 1;
    }
    settings.config_version = CURRENT_CONFIG_VERSION;
}

#[cfg(test)]
mod tests {
    use super::SettingsManager;
    use crate::config::{AppSettings, JavaInfo, PartialAppSettings};

    #[tokio::test]
    async fn empty_partial_update_does_not_rewrite_settings() {
        let root = tempfile::tempdir().expect("temporary config directory should be created");
        let path = root.path().join("settings.json");
        let original =
            serde_json::to_string(&AppSettings::default()).expect("settings should serialize");
        tokio::fs::write(&path, &original)
            .await
            .expect("settings fixture should be written");
        let mut manager = SettingsManager::load(&path)
            .await
            .expect("settings should load");

        let result = manager
            .update_partial(PartialAppSettings::default())
            .await
            .expect("empty update should succeed");

        assert!(result.changed_groups.is_empty());
        let persisted = tokio::fs::read_to_string(&path)
            .await
            .expect("settings should remain readable");
        assert_eq!(persisted, original);
    }

    #[tokio::test]
    async fn java_cache_persists_confidence() {
        let root = tempfile::tempdir().expect("temporary config directory should be created");
        let path = root.path().join("settings.json");
        let mut manager = SettingsManager::load(&path)
            .await
            .expect("settings should load");

        manager
            .update_java_cache(vec![JavaInfo {
                path: "/opt/jdk/bin/java".to_string(),
                version: "21.0.1".to_string(),
                vendor: "OpenJDK".to_string(),
                is_64bit: true,
                major_version: 21,
                confidence: 87,
            }])
            .await
            .expect("Java cache should persist");

        let reloaded = SettingsManager::load(&path)
            .await
            .expect("persisted settings should reload");
        assert_eq!(reloaded.get().cached_java_list[0].confidence, 87);
    }
}
