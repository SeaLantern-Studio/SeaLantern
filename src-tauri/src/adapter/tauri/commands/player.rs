/*
 * @Author: hjcba 1174368998@qq.com
 * @Date: 2026-08-20 20:10:28
 * @LastEditors: hjcba 1174368998@qq.com
 * @LastEditTime: 2026-08-21 10:04:47
 * @FilePath: \SeaLantern\src-tauri\src\adapter\tauri\commands\player.rs
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
//! 玩家查询 Tauri 命令。
//!
//! 参数/响应统一遵循 snake_case：命令参数与结构体 DTO 均显式声明
//! `rename_all = "snake_case"`，与后端契约模型保持一致。
use sealantern_application::services::AppServices;
use sealantern_interface::{PlayerLookupError, PlayerLookupService, PlayerProfile};

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
