//! 旧版实例配置迁移服务实现。
//!
//! 实现 [`crate::port::LegacyMigrationService`]：检测旧版集中式配置
//! `sea_lantern_servers.json`，并在用户确认后把其中的服务器登记为新版
//! 受管实例（每个目录一份 `sl.json` + 设置注册表的信任/附加目录登记）。
//!
//! 设计的三个关键点：
//! - **只读检测**：`status` 完全不写盘，启动流程可以放心调用；
//! - **复用实例创建语义**：写文档后走与 `create` 相同的登记步骤（注册附加
//!   目录、解除忽略、登记信任），失败则回滚已写文档；
//! - **一次性提示**：迁移成功后旧文件被重命名归档；用户选择「不再提示」
//!   时记录到设置（`legacy_migration_dismissed`），二者都不会破坏旧数据。

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;

use sealantern_contract::LegacyMigrationServiceError;
use sealantern_contract::legacy_migration::{
    LegacyMigrationEntry, LegacyMigrationEntryState, LegacyMigrationReport,
    LegacyMigrationSkipReason, LegacyMigrationStatus, MigratedLegacyInstance, SkippedLegacyInstance,
};
use sealantern_feature::config::instance::{
    DocumentStore, InstanceDocument, LEGACY_SERVERS_FILE_NAME, LegacyConfigError,
    LegacyServerInstance, has_document, legacy_servers_path, read_legacy_servers,
    register_extra_dir, trust_and_unignore,
};
use sealantern_infra::platform::AppLayout;
use uuid::Uuid;

use crate::error::LegacyMigrationError;
use crate::port::{InstanceService, LegacyMigrationService};
use crate::service::{CoreInstanceService, CoreSettingsService};

/// 基于实例与设置服务的旧版配置迁移实现。
pub struct CoreLegacyMigrationService {
    instance: Arc<CoreInstanceService>,
    settings: Arc<CoreSettingsService>,
    /// 串行化迁移：两个并发的 `migrate` 会各自读到同一批旧条目并重复写文档。
    migration_ops: tokio::sync::Mutex<()>,
}

impl CoreLegacyMigrationService {
    /// 以共享的实例与设置服务构造迁移服务。
    pub fn new(instance: Arc<CoreInstanceService>, settings: Arc<CoreSettingsService>) -> Self {
        Self { instance, settings, migration_ops: tokio::sync::Mutex::new(()) }
    }

    /// 旧配置文件路径（主配置目录下）。
    async fn legacy_path(&self) -> Result<PathBuf, LegacyMigrationError> {
        let layout = self.settings.layout().await?;
        Ok(legacy_servers_path(layout.config_dir()))
    }

    /// 读取旧条目；文件不存在视为「没有旧数据」而不是错误。
    async fn legacy_entries(&self) -> Result<Vec<LegacyServerInstance>, LegacyMigrationError> {
        let path = self.legacy_path().await?;
        match read_legacy_servers(&path).await {
            Ok(entries) => Ok(entries),
            Err(LegacyConfigError::NotFound { .. }) => Ok(Vec::new()),
            Err(error @ LegacyConfigError::Invalid { .. }) => {
                tracing::warn!(
                    target: "sealantern.application.legacy_migration",
                    error = %error,
                    "legacy server list could not be parsed"
                );
                Ok(Vec::new())
            }
            Err(LegacyConfigError::Storage { source }) => Err(LegacyMigrationError::Io { source }),
        }
    }

    /// 只读检查单个旧条目的迁移可行性。
    async fn entry_state(entry: &LegacyServerInstance) -> LegacyMigrationEntryState {
        let directory = match entry.directory() {
            Ok(directory) => directory,
            Err(_) => return LegacyMigrationEntryState::Invalid,
        };
        match tokio::fs::metadata(&directory).await {
            Ok(metadata) if metadata.is_dir() => {}
            _ => return LegacyMigrationEntryState::MissingDirectory,
        }
        if has_document(&directory) {
            return LegacyMigrationEntryState::AlreadyMigrated;
        }
        match entry.to_document() {
            Ok(mapped) if mapped.document.validate().is_ok() => {
                LegacyMigrationEntryState::Migratable
            }
            _ => LegacyMigrationEntryState::Invalid,
        }
    }

