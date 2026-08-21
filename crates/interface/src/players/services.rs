/*
 * @Author: hjcba 1174368998@qq.com
 * @Date: 2026-08-20 18:54:01
 * @LastEditors: hjcba 1174368998@qq.com
 * @LastEditTime: 2026-08-21 10:03:37
 * @FilePath: \SeaLantern\crates\interface\src\players\services.rs
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
//! 玩家查询服务契约。

use async_trait::async_trait;
use serde::Serialize;

use crate::error::PlayerLookupError;

/// 按用户名查询到的玩家档案。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlayerProfile {
    /// 玩家名。
    pub name: String,
    /// 玩家 UUID（无连字符形式）。
    ///
    /// 来源为服务器本地的 usercache.json，原始格式为 8-4-4-4-12 带连字符，
    /// 此处统一去掉连字符返回 32 位 hex。
    pub uuid: String,
}

/// 玩家查询宿主能力端口。
#[async_trait]
pub trait PlayerLookupService: Send + Sync {
    /// 按用户名查询玩家档案。
    ///
    /// 输入为空或含非法字符返回 [`PlayerLookupError::InvalidInput`]；
    /// 服务器路径为空返回 [`PlayerLookupError::ServerNotSelected`]；
    /// 目标不存在返回 [`PlayerLookupError::NotFound`]；
    /// 本地文件读取/解析失败返回 [`PlayerLookupError::ServiceUnavailable`]。
    async fn lookup(
        &self,
        server_path: String,
        username: String,
    ) -> Result<PlayerProfile, PlayerLookupError>;
}
