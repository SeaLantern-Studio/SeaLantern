use super::i18n::{tunnel_t, tunnel_t1, tunnel_t2, tunnel_t3};
use super::state::{push_log, TunnelConnection};
use sculk::tunnel::{ConnectionSnapshot, TunnelEvent};
use tauri::async_runtime::JoinHandle;

pub(super) fn map_connection(snapshot: ConnectionSnapshot) -> TunnelConnection {
    TunnelConnection {
        remote_id: snapshot.remote_id.to_string(),
        is_relay: snapshot.is_relay,
        rtt_ms: snapshot.rtt_ms,
        tx_bytes: snapshot.tx_bytes,
        rx_bytes: snapshot.rx_bytes,
        alive: snapshot.alive,
        elapsed_secs: snapshot.elapsed.as_secs(),
    }
}

fn format_event(event: &TunnelEvent) -> String {
    match event {
        TunnelEvent::PlayerJoined { id } => tunnel_t1("tunnel.log.player_joined", id.to_string()),
        TunnelEvent::PlayerLeft { id, reason } => {
            tunnel_t2("tunnel.log.player_left", id.to_string(), reason.to_string())
        }
        TunnelEvent::Connected => tunnel_t("tunnel.log.connected_host"),
        TunnelEvent::Disconnected { reason } => {
            tunnel_t1("tunnel.log.disconnected", reason.to_string())
        }
        TunnelEvent::PathChanged { remote_id, is_relay, rtt_ms } => {
            let route_label = if *is_relay {
                tunnel_t("tunnel.log.route_relay")
            } else {
                tunnel_t("tunnel.log.route_direct")
            };
            tunnel_t3(
                "tunnel.log.path_changed",
                remote_id.to_string(),
                route_label,
                rtt_ms.to_string(),
            )
        }
        TunnelEvent::Reconnecting { attempt } => {
            tunnel_t1("tunnel.log.reconnecting", attempt.to_string())
        }
        TunnelEvent::Reconnected => tunnel_t("tunnel.log.reconnected"),
        TunnelEvent::AuthFailed { id } => tunnel_t1("tunnel.log.auth_failed", id.to_string()),
        TunnelEvent::PlayerRejected { id, reason } => {
            tunnel_t2("tunnel.log.player_rejected", id.to_string(), reason.to_string())
        }
        TunnelEvent::Error { message } => tunnel_t1("tunnel.log.error_event", message.clone()),
        _ => tunnel_t1("tunnel.log.event_unknown", format!("{event:?}")),
    }
}

pub(super) fn spawn_event_task(
    mut events: tokio::sync::mpsc::Receiver<TunnelEvent>,
) -> JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        while let Some(event) = events.recv().await {
            push_log(format_event(&event));
        }
    })
}
