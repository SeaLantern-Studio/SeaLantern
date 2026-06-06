use std::sync::{Mutex, OnceLock};

#[derive(Clone, serde::Serialize)]
pub struct BufferedContextMenuEvent {
    pub plugin_id: String,
    pub action: String,
    pub context: String,
    pub items: String,
}

static CONTEXT_MENU_SNAPSHOT: OnceLock<Mutex<Vec<BufferedContextMenuEvent>>> = OnceLock::new();

fn get_context_menu_snapshot_store() -> &'static Mutex<Vec<BufferedContextMenuEvent>> {
    CONTEXT_MENU_SNAPSHOT.get_or_init(|| Mutex::new(Vec::new()))
}

pub(crate) fn buffer_context_menu_event(
    plugin_id: &str,
    action: &str,
    context: &str,
    items_json: &str,
) {
    let mut store = get_context_menu_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    match action {
        "register" => {
            store.retain(|e| !(e.plugin_id == plugin_id && e.context == context));
            store.push(BufferedContextMenuEvent {
                plugin_id: plugin_id.to_string(),
                action: action.to_string(),
                context: context.to_string(),
                items: items_json.to_string(),
            });
        }
        "unregister" => {
            store.retain(|e| !(e.plugin_id == plugin_id && e.context == context));
        }
        _ => {}
    }
}

pub fn take_context_menu_snapshot() -> Vec<BufferedContextMenuEvent> {
    let store = get_context_menu_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.clone()
}

pub fn clear_plugin_context_menu_snapshot(plugin_id: &str) {
    let mut store = get_context_menu_snapshot_store()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    store.retain(|e| e.plugin_id != plugin_id);
}
