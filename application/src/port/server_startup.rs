//! 实例启动配置管理能力端口。
//!
//! 以受信任的实例标识定位：调用方传入 `InstanceId`，服务经
//! [`crate::port::InstanceService`] 解析为已登记信任的实例目录后，再在其
//! `sl.json` 上读写 `startup` 段的内存覆盖——不接受任意文件系统路径，
//! 避免越权读写未受管目录下的 `sl.json`。

use async_trait::async_trait;
use sealantern_core::instance::InstanceId;

use sealantern_contract::ServerStartupServiceError;
use sealantern_contract::server_startup::SLStartupConfig;

/// 实例启动配置（`sl.json` 的 `startup` 段）管理能力。
#[async_trait]
pub trait ServerStartupService: Send + Sync {
    /// 读取实例的启动内存覆盖；无文档或无覆盖时字段为 `None`。
    async fn read(&self, id: &InstanceId) -> Result<SLStartupConfig, ServerStartupServiceError>;

    /// 写入实例的启动内存覆盖。
    async fn write(
        &self,
        id: &InstanceId,
        config: &SLStartupConfig,
    ) -> Result<(), ServerStartupServiceError>;
}
