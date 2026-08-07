//! 系统资源信息服务端口。

use std::path::Path;

use async_trait::async_trait;

use crate::error::SystemServiceError;

use super::models::{DirectoryUsage, ProcessResourceUsage, SystemSnapshot};

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

    /// 采集指定进程的资源使用。
    ///
    /// 进程不存在或无权访问时返回 `pid = None` 的空结果。
    async fn process_usage(&self, pid: u32) -> Result<ProcessResourceUsage, SystemServiceError>;

    /// 计算目录磁盘占用。
    async fn directory_usage(&self, path: &Path) -> Result<DirectoryUsage, SystemServiceError>;
}
