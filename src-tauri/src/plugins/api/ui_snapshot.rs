use std::sync::{Mutex, OnceLock};

#[derive(Clone, serde::Serialize)]
pub struct BufferedUiEvent {
    pub plugin_id: String,
    pub action: String,
    pub element_id: String,
    pub html: String,
}

static UI_EVENT_SNAPSHOT: OnceLock<Mutex<Vec<BufferedUiEvent>>> = OnceLock::new();

fn get_ui_snapshot_store() -> &'static Mutex<Vec<BufferedUiEvent>> {
    UI_EVENT_SNAPSHOT.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn buffer_ui_event(plugin_id: &str, action: &str, element_id: &str, html: &str) {
    let mut store = get_ui_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    match action {
        "inject" | "insert" => {
            store.retain(|e| {
                !(e.plugin_id == plugin_id && e.element_id == element_id && e.action == action)
            });
            store.push(BufferedUiEvent {
                plugin_id: plugin_id.to_string(),
                action: action.to_string(),
                element_id: element_id.to_string(),
                html: html.to_string(),
            });
        }
        "update" => {
            if let Some(existing) = store
                .iter_mut()
                .find(|e| e.plugin_id == plugin_id && e.element_id == element_id)
            {
                existing.html = html.to_string();
                existing.action = "inject".to_string();
            } else {
                store.push(BufferedUiEvent {
                    plugin_id: plugin_id.to_string(),
                    action: "inject".to_string(),
                    element_id: element_id.to_string(),
                    html: html.to_string(),
                });
            }
        }
        "inject_css" => {
            store.retain(|e| {
                !(e.plugin_id == plugin_id && e.element_id == element_id && e.action == "inject_css")
            });
            store.push(BufferedUiEvent {
                plugin_id: plugin_id.to_string(),
                action: action.to_string(),
                element_id: element_id.to_string(),
                html: html.to_string(),
            });
        }
        "remove_css" => {
            store.retain(|e| {
                !(e.plugin_id == plugin_id && e.element_id == element_id && e.action == "inject_css")
            });
        }
        "remove" => {
            store.retain(|e| !(e.plugin_id == plugin_id && e.element_id == element_id));
        }
        "remove_all" => {
            store.retain(|e| e.plugin_id != plugin_id);
        }
        _ => {}
    }
}

pub fn take_ui_event_snapshot() -> Vec<BufferedUiEvent> {
    let store = get_ui_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.clone()
}

#[allow(dead_code)] // 外部调用
pub fn clear_plugin_ui_snapshot(plugin_id: &str) {
    let mut store = get_ui_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.retain(|e| e.plugin_id != plugin_id);
}
