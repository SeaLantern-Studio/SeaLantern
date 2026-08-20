//! 玩家查询 Tauri 命令。
//!
//! 参数/响应统一遵循 snake_case：命令参数与结构体 DTO 均显式声明
//! `rename_all = "snake_case"`，与后端契约模型保持一致。
use std::sync::Arc;

use sealantern_application::service::CorePlayerService;
use sealantern_application::services::AppServices;
use sealantern_interface::{PlayerLookupError, PlayerLookupService, PlayerProfile};

/// 获取全局玩家查询服务句柄（惰性初始化容器）。
async fn player_service() -> Result<Arc<CorePlayerService>, PlayerLookupError> {
    let services = AppServices::get()
        .await
        .map_err(|_| PlayerLookupError::ServiceUnavailable)?;
    Ok(services.player().clone())
}

/// 按用户名查询玩家档案（UUID 等）。
#[tauri::command(rename_all = "snake_case")]
pub async fn lookup_player(username: String) -> Result<PlayerProfile, PlayerLookupError> {
    let service = player_service().await?;
    service.lookup(username).await
}
