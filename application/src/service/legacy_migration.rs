//! 旧版实例配置迁移服务实现。
//!
//! 实现 [`crate::port::LegacyMigrationService`]：检测旧版集中式配置
//! `sea_lantern_servers.json`，并在用户确认后把其中的服务器登记为新版
//! 受管实例（每个目录一份 `sl.json` + 设置注册表的信任/附加目录登记）。
//!
//! 设计的三个关键点：
//! - **只读检测**：`status` 完全不写盘，供设置页在用户手动触发前预览；
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
    LegacyMigrationSkipReason, LegacyMigrationStatus, MigratedLegacyInstance,
    SkippedLegacyInstance,
};
use sealantern_feature::config::instance::{
    DocumentStore, InstanceDocument, LEGACY_SERVERS_FILE_NAME, LegacyConfigError,
    LegacyServerInstance, LegacyServers, has_document, legacy_servers_candidates,
    read_legacy_servers, register_extra_dir, trust_and_unignore, unregister_extra_dir,
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
        Self {
            instance,
            settings,
            migration_ops: tokio::sync::Mutex::new(()),
        }
    }

    /// 解析实际使用的旧配置文件位置。
    ///
    /// 优先当前配置目录；1.2.x 在 Windows / macOS 上用的是带空格的
    /// `Sea Lantern` 目录，升级后旧文件仍留在那里，只看当前目录会永远
    /// 检测不到。返回 `None` 表示两处都没有旧文件。
    async fn resolve_legacy_path(&self) -> Result<Option<PathBuf>, LegacyMigrationError> {
        let layout = self.settings.layout().await?;
        for candidate in legacy_servers_candidates(layout.config_dir()) {
            if tokio::fs::try_exists(&candidate).await.unwrap_or(false) {
                return Ok(Some(candidate));
            }
        }
        Ok(None)
    }

    /// 读取旧条目；文件不存在视为「没有旧数据」而不是错误。
    async fn legacy_entries(&self) -> Result<LegacyServers, LegacyMigrationError> {
        let Some(path) = self.resolve_legacy_path().await? else {
            return Ok(LegacyServers::default());
        };
        match read_legacy_servers(&path).await {
            Ok(servers) => Ok(servers),
            Err(LegacyConfigError::NotFound { .. }) => Ok(LegacyServers::default()),
            Err(error @ LegacyConfigError::Invalid { .. }) => {
                tracing::warn!(
                    target: "sealantern.application.legacy_migration",
                    error = %error,
                    "legacy server list could not be parsed"
                );
                Ok(LegacyServers::default())
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

    /// 当前磁盘上已被占用的实例 id，用于迁移时避免 id 冲突。
    ///
    /// 取**全部**已发现的文档 id（受信任 + 待处理 + 冲突组），而不只是受信任
    /// 实例：迁移会写入受信任名单，若沿用某个未信任目录已占用的 id，发现层
    /// 会把两份同 id 文档都移出列表——用户看到的是「迁移成功但服务器消失」。
    ///
    /// 发现失败时直接报错而不是退回空集合：无法枚举已有 id 就没法安全地分配
    /// 新 id，静默继续只会制造覆盖或重复。
    async fn occupied_ids(
        &self,
    ) -> Result<(HashSet<String>, HashSet<String>), LegacyMigrationError> {
        let discovery = self
            .instance
            .discovery()
            .await
            .map_err(|source| LegacyMigrationError::Discovery { source })?;
        let mut occupied =
            HashSet::with_capacity(discovery.trusted.len() + discovery.pending.len());
        let trusted: HashSet<String> = discovery
            .trusted
            .iter()
            .map(|inst| inst.id.as_str().to_string())
            .collect();
        occupied.extend(trusted.iter().cloned());
        for pending in &discovery.pending {
            occupied.insert(pending.instance_id.clone());
        }
        for problem in &discovery.problems {
            if let Some(id) = &problem.problem.id {
                occupied.insert(id.clone());
            }
        }
        Ok((occupied, trusted))
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

        let registration = self.register_in_registry(layout, directory, &id).await;

        if let Err(error) = registration {
            // 撤销「附加目录」登记：它可能已在上一行落盘，否则会留下一个没有
            // 实例文档、却仍会被扫描的孤立目录。
            if !is_managed_dir(directory, layout) {
                self.rollback_extra_dir(directory).await;
            }
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

    /// 在设置名单中登记实例：受管目录外先登记附加目录，再登记信任并解除忽略。
    ///
    /// 两个原语都幂等，重复登记不会重复写入。
    async fn register_in_registry(
        &self,
        layout: &AppLayout,
        directory: &Path,
        id: &str,
    ) -> Result<(), LegacyMigrationError> {
        let extra_dir = !is_managed_dir(directory, layout);
        let mut manager = self
            .settings
            .lock_manager()
            .await
            .map_err(|source| LegacyMigrationError::Settings { source })?;
        if extra_dir {
            register_extra_dir(&mut manager, directory)
                .await
                .map_err(crate::error::SettingsError::from)
                .map_err(|source| LegacyMigrationError::Settings { source })?;
        }
        trust_and_unignore(&mut manager, id, directory)
            .await
            .map_err(crate::error::SettingsError::from)
            .map_err(|source| LegacyMigrationError::Settings { source })?;
        Ok(())
    }

    /// 尽力撤销附加目录登记，失败只告警。
    ///
    /// 回滚本身失败时保持「文档已删除、目录仍登记」——发现层会把这种目录当作
    /// 没有文档的裸目录，用户可以重新导入或忽略，不会造成数据丢失。
    async fn rollback_extra_dir(&self, directory: &Path) {
        match self.settings.lock_manager().await {
            Ok(mut manager) => {
                if let Err(error) = unregister_extra_dir(&mut manager, directory).await {
                    tracing::warn!(
                        target: "sealantern.application.legacy_migration",
                        dir = %directory.display(),
                        error = %error,
                        "failed to roll back extra server directory registration"
                    );
                }
            }
            Err(error) => {
                tracing::warn!(
                    target: "sealantern.application.legacy_migration",
                    dir = %directory.display(),
                    error = %error,
                    "failed to lock settings while rolling back extra server directory"
                );
            }
        }
    }

    /// 目录已有 `sl.json` 时尝试补完登记（崩溃在写入与登记之间的恢复路径）。
    ///
    /// 正常路径下 `DocumentStore::create` 与名单登记之间没有等待点；但进程若
    /// 在两者之间退出，重试时 `has_document` 会把它判为「已迁移」而跳过，于是
    /// 文档存在却永远不受信任。这里读回文档并补齐附加目录与信任登记——两个
    /// 原语都是幂等的，已在名单中的条目不会重复写入。
    async fn finalize_registration(
        &self,
        layout: &AppLayout,
        directory: &Path,
    ) -> Result<MigratedLegacyInstance, LegacyMigrationError> {
        let store = DocumentStore::load(DocumentStore::path_for(directory))
            .await
            .map_err(|source| LegacyMigrationError::DocumentWrite {
                directory: directory.to_path_buf(),
                source,
            })?;
        let document = store.get();
        let id = document.id.as_str().to_string();
        self.register_in_registry(layout, directory, &id).await?;

        Ok(MigratedLegacyInstance {
            instance_id: id,
            name: document.name.clone(),
            directory: directory.to_path_buf(),
        })
    }

    /// 迁移成功后归档旧文件：重命名为 `<name>.migrated[-<ts>]`。
    ///
    /// 归档而非删除，保留用户原始数据；若同名的归档文件已存在，追加时间戳
    /// 避免覆盖。归档失败不影响已完成的迁移——只在日志中告警并返回
    /// `false`，设置页仍会显示可迁移条目（旧文件仍在），用户可重试。
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
        let entries = self.legacy_entries().await?.entries;
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

        // 迁移提示只由用户在设置页手动触发（设置项「旧版实例配置迁移」），
        // 因此这里只报告「存在可迁移条目」这一事实，不再依据受信任实例数量
        // 猜测是否该弹窗——设置页需要在已有服务器时也能看到并手动迁移旧配置。
        Ok(LegacyMigrationStatus {
            available: !dismissed && migratable > 0,
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
        let Some(legacy_path) = self
            .resolve_legacy_path()
            .await
            .map_err(LegacyMigrationServiceError::from)?
        else {
            return Err(LegacyMigrationServiceError::NothingToMigrate);
        };
        let servers = match read_legacy_servers(&legacy_path).await {
            Ok(servers) => servers,
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
                return Err(LegacyMigrationServiceError::from(LegacyMigrationError::Io { source }));
            }
        };
        if servers.entries.is_empty() {
            return Err(LegacyMigrationServiceError::NothingToMigrate);
        }
        let entries = servers.entries;
        let unparsable = servers.unparsable;

        let (mut occupied, trusted) = self.occupied_ids().await?;
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
                // 文档已在但未登记信任：多半是上次迁移写完文档就中断了，
                // 补完登记即可恢复；已在受信任集合里的条目按「已迁移」跳过。
                match self.finalize_registration(&layout, &directory).await {
                    Ok(instance) if !trusted.contains(&instance.instance_id) => {
                        occupied.insert(instance.instance_id.clone());
                        migrated.push(instance);
                    }
                    Ok(_) => skipped.push(SkippedLegacyInstance {
                        name,
                        directory,
                        reason: LegacyMigrationSkipReason::AlreadyMigrated,
                    }),
                    Err(error) => {
                        tracing::warn!(
                            target: "sealantern.application.legacy_migration",
                            dir = %directory.display(),
                            error = %error,
                            "failed to complete registration for an existing instance document"
                        );
                        skipped.push(SkippedLegacyInstance {
                            name,
                            directory,
                            reason: LegacyMigrationSkipReason::CreateFailed,
                        });
                    }
                }
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
                    migrated.push(MigratedLegacyInstance { instance_id, name, directory });
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

        // 仍有条目没搬过去时保留旧文件，否则它们会随归档永久消失、无法重试：
        // - 无法解析的坏行，需要用户自行修复；
        // - 目录暂时缺失 / 创建失败 / 内容非法，修复后重试即可成功。
        //
        // 「目录已有 sl.json」是终态，不算未完成，不应因此长期留着重启提示。
        let retryable = skipped
            .iter()
            .any(|item| item.reason != LegacyMigrationSkipReason::AlreadyMigrated);
        let legacy_file_archived = if unparsable > 0 || retryable {
            tracing::warn!(
                target: "sealantern.application.legacy_migration",
                unparsable,
                skipped = skipped.len(),
                "legacy server list kept on disk because some entries were not migrated"
            );
            false
        } else {
            self.archive_legacy_file(&legacy_path).await
        };

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
    use sealantern_feature::config::instance::{LEGACY_APP_DIR_NAME, legacy_servers_path};
    use sealantern_feature::config::settings::SettingsManager;
    use tempfile::TempDir;

    use super::*;
    use crate::service::{CoreInstanceService, CoreSettingsService};

    /// 构造以临时目录为配置根的迁移服务。
    async fn test_service(root: &Path) -> CoreLegacyMigrationService {
        test_service_in_config_dir(root).await
    }

    /// 构造以指定配置目录（`settings.json` 所在目录）为根的迁移服务。
    async fn test_service_in_config_dir(config_dir: &Path) -> CoreLegacyMigrationService {
        let manager = SettingsManager::load(config_dir.join("settings.json"))
            .await
            .expect("load settings manager");
        let settings = Arc::new(CoreSettingsService::with_manager(manager));
        let instance = Arc::new(CoreInstanceService::new(settings.clone()));
        CoreLegacyMigrationService::new(instance, settings)
    }

    /// 写一份旧版 `sea_lantern_servers.json`。
    async fn write_legacy(root: &Path, body: &str) {
        tokio::fs::write(legacy_servers_path(root), body)
            .await
            .expect("write legacy list");
    }

    fn sl_json(id: &str, name: &str) -> String {
        serde_json::json!({
            "schema_version": 1,
            "id": id,
            "name": name,
            "core": {"type": "paper", "version": "1.20.4"},
            "startup": {"mode": "jar", "target": "server.jar", "port": 25565},
        })
        .to_string()
    }

    fn legacy_entry(id: &str, name: &str, dir: &Path) -> String {
        let entry = serde_json::json!({
            "id": id,
            "name": name,
            "core_type": "paper",
            "core_version": "1.20.4",
            "mc_version": "1.20.4",
            "path": dir,
            "jar_path": dir.join("server.jar"),
            "startup_mode": "jar",
            "java_path": "",
            "max_memory": 4096,
            "min_memory": 1024,
            "jvm_args": [],
            "port": 25565,
            "created_at": 1_700_000_000u64,
        });
        entry.to_string()
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
    async fn detects_legacy_file_in_previous_app_directory() {
        let root = TempDir::new().unwrap();
        let config_dir = root.path().join("SeaLantern");
        let previous_dir = root.path().join(LEGACY_APP_DIR_NAME);
        let server_dir = root.path().join("srv").join("paper");
        tokio::fs::create_dir_all(&config_dir).await.unwrap();
        tokio::fs::create_dir_all(&previous_dir).await.unwrap();
        tokio::fs::create_dir_all(&server_dir).await.unwrap();
        write_legacy(&previous_dir, &format!("[{}]", legacy_entry("abc", "Paper", &server_dir)))
            .await;
        let service = test_service_in_config_dir(&config_dir).await;

        let status = service.status().await.expect("status");

        assert!(status.available, "legacy file in previous app dir must be found");
        assert_eq!(status.entries.len(), 1);

        let report = service.migrate().await.expect("migrate");

        assert_eq!(report.migrated.len(), 1);
        assert!(report.legacy_file_archived);
        assert!(
            !legacy_servers_path(&previous_dir).exists(),
            "legacy file in previous app dir should be archived away"
        );
    }

    #[tokio::test]
    async fn status_reports_migratable_entry_without_writing() {
        let root = TempDir::new().unwrap();
        let server_dir = root.path().join("srv").join("paper");
        tokio::fs::create_dir_all(&server_dir).await.unwrap();
        write_legacy(root.path(), &format!("[{}]", legacy_entry("abc", "Paper", &server_dir)))
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
        write_legacy(root.path(), &format!("[{}]", legacy_entry("abc", "Paper", &server_dir)))
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
        let archived = root
            .path()
            .join(format!("{LEGACY_SERVERS_FILE_NAME}.migrated"));
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
    async fn malformed_entry_keeps_legacy_file_for_retry() {
        let root = TempDir::new().unwrap();
        let server_dir = root.path().join("srv").join("paper");
        tokio::fs::create_dir_all(&server_dir).await.unwrap();
        let good = legacy_entry("abc", "Paper", &server_dir);
        let body = format!("[{good},{}]", r#"{"id":"bad","path":42}"#);
        write_legacy(root.path(), &body).await;
        let service = test_service(root.path()).await;

        let report = service.migrate().await.expect("migrate");

        assert_eq!(report.migrated.len(), 1, "valid entry must still migrate");
        assert!(
            !report.legacy_file_archived,
            "legacy file must be kept while unparsable rows remain"
        );
        assert!(
            legacy_servers_path(root.path()).is_file(),
            "legacy file must stay on disk so the bad row can be fixed"
        );
    }

    #[tokio::test]
    async fn duplicate_id_with_untrusted_instance_is_reassigned() {
        let root = TempDir::new().unwrap();
        let existing_dir = root.path().join("instances").join("existing");
        let server_dir = root.path().join("srv").join("paper");
        for dir in [&existing_dir, &server_dir] {
            tokio::fs::create_dir_all(dir).await.unwrap();
        }
        // 未登记信任的目录：list() 看不到它，但迁移沿用它的 id 会让发现层把
        // 两份同 id 文档一起移出列表。
        tokio::fs::write(existing_dir.join("sl.json"), sl_json("abc", "Existing"))
            .await
            .unwrap();
        write_legacy(root.path(), &format!("[{}]", legacy_entry("abc", "Paper", &server_dir)))
            .await;
        let service = test_service(root.path()).await;

        let report = service.migrate().await.expect("migrate");

        assert_eq!(report.migrated.len(), 1);
        assert_ne!(
            report.migrated[0].instance_id, "abc",
            "must not reuse an id already present on disk"
        );

        let discovery = service.instance.discovery().await.expect("discovery");
        assert_eq!(discovery.trusted.len(), 1, "migrated instance must be visible");
        assert!(discovery.problems.is_empty(), "no duplicate-id problem should be left behind");
    }

    async fn trust(service: &CoreLegacyMigrationService, id: &str) {
        service
            .settings
            .lock_manager()
            .await
            .unwrap()
            .update_registry(|registry| {
                registry.trusted_instances.push(id.to_string());
                true
            })
            .await
            .expect("register trusted instance");
    }

    #[tokio::test]
    async fn existing_document_without_trust_is_registered() {
        let root = TempDir::new().unwrap();
        let server_dir = root.path().join("srv").join("paper");
        tokio::fs::create_dir_all(&server_dir).await.unwrap();
        // 模拟上次迁移写完 sl.json 后进程退出：文档在、信任与附加目录都缺。
        tokio::fs::write(server_dir.join("sl.json"), sl_json("abc", "Paper"))
            .await
            .unwrap();
        write_legacy(root.path(), &format!("[{}]", legacy_entry("abc", "Paper", &server_dir)))
            .await;
        let service = test_service(root.path()).await;

        let report = service.migrate().await.expect("migrate");

        assert_eq!(report.migrated.len(), 1, "interrupted migration must be resumed");
        assert_eq!(report.migrated[0].instance_id, "abc");

        let discovery = service.instance.discovery().await.expect("discovery");
        assert_eq!(discovery.trusted.len(), 1);
        assert_eq!(discovery.trusted[0].id.as_str(), "abc");
    }

    #[tokio::test]
    async fn rollback_extra_dir_removes_registration() {
        let root = TempDir::new().unwrap();
        let server_dir = root.path().join("srv").join("paper");
        tokio::fs::create_dir_all(&server_dir).await.unwrap();
        let service = test_service(root.path()).await;
        {
            let mut manager = service.settings.lock_manager().await.unwrap();
            register_extra_dir(&mut manager, &server_dir)
                .await
                .expect("register");
        }

        service.rollback_extra_dir(&server_dir).await;

        let registry = service.settings.registry().await.expect("registry");
        assert!(
            !registry
                .extra_server_dirs
                .iter()
                .any(|dir| dir == &server_dir),
            "rollback must drop the registered extra directory"
        );
    }

    #[tokio::test]
    async fn skipped_entry_keeps_legacy_file_for_retry() {
        let root = TempDir::new().unwrap();
        let server_dir = root.path().join("srv").join("paper");
        let missing_dir = root.path().join("srv").join("gone");
        tokio::fs::create_dir_all(&server_dir).await.unwrap();
        write_legacy(
            root.path(),
            &format!(
                "[{},{}]",
                legacy_entry("abc", "Paper", &server_dir),
                legacy_entry("gone", "Gone", &missing_dir)
            ),
        )
        .await;
        let service = test_service(root.path()).await;

        let report = service.migrate().await.expect("migrate");

        assert_eq!(report.migrated.len(), 1);
        assert_eq!(report.skipped.len(), 1);
        assert!(
            !report.legacy_file_archived,
            "missing directory may reappear, so the entry must stay retryable"
        );
        assert!(legacy_servers_path(root.path()).is_file());
    }

    #[tokio::test]
    async fn already_migrated_entry_still_allows_archiving() {
        let root = TempDir::new().unwrap();
        let done_dir = root.path().join("srv").join("done");
        let server_dir = root.path().join("srv").join("paper");
        for dir in [&done_dir, &server_dir] {
            tokio::fs::create_dir_all(dir).await.unwrap();
        }
        tokio::fs::write(done_dir.join("sl.json"), sl_json("done", "Done"))
            .await
            .unwrap();
        write_legacy(
            root.path(),
            &format!(
                "[{},{}]",
                legacy_entry("done", "Done", &done_dir),
                legacy_entry("abc", "Paper", &server_dir)
            ),
        )
        .await;
        let service = test_service(root.path()).await;
        // 真正「已迁移」是「文档在 + 已受信任」；只有文档属于中断的迁移。
        trust(&service, "done").await;

        let report = service.migrate().await.expect("migrate");

        assert_eq!(report.migrated.len(), 1);
        assert_eq!(report.skipped.len(), 1);
        assert!(
            report.legacy_file_archived,
            "already-migrated entries are terminal and must not block archiving"
        );
        assert!(!legacy_servers_path(root.path()).exists());
    }

    #[tokio::test]
    async fn migrate_skips_missing_and_already_migrated() {
        let root = TempDir::new().unwrap();
        let migrated_dir = root.path().join("srv").join("done");
        let missing_dir = root.path().join("srv").join("gone");
        tokio::fs::create_dir_all(&migrated_dir).await.unwrap();
        tokio::fs::write(migrated_dir.join("sl.json"), sl_json("done", "Done"))
            .await
            .unwrap();
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
        trust(&service, "done").await;

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
        write_legacy(root.path(), &format!("[{}]", legacy_entry("abc", "Paper", &server_dir)))
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
            reloaded
                .settings
                .legacy_migration_dismissed()
                .await
                .expect("read flag"),
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
    async fn status_is_independent_of_trusted_instances() {
        let root = TempDir::new().unwrap();
        let server_dir = root.path().join("srv").join("paper");
        tokio::fs::create_dir_all(&server_dir).await.unwrap();
        write_legacy(root.path(), &format!("[{}]", legacy_entry("abc", "Paper", &server_dir)))
            .await;
        let service = test_service(root.path()).await;

        // 故意把主实例目录弄成不可扫描的文件，模拟实例发现失败：
        // 迁移入口只依赖旧配置本身，不应被实例枚举拖垮。
        tokio::fs::remove_dir_all(root.path().join("instances"))
            .await
            .unwrap();
        tokio::fs::write(root.path().join("instances"), b"not a directory")
            .await
            .unwrap();

        let status = service.status().await.expect("status must not fail");

        assert!(status.available);
        assert_eq!(status.entries.len(), 1);
    }

    /// 迁移只在设置页由用户手动触发，所以已有实例时也必须照常报告可迁移条目。
    #[tokio::test]
    async fn existing_trusted_instance_does_not_hide_prompt() {
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
        write_legacy(root.path(), &format!("[{}]", legacy_entry("abc", "Paper", &server_dir)))
            .await;

        let status = service.status().await.expect("status");

        assert!(status.available, "manual trigger from settings must still offer the migration");
        assert_eq!(status.entries[0].state, LegacyMigrationEntryState::Migratable);
    }
}
