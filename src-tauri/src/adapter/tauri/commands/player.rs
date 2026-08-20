//! 玩家查询 Tauri 命令。
//!
//! 参数/响应统一遵循 snake_case：命令参数与结构体 DTO 均显式声明
//! `rename_all = "snake_case"`，与后端契约模型保持一致。
use sealantern_application::services::AppServices;
use sealantern_interface::{PlayerLookupError, PlayerLookupService, PlayerProfile};

/// 按用户名查询玩家档案（UUID），从服务器本地 usercache.json 读取。
#[tauri::command(rename_all = "snake_case")]
pub async fn lookup_player(server_path: String, username: String) -> Result<PlayerProfile, PlayerLookupError> {
    let service = AppServices::player_service()
        .await
        .map_err(|_| PlayerLookupError::ServiceUnavailable)?;
    service.lookup(server_path, username).await
}
