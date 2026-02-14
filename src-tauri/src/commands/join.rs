use tauri::command;
use crate::services::global;
use crate::services::join_manager::ServerAddress;

#[command]
pub async fn resolve_server_id(id: String) -> Result<ServerAddress, String> {
    let join_manager = global::join_manager();
    join_manager.resolve_id(&id).await
}

#[command]
pub async fn join_server_by_id(id: String) -> Result<String, String> {
    let addr = resolve_server_id(id).await?;
    
    // 这里返回连接字符串，前端可以据此启动游戏并自动连接
    // 格式通常为: mc://join/<host>:<port> (取决于启动器支持)
    Ok(format!("{}:{}", addr.host, addr.port))
}
