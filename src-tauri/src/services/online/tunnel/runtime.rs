use super::config::{
    apply_relay_preference, ensure_secret_key, normalize_optional_string, save_profile_in_state,
};
use super::events::{map_connection, spawn_event_task};
use super::i18n::{tunnel_t, tunnel_t1};
use super::state::{
    active_tunnel, clear_running_state, push_log, replace_with_active_tunnel, runtime_state,
    set_started, stop_previous_for_restart, ActiveTunnel, TunnelMode, TunnelStatus,
};
use sculk::persist::generate_new_key;
use sculk::tunnel::{HostConfig, IrohTunnel, JoinConfig, Ticket};

pub async fn host(
    port: u16,
    password: Option<String>,
    max_players: Option<u32>,
    relay_url: Option<String>,
) -> Result<TunnelStatus, String> {
    if port == 0 {
        return Err(tunnel_t("tunnel.err.port_zero_host"));
    }

    let relay = {
        let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
        state.profile.host.port = port;
        let relay = apply_relay_preference(&mut state.profile, relay_url)?;
        save_profile_in_state(&mut state);
        relay
    };
    let secret_key = {
        let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
        ensure_secret_key(&mut state)?
    };
    stop_previous_for_restart().await;

    let normalized_password = normalize_optional_string(password);
    let normalized_max_players = max_players.filter(|value| *value > 0);
    let config = HostConfig::default()
        .password(normalized_password.clone())
        .max_players(normalized_max_players);

    let (tunnel, ticket, events) = IrohTunnel::host(port, Some(secret_key), relay, config)
        .await
        .map_err(|e| tunnel_t1("tunnel.err.host_start_failed", e.to_string()))?;

    let active = ActiveTunnel {
        mode: TunnelMode::Host,
        tunnel: std::sync::Arc::new(tunnel),
        event_task: spawn_event_task(events),
    };
    let ticket_str = ticket.to_string();

    replace_with_active_tunnel(active).await;
    set_started(TunnelMode::Host, Some(ticket_str.clone()));
    push_log(tunnel_t1("tunnel.log.host_started", format!("{port}")));
    push_log(tunnel_t1("tunnel.log.share_ticket", ticket_str.clone()));
    if sculk::clipboard::clipboard_copy(&ticket_str) {
        push_log(tunnel_t("tunnel.log.ticket_copied"));
    } else {
        push_log(tunnel_t("tunnel.log.ticket_copy_failed_manual"));
    }

    Ok(status().await)
}

pub async fn join(
    ticket: String,
    local_port: u16,
    password: Option<String>,
) -> Result<TunnelStatus, String> {
    if local_port == 0 {
        return Err(tunnel_t("tunnel.err.local_port_zero"));
    }

    let ticket_trimmed = ticket.trim().to_string();
    if ticket_trimmed.is_empty() {
        return Err(tunnel_t("tunnel.err.ticket_empty"));
    }
    let parsed_ticket = ticket_trimmed
        .parse::<Ticket>()
        .map_err(|e| tunnel_t1("tunnel.err.ticket_format", e.to_string()))?;

    stop_previous_for_restart().await;

    let config = JoinConfig::default().password(normalize_optional_string(password));
    let (tunnel, events) = IrohTunnel::join(&parsed_ticket, local_port, config)
        .await
        .map_err(|e| tunnel_t1("tunnel.err.join_failed", e.to_string()))?;

    let active = ActiveTunnel {
        mode: TunnelMode::Join,
        tunnel: std::sync::Arc::new(tunnel),
        event_task: spawn_event_task(events),
    };

    replace_with_active_tunnel(active).await;
    set_started(TunnelMode::Join, None);
    {
        let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
        state.profile.join.port = local_port;
        state.profile.join.last_ticket = Some(ticket_trimmed);
        save_profile_in_state(&mut state);
    }
    push_log(tunnel_t1("tunnel.log.join_started", format!("{local_port}")));

    Ok(status().await)
}

pub async fn regenerate_ticket() -> Result<TunnelStatus, String> {
    let active_running = {
        let active = active_tunnel().lock().await;
        active.is_some()
    };
    if active_running {
        return Err(tunnel_t("tunnel.err.tunnel_running_generate"));
    }

    let key_path = super::state::tunnel_key_path();
    if let Err(e) = std::fs::remove_file(&key_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(tunnel_t1("tunnel.err.delete_old_key_failed", e.to_string()));
        }
    }

    let key = generate_new_key(&key_path)
        .map_err(|e| tunnel_t1("tunnel.err.regenerate_key_failed", e.to_string()))?;

    {
        let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
        state.secret_key = Some(key);
        state.ticket = super::state::derive_ticket_for_state(&state);
    }

    push_log(tunnel_t("tunnel.log.ticket_regenerated"));
    Ok(status().await)
}

pub async fn generate_ticket() -> Result<TunnelStatus, String> {
    let active_running = {
        let active = active_tunnel().lock().await;
        active.is_some()
    };
    if active_running {
        return Err(tunnel_t("tunnel.err.tunnel_running_generate"));
    }

    {
        let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
        if state.secret_key.is_none() {
            let key = generate_new_key(&super::state::tunnel_key_path())
                .map_err(|e| tunnel_t1("tunnel.err.generate_key_failed", e.to_string()))?;
            state.secret_key = Some(key);
        }
        state.ticket = super::state::derive_ticket_for_state(&state);
    }
    push_log(tunnel_t("tunnel.log.ticket_generated"));

    Ok(status().await)
}

pub async fn stop() -> Result<TunnelStatus, String> {
    if let Some(active) = super::state::take_active_tunnel().await {
        active.tunnel.close().await;
        active.event_task.abort();
        clear_running_state();
        push_log(tunnel_t("tunnel.log.tunnel_stopped"));
    } else {
        push_log(tunnel_t("tunnel.log.no_tunnel_running"));
    }

    Ok(status().await)
}

pub async fn copy_ticket() -> Result<bool, String> {
    let ticket = {
        let state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
        state.ticket.clone()
    };
    let Some(ticket) = ticket else {
        return Err(tunnel_t("tunnel.err.no_ticket_to_copy"));
    };
    let copied = sculk::clipboard::clipboard_copy(&ticket);
    if copied {
        push_log(tunnel_t("tunnel.log.ticket_copied"));
    } else {
        push_log(tunnel_t("tunnel.log.ticket_copy_failed_short"));
    }
    Ok(copied)
}

pub async fn status() -> TunnelStatus {
    let (tunnel_ref, mode_from_active) = {
        let active = active_tunnel().lock().await;
        (
            active
                .as_ref()
                .map(|tunnel| std::sync::Arc::clone(&tunnel.tunnel)),
            active.as_ref().map(|tunnel| tunnel.mode),
        )
    };

    let connections = tunnel_ref
        .as_ref()
        .map(|tunnel| {
            tunnel
                .connections()
                .map(|items| items.into_iter().map(map_connection).collect())
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
    let mode = mode_from_active
        .or(state.mode)
        .map(|mode| mode.as_str().to_string());
    let ticket = state.ticket.clone();
    let logs = state.logs.clone();
    let running = tunnel_ref.is_some();
    let host_port = state.profile.host.port;
    let join_port = state.profile.join.port;
    let last_ticket = state.profile.join.last_ticket.clone();
    let relay_url = if state.profile.relay.custom {
        state.profile.relay.url.clone()
    } else {
        None
    };

    TunnelStatus {
        running,
        mode,
        ticket,
        connections,
        logs,
        host_port,
        join_port,
        last_ticket,
        relay_url,
    }
}
