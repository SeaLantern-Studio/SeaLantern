use std::sync::{Mutex, OnceLock};

#[derive(Clone, serde::Serialize)]
pub struct BufferedSidebarEvent {
    pub plugin_id: String,
    pub action: String,
    pub label: String,
    pub icon: String,
}

static SIDEBAR_EVENT_SNAPSHOT: OnceLock<Mutex<Vec<BufferedSidebarEvent>>> = OnceLock::new();

fn get_sidebar_snapshot_store() -> &'static Mutex<Vec<BufferedSidebarEvent>> {
    SIDEBAR_EVENT_SNAPSHOT.get_or_init(|| Mutex::new(Vec::new()))
}

pub(crate) fn buffer_sidebar_event(plugin_id: &str, action: &str, label: &str, icon: &str) {
    let mut store = get_sidebar_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    match action {
        "register" => {
            store.retain(|e| e.plugin_id != plugin_id);
            store.push(BufferedSidebarEvent {
                plugin_id: plugin_id.to_string(),
                action: action.to_string(),
                label: label.to_string(),
                icon: icon.to_string(),
            });
        }
        "unregister" => {
            store.retain(|e| e.plugin_id != plugin_id);
        }
        _ => {}
    }
}

pub fn take_sidebar_event_snapshot() -> Vec<BufferedSidebarEvent> {
    let store = get_sidebar_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.clone()
}

pub fn clear_plugin_sidebar_snapshot(plugin_id: &str) {
    let mut store = get_sidebar_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.retain(|e| e.plugin_id != plugin_id);
}
