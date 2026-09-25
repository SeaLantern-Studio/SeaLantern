//! 实例启动配置管理能力端口。
//!
//! 方法以实例标识定位：服务器目录由应用层从实例注册表解析，
//! 因此调用方（无论 Tauri 还是 HTTP）都无法指定任意路径。

use async_trait::async_trait;

use sealantern_contract::ServerStartupServiceError;
use sealantern_contract::server_startup::SLStartupConfig;
use sealantern_core::instance::InstanceId;

/// 实例启动配置（`SeaLantern/config.toml`）管理能力。
#[async_trait]
pub trait ServerStartupService: Send + Sync {
    /// 读取实例的启动配置覆盖；无覆盖时字段为 `None`。
    async fn read(&self, id: &InstanceId) -> Result<SLStartupConfig, ServerStartupServiceError>;

    /// 写入实例的启动配置覆盖。
    async fn write(
        &self,
        id: &InstanceId,
        config: &SLStartupConfig,
    ) -> Result<(), ServerStartupServiceError>;
}
