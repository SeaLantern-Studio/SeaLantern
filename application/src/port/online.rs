//! 在线隧道宿主能力端口。

use async_trait::async_trait;
use sealantern_contract::OnlineTunnelServiceError;
use sealantern_contract::online::{
    OnlineTunnelEvent, OnlineTunnelHostRequest, OnlineTunnelJoinRequest, OnlineTunnelStatus,
};
use tokio::sync::broadcast;

/// 在线隧道宿主能力端口。
///
/// 一个服务实例同一时刻至多持有一条活动隧道；隧道启动或停止期间，新的
/// host / join 请求会被拒绝。
#[async_trait]
pub trait OnlineTunnelService: Send + Sync {
    /// 以 Host 角色开启隧道，返回建立完成后的状态。
    async fn host(
        &self,
        request: OnlineTunnelHostRequest,
    ) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError>;
    /// 以 Join 角色加入票据指定的隧道，返回建立完成后的状态。
    async fn join(
        &self,
        request: OnlineTunnelJoinRequest,
    ) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError>;
    /// 停止当前活动隧道，返回空闲状态。
    async fn stop(&self) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError>;
    /// 查询当前隧道状态；隧道正在启动或停止时返回忙错误。
    async fn status(&self) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError>;
    /// 订阅隧道事件广播流，供调用方实时监听对端接入、断线与错误等事件。
    async fn subscribe(
        &self,
    ) -> Result<broadcast::Receiver<OnlineTunnelEvent>, OnlineTunnelServiceError>;
    /// 幂等关闭服务持有的活动隧道，供宿主退出流程调用。
    async fn shutdown(&self) -> Result<(), OnlineTunnelServiceError>;
}
