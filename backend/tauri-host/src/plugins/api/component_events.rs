use std::sync::{Mutex, OnceLock};

#[derive(Clone, serde::Serialize)]
pub struct BufferedComponentEvent {
    pub plugin_id: String,
    pub payload_json: String,
}

static COMPONENT_EVENT_SNAPSHOT: OnceLock<Mutex<Vec<BufferedComponentEvent>>> = OnceLock::new();

fn get_component_snapshot_store() -> &'static Mutex<Vec<BufferedComponentEvent>> {
    COMPONENT_EVENT_SNAPSHOT.get_or_init(|| Mutex::new(Vec::new()))
}

pub(crate) fn buffer_component_event(plugin_id: &str, payload_json: &str) {
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(payload_json) {
        let action = parsed.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let component_id = parsed
            .get("component_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let prop = parsed.get("prop").and_then(|v| v.as_str()).unwrap_or("");
        let mut store = get_component_snapshot_store()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        match action {
            "create" => {
                store.retain(|e| e.plugin_id != plugin_id);
                store.push(BufferedComponentEvent {
                    plugin_id: plugin_id.to_string(),
                    payload_json: payload_json.to_string(),
                });
            }
            "set" => {
                store.retain(|e| {
                    if e.plugin_id != plugin_id {
                        return true;
                    }
                    if let Ok(p) = serde_json::from_str::<serde_json::Value>(&e.payload_json) {
                        let a = p.get("action").and_then(|v| v.as_str()).unwrap_or("");
                        let c = p.get("component_id").and_then(|v| v.as_str()).unwrap_or("");
                        let pr = p.get("prop").and_then(|v| v.as_str()).unwrap_or("");
                        !(a == "set" && c == component_id && pr == prop)
                    } else {
                        true
                    }
                });
                store.push(BufferedComponentEvent {
                    plugin_id: plugin_id.to_string(),
                    payload_json: payload_json.to_string(),
                });
            }
            _ => {}
        }
    }
}

pub fn take_component_event_snapshot() -> Vec<BufferedComponentEvent> {
    let store = get_component_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.clone()
}

pub fn clear_plugin_component_snapshot(plugin_id: &str) {
    let mut store = get_component_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.retain(|e| e.plugin_id != plugin_id);
}
