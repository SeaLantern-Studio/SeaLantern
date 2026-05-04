use sculk::persist::Profile;
use sculk::tunnel::{IrohTunnel, Ticket};
use sculk::types::SecretKey;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::async_runtime::JoinHandle;
use tokio::sync::Mutex as AsyncMutex;

pub(super) const MAX_LOG_LINES: usize = 200;
pub(super) const ONLINE_DIR: &str = "online";
pub(super) const PROFILE_FILE: &str = "profile.toml";
pub(super) const SECRET_KEY_FILE: &str = "secret.key";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TunnelMode {
    Host,
    Join,
}

impl TunnelMode {
    pub(super) fn as_str(self) -> &'static str {
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
pub(super) struct TunnelRuntimeState {
    pub mode: Option<TunnelMode>,
    pub ticket: Option<String>,
    pub logs: Vec<String>,
    pub profile: Profile,
    pub secret_key: Option<SecretKey>,
}

pub(super) struct ActiveTunnel {
    pub mode: TunnelMode,
    pub tunnel: Arc<IrohTunnel>,
    pub event_task: JoinHandle<()>,
}

pub(super) fn active_tunnel() -> &'static AsyncMutex<Option<ActiveTunnel>> {
    static INSTANCE: OnceLock<AsyncMutex<Option<ActiveTunnel>>> = OnceLock::new();
    INSTANCE.get_or_init(|| AsyncMutex::new(None))
}

pub(super) fn runtime_state() -> &'static Mutex<TunnelRuntimeState> {
    static INSTANCE: OnceLock<Mutex<TunnelRuntimeState>> = OnceLock::new();
    INSTANCE.get_or_init(|| Mutex::new(super::config::load_runtime_state()))
}

pub(super) fn push_log(message: impl Into<String>) {
    let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
    state.logs.push(message.into());
    if state.logs.len() > MAX_LOG_LINES {
        let overflow = state.logs.len() - MAX_LOG_LINES;
        state.logs.drain(0..overflow);
    }
}

pub(super) fn set_started(mode: TunnelMode, ticket: Option<String>) {
    let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
    state.mode = Some(mode);
    if let Some(ticket) = ticket {
        state.ticket = Some(ticket);
    }
    state.logs.clear();
}

pub(super) fn clear_running_state() {
    let mut state = runtime_state().lock().unwrap_or_else(|e| e.into_inner());
    state.mode = None;
}

pub(super) fn tunnel_data_dir() -> PathBuf {
    crate::utils::path::get_app_data_dir().join(ONLINE_DIR)
}

pub(super) fn tunnel_profile_path() -> PathBuf {
    tunnel_data_dir().join(PROFILE_FILE)
}

pub(super) fn tunnel_key_path() -> PathBuf {
    tunnel_data_dir().join(SECRET_KEY_FILE)
}

pub(super) fn derive_ticket_for_state(state: &TunnelRuntimeState) -> Option<String> {
    let key = state.secret_key.as_ref()?;
    let relay = state.profile.resolve_relay_url(None).ok().flatten();
    Some(Ticket::new(key.public(), relay).to_string())
}

pub(super) async fn take_active_tunnel() -> Option<ActiveTunnel> {
    let mut active = active_tunnel().lock().await;
    active.take()
}

pub(super) async fn replace_with_active_tunnel(next: ActiveTunnel) {
    let mut active = active_tunnel().lock().await;
    *active = Some(next);
}

pub(super) async fn stop_previous_for_restart() {
    if let Some(previous) = take_active_tunnel().await {
        previous.tunnel.close().await;
        previous.event_task.abort();
        clear_running_state();
    }
}
