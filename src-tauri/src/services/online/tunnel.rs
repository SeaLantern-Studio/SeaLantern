use sculk::persist::{generate_new_key, Profile};
use sculk::tunnel::{ConnectionSnapshot, HostConfig, IrohTunnel, JoinConfig, Ticket, TunnelEvent};
use sculk::types::{RelayUrl, SecretKey};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::async_runtime::JoinHandle;
use tokio::sync::Mutex as AsyncMutex;

const MAX_LOG_LINES: usize = 200;
const ONLINE_DIR: &str = "online";
const PROFILE_FILE: &str = "profile.toml";
const SECRET_KEY_FILE: &str = "secret.key";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TunnelMode {
    Host,
    Join,
}

impl TunnelMode {
    fn as_str(self) -> &'static str {
        match self {
            TunnelMode::Host => "host",
            TunnelMode::Join => "join",
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct TunnelConnection {
    pub remote_id: String,
    pub is_relay: bool,
    pub rtt_ms: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub alive: bool,
    pub elapsed_secs: u64,
}

#[derive(Debug, Serialize, Clone)]
pub struct TunnelStatus {
    pub running: bool,
    pub mode: Option<String>,
    pub ticket: Option<String>,
    pub connections: Vec<TunnelConnection>,
    pub logs: Vec<String>,
    pub host_port: u16,
    pub join_port: u16,
    pub last_ticket: Option<String>,
    pub relay_url: Option<String>,
}

#[derive(Default)]
struct TunnelRuntimeState {
    mode: Option<TunnelMode>,
    ticket: Option<String>,
    logs: Vec<String>,
    profile: Profile,
    secret_key: Option<SecretKey>,
    last_host_config: Option<LastHostConfig>,
}

struct ActiveTunnel {
    mode: TunnelMode,
    tunnel: Arc<IrohTunnel>,
    event_task: JoinHandle<()>,
}

fn active_tunnel() -> &'static AsyncMutex<Option<ActiveTunnel>> {
    static INSTANCE: OnceLock<AsyncMutex<Option<ActiveTunnel>>> = OnceLock::new();
    INSTANCE.get_or_init(|| AsyncMutex::new(None))
}

fn runtime_state() -> &'static Mutex<TunnelRuntimeState> {
    static INSTANCE: OnceLock<Mutex<TunnelRuntimeState>> = OnceLock::new();
    INSTANCE.get_or_init(|| Mutex::new(load_runtime_state()))
}

fn push_log(message: impl Into<String>) {
    let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
    state.logs.push(message.into());
    if state.logs.len() > MAX_LOG_LINES {
        let overflow = state.logs.len() - MAX_LOG_LINES;
        state.logs.drain(0..overflow);
    }
}

fn set_started(mode: TunnelMode, ticket: Option<String>) {
    let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
    state.mode = Some(mode);
    if let Some(ticket) = ticket {
        state.ticket = Some(ticket);
    }
    state.logs.clear();
}

fn clear_running_state() {
    let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
    state.mode = None;
}

fn tunnel_data_dir() -> PathBuf {
    crate::utils::path::get_app_data_dir().join(ONLINE_DIR)
}

fn tunnel_profile_path() -> PathBuf {
    tunnel_data_dir().join(PROFILE_FILE)
}

fn tunnel_key_path() -> PathBuf {
    tunnel_data_dir().join(SECRET_KEY_FILE)
}

fn load_existing_secret_key(path: &std::path::Path) -> Result<Option<SecretKey>, String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(format!("读取密钥失败: {e}"));
        }
    };
    if bytes.len() != 32 {
        return Err(format!("密钥长度错误: expected 32, got {}", bytes.len()));
    }
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|v: Vec<u8>| format!("密钥长度错误: expected 32, got {}", v.len()))?;
    Ok(Some(SecretKey::from_bytes(&arr)))
}

fn derive_ticket_for_state(state: &TunnelRuntimeState) -> Option<String> {
    let key = state.secret_key.as_ref()?;
    let relay = state.profile.resolve_relay_url(None).ok().flatten();
    Some(Ticket::new(key.public(), relay).to_string())
}

fn load_runtime_state() -> TunnelRuntimeState {
    let mut logs = Vec::new();
    let profile_path = tunnel_profile_path();
    let profile = match Profile::load_from(&profile_path) {
        Ok(p) => p,
        Err(e) => {
            logs.push(format!("加载 sculk Profile 失败，使用默认值: {e}"));
            Profile::default()
        }
    };
    let key_path = tunnel_key_path();
    let secret_key = match load_existing_secret_key(&key_path) {
        Ok(key) => key,
        Err(e) => {
            logs.push(format!("加载 sculk SecretKey 失败: {e}"));
            None
        }
    };

    let mut state = TunnelRuntimeState {
        mode: None,
        ticket: None,
        logs,
        profile,
        secret_key,
        last_host_config: None,
    };
    state.ticket = derive_ticket_for_state(&state);
    state
}

