//! 玩家查询与玩家列表的契约模型。

use serde::Serialize;

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

/// 单条玩家条目（含 UUID）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlayerEntryDto {
    /// 玩家 UUID（无连字符形式）。
    pub uuid: String,
    /// 玩家名。
    pub name: String,
}

/// 封禁条目。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BanEntryDto {
    /// 玩家 UUID（无连字符形式）。
    pub uuid: String,
    /// 玩家名。
    pub name: String,
    /// 封禁原因。
    pub reason: String,
}

/// OP 条目。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OpEntryDto {
    /// 玩家 UUID（无连字符形式）。
    pub uuid: String,
    /// 玩家名。
    pub name: String,
    /// OP 等级。
    pub level: i32,
}
