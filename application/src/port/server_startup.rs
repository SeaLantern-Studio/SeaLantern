//! 实例启动配置管理能力端口。
//!
//! 以服务器目录路径定位：与 `ServerConfigService` 一致，调用方传入实例
//! 目录，服务在其 `sl.json` 上读写 `startup` 段的内存覆盖。

use async_trait::async_trait;

use sealantern_contract::ServerStartupServiceError;
use sealantern_contract::server_startup::SLStartupConfig;

/// 实例启动配置（`sl.json` 的 `startup` 段）管理能力。
#[async_trait]
pub trait ServerStartupService: Send + Sync {
    /// 读取实例的启动内存覆盖；无文档或无覆盖时字段为 `None`。
    async fn read(&self, server_path: &str) -> Result<SLStartupConfig, ServerStartupServiceError>;

    /// 写入实例的启动内存覆盖。
    async fn write(
        &self,
        server_path: &str,
        config: &SLStartupConfig,
    ) -> Result<(), ServerStartupServiceError>;
}
