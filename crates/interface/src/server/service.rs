//! 服务器进程管理服务端口。

use async_trait::async_trait;
use sealantern_core::instance::InstanceId;

use crate::error::ServerServiceError;

use super::models::ServerSnapshot;

/// 服务器进程管理宿主能力端口。
///
/// 管理实例对应的服务器进程生命周期：启动、优雅停止、强制停止、状态查询与
/// 控制台命令。实例记录本身（CRUD）由 [`InstanceService`](crate::InstanceService)
/// 负责；本端口以实例标识定位进程，实现方需从实例记录读取启动配置。
///
/// 方法均为异步：涉及进程 IO 与状态轮询。实现方组合 `core` 的 process 能力，
/// 不依赖任何具体宿主。
#[async_trait]
pub trait ServerService: Send + Sync {
    /// 查询实例对应服务器进程的状态快照。
    async fn status(&self, id: &InstanceId) -> Result<ServerSnapshot, ServerServiceError>;

    /// 启动实例对应的服务器进程。
    async fn start(&self, id: &InstanceId) -> Result<(), ServerServiceError>;

    /// 按实例当前生命周期状态完成一次重启。
    ///
    /// 已停止实例直接启动；活动实例必须先确认停止后才能再次启动。
    async fn restart(&self, id: &InstanceId) -> Result<(), ServerServiceError>;

    /// 优雅停止服务器进程（向控制台发送 stop 并等待退出，超时后强制终止）。
    async fn stop(&self, id: &InstanceId) -> Result<(), ServerServiceError>;

    /// 强制停止服务器进程（终止整个进程树）。
    async fn force_stop(&self, id: &InstanceId) -> Result<(), ServerServiceError>;

    /// 向服务器控制台发送单行命令。
    async fn send_command(&self, id: &InstanceId, command: &str) -> Result<(), ServerServiceError>;
}
