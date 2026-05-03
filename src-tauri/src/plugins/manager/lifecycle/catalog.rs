use super::{PluginManager, PluginState};

pub(super) fn get_nav_items(manager: &PluginManager) -> Vec<serde_json::Value> {
    let mut nav_items = Vec::new();

    for (plugin_id, info) in &manager.plugins {
        if !matches!(info.state, PluginState::Enabled) {
            continue;
        }

        if let Some(ref ui) = info.manifest.ui {
            if let Some(ref sidebar) = ui.sidebar {
                nav_items.push(serde_json::json!({
                    "plugin_id": plugin_id,
                    "group": sidebar.group,
                    "label": sidebar.label,
                    "icon": sidebar.icon,
                    "priority": sidebar.priority.unwrap_or(0),
                    "pages": ui.pages.iter().map(|p| {
                        serde_json::json!({
                            "id": p.id,
                            "title": p.title,
                            "path": p.path,
                            "icon": p.icon,
                        })
                    }).collect::<Vec<_>>(),
                }));
            }
        }
    }

    nav_items
}
