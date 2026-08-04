use std::sync::Arc;

use sculk::tunnel::{HostConfig, IrohTunnel, JoinConfig, TunnelEvent as SculkTunnelEvent};
use sculk::types::{RelayUrl, SecretKey};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

use super::{
    HostTunnelRequest, JoinTunnelRequest, OnlineTunnelError, TunnelConnection, TunnelEvent,
    TunnelIdentity, TunnelMode, TunnelStatus, TunnelTicket,
};

pub(super) struct ActiveTunnel {
    pub(super) mode: TunnelMode,
    pub(super) ticket: Option<TunnelTicket>,
    tunnel: Arc<IrohTunnel>,
    events: broadcast::Sender<TunnelEvent>,
    event_task: JoinHandle<()>,
}

impl ActiveTunnel {
    pub(super) fn status(&self) -> Result<TunnelStatus, OnlineTunnelError> {
        Ok(TunnelStatus {
            active: true,
            mode: Some(self.mode),
            ticket: self.ticket.clone(),
            connections: connection_snapshot(&self.tunnel)?,
        })
    }

    pub(super) fn subscribe(&self) -> broadcast::Receiver<TunnelEvent> {
        self.events.subscribe()
    }

    pub(super) async fn close(self) {
        self.tunnel.close().await;
        self.event_task.abort();
        let _ = self.event_task.await;
    }
}

pub(super) async fn host(request: HostTunnelRequest) -> Result<ActiveTunnel, OnlineTunnelError> {
    validate_host_request(&request)?;
    let relay = parse_relay_url(request.relay_url.as_deref())?;
    let identity = request.identity.as_ref().map(identity_to_secret_key);
    let config = HostConfig::default()
        .password(normalize_optional_string(request.password))
        .max_players(request.max_players);

    let (tunnel, ticket, events) =
        IrohTunnel::host(request.minecraft_port, identity, relay, config)
            .await
            .map_err(|error| OnlineTunnelError::provider("start host tunnel", error))?;

    Ok(active_tunnel(
        TunnelMode::Host,
        Some(TunnelTicket::from_provider(ticket.to_string())),
        tunnel,
        events,
    ))
}

pub(super) async fn join(request: JoinTunnelRequest) -> Result<ActiveTunnel, OnlineTunnelError> {
    if request.local_port == 0 {
        return Err(OnlineTunnelError::invalid_request("local port must be non-zero"));
    }

    let ticket = request
        .ticket
        .as_str()
        .parse()
        .map_err(|error| OnlineTunnelError::provider("parse tunnel ticket", error))?;
    let config = JoinConfig::default()
        .password(normalize_optional_string(request.password))
        .max_retries(request.max_retries);
    let (tunnel, events) = IrohTunnel::join(&ticket, request.local_port, config)
        .await
        .map_err(|error| OnlineTunnelError::provider("join host tunnel", error))?;

    Ok(active_tunnel(TunnelMode::Join, None, tunnel, events))
}

fn active_tunnel(
    mode: TunnelMode,
    ticket: Option<TunnelTicket>,
    tunnel: IrohTunnel,
    events: mpsc::Receiver<SculkTunnelEvent>,
) -> ActiveTunnel {
    let (event_sender, _) = broadcast::channel(128);
    let event_task = spawn_event_forwarder(events, event_sender.clone());

    ActiveTunnel {
        mode,
        ticket,
        tunnel: Arc::new(tunnel),
        events: event_sender,
        event_task,
    }
}

fn validate_host_request(request: &HostTunnelRequest) -> Result<(), OnlineTunnelError> {
    if request.minecraft_port == 0 {
        return Err(OnlineTunnelError::invalid_request("minecraft port must be non-zero"));
    }
    if request.max_players == Some(0) {
        return Err(OnlineTunnelError::invalid_request("max players must be greater than zero"));
    }
    Ok(())
}

fn parse_relay_url(value: Option<&str>) -> Result<Option<RelayUrl>, OnlineTunnelError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    value
        .parse::<RelayUrl>()
        .map(Some)
        .map_err(|error| OnlineTunnelError::provider("parse relay URL", error))
}