    /// 条目显示名（空名回落到目录名）。
    fn entry_name(entry: &LegacyServerInstance) -> String {
        let trimmed = entry.name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
        Path::new(&entry.path)
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.trim().is_empty())
            .unwrap_or(entry.id.as_str())
            .to_string()
    }

    /// 当前已被占用的实例 id，用于迁移时避免 id 冲突。
    ///
    /// 只取受信任实例：未信任目录上的 `sl.json` 会被 `has_document` 判为
    /// 「已迁移」而跳过，不会参与本条冲突判定。发现过程出错时按「无占用」
    /// 继续——单条迁移失败由逐条结果报告，不应整体阻断。
    async fn occupied_ids(&self) -> HashSet<String> {
        match self.instance.list().await {
            Ok(instances) => instances
                .iter()
                .map(|instance| instance.id.as_str().to_string())
                .collect(),
            Err(error) => {
                tracing::warn!(
                    target: "sealantern.application.legacy_migration",
                    error = %error,
                    "failed to list existing instances before migration"
                );
                HashSet::new()
            }
        }
    }

    /// 写入一份实例文档并完成名单登记；失败时回滚文档。
    async fn register_instance(
        &self,
        layout: &AppLayout,
        directory: &Path,
        document: InstanceDocument,
    ) -> Result<(), LegacyMigrationError> {
        let path = DocumentStore::path_for(directory);
        let store = DocumentStore::create(path.clone(), document)
            .await
            .map_err(|source| LegacyMigrationError::DocumentWrite {
                directory: directory.to_path_buf(),
                source,
            })?;
        let id = store.get().id.as_str().to_string();

        let registration = async {
            let mut manager = self.settings.lock_manager().await?;
            if !is_managed_dir(directory, layout) {
                register_extra_dir(&mut manager, directory).await?;
            }
            trust_and_unignore(&mut manager, &id, directory).await?;
            Ok::<(), crate::error::SettingsError>(())
        }
        .await;

        if let Err(error) = registration {
            if let Err(rollback) = DocumentStore::remove(path.clone()).await {
                tracing::warn!(
                    target: "sealantern.application.legacy_migration",
                    dir = %directory.display(),
                    error = %rollback,
                    "failed to roll back instance document after registration failure"
                );
            }
            return Err(LegacyMigrationError::Settings { source: error });
        }
        Ok(())
    }

    /// 迁移成功后归档旧文件：重命名为 `<name>.migrated[-<ts>]`。
    ///
    /// 归档而非删除，保留用户原始数据；若同名的归档文件已存在，追加时间戳
    /// 避免覆盖。归档失败不影响已完成的迁移——只在日志中告警并返回
    /// `false`，下次启动会再次提示（旧文件仍在），用户可重试。
    async fn archive_legacy_file(&self, path: &Path) -> bool {
        if !tokio::fs::try_exists(path).await.unwrap_or(false) {
            return false;
        }
        let mut target = path.with_file_name(format!("{LEGACY_SERVERS_FILE_NAME}.migrated"));
        if tokio::fs::try_exists(&target).await.unwrap_or(false) {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_millis())
                .unwrap_or_default();
            target = path.with_file_name(format!(
                "{LEGACY_SERVERS_FILE_NAME}.migrated-{timestamp}-{}",
                Uuid::new_v4()
            ));
        }
        match tokio::fs::rename(path, &target).await {
            Ok(()) => true,
            // 并发进程已完成迁移并归档：视为已归档。
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => {
                tracing::warn!(
                    target: "sealantern.application.legacy_migration",
                    path = %path.display(),
                    target = %target.display(),
                    error = %error,
                    "failed to archive legacy server list after migration"
                );
                false
            }
        }
    }
}

#[async_trait]
impl LegacyMigrationService for CoreLegacyMigrationService {
    async fn status(&self) -> Result<LegacyMigrationStatus, LegacyMigrationServiceError> {
        let dismissed = self
            .settings
            .legacy_migration_dismissed()
            .await
            .map_err(LegacyMigrationError::from)
            .map_err(LegacyMigrationServiceError::from)?;
        let entries = self.legacy_entries().await?;
        if entries.is_empty() {
            return Ok(LegacyMigrationStatus::empty(dismissed));
        }

        let mut views = Vec::with_capacity(entries.len());
        let mut migratable = 0usize;
        for entry in &entries {
            let state = Self::entry_state(entry).await;
            if state == LegacyMigrationEntryState::Migratable {
                migratable += 1;
            }
            views.push(LegacyMigrationEntry {
                name: Self::entry_name(entry),
                directory: entry.directory().unwrap_or_default(),
                state,
            });
        }

        // 「没有服务器」才提示：已有受信任实例说明列表不为空，旧文件只是历史
        // 残留（或用户主动保留的备份），不应再打扰。
        //
        // 发现失败时按「没有已管理实例」继续：检测本身是只读辅助能力，
        // 不应因某个目录不可读就让启动流程拿不到结果；真正的迁移失败会
        // 在用户确认后由 `migrate` 逐条报告。
        let has_trusted = match self.instance.list().await {
            Ok(instances) => !instances.is_empty(),
            Err(error) => {
                tracing::warn!(
                    target: "sealantern.application.legacy_migration",
                    error = %error,
                    "failed to list instances while detecting legacy migration"
                );
                false
            }
        };

        Ok(LegacyMigrationStatus {
            available: !dismissed && !has_trusted && migratable > 0,
            dismissed,
            entries: views,
        })
    }

