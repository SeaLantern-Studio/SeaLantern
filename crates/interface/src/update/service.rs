use async_trait::async_trait;

use crate::error::UpdateCheckServiceError;
use crate::error::UpdateInstallServiceError;
use sealantern_extra::update::PendingUpdate;

use super::models::UpdateInfo;

/// 应用更新检查宿主能力端口。
///
/// 当前版本由实现方从应用构建信息获取，宿主不得覆盖；下载与安装属于独立能力，
/// 后续通过单独的 trait 扩展。
#[async_trait]
pub trait UpdateCheckService: Send + Sync {
    /// 检查当前平台是否存在新版本。
    async fn check(&self) -> Result<UpdateInfo, UpdateCheckServiceError>;
}

#[async_trait]
pub trait UpdateInstallService: Send + Sync {
    async fn download(
        &self,
        url: String,
        expected_hash: Option<String>,
        version: String,
    ) -> Result<String, UpdateInstallServiceError>;
    async fn pending(&self) -> Result<Option<PendingUpdate>, UpdateInstallServiceError>;
    async fn clear_pending(&self) -> Result<(), UpdateInstallServiceError>;
    async fn install(
        &self,
        file_path: String,
        arguments: Vec<String>,
    ) -> Result<(), UpdateInstallServiceError>;
}