fn save_profile_in_state(state: &mut TunnelRuntimeState) {
    if let Err(e) = state.profile.save_to(&tunnel_profile_path()) {
        state.logs.push(format!("保存 sculk Profile 失败: {e}"));
    }
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn parse_relay_url(value: Option<String>) -> Result<Option<RelayUrl>, String> {
    let Some(raw) = normalize_optional_string(value) else {
        return Ok(None);
    };
    raw.parse::<RelayUrl>()
        .map(Some)
        .map_err(|e| format!("无效中继地址: {e}"))
}

fn apply_relay_preference(
    profile: &mut Profile,
    relay_input: Option<String>,
) -> Result<Option<RelayUrl>, String> {
    let normalized = normalize_optional_string(relay_input);
    let parsed = parse_relay_url(normalized.clone())?;

    if let Some(url) = normalized {
        profile.relay.custom = true;
        profile.relay.url = Some(url);
        Ok(parsed)
    } else {
        profile.relay.custom = false;
        profile.relay.url = None;
        Ok(None)
    }
}

fn map_connection(snapshot: ConnectionSnapshot) -> TunnelConnection {
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
        TunnelEvent::PlayerJoined { id } => format!("玩家加入: {id}"),
        TunnelEvent::PlayerLeft { id, reason } => format!("玩家离开: {id} ({reason})"),
        TunnelEvent::Connected => "已连接到房主".to_string(),
        TunnelEvent::Disconnected { reason } => format!("连接断开: {reason}"),
        TunnelEvent::PathChanged { remote_id, is_relay, rtt_ms } => {
            let route = if *is_relay { "relay" } else { "direct" };
            format!("路径变化: {remote_id}, route={route}, rtt={rtt_ms}ms")
        }
        TunnelEvent::Reconnecting { attempt } => format!("重连中，第 {attempt} 次尝试"),
        TunnelEvent::Reconnected => "重连成功".to_string(),
        TunnelEvent::AuthFailed { id } => format!("认证失败: {id}"),
        TunnelEvent::PlayerRejected { id, reason } => format!("连接被拒绝: {id} ({reason})"),
        TunnelEvent::Error { message } => format!("错误: {message}"),
        _ => format!("{event:?}"),
    }
}

fn spawn_event_task(mut events: tokio::sync::mpsc::Receiver<TunnelEvent>) -> JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        while let Some(event) = events.recv().await {
            push_log(format_event(&event));
        }
    })
}

async fn take_active_tunnel() -> Option<ActiveTunnel> {
    let mut active = active_tunnel().lock().await;
    active.take()
}

async fn replace_with_active_tunnel(next: ActiveTunnel) {
    let mut active = active_tunnel().lock().await;
    *active = Some(next);
}

async fn stop_previous_for_restart() {
    if let Some(previous) = take_active_tunnel().await {
        previous.tunnel.close().await;
        previous.event_task.abort();
        clear_running_state();
    }
}

pub async fn host(
    port: u16,
    password: Option<String>,
    max_players: Option<u32>,
    relay_url: Option<String>,
) -> Result<TunnelStatus, String> {
    if port == 0 {
        return Err("端口不能为 0".to_string());
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
        if state.secret_key.is_none() {
            let key = generate_new_key(&tunnel_key_path())
                .map_err(|e| format!("生成隧道密钥失败: {e}"))?;
            state.secret_key = Some(key);
            state.ticket = derive_ticket_for_state(&state);
        }
        state.secret_key.clone()
    };
    stop_previous_for_restart().await;

    let normalized_password = normalize_optional_string(password);
    let normalized_max_players = max_players.filter(|v| *v > 0);
    let config = HostConfig::default()
        .password(normalized_password.clone())
        .max_players(normalized_max_players);

    let (tunnel, ticket, events) = IrohTunnel::host(port, secret_key, relay, config)
        .await
        .map_err(|e| format!("启动 Host 隧道失败: {e}"))?;

    let active = ActiveTunnel {
        mode: TunnelMode::Host,
        tunnel: Arc::new(tunnel),
        event_task: spawn_event_task(events),
    };
    let ticket_str = ticket.to_string();

    replace_with_active_tunnel(active).await;
    {
        let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
        state.last_host_config = Some(LastHostConfig {
            password: normalized_password,
            max_players: normalized_max_players,
        });
    }
    set_started(TunnelMode::Host, Some(ticket_str.clone()));
    push_log(format!("Host 隧道已启动，端口={port}"));
    push_log(format!("分享票据: {ticket_str}"));
    if sculk::clipboard::clipboard_copy(&ticket_str) {
        push_log("票据已复制到系统剪贴板".to_string());
    } else {
        push_log("票据复制失败，请手动复制".to_string());
    }

    Ok(status().await)
}

