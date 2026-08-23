/*
 * @Author: hjcba 1174368998@qq.com
 * @Date: 2026-08-20 20:10:28
 * @LastEditors: hjcba 1174368998@qq.com
 * @LastEditTime: 2026-08-23 09:08:44
 * @FilePath: \SeaLantern\src-tauri\src\adapter\tauri\commands\player.rs
 * @Description: 玩家查询 Tauri 命令。
 */
//! 玩家查询 Tauri 命令。
//!

//! 解析逻辑、UUID 反查等业务逻辑在 `application::service::player`。

use sealantern_application::services::AppServices;
use sealantern_interface::{
    BanEntryDto, OpEntryDto, PlayerEntryDto, PlayerListError, PlayerListService, PlayerLookupError,
    PlayerLookupService, PlayerProfile,
};

/// 按用户名查询玩家档案（UUID），从服务器本地 usercache.json 读取。
#[tauri::command(rename_all = "snake_case")]
pub async fn lookup_player(
    server_path: String,
    username: String,
) -> Result<PlayerProfile, PlayerLookupError> {
    let service = AppServices::player_service()
        .await
        .map_err(|_| PlayerLookupError::ServiceUnavailable)?;
    service.lookup(server_path, username).await
}

/// 在线玩家：发 `list` 命令，捕获回显解析玩家名。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_online_players(server_id: String) -> Result<Vec<String>, PlayerListError> {
    let service = AppServices::player_service()
        .await
        .map_err(|_| PlayerListError::ServiceUnavailable)?;
    service.get_online_players(server_id).await
}

/// 白名单：发 `whitelist list`，解析名字后用 usercache 反查 UUID。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_whitelist(
    server_id: String,
    server_path: String,
) -> Result<Vec<PlayerEntryDto>, PlayerListError> {
    let service = AppServices::player_service()
        .await
        .map_err(|_| PlayerListError::ServiceUnavailable)?;
    service.get_whitelist(server_id, server_path).await
}

/// 封禁列表：发 `banlist`，解析名字+原因，UUID 用 usercache 反查。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_banned_players(
    server_id: String,
    server_path: String,
) -> Result<Vec<BanEntryDto>, PlayerListError> {
    let service = AppServices::player_service()
        .await
        .map_err(|_| PlayerListError::ServiceUnavailable)?;
    service.get_banned_players(server_id, server_path).await
}

/// OP 列表：发 `list` 命令，解析带 `*` 前缀的在线玩家。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_ops(
    server_id: String,
    server_path: String,
) -> Result<Vec<OpEntryDto>, PlayerListError> {
    let service = AppServices::player_service()
        .await
        .map_err(|_| PlayerListError::ServiceUnavailable)?;
    service.get_ops(server_id, server_path).await
}
