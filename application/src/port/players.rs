//! 玩家查询与玩家列表服务端口。

use async_trait::async_trait;
use sealantern_contract::{
    BanEntryDto, OpEntryDto, PlayerAdminError, PlayerEntryDto, PlayerListError, PlayerLookupError,
    PlayerProfile,
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
/// 数据来源分两类：
/// - **在线玩家**：向运行中的服务器发 `list` 命令并捕获回显，要求服务器运行中。
/// - **白名单 / 封禁 / OP**：直接读取服务器目录下的配置文件
///   （`whitelist.json` / `banned-players.json` / `ops.json`），
///   **不要求服务器运行**，且能拿到离线玩家与真实等级。
#[async_trait]
pub trait PlayerListService: Send + Sync {
    /// 获取在线玩家名列表（发 `list` 命令，要求服务器运行中）。
    async fn get_online_players(&self, server_id: String) -> Result<Vec<String>, PlayerListError>;

    /// 获取白名单（读取服务器目录下的 `whitelist.json`）。
    ///
    /// 只收 `server_id`；内部经实例注册表解析出唯一可信目录，不信任前端传入
    /// 的 `server_path`（见 code review：server_id 与 server_path 分开信任）。
    async fn get_whitelist(
        &self,
        server_id: String,
    ) -> Result<Vec<PlayerEntryDto>, PlayerListError>;

    /// 获取封禁列表（读取服务器目录下的 `banned-players.json`）。
    ///
    /// 只收 `server_id`；内部经实例注册表解析出唯一可信目录。
    async fn get_banned_players(
        &self,
        server_id: String,
    ) -> Result<Vec<BanEntryDto>, PlayerListError>;

    /// 获取 OP 列表（读取服务器目录下的 `ops.json`，含离线 OP 与真实等级）。
    ///
    /// 只收 `server_id`；内部经实例注册表解析出唯一可信目录。
    async fn get_ops(&self, server_id: String) -> Result<Vec<OpEntryDto>, PlayerListError>;
}

/// 玩家管理写操作宿主能力端口。
///
/// 所有写操作都要求服务器处于运行状态：通过向其 stdin 写入一条控制台命令
/// （`whitelist add` / `whitelist remove` / `ban` / `pardon` / `op` / `deop` /
/// `kick`）实现，并捕获回显以确认命令确已送达服务器。
///
/// Minecraft 服务端在命令生效后会**立即**把结果落盘
/// （`whitelist.json` / `banned-players.json` / `ops.json`），因此写操作返回后，
/// [`PlayerListService`] 的读操作可以立刻看到变更，无需 `whitelist reload`，
/// 也无需由调用方手动改写文件。
#[async_trait]
pub trait PlayerAdminService: Send + Sync {
    /// 把玩家加入白名单（发 `whitelist add <name>`）。
    async fn add_to_whitelist(
        &self,
        server_id: String,
        name: String,
    ) -> Result<String, PlayerAdminError>;

    /// 把玩家移出白名单（发 `whitelist remove <name>`）。
    async fn remove_from_whitelist(
        &self,
        server_id: String,
        name: String,
    ) -> Result<String, PlayerAdminError>;

    /// 封禁玩家（发 `ban <name> [reason]`，`reason` 为空时不带原因）。
    async fn ban_player(
        &self,
        server_id: String,
        name: String,
        reason: String,
    ) -> Result<String, PlayerAdminError>;

    /// 解除封禁（发 `pardon <name>`）。
    async fn unban_player(
        &self,
        server_id: String,
        name: String,
    ) -> Result<String, PlayerAdminError>;

    /// 授予 OP（发 `op <name>`）。
    async fn add_op(&self, server_id: String, name: String) -> Result<String, PlayerAdminError>;

    /// 撤销 OP（发 `deop <name>`）。
    async fn remove_op(&self, server_id: String, name: String) -> Result<String, PlayerAdminError>;

    /// 踢出在线玩家（发 `kick <name> [reason]`，`reason` 为空时不带原因）。
    async fn kick_player(
        &self,
        server_id: String,
        name: String,
        reason: String,
    ) -> Result<String, PlayerAdminError>;
}
