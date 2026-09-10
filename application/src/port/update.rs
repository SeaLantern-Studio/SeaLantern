use async_trait::async_trait;
use sealantern_contract::update::{PendingUpdate, UpdateInfo};
use sealantern_contract::{UpdateCheckServiceError, UpdateInstallServiceError};

/// 应用更新检查宿主能力端口。
///
/// 当前版本由实现方从应用构建信息获取，宿主不得覆盖；下载与安装属于独立能力，
/// 由 [`UpdateInstallService`] 承担。
#[async_trait]
pub trait UpdateCheckService: Send + Sync {
    /// 检查当前平台是否存在新版本。
    async fn check(&self) -> Result<UpdateInfo, UpdateCheckServiceError>;
}

/// 应用更新下载与安装宿主能力端口。
#[async_trait]
pub trait UpdateInstallService: Send + Sync {
    /// 下载指定版本的更新资源到本地，返回文件路径。
    ///
    /// `expected_hash` 非空时校验下载内容的 SHA-256，不一致视为失败。
    async fn download(
        &self,
        url: String,
        expected_hash: Option<String>,
        version: String,
    ) -> Result<String, UpdateInstallServiceError>;
    /// 查询已下载、待安装的更新；没有待安装内容时返回 `None`。
    async fn pending(&self) -> Result<Option<PendingUpdate>, UpdateInstallServiceError>;
    /// 清除已下载的待安装更新记录。
    async fn clear_pending(&self) -> Result<(), UpdateInstallServiceError>;
    /// 安装指定的更新文件，`arguments` 为安装过程附加参数。
    async fn install(
        &self,
        file_path: String,
        arguments: Vec<String>,
    ) -> Result<(), UpdateInstallServiceError>;
}
