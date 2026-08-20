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
    pub uuid: String,
}

/// 玩家查询宿主能力端口。
#[async_trait]
pub trait PlayerLookupService: Send + Sync {
    /// 按用户名查询玩家档案。
    ///
    /// 输入为空或含非法字符返回 [`PlayerLookupError::InvalidInput`]；
    /// 目标不存在返回 [`PlayerLookupError::NotFound`]；上游限流或不可用时
    /// 返回 [`PlayerLookupError::RateLimited`] / [`PlayerLookupError::ServiceUnavailable`]。
    async fn lookup(&self, username: String) -> Result<PlayerProfile, PlayerLookupError>;
}
