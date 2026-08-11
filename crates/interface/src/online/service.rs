use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::error::OnlineTunnelServiceError;

use super::{
    OnlineTunnelEvent, OnlineTunnelHostRequest, OnlineTunnelJoinRequest, OnlineTunnelStatus,
};

#[async_trait]
pub trait OnlineTunnelService: Send + Sync {
    async fn host(
        &self,
        request: OnlineTunnelHostRequest,
    ) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError>;
    async fn join(
        &self,
        request: OnlineTunnelJoinRequest,
    ) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError>;
    async fn stop(&self) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError>;
    async fn status(&self) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError>;
    async fn subscribe(
        &self,
    ) -> Result<broadcast::Receiver<OnlineTunnelEvent>, OnlineTunnelServiceError>;
    async fn shutdown(&self) -> Result<(), OnlineTunnelServiceError>;
}
