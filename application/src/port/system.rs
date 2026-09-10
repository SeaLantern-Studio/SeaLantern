//! 系统资源信息服务端口。

use async_trait::async_trait;
use sealantern_contract::SystemServiceError;
use sealantern_contract::system::{ServerResourceUsage, SystemSnapshot};

/// 系统资源信息宿主能力端口。
///
/// 方法均为异步：目录遍历等可能涉及阻塞 IO。实现方组合 `infra` 的系统采集
/// 能力，不依赖任何具体宿主。
#[async_trait]
pub trait SystemService: Send + Sync {
    /// 采集整机资源快照。
    ///
    /// CPU 使用率为采样时刻瞬时值，调用方如需平滑值应自行间隔采样。
    async fn system_snapshot(&self) -> Result<SystemSnapshot, SystemServiceError>;

    /// 获取默认运行路径。
    ///
    /// 路径优先级：标准数据目录 → 文档目录 → 当前工作目录。
    async fn default_run_path(&self) -> Result<String, SystemServiceError>;

    /// 按实例标识采集服务器资源占用。
    ///
    /// 未运行或进程不存在时返回 `pid = None` 的空资源结果。
    async fn server_resource_usage(
        &self,
        instance_id: &str,
    ) -> Result<ServerResourceUsage, SystemServiceError>;
}
