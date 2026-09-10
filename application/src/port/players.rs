//! 玩家查询与玩家列表服务端口。

use async_trait::async_trait;
use sealantern_contract::{
    BanEntryDto, OpEntryDto, PlayerEntryDto, PlayerListError, PlayerLookupError, PlayerProfile,
};

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

/// 玩家列表查询宿主能力端口。
///
/// 在线玩家、白名单、封禁、OP 列表等通过控制台命令捕获的服务契约。
#[async_trait]
pub trait PlayerListService: Send + Sync {
    /// 获取在线玩家名列表（发 `list` 命令）。
    async fn get_online_players(&self, server_id: String) -> Result<Vec<String>, PlayerListError>;

    /// 获取白名单（发 `whitelist list`，UUID 由 usercache 反查）。
    ///
    /// 只收 `server_id`；内部经实例注册表解析出唯一可信目录，不信任前端传入
    /// 的 `server_path`（见 code review：server_id 与 server_path 分开信任）。
    async fn get_whitelist(
        &self,
        server_id: String,
    ) -> Result<Vec<PlayerEntryDto>, PlayerListError>;

    /// 获取封禁列表（发 `banlist`，UUID 由 usercache 反查）。
    ///
    /// 只收 `server_id`；内部经实例注册表解析出唯一可信目录。
    async fn get_banned_players(
        &self,
        server_id: String,
    ) -> Result<Vec<BanEntryDto>, PlayerListError>;

    /// 获取在线 OP 列表（从 `list` 输出里 `*` 前缀的玩家）。
    ///
    /// 只收 `server_id`；内部经实例注册表解析出唯一可信目录。
    async fn get_ops(&self, server_id: String) -> Result<Vec<OpEntryDto>, PlayerListError>;
}
