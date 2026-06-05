use super::models::{CreateServerIdRequest, ServerIdEntry};
use super::shared::{current_unix_secs, generate_id_from_name, validate_custom_id};
use super::SharedServerIdEntries;

/// 创建短 ID
pub(super) async fn create_id(
    entries: &SharedServerIdEntries,
    req: CreateServerIdRequest,
) -> Result<ServerIdEntry, String> {
    let id = if let Some(custom_id) = req.id {
        validate_custom_id(&custom_id)?;
        custom_id
    } else {
        generate_id_from_name(&req.name)
    };

    let current_entries = entries.read().await;
    if current_entries.contains_key(&id) {
        return Err(format!("Server ID '{}' already exists", id));
    }
    drop(current_entries);

    let entry = ServerIdEntry {
        id: id.clone(),
        name: req.name,
        address: req.address,
        port: req.port,
        description: req.description,
        created_at: current_unix_secs(),
        last_accessed_at: None,
        is_active: true,
        tags: req.tags.unwrap_or_default(),
    };

    let mut current_entries = entries.write().await;
    current_entries.insert(id, entry.clone());
    Ok(entry)
}

/// 解析短 ID 到地址
pub(super) async fn resolve_id(
    entries: &SharedServerIdEntries,
    id: &str,
) -> Result<(String, u16), String> {
    let mut current_entries = entries.write().await;

    match current_entries.get_mut(id) {
        Some(entry) => {
            if !entry.is_active {
                return Err(format!("Server ID '{}' is inactive", id));
            }
            entry.last_accessed_at = Some(current_unix_secs());
            Ok((entry.address.clone(), entry.port))
        }
        None => Err(format!("Server ID '{}' not found", id)),
    }
}

/// 读取短 ID 详情
pub(super) async fn get_id(
    entries: &SharedServerIdEntries,
    id: &str,
) -> Result<ServerIdEntry, String> {
    let current_entries = entries.read().await;
    current_entries
        .get(id)
        .cloned()
        .ok_or_else(|| format!("Server ID '{}' not found", id))
}

/// 列出全部启用中的短 ID
pub(super) async fn list_ids(entries: &SharedServerIdEntries) -> Vec<ServerIdEntry> {
    let current_entries = entries.read().await;
    current_entries
        .values()
        .filter(|entry| entry.is_active)
        .cloned()
        .collect()
}

/// 更新短 ID
pub(super) async fn update_id(
    entries: &SharedServerIdEntries,
    id: &str,
    name: Option<String>,
    address: Option<String>,
    port: Option<u16>,
) -> Result<ServerIdEntry, String> {
    let mut current_entries = entries.write().await;

    match current_entries.get_mut(id) {
        Some(entry) => {
            if let Some(name) = name {
                entry.name = name;
            }
            if let Some(address) = address {
                entry.address = address;
            }
            if let Some(port) = port {
                entry.port = port;
            }
            Ok(entry.clone())
        }
        None => Err(format!("Server ID '{}' not found", id)),
    }
}

/// 停用短 ID
pub(super) async fn deactivate_id(entries: &SharedServerIdEntries, id: &str) -> Result<(), String> {
    let mut current_entries = entries.write().await;

    match current_entries.get_mut(id) {
        Some(entry) => {
            entry.is_active = false;
            Ok(())
        }
        None => Err(format!("Server ID '{}' not found", id)),
    }
}

/// 删除短 ID
pub(super) async fn delete_id(entries: &SharedServerIdEntries, id: &str) -> Result<(), String> {
    let mut current_entries = entries.write().await;

    if current_entries.remove(id).is_some() {
        Ok(())
    } else {
        Err(format!("Server ID '{}' not found", id))
    }
}

/// 搜索短 ID
pub(super) async fn search_ids(entries: &SharedServerIdEntries, query: &str) -> Vec<ServerIdEntry> {
    let current_entries = entries.read().await;
    let query_lower = query.to_lowercase();

    current_entries
        .values()
        .filter(|entry| {
            entry.is_active
                && (entry.name.to_lowercase().contains(&query_lower)
                    || entry.id.to_lowercase().contains(&query_lower)
                    || entry
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower)))
        })
        .cloned()
        .collect()
}
