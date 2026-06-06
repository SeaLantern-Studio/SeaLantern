use std::sync::{Mutex, OnceLock};

#[derive(Clone, serde::Serialize)]
pub struct BufferedComponentEvent {
    pub plugin_id: String,
    pub payload_json: String,
}

fn parse_component_payload(payload_json: &str) -> Result<serde_json::Value, String> {
    serde_json::from_str::<serde_json::Value>(payload_json)
        .map_err(|error| format!("invalid component event payload JSON: {}", error))
}

static COMPONENT_EVENT_SNAPSHOT: OnceLock<Mutex<Vec<BufferedComponentEvent>>> = OnceLock::new();

fn get_component_snapshot_store() -> &'static Mutex<Vec<BufferedComponentEvent>> {
    COMPONENT_EVENT_SNAPSHOT.get_or_init(|| Mutex::new(Vec::new()))
}

pub(crate) fn buffer_component_event(plugin_id: &str, payload_json: &str) -> Result<(), String> {
    let parsed = parse_component_payload(payload_json)?;
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
                let Ok(parsed_existing) = parse_component_payload(&e.payload_json) else {
                    return true;
                };
                let existing_action = parsed_existing
                    .get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let existing_component_id = parsed_existing
                    .get("component_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let existing_prop = parsed_existing
                    .get("prop")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                !(existing_action == "set"
                    && existing_component_id == component_id
                    && existing_prop == prop)
            });
            store.push(BufferedComponentEvent {
                plugin_id: plugin_id.to_string(),
                payload_json: payload_json.to_string(),
            });
        }
        _ => {}
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::{
        buffer_component_event, clear_plugin_component_snapshot, take_component_event_snapshot,
    };

    #[test]
    fn buffer_component_event_rejects_invalid_json_without_snapshot_mutation() {
        clear_plugin_component_snapshot("plugin-a");

        let error = buffer_component_event("plugin-a", "{")
            .expect_err("invalid component payload should not be silently ignored");

        assert!(error.contains("invalid component event payload JSON"));
        assert!(take_component_event_snapshot()
            .iter()
            .all(|event| event.plugin_id != "plugin-a"));
    }
}
