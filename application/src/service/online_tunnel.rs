use async_trait::async_trait;
use sealantern_extra::online::{
    HostTunnelRequest, JoinTunnelRequest, OnlineTunnelError,
    OnlineTunnelService as ExtraOnlineTunnelService, TunnelConnection, TunnelEvent, TunnelIdentity,
    TunnelMode, TunnelStatus, TunnelTicket,
};
use sealantern_interface::{
    OnlineTunnelConnection, OnlineTunnelEvent, OnlineTunnelHostRequest, OnlineTunnelJoinRequest,
    OnlineTunnelMode, OnlineTunnelService, OnlineTunnelServiceError, OnlineTunnelStatus,
};
use tokio::sync::broadcast;

#[derive(Clone, Default)]
pub struct CoreOnlineTunnelService {
    inner: ExtraOnlineTunnelService,
}

#[async_trait]
impl OnlineTunnelService for CoreOnlineTunnelService {
    async fn host(
        &self,
        request: OnlineTunnelHostRequest,
    ) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
        let identity = request
            .identity
            .map(|identity| {
                identity
                    .try_into()
                    .map(TunnelIdentity::from_bytes)
                    .map_err(|_| OnlineTunnelServiceError::InvalidInput)
            })
            .transpose()?;
        self.inner
            .host(HostTunnelRequest {
                minecraft_port: request.minecraft_port,
                password: request.password,
                max_players: request.max_players,
                relay_url: request.relay_url,
                identity,
            })
            .await
            .map(map_status)
            .map_err(map_error)
    }

    async fn join(
        &self,
        request: OnlineTunnelJoinRequest,
    ) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
        let ticket = TunnelTicket::parse(request.ticket)
            .map_err(|_| OnlineTunnelServiceError::InvalidInput)?;
        self.inner
            .join(JoinTunnelRequest {
                ticket,
                local_port: request.local_port,
                password: request.password,
                max_retries: request.max_retries,
            })
            .await
            .map(map_status)
            .map_err(map_error)
    }

    async fn stop(&self) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
        self.inner.stop().await.map(map_status).map_err(map_error)
    }

    async fn status(&self) -> Result<OnlineTunnelStatus, OnlineTunnelServiceError> {
        self.inner.status().await.map(map_status).map_err(map_error)
    }

    async fn subscribe(
        &self,
    ) -> Result<broadcast::Receiver<OnlineTunnelEvent>, OnlineTunnelServiceError> {
        let mut source = self.inner.subscribe().await.map_err(map_error)?;
        let (sender, receiver) = broadcast::channel(128);
        tokio::spawn(async move {
            while let Ok(event) = source.recv().await {
                let _ = sender.send(map_event(event));
            }
        });
        Ok(receiver)
    }

    async fn shutdown(&self) -> Result<(), OnlineTunnelServiceError> {
        self.inner.shutdown().await.map_err(map_error)
    }
}

fn map_error(error: OnlineTunnelError) -> OnlineTunnelServiceError {
    match error {
        OnlineTunnelError::InvalidRequest { .. } => OnlineTunnelServiceError::InvalidInput,
        OnlineTunnelError::Busy => OnlineTunnelServiceError::Busy,
        OnlineTunnelError::NotRunning => OnlineTunnelServiceError::NotRunning,
        OnlineTunnelError::Provider { .. } => OnlineTunnelServiceError::OperationFailed,
    }
}

fn map_mode(value: TunnelMode) -> OnlineTunnelMode {
    match value {
        TunnelMode::Host => OnlineTunnelMode::Host,
        TunnelMode::Join => OnlineTunnelMode::Join,
    }
}

fn map_connection(value: TunnelConnection) -> OnlineTunnelConnection {
    OnlineTunnelConnection {
        remote_id: value.remote_id,
        is_relay: value.is_relay,
        rtt_ms: value.rtt_ms,
        tx_bytes: value.tx_bytes,
        rx_bytes: value.rx_bytes,
        alive: value.alive,
        elapsed_ms: value.elapsed_ms,
    }
}

fn map_status(value: TunnelStatus) -> OnlineTunnelStatus {
    OnlineTunnelStatus {
        active: value.active,
        mode: value.mode.map(map_mode),
        ticket: value.ticket.map(|ticket| ticket.as_str().to_owned()),
        connections: value.connections.into_iter().map(map_connection).collect(),
    }
}

fn map_event(value: TunnelEvent) -> OnlineTunnelEvent {
    match value {
        TunnelEvent::PlayerJoined { remote_id } => OnlineTunnelEvent::PlayerJoined { remote_id },
        TunnelEvent::PlayerLeft { remote_id, reason } => {
            OnlineTunnelEvent::PlayerLeft { remote_id, reason }
        }
        TunnelEvent::Connected => OnlineTunnelEvent::Connected,
        TunnelEvent::Disconnected { reason } => OnlineTunnelEvent::Disconnected { reason },
        TunnelEvent::PathChanged { remote_id, is_relay, rtt_ms } => {
            OnlineTunnelEvent::PathChanged { remote_id, is_relay, rtt_ms }
        }
        TunnelEvent::Reconnecting { attempt } => OnlineTunnelEvent::Reconnecting { attempt },
        TunnelEvent::Reconnected => OnlineTunnelEvent::Reconnected,
        TunnelEvent::AuthenticationFailed { remote_id } => {
            OnlineTunnelEvent::AuthenticationFailed { remote_id }
        }
        TunnelEvent::PlayerRejected { remote_id, reason } => {
            OnlineTunnelEvent::PlayerRejected { remote_id, reason }
        }
        TunnelEvent::Error { message } => OnlineTunnelEvent::Error { message },
        TunnelEvent::ProviderMessage { message } => OnlineTunnelEvent::ProviderMessage { message },
    }
}
