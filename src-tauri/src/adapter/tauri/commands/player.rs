/*
 * @Author: hjcba 1174368998@qq.com
 * @Date: 2026-08-20 20:10:28
 * @LastEditors: hjcba 1174368998@qq.com
 * @LastEditTime: 2026-08-26 19:49:08
 * @FilePath: \SeaLantern\src-tauri\src\adapter\tauri\commands\player.rs
 * @Description: 玩家查询 Tauri 命令。
 */
//! 玩家查询 Tauri 命令。
//!
//! 读取/解析逻辑在 `application::service::player`：白名单、封禁、OP 直接读
//! 服务器目录下的配置文件（不要求服务器运行），在线玩家发 `list` 命令捕获。

use sealantern_application::port::{InstanceService, PlayerListService, PlayerLookupService};
use sealantern_application::services::AppServices;
use sealantern_contract::{
    BanEntryDto, OpEntryDto, PlayerEntryDto, PlayerListError, PlayerLookupError, PlayerProfile,
};
use sealantern_core::instance::InstanceId;
use tauri::State;

/// 由 server_id 经实例注册表解析出唯一可信的服务器目录。
///
/// 只信任 server_id，不接收前端传入的 server_path：避免 A 服命令输出与
/// B 服 usercache 拼出错误 UUID（见 code review：server_id 与 server_path
/// 分开信任）。
async fn resolve_server_path(
    services: &AppServices,
    server_id: &str,
) -> Result<String, PlayerLookupError> {
    let id = InstanceId::new(server_id).map_err(|_| PlayerLookupError::InvalidInput)?;
    let instance = services
        .instance()
        .find(&id)
        .await
        .map_err(|_| PlayerLookupError::ServiceUnavailable)?
        .ok_or(PlayerLookupError::NotFound)?;
    Ok(instance.directory.to_string_lossy().into_owned())
}

/// 按用户名查询玩家档案（UUID），从服务器本地 usercache.json 读取。
#[tauri::command(rename_all = "snake_case")]
pub async fn lookup_player(
    services: State<'_, AppServices>,
    server_id: String,
    username: String,
) -> Result<PlayerProfile, PlayerLookupError> {
    let server_path = resolve_server_path(&services, &server_id).await?;
    services.player().lookup(server_path, username).await
}

/// 在线玩家：发 `list` 命令，捕获回显解析玩家名（要求服务器运行中）。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_online_players(
    services: State<'_, AppServices>,
    server_id: String,
) -> Result<Vec<String>, PlayerListError> {
    services.player().get_online_players(server_id).await
}

/// 白名单：读取服务器目录下的 `whitelist.json`（不要求服务器运行）。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_whitelist(
    services: State<'_, AppServices>,
    server_id: String,
) -> Result<Vec<PlayerEntryDto>, PlayerListError> {
    services.player().get_whitelist(server_id).await
}

/// 封禁列表：读取服务器目录下的 `banned-players.json`（不要求服务器运行）。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_banned_players(
    services: State<'_, AppServices>,
    server_id: String,
) -> Result<Vec<BanEntryDto>, PlayerListError> {
    services.player().get_banned_players(server_id).await
}

/// OP 列表：读取服务器目录下的 `ops.json`（不要求服务器运行，含离线 OP）。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_ops(
    services: State<'_, AppServices>,
    server_id: String,
) -> Result<Vec<OpEntryDto>, PlayerListError> {
    services.player().get_ops(server_id).await
}
