use crate::services::global::server_id_manager;
use crate::services::server_id_manager::{CreateServerIdRequest, ServerIdEntry, ServerIdResponse};

/// Create a new server ID
#[tauri::command]
pub async fn create_server_id(
    id: Option<String>,
    name: String,
    address: String,
    port: u16,
    description: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<ServerIdResponse, String> {
    let req = CreateServerIdRequest {
        id,
        name,
        address,
        port,
        description,
        tags,
    };

    let entry = server_id_manager().create_id(req).await?;

    Ok(ServerIdResponse {
        id: entry.id,
        name: entry.name,
        address: entry.address,
        port: entry.port,
        created_at: entry.created_at,
        is_active: entry.is_active,
    })
}

/// Resolve a server ID to its address
#[tauri::command]
pub async fn resolve_server_id(id: String) -> Result<(String, u16), String> {
    server_id_manager().resolve_id(&id).await
}

/// Get details of a server ID
#[tauri::command]
pub async fn get_server_id(id: String) -> Result<ServerIdEntry, String> {
    server_id_manager().get_id(&id).await
}

/// List all active server IDs
#[tauri::command]
pub async fn list_server_ids() -> Result<Vec<ServerIdEntry>, String> {
    Ok(server_id_manager().list_ids().await)
}

/// Update a server ID
#[tauri::command]
pub async fn update_server_id(
    id: String,
    name: Option<String>,
    address: Option<String>,
    port: Option<u16>,
) -> Result<ServerIdEntry, String> {
    server_id_manager()
        .update_id(&id, name, address, port)
        .await
}

/// Deactivate a server ID
#[tauri::command]
pub async fn deactivate_server_id(id: String) -> Result<(), String> {
    server_id_manager().deactivate_id(&id).await
}

/// Delete a server ID
#[tauri::command]
pub async fn delete_server_id(id: String) -> Result<(), String> {
    server_id_manager().delete_id(&id).await
}

/// Search server IDs by name or tags
#[tauri::command]
pub async fn search_server_ids(query: String) -> Result<Vec<ServerIdEntry>, String> {
    Ok(server_id_manager().search_ids(&query).await)
}
