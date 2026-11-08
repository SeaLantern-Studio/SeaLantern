//! 实例启动配置（`sl.json` 的 `startup` 段）管理服务实现。
//!
//! 实现 [`crate::port::ServerStartupService`] 能力端口。定位实例时**不接受**
//! 任意文件系统路径：调用方传入 `InstanceId`，本服务经
//! [`crate::service::CoreInstanceService`] 解析为已登记信任的实例目录后，
//! 才在其 `sl.json` 上读写——复用 `DocumentStore` 的锁内更新与结构校验。
//! 未登记信任的实例 id 一律拒绝，杜绝越权读写未受管目录下的 `sl.json`。
//!
//! 错误分层：内部以应用层主错误 [`ServerStartupError`] 为源头，暴露
//! [`ServerStartupService`] 时统一转为接口契约错误 [`ServerStartupServiceError`]。

use std::sync::Arc;

use async_trait::async_trait;
use sealantern_core::instance::InstanceId;

use sealantern_contract::ServerStartupServiceError;
use sealantern_contract::server_startup::SLStartupConfig;
use sealantern_feature::config::instance::{DocumentError, DocumentStore, MemorySpec};
use sealantern_infra::fs::FsError;

use crate::error::ServerStartupError;
use crate::port::{InstanceService, ServerStartupService};
use crate::service::CoreInstanceService;

/// 基于实例文档的启动配置服务实现。
pub struct CoreServerStartupService {
    instance: Arc<CoreInstanceService>,
}

impl CoreServerStartupService {
    /// 以共享的实例服务构造（实例服务提供受信任实例目录的解析）。
    pub fn new(instance: Arc<CoreInstanceService>) -> Self {
        Self { instance }
    }

    /// 将受信任实例 id 解析为其 `sl.json` 路径。
    ///
    /// 仅返回已登记信任的实例目录；未信任 / 不存在的 id 上抛
    /// `InvalidInput`，从源头阻断对任意目录 `sl.json` 的访问。
    async fn trusted_document_path(
        &self,
        id: &InstanceId,
    ) -> Result<std::path::PathBuf, ServerStartupError> {
        let instance = self
            .instance
            .find(id)
            .await
            .map_err(|error| ServerStartupError::OperationFailed { source: Box::new(error) })?
            .ok_or(ServerStartupError::InvalidInput)?;
        Ok(DocumentStore::path_for(&instance.directory))
    }
}

#[async_trait]
impl ServerStartupService for CoreServerStartupService {
    async fn read(&self, id: &InstanceId) -> Result<SLStartupConfig, ServerStartupServiceError> {
        let path = self.trusted_document_path(id).await?;
        // 仅在文档确实不存在（裸目录/未建档）时按「无覆盖」返回默认值；
        // 文档存在但不可读（损坏、版本超前、权限、校验失败等）则上抛错误，
        // 避免把不可恢复的故障掩盖成「该实例没有内存覆盖」。
        match DocumentStore::load(path).await {
            Ok(store) => {
                let memory = store.get().startup.memory_mib;
                Ok(SLStartupConfig {
                    max_memory: (memory.max != 0).then_some(memory.max),
                    min_memory: (memory.min != 0).then_some(memory.min),
                })
            }
            Err(DocumentError::Storage { source: FsError::Io { source, .. } })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                Ok(SLStartupConfig::default())
            }
            Err(error) => Err(ServerStartupError::from(error).into()),
        }
    }

    async fn write(
        &self,
        id: &InstanceId,
        config: &SLStartupConfig,
    ) -> Result<(), ServerStartupServiceError> {
        if let (Some(min), Some(max)) = (config.min_memory, config.max_memory)
            && min > max
        {
            return Err(ServerStartupError::InvalidInput.into());
        }

        let path = self.trusted_document_path(id).await?;
        let mut store = DocumentStore::load(path)
            .await
            .map_err(ServerStartupError::from)?;
        store
            .update(|document| {
                document.startup.memory_mib = MemorySpec {
                    min: config.min_memory.unwrap_or(0),
                    max: config.max_memory.unwrap_or(0),
                };
                Ok(true)
            })
            .await
            .map_err(ServerStartupError::from)?;
        Ok(())
    }
}
