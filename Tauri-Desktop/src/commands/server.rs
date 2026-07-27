//! 服务器相关命令

use sealantern_extra::models::ServerInstance;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

/// 服务器状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusInfo {
    pub id: String,
    pub status: String,
    pub pid: Option<u32>,
    pub uptime: Option<u64>,
}

/// 服务器管理状态
pub struct ServerState {
    pub servers: Mutex<HashMap<String, ServerInstance>>,
    pub statuses: Mutex<HashMap<String, ServerStatusInfo>>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            servers: Mutex::new(HashMap::new()),
            statuses: Mutex::new(HashMap::new()),
        }
    }
}

/// 获取服务器列表
#[tauri::command]
pub async fn get_server_list(state: State<'_, ServerState>) -> Result<Vec<ServerInstance>, String> {
    let servers = state.servers.lock().map_err(|e| e.to_string())?;
    Ok(servers.values().cloned().collect())
}

/// 获取服务器状态
#[tauri::command]
pub async fn get_server_status(
    id: String,
    state: State<'_, ServerState>,
) -> Result<ServerStatusInfo, String> {
    let statuses = state.statuses.lock().map_err(|e| e.to_string())?;
    statuses
        .get(&id)
        .cloned()
        .ok_or_else(|| format!("Server {} not found", id))
}

/// 启动服务器
#[tauri::command]
pub async fn start_server(id: String, _state: State<'_, ServerState>) -> Result<(), String> {
    // TODO: 实现真正的服务器启动逻辑
    tracing::info!("Starting server: {}", id);
    Ok(())
}

/// 停止服务器
#[tauri::command]
pub async fn stop_server(id: String, _state: State<'_, ServerState>) -> Result<(), String> {
    // TODO: 实现真正的服务器停止逻辑
    tracing::info!("Stopping server: {}", id);
    Ok(())
}

/// 删除服务器
#[tauri::command]
pub async fn delete_server(id: String, state: State<'_, ServerState>) -> Result<(), String> {
    let mut servers = state.servers.lock().map_err(|e| e.to_string())?;
    servers.remove(&id);
    Ok(())
}

/// 更新服务器名称
#[tauri::command]
pub async fn update_server_name(
    id: String,
    name: String,
    state: State<'_, ServerState>,
) -> Result<(), String> {
    let mut servers = state.servers.lock().map_err(|e| e.to_string())?;
    if let Some(server) = servers.get_mut(&id) {
        server.name = name;
    }
    Ok(())
}