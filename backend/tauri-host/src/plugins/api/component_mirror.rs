use std::sync::RwLock;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ComponentEntry {
    pub id: String,
    pub component_type: String,
}

static COMPONENT_MIRROR: RwLock<Vec<ComponentEntry>> = RwLock::new(Vec::new());

pub fn component_mirror_register(id: &str, component_type: &str) {
    let mut mirror = COMPONENT_MIRROR.write().unwrap_or_else(|e| e.into_inner());

    mirror.retain(|e| e.id != id);
    mirror.push(ComponentEntry {
        id: id.to_string(),
        component_type: component_type.to_string(),
    });
}

pub fn component_mirror_unregister(id: &str) {
    let mut mirror = COMPONENT_MIRROR.write().unwrap_or_else(|e| e.into_inner());
    mirror.retain(|e| e.id != id);
}

pub fn component_mirror_list(page_filter: Option<&str>) -> Vec<ComponentEntry> {
    let mirror = COMPONENT_MIRROR.read().unwrap_or_else(|e| e.into_inner());
    match page_filter {
        Some(f) => mirror
            .iter()
            .filter(|e| e.id.starts_with(&format!("{}/", f)))
            .cloned()
            .collect(),
        None => mirror.clone(),
    }
}

pub fn component_mirror_clear() {
    let mut mirror = COMPONENT_MIRROR.write().unwrap_or_else(|e| e.into_inner());
    mirror.clear();
}
