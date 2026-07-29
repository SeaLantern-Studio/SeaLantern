use std::sync::{Mutex, OnceLock};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct BufferedPermissionLog {
    pub plugin_id: String,
    pub log_type: String,
    pub action: String,
    pub detail: String,
    pub timestamp: u64,
}

static PERMISSION_LOG_SNAPSHOT: OnceLock<Mutex<Vec<BufferedPermissionLog>>> = OnceLock::new();

fn get_permission_log_snapshot_store() -> &'static Mutex<Vec<BufferedPermissionLog>> {
    PERMISSION_LOG_SNAPSHOT.get_or_init(|| Mutex::new(Vec::new()))
}

pub(crate) fn buffer_permission_log(
    plugin_id: &str,
    log_type: &str,
    action: &str,
    detail: &str,
    timestamp: u64,
) {
    let mut store = get_permission_log_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    store.push(BufferedPermissionLog {
        plugin_id: plugin_id.to_string(),
        log_type: log_type.to_string(),
        action: action.to_string(),
        detail: detail.to_string(),
        timestamp,
    });

    const MAX_PERMISSION_LOGS: usize = 1000;
    if store.len() > MAX_PERMISSION_LOGS {
        let overflow = store.len() - MAX_PERMISSION_LOGS;
        store.drain(0..overflow);
    }
}

pub fn take_permission_log_snapshot() -> Vec<BufferedPermissionLog> {
    let store = get_permission_log_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.clone()
}

pub fn get_plugin_permission_logs(plugin_id: &str) -> Vec<BufferedPermissionLog> {
    let store = get_permission_log_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store
        .iter()
        .filter(|e| e.plugin_id == plugin_id)
        .cloned()
        .collect()
}

pub fn clear_plugin_permission_logs(plugin_id: &str) {
    let mut store = get_permission_log_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.retain(|e| e.plugin_id != plugin_id);
}