pub async fn join(
    ticket: String,
    local_port: u16,
    password: Option<String>,
) -> Result<TunnelStatus, String> {
    if local_port == 0 {
        return Err("本地端口不能为 0".to_string());
    }

    let ticket_trimmed = ticket.trim().to_string();
    if ticket_trimmed.is_empty() {
        return Err("票据不能为空".to_string());
    }
    let parsed_ticket = ticket_trimmed
        .parse::<Ticket>()
        .map_err(|e| format!("票据格式错误: {e}"))?;

    stop_previous_for_restart().await;

    let config = JoinConfig::default().password(normalize_optional_string(password));
    let (tunnel, events) = IrohTunnel::join(&parsed_ticket, local_port, config)
        .await
        .map_err(|e| format!("加入隧道失败: {e}"))?;

    let active = ActiveTunnel {
        mode: TunnelMode::Join,
        tunnel: Arc::new(tunnel),
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
    push_log(format!("Join 隧道已启动，本地端口={local_port}"));

    Ok(status().await)
}

pub async fn regenerate_ticket() -> Result<TunnelStatus, String> {
    let (port, relay, key, host_cfg) = {
        let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
        if state.mode != Some(TunnelMode::Host) {
            return Err("当前不是建房模式，无法重新生成票据".to_string());
        }
        let host_cfg = state
            .last_host_config
            .clone()
            .ok_or_else(|| "缺少建房配置，无法重新生成票据".to_string())?;
        let key_path = tunnel_key_path();
        if let Err(e) = std::fs::remove_file(&key_path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(format!("删除旧密钥失败: {e}"));
            }
        }
        let key = generate_new_key(&key_path).map_err(|e| format!("重新生成密钥失败: {e}"))?;
        state.secret_key = Some(key.clone());
        let relay = state
            .profile
            .resolve_relay_url(None)
            .map_err(|e| format!("解析中继地址失败: {e}"))?;
        (state.profile.host.port, relay, key, host_cfg)
    };

    stop_previous_for_restart().await;

    let config = HostConfig::default()
        .password(host_cfg.password)
        .max_players(host_cfg.max_players);
    let (tunnel, ticket, events) = IrohTunnel::host(port, Some(key), relay, config)
        .await
        .map_err(|e| format!("重新生成票据失败: {e}"))?;

    let active = ActiveTunnel {
        mode: TunnelMode::Host,
        tunnel: Arc::new(tunnel),
        event_task: spawn_event_task(events),
    };
    let ticket_str = ticket.to_string();

    replace_with_active_tunnel(active).await;
    set_started(TunnelMode::Host, Some(ticket_str.clone()));
    push_log("票据已重新生成".to_string());
    push_log(format!("新票据: {ticket_str}"));
    if sculk::clipboard::clipboard_copy(&ticket_str) {
        push_log("新票据已复制到系统剪贴板".to_string());
    }

    Ok(status().await)
}

pub async fn generate_ticket() -> Result<TunnelStatus, String> {
    let active_running = {
        let active = active_tunnel().lock().await;
        active.is_some()
    };
    if active_running {
        return Err("隧道运行中，无法生成新票据".to_string());
    }

    {
        let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
        if state.secret_key.is_none() {
            let key = generate_new_key(&tunnel_key_path())
                .map_err(|e| format!("生成隧道密钥失败: {e}"))?;
            state.secret_key = Some(key);
        }
        state.ticket = derive_ticket_for_state(&state);
    }
    push_log("票据已生成".to_string());

    Ok(status().await)
}

pub async fn stop() -> Result<TunnelStatus, String> {
    if let Some(active) = take_active_tunnel().await {
        active.tunnel.close().await;
        active.event_task.abort();
        clear_running_state();
        push_log("隧道已停止".to_string());
    } else {
        push_log("当前没有运行中的隧道".to_string());
    }

    Ok(status().await)
}

pub async fn copy_ticket() -> Result<bool, String> {
    let ticket = {
        let state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
        state.ticket.clone()
    };
    let Some(ticket) = ticket else {
        return Err("当前没有可复制的票据".to_string());
    };
    let copied = sculk::clipboard::clipboard_copy(&ticket);
    if copied {
        push_log("票据已复制到系统剪贴板".to_string());
    } else {
        push_log("票据复制失败".to_string());
    }
    Ok(copied)
}

pub async fn status() -> TunnelStatus {
    let (tunnel_ref, mode_from_active) = {
        let active = active_tunnel().lock().await;
        (active.as_ref().map(|t| Arc::clone(&t.tunnel)), active.as_ref().map(|t| t.mode))
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
        .map(|m| m.as_str().to_string());
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
#[derive(Debug, Clone)]
struct LastHostConfig {
    password: Option<String>,
    max_players: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_relay_preference_rejects_invalid_input_without_mutating_profile() {
        let mut profile = Profile::default();
        profile.relay.custom = true;
        profile.relay.url = Some("https://relay.example.com".to_string());
        let before = profile.clone();

        let result = apply_relay_preference(&mut profile, Some("not a relay url".to_string()));

        assert!(result.is_err());
        assert_eq!(profile.relay.custom, before.relay.custom);
        assert_eq!(profile.relay.url, before.relay.url);
    }

    #[test]
    fn apply_relay_preference_clears_custom_relay_when_input_empty() {
        let mut profile = Profile::default();
        profile.relay.custom = true;
        profile.relay.url = Some("https://relay.example.com".to_string());

        let result = apply_relay_preference(&mut profile, None);

        assert!(result.is_ok());
        assert!(!profile.relay.custom);
        assert!(profile.relay.url.is_none());
    }
}
