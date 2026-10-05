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
use sealantern_feature::config::instance::{DocumentStore, MemorySpec};

use crate::error::ServerStartupError;
use crate::port::ServerStartupService;

/// 基于实例文档的启动配置服务实现。
#[derive(Debug, Default)]
pub struct CoreServerStartupService;

#[async_trait]
impl ServerStartupService for CoreServerStartupService {
    async fn read(&self, server_path: &str) -> Result<SLStartupConfig, ServerStartupServiceError> {
        let path = DocumentStore::path_for(Path::new(server_path));
        // 无文档（裸目录/未建档）或文档不可读时按「无覆盖」返回，由前端
        // 回落到默认值；文档层面的问题会在实例列表/发现页另行暴露。
        let Ok(store) = DocumentStore::load(path).await else {
            return Ok(SLStartupConfig::default());
        };
        let memory = store.get().startup.memory_mib;
        Ok(SLStartupConfig {
            max_memory: (memory.max != 0).then_some(memory.max),
            min_memory: (memory.min != 0).then_some(memory.min),
        })
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
