//! 服务器插件（plugins 目录）管理服务实现。
//!
//! 实现 [`crate::port::ServerPluginService`] 能力端口，组合
//! `feature::server_plugin` 的插件目录读写能力。
//!
//! 所有方法都以实例标识定位：服务器目录由本服务从实例注册表解析后交给
//! feature 层，因此宿主的调用方无法指定任意路径。
//!
//! 错误分层：内部以应用层主错误 [`ServerPluginError`] 为源头，暴露
//! [`ServerPluginService`] 时统一转为接口契约错误 [`ServerPluginServiceError`]。

use std::sync::Arc;

use async_trait::async_trait;

use sealantern_contract::ServerPluginServiceError;
use sealantern_contract::server_plugin::{PluginConfigFile, PluginSummary};
use sealantern_core::instance::InstanceId;
use sealantern_feature::server_plugin::{
    ServerPluginError as FeaturePluginError, ServerPluginManager,
};

use crate::error::ServerPluginError;
use crate::port::ServerPluginService;
use crate::service::{CoreInstanceService, resolve_instance_directory};

/// 将阻塞的目录与归档操作调度到阻塞线程池，统一收敛错误。
async fn run_blocking<T, F>(operation: F) -> Result<T, ServerPluginError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, FeaturePluginError> + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(ServerPluginError::from)?
        .map_err(ServerPluginError::from)
}

/// 服务器插件管理服务。
pub struct CoreServerPluginService {
    instances: Arc<CoreInstanceService>,
}

impl CoreServerPluginService {
    /// 以实例服务构造；每次操作都先按实例标识解析服务器目录。
    pub fn new(instances: Arc<CoreInstanceService>) -> Self {
        Self { instances }
    }

    /// 解析实例目录，缺失时按「实例不存在」上报。
    async fn resolve_directory(&self, id: &InstanceId) -> Result<String, ServerPluginError> {
        resolve_instance_directory(self.instances.as_ref(), id)
            .await
            .map_err(ServerPluginError::from)?
            .map(|directory| directory.to_string_lossy().into_owned())
            .ok_or(ServerPluginError::InstanceNotFound)
    }
}

#[async_trait]
impl ServerPluginService for CoreServerPluginService {
    async fn list(&self, id: &InstanceId) -> Result<Vec<PluginSummary>, ServerPluginServiceError> {
        let server_path = self.resolve_directory(id).await?;
        run_blocking(move || ServerPluginManager::new(&server_path).list())
            .await
            .map_err(Into::into)
    }

    async fn read_config_files(
        &self,
        id: &InstanceId,
        _file_name: &str,
        plugin_name: &str,
    ) -> Result<Vec<PluginConfigFile>, ServerPluginServiceError> {
        let server_path = self.resolve_directory(id).await?;
        let plugin_name = plugin_name.to_owned();
        run_blocking(move || ServerPluginManager::new(&server_path).read_config_files(&plugin_name))
            .await
            .map_err(Into::into)
    }

    async fn set_enabled(
        &self,
        id: &InstanceId,
        file_name: &str,
        enabled: bool,
    ) -> Result<(), ServerPluginServiceError> {
        let server_path = self.resolve_directory(id).await?;
        let file_name = file_name.to_owned();
        run_blocking(move || {
            ServerPluginManager::new(&server_path).set_enabled(&file_name, enabled)
        })
        .await
        .map_err(Into::into)
    }

    async fn delete(
        &self,
        id: &InstanceId,
        file_name: &str,
    ) -> Result<(), ServerPluginServiceError> {
        let server_path = self.resolve_directory(id).await?;
        let file_name = file_name.to_owned();
        run_blocking(move || ServerPluginManager::new(&server_path).delete(&file_name))
            .await
            .map_err(Into::into)
    }

    async fn install(
        &self,
        id: &InstanceId,
        file_name: &str,
        data: &[u8],
    ) -> Result<(), ServerPluginServiceError> {
        let server_path = self.resolve_directory(id).await?;
        let file_name = file_name.to_owned();
        // 插件字节内容由前端一次性给出，这里按值移交阻塞任务。
        let data = data.to_vec();
        run_blocking(move || ServerPluginManager::new(&server_path).install(&file_name, &data))
            .await
            .map_err(Into::into)
    }
}
