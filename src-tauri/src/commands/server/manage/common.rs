use crate::services::global;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceStopPreparationResponse {
    pub token: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStartFallbackEvent {
    pub server_id: String,
    pub server_name: String,
    pub from_mode: String,
    pub to_mode: String,
    pub reason: String,
}

pub(super) fn manager() -> &'static crate::services::server::manager::ServerManager {
    global::server_manager()
}
