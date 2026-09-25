//! 实例启动配置（SeaLantern/config.toml）管理服务实现。
//!
//! 实现 [`crate::port::ServerStartupService`] 能力端口，组合
//! `feature::server_startup` 的配置文件读写能力。
//!
//! 错误分层：内部以应用层主错误 [`ServerStartupError`] 为源头，暴露
//! [`ServerStartupService`] 时统一转为接口契约错误 [`ServerStartupServiceError`]。

use std::sync::Arc;

use async_trait::async_trait;

use sealantern_contract::ServerStartupServiceError;
use sealantern_contract::server_startup::SLStartupConfig;
use sealantern_core::instance::InstanceId;
use sealantern_feature::server_startup::{
    ServerStartupError as FeatureStartupError, ServerStartupManager,
};

use crate::error::ServerStartupError;
use crate::port::ServerStartupService;
use crate::service::{CoreInstanceService, resolve_instance_directory};

/// 将阻塞的配置文件读写调度到阻塞线程池，统一收敛错误。
async fn run_blocking<T, F>(operation: F) -> Result<T, ServerStartupError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, FeatureStartupError> + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(ServerStartupError::from)?
        .map_err(ServerStartupError::from)
}

/// 实例启动配置管理服务。
pub struct CoreServerStartupService {
    instances: Arc<CoreInstanceService>,
}

impl CoreServerStartupService {
    /// 以实例服务构造；每次操作都先按实例标识解析服务器目录。
    pub fn new(instances: Arc<CoreInstanceService>) -> Self {
        Self { instances }
    }

    /// 解析实例目录，缺失时按「实例不存在」上报。
    async fn resolve_directory(&self, id: &InstanceId) -> Result<String, ServerStartupError> {
        resolve_instance_directory(self.instances.as_ref(), id)
            .await
            .map_err(ServerStartupError::from)?
            .map(|directory| directory.to_string_lossy().into_owned())
            .ok_or(ServerStartupError::InstanceNotFound)
    }
}

#[async_trait]
impl ServerStartupService for CoreServerStartupService {
    async fn read(&self, id: &InstanceId) -> Result<SLStartupConfig, ServerStartupServiceError> {
        let server_path = self.resolve_directory(id).await?;
        run_blocking(move || ServerStartupManager::new(&server_path).read())
            .await
            .map_err(Into::into)
    }

    async fn write(
        &self,
        id: &InstanceId,
        config: &SLStartupConfig,
    ) -> Result<(), ServerStartupServiceError> {
        let server_path = self.resolve_directory(id).await?;
        let config = config.clone();
        run_blocking(move || ServerStartupManager::new(&server_path).write(&config))
            .await
            .map_err(Into::into)
    }
}