    async fn migrate(&self) -> Result<LegacyMigrationReport, LegacyMigrationServiceError> {
        let _ops = self.migration_ops.lock().await;
        let layout = self
            .settings
            .layout()
            .await
            .map_err(LegacyMigrationError::from)?;
        let legacy_path = legacy_servers_path(layout.config_dir());
        let entries = match read_legacy_servers(&legacy_path).await {
            Ok(entries) => entries,
            Err(LegacyConfigError::NotFound { .. }) => {
                return Err(LegacyMigrationServiceError::NothingToMigrate);
            }
            Err(error @ LegacyConfigError::Invalid { .. }) => {
                tracing::warn!(
                    target: "sealantern.application.legacy_migration",
                    error = %error,
                    "legacy server list could not be parsed"
                );
                return Err(LegacyMigrationServiceError::InvalidLegacyData);
            }
            Err(LegacyConfigError::Storage { source }) => {
                return Err(LegacyMigrationServiceError::from(
                    LegacyMigrationError::Io { source },
                ));
            }
        };
        if entries.is_empty() {
            return Err(LegacyMigrationServiceError::NothingToMigrate);
        }

        let mut occupied = self.occupied_ids().await;
        let mut migrated = Vec::new();
        let mut skipped = Vec::new();

        for entry in &entries {
            let name = Self::entry_name(entry);
            let directory = match entry.directory() {
                Ok(directory) => directory,
                Err(_) => {
                    skipped.push(SkippedLegacyInstance {
                        name,
                        directory: PathBuf::new(),
                        reason: LegacyMigrationSkipReason::Invalid,
                    });
                    continue;
                }
            };
            if !tokio::fs::metadata(&directory)
                .await
                .map(|metadata| metadata.is_dir())
                .unwrap_or(false)
            {
                skipped.push(SkippedLegacyInstance {
                    name,
                    directory,
                    reason: LegacyMigrationSkipReason::MissingDirectory,
                });
                continue;
            }
            if has_document(&directory) {
                skipped.push(SkippedLegacyInstance {
                    name,
                    directory,
                    reason: LegacyMigrationSkipReason::AlreadyMigrated,
                });
                continue;
            }

            let mapped = match entry.to_document() {
                Ok(mapped) => mapped,
                Err(error) => {
                    tracing::warn!(
                        target: "sealantern.application.legacy_migration",
                        dir = %directory.display(),
                        error = %error,
                        "legacy server entry cannot be mapped"
                    );
                    skipped.push(SkippedLegacyInstance {
                        name,
                        directory,
                        reason: LegacyMigrationSkipReason::Invalid,
                    });
                    continue;
                }
            };
            if let Err(error) = mapped.document.validate() {
                tracing::warn!(
                    target: "sealantern.application.legacy_migration",
                    dir = %directory.display(),
                    error = %error,
                    "legacy server entry produced an invalid document"
                );
                skipped.push(SkippedLegacyInstance {
                    name,
                    directory,
                    reason: LegacyMigrationSkipReason::Invalid,
                });
                continue;
            }

            // id 冲突（旧配置重复 id / 与现有实例撞车）时重新分配，保持
            // 迁移可继续；目录本身不变，实例身份由新文档决定。
            let mut document = mapped.document;
            if occupied.contains(document.id.as_str()) {
                match sealantern_core::instance::InstanceId::new(Uuid::new_v4().to_string()) {
                    Ok(new_id) => document.id = new_id,
                    Err(error) => {
                        tracing::warn!(
                            target: "sealantern.application.legacy_migration",
                            error = %error,
                            "failed to regenerate a conflicting instance id"
                        );
                        skipped.push(SkippedLegacyInstance {
                            name,
                            directory,
                            reason: LegacyMigrationSkipReason::Invalid,
                        });
                        continue;
                    }
                }
            }

            let instance_id = document.id.as_str().to_string();
            match self.register_instance(&layout, &directory, document).await {
                Ok(()) => {
                    occupied.insert(instance_id.clone());
                    migrated.push(MigratedLegacyInstance {
                        instance_id,
                        name,
                        directory,
                    });
                }
                Err(error) => {
                    tracing::warn!(
                        target: "sealantern.application.legacy_migration",
                        dir = %directory.display(),
                        error = %error,
                        "legacy server migration failed"
                    );
                    skipped.push(SkippedLegacyInstance {
                        name,
                        directory,
                        reason: LegacyMigrationSkipReason::CreateFailed,
                    });
                }
            }
        }

        if migrated.is_empty() {
            return Err(LegacyMigrationServiceError::NothingToMigrate);
        }

        let legacy_file_archived = self.archive_legacy_file(&legacy_path).await;

        tracing::info!(
            target: "sealantern.application.legacy_migration",
            migrated = migrated.len(),
            skipped = skipped.len(),
            archived = legacy_file_archived,
            "legacy server configuration migrated"
        );
        Ok(LegacyMigrationReport { migrated, skipped, legacy_file_archived })
    }

