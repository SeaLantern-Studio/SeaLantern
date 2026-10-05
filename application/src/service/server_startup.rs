//! 实例启动配置（`sl.json` 的 `startup` 段）管理服务实现。
//!
//! 实现 [`crate::port::ServerStartupService`] 能力端口，直接经
//! `feature::config::instance::DocumentStore` 读写实例文档，复用其锁内更新与
//! 结构校验。
//!
//! 错误分层：内部以应用层主错误 [`ServerStartupError`] 为源头，暴露
//! [`ServerStartupService`] 时统一转为接口契约错误 [`ServerStartupServiceError`]。

use std::path::Path;

use async_trait::async_trait;

use sealantern_contract::ServerStartupServiceError;
use sealantern_contract::server_startup::SLStartupConfig;
use sealantern_feature::config::instance::{DocumentError, DocumentStore, MemorySpec};
use sealantern_infra::fs::FsError;

use crate::error::ServerStartupError;
use crate::port::ServerStartupService;

/// 基于实例文档的启动配置服务实现。
#[derive(Debug, Default)]
pub struct CoreServerStartupService;

#[async_trait]
impl ServerStartupService for CoreServerStartupService {
    async fn read(&self, server_path: &str) -> Result<SLStartupConfig, ServerStartupServiceError> {
        let path = DocumentStore::path_for(Path::new(server_path));
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
        server_path: &str,
        config: &SLStartupConfig,
    ) -> Result<(), ServerStartupServiceError> {
        if let (Some(min), Some(max)) = (config.min_memory, config.max_memory)
            && min > max
        {
            return Err(ServerStartupError::InvalidInput.into());
        }

        let path = DocumentStore::path_for(Path::new(server_path));
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
