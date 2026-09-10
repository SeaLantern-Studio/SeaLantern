//! 服务器备份服务实现。
//!
//! 实现 [`crate::port::BackupService`] 能力端口，组合
//! [`CoreInstanceService`]（按实例定位服务器目录）与
//! [`CoreServerService`]（校验服务器已停止，冷备份前提）以及
//! `feature::backup` 的备份能力，向宿主提供备份的查询、创建、删除、
//! 恢复与设置管理。
//!
//! 错误分层：内部以应用层主错误 [`BackupError`] 为源头，暴露
//! [`BackupService`] 时统一转为接口契约错误 [`BackupServiceError`]。

use std::sync::Arc;

use async_trait::async_trait;
use sealantern_contract::BackupServiceError;
use sealantern_contract::backup::{BackupItem, BackupSettings, CreateBackupRequest};
use sealantern_contract::server::ServerState;
use sealantern_core::instance::InstanceId;

use super::{CoreInstanceService, CoreServerService};
use crate::error::BackupError;
use crate::port::{BackupService, InstanceService, ServerService};

/// 基于实例目录与服务器状态校验的备份服务实现。
pub struct CoreBackupService {
    instance_service: Arc<CoreInstanceService>,
    server_service: Arc<CoreServerService>,
}

impl CoreBackupService {
    /// 创建使用指定实例与服务器进程服务的备份服务。
    pub fn new(
        instance_service: Arc<CoreInstanceService>,
        server_service: Arc<CoreServerService>,
    ) -> Self {
        Self { instance_service, server_service }
    }

    /// 校验服务器已停止（冷备份前提），返回其实例记录。
    ///
    /// 实例不存在或服务器仍在运行时分别返回 [`BackupError::NotFound`]
    /// 与 [`BackupError::ServerRunning`]。
    async fn require_stopped_instance(
        &self,
        server_id: &str,
    ) -> Result<sealantern_core::instance::Instance, BackupError> {
        let instance_id =
            InstanceId::new(server_id.to_owned()).map_err(|_| BackupError::InvalidInput)?;

        let instance = self
            .instance_service
            .find(&instance_id)
            .await
            .map_err(BackupError::from)?
            .ok_or(BackupError::NotFound)?;

        let status = self
            .server_service
            .status(&instance_id)
            .await
            .map_err(BackupError::from)?;

        if status.state != ServerState::Stopped {
            return Err(BackupError::ServerRunning);
        }

        Ok(instance)
    }
}

#[async_trait]
impl BackupService for CoreBackupService {
    async fn list(&self, server_id: &str) -> Result<Vec<BackupItem>, BackupServiceError> {
        sealantern_feature::backup::get_backup_list(server_id.to_owned())
            .await
            .map_err(BackupError::from)
            .map_err(Into::into)
    }

    async fn create(&self, request: CreateBackupRequest) -> Result<BackupItem, BackupServiceError> {
        let instance = self.require_stopped_instance(&request.server_id).await?;
        let directory = instance.directory.clone();

        // 回调在 feature 的阻塞任务内部被再次调用，用于关闭「状态检查后、实际
        // 落盘前服务器被并发启动」的竞态窗口：此时同步查询一次进程表复核。
        let check_server_stopped = {
            let server = self.server_service.clone();
            let instance = instance.clone();
            move |_server_id: &str| server.server_stopped(&instance)
        };

        sealantern_feature::backup::create_backup(request, directory, check_server_stopped)
            .await
            .map_err(BackupError::from)
            .map_err(Into::into)
    }

    async fn delete(&self, backup_id: &str) -> Result<(), BackupServiceError> {
        sealantern_feature::backup::delete_backup(backup_id.to_owned())
            .await
            .map_err(BackupError::from)
            .map_err(Into::into)
    }

    async fn restore(&self, backup_id: &str, server_id: &str) -> Result<(), BackupServiceError> {
        let instance = self.require_stopped_instance(server_id).await?;
        let directory = instance.directory.clone();

        // 与 create 相同的竞态窗口防护：阻塞任务内部同步复核服务器状态。
        let check_server_stopped = {
            let server = self.server_service.clone();
            let instance = instance.clone();
            move |_server_id: &str| server.server_stopped(&instance)
        };

        sealantern_feature::backup::restore_backup(
            backup_id.to_owned(),
            server_id.to_owned(),
            directory,
            check_server_stopped,
        )
        .await
        .map_err(BackupError::from)
        .map_err(Into::into)
    }

    async fn settings(&self, server_id: &str) -> Result<BackupSettings, BackupServiceError> {
        sealantern_feature::backup::get_backup_settings(server_id.to_owned())
            .await
            .map_err(BackupError::from)
            .map_err(Into::into)
    }

    async fn update_settings(
        &self,
        server_id: &str,
        settings: BackupSettings,
    ) -> Result<(), BackupServiceError> {
        sealantern_feature::backup::update_backup_settings(server_id.to_owned(), settings)
            .await
            .map_err(BackupError::from)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tempfile::tempdir;

    use super::*;

    /// 构造测试用备份服务（临时实例目录 + 独立 server 服务）。
    async fn test_service() -> CoreBackupService {
        let dir = tempdir().expect("create temp dir");
        let instance = Arc::new(
            CoreInstanceService::with_path(dir.path().join("instances.json"))
                .await
                .expect("create instance service"),
        );
        let server = Arc::new(CoreServerService::new(
            instance.clone(),
            Arc::new(crate::service::CoreSettingsService::new()),
        ));
        CoreBackupService::new(instance, server)
    }

    #[tokio::test]
    async fn list_returns_empty_for_unknown_server() {
        let service = test_service().await;
        let items = service.list("missing").await.expect("list should succeed");
        assert!(items.is_empty(), "unknown server should have no backups");
    }

    #[tokio::test]
    async fn settings_returns_defaults_for_unknown_server() {
        let service = test_service().await;
        let settings = service
            .settings("missing")
            .await
            .expect("settings should succeed");
        assert_eq!(settings, BackupSettings::default());
    }
}