    async fn dismiss(&self) -> Result<(), LegacyMigrationServiceError> {
        self.settings
            .dismiss_legacy_migration()
            .await
            .map_err(LegacyMigrationError::from)
            .map_err(LegacyMigrationServiceError::from)
    }
}

/// 目录是否位于主资源 `instances/` 容器下（词法比较，不做 IO）。
fn is_managed_dir(dir: &Path, layout: &AppLayout) -> bool {
    dir.parent()
        .map(|parent| parent == layout.instances_dir().as_path())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use sealantern_contract::legacy_migration::LegacyMigrationEntryState;
    use sealantern_feature::config::settings::SettingsManager;
    use tempfile::TempDir;

    use super::*;
    use crate::service::{CoreInstanceService, CoreSettingsService};

    /// 构造以临时目录为配置根的迁移服务。
    async fn test_service(root: &Path) -> CoreLegacyMigrationService {
        let manager = SettingsManager::load(root.join("settings.json"))
            .await
            .expect("load settings manager");
        let settings = Arc::new(CoreSettingsService::with_manager(manager));
        let instance = Arc::new(CoreInstanceService::new(settings.clone()));
        CoreLegacyMigrationService::new(instance, settings)
    }

    /// 写一份旧版 `sea_lantern_servers.json`。
    async fn write_legacy(root: &Path, body: &str) {
        tokio::fs::write(legacy_servers_path(root), body).await.expect("write legacy list");
    }

    fn legacy_entry(id: &str, name: &str, dir: &Path) -> String {
        format!(
            r#"{{"id":"{id}","name":"{name}","core_type":"paper","core_version":"1.20.4","mc_version":"1.20.4","path":"{}","jar_path":"{}","startup_mode":"jar","java_path":"","max_memory":4096,"min_memory":1024,"jvm_args":[],"port":25565,"created_at":1700000000}}"#,
            dir.display(),
            dir.join("server.jar").display()
        )
    }

    #[tokio::test]
    async fn empty_when_no_legacy_file() {
        let root = TempDir::new().unwrap();
        let service = test_service(root.path()).await;

        let status = service.status().await.expect("status");

        assert!(!status.available);
        assert!(status.entries.is_empty());
    }

    #[tokio::test]
    async fn status_reports_migratable_entry_without_writing() {
        let root = TempDir::new().unwrap();
        let server_dir = root.path().join("srv").join("paper");
        tokio::fs::create_dir_all(&server_dir).await.unwrap();
        write_legacy(
            root.path(),
            &format!("[{}]", legacy_entry("abc", "Paper", &server_dir)),
        )
        .await;
        let service = test_service(root.path()).await;

        let status = service.status().await.expect("status");

        assert!(status.available);
        assert_eq!(status.entries.len(), 1);
        assert_eq!(status.entries[0].state, LegacyMigrationEntryState::Migratable);
        assert!(!has_document(&server_dir), "status must not write sl.json");
    }

    #[tokio::test]
    async fn migrate_writes_document_registers_and_archives() {
        let root = TempDir::new().unwrap();
        let server_dir = root.path().join("srv").join("paper");
        tokio::fs::create_dir_all(&server_dir).await.unwrap();
        write_legacy(
            root.path(),
            &format!("[{}]", legacy_entry("abc", "Paper", &server_dir)),
        )
        .await;
        let service = test_service(root.path()).await;

        let report = service.migrate().await.expect("migrate");

        assert!(report.has_migrated());
        assert_eq!(report.migrated.len(), 1);
        assert_eq!(report.migrated[0].instance_id, "abc");
        assert!(report.legacy_file_archived);
        assert!(has_document(&server_dir), "sl.json should be written");
        assert!(
            !legacy_servers_path(root.path()).exists(),
            "legacy file should be archived away"
        );
        let archived = root.path().join(format!("{LEGACY_SERVERS_FILE_NAME}.migrated"));
        assert!(archived.is_file(), "archived legacy file should remain on disk");

        let registry = service.settings.registry().await.expect("registry");
        assert!(registry.trusted_instances.iter().any(|id| id == "abc"));
        assert!(
            registry
                .extra_server_dirs
                .iter()
                .any(|dir| dir.ends_with("paper")),
            "directory outside instances/ must be registered as extra"
        );

        let status = service.status().await.expect("status after migrate");
        assert!(!status.available, "no prompt once migrated");
    }

    #[tokio::test]
    async fn migrate_skips_missing_and_already_migrated() {
        let root = TempDir::new().unwrap();
        let migrated_dir = root.path().join("srv").join("done");
        let missing_dir = root.path().join("srv").join("gone");
        tokio::fs::create_dir_all(&migrated_dir).await.unwrap();
        tokio::fs::write(migrated_dir.join("sl.json"), "{}").await.unwrap();
        write_legacy(
            root.path(),
            &format!(
                "[{},{}]",
                legacy_entry("done", "Done", &migrated_dir),
                legacy_entry("gone", "Gone", &missing_dir)
            ),
        )
        .await;
        let service = test_service(root.path()).await;

        let error = service.migrate().await.expect_err("nothing migratable");

        assert_eq!(error, LegacyMigrationServiceError::NothingToMigrate);

        let status = service.status().await.expect("status");
        let states: Vec<_> = status.entries.iter().map(|entry| entry.state).collect();
        assert!(states.contains(&LegacyMigrationEntryState::AlreadyMigrated));
        assert!(states.contains(&LegacyMigrationEntryState::MissingDirectory));
        assert!(!status.available, "nothing migratable means no prompt");
    }

    #[tokio::test]
    async fn dismissal_is_persisted_and_idempotent() {
        let root = TempDir::new().unwrap();
        let server_dir = root.path().join("srv").join("paper");
        tokio::fs::create_dir_all(&server_dir).await.unwrap();
        write_legacy(
            root.path(),
            &format!("[{}]", legacy_entry("abc", "Paper", &server_dir)),
        )
        .await;
        let service = test_service(root.path()).await;

        service.dismiss().await.expect("dismiss");
        service.dismiss().await.expect("dismiss twice");

        let status = service.status().await.expect("status");
        assert!(status.dismissed);
        assert!(!status.available);
        assert_eq!(status.entries.len(), 1, "dismissal keeps entries listed");

        let reloaded = test_service(root.path()).await;
        assert!(
            reloaded.settings.legacy_migration_dismissed().await.expect("read flag"),
            "dismissal must survive reload"
        );
    }

    #[tokio::test]
    async fn invalid_legacy_file_is_reported_as_invalid_data() {
        let root = TempDir::new().unwrap();
        write_legacy(root.path(), "{ not json").await;
        let service = test_service(root.path()).await;

        let error = service.migrate().await.expect_err("invalid data");

        assert_eq!(error, LegacyMigrationServiceError::InvalidLegacyData);
    }

    #[tokio::test]
    async fn existing_trusted_instance_suppresses_prompt() {
        let root = TempDir::new().unwrap();
        let managed_dir = root.path().join("instances").join("existing");
        tokio::fs::create_dir_all(&managed_dir).await.unwrap();
        let service = test_service(root.path()).await;
        service
            .settings
            .lock_manager()
            .await
            .unwrap()
            .update_registry(|registry| {
                registry.trusted_instances.push("existing".to_string());
                true
            })
            .await
            .expect("register trusted instance");
        tokio::fs::write(
            managed_dir.join("sl.json"),
            r#"{"schema_version":1,"id":"existing","name":"Existing","core":{"type":"paper","version":"1.20.4"},"startup":{"mode":"jar","target":"server.jar","port":25565}}"#,
        )
        .await
        .unwrap();

        let server_dir = root.path().join("srv").join("paper");
        tokio::fs::create_dir_all(&server_dir).await.unwrap();
        write_legacy(
            root.path(),
            &format!("[{}]", legacy_entry("abc", "Paper", &server_dir)),
        )
        .await;

        let status = service.status().await.expect("status");

        assert!(!status.available, "existing instances mean the list is not empty");
        assert_eq!(status.entries[0].state, LegacyMigrationEntryState::Migratable);
    }
}