fn identity_to_secret_key(identity: &TunnelIdentity) -> SecretKey {
    SecretKey::from_bytes(identity.as_bytes())
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_owned();
        (!value.is_empty()).then_some(value)
    })
}

fn connection_snapshot(tunnel: &IrohTunnel) -> Result<Vec<TunnelConnection>, OnlineTunnelError> {
    tunnel
        .connections()
        .map_err(|error| OnlineTunnelError::provider("read tunnel connections", error))
        .map(|connections| {
            connections
                .into_iter()
                .map(|connection| TunnelConnection {
                    remote_id: connection.remote_id.to_string(),
                    is_relay: connection.is_relay,
                    rtt_ms: connection.rtt_ms,
                    tx_bytes: connection.tx_bytes,
                    rx_bytes: connection.rx_bytes,
                    alive: connection.alive,
                    elapsed_ms: connection.elapsed.as_millis() as u64,
                })
                .collect()
        })
}

fn spawn_event_forwarder(
    mut receiver: mpsc::Receiver<SculkTunnelEvent>,
    sender: broadcast::Sender<TunnelEvent>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            let event = map_event(event);
            if let TunnelEvent::Error { message } = &event {
                crate::observability::online_tunnel_event_error(message);
            }
            let _ = sender.send(event);
        }
    })
}

fn map_event(event: SculkTunnelEvent) -> TunnelEvent {
    match event {
        SculkTunnelEvent::PlayerJoined { id } => {
            TunnelEvent::PlayerJoined { remote_id: id.to_string() }
        }
        SculkTunnelEvent::PlayerLeft { id, reason } => {
            TunnelEvent::PlayerLeft { remote_id: id.to_string(), reason }
        }
        SculkTunnelEvent::Connected => TunnelEvent::Connected,
        SculkTunnelEvent::Disconnected { reason } => TunnelEvent::Disconnected { reason },
        SculkTunnelEvent::PathChanged { remote_id, is_relay, rtt_ms } => TunnelEvent::PathChanged {
            remote_id: remote_id.to_string(),
            is_relay,
            rtt_ms,
        },
        SculkTunnelEvent::Reconnecting { attempt } => TunnelEvent::Reconnecting { attempt },
        SculkTunnelEvent::Reconnected => TunnelEvent::Reconnected,
        SculkTunnelEvent::AuthFailed { id } => {
            TunnelEvent::AuthenticationFailed { remote_id: id.to_string() }
        }
        SculkTunnelEvent::PlayerRejected { id, reason } => {
            TunnelEvent::PlayerRejected { remote_id: id.to_string(), reason }
        }
        SculkTunnelEvent::Error { message } => TunnelEvent::Error { message },
        _ => TunnelEvent::ProviderMessage {
            message: "received an unsupported provider event".to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_host_port_before_opening_a_tunnel() {
        let error = validate_host_request(&HostTunnelRequest {
            minecraft_port: 0,
            password: None,
            max_players: None,
            relay_url: None,
            identity: None,
        })
        .expect_err("zero port must be rejected");

        assert!(matches!(error, OnlineTunnelError::InvalidRequest { .. }));
    }

    #[test]
    fn normalizes_blank_optional_password() {
        assert_eq!(normalize_optional_string(Some("  ".to_owned())), None);
        assert_eq!(
            normalize_optional_string(Some(" secret ".to_owned())),
            Some("secret".to_owned())
        );
    }

    #[test]
    fn maps_provider_events_without_exposing_provider_types() {
        let event = map_event(SculkTunnelEvent::Connected);
        assert_eq!(event, TunnelEvent::Connected);
    }

    #[test]
    fn rejects_an_invalid_ticket_at_the_public_boundary() {
        let error = TunnelTicket::parse("not-a-ticket").expect_err("ticket must be validated");

        assert!(matches!(error, OnlineTunnelError::Provider { .. }));
    }
}
