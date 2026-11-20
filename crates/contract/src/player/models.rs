//! 玩家查询与玩家列表的契约模型。

use crate::dimension::Dimension;
use serde::{Deserialize, Serialize};

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
///
/// 直接对应服务器目录下 `whitelist.json` 的元素：读取该文件即可得到
/// 白名单，不要求服务器处于运行状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlayerEntryDto {
    /// 玩家 UUID（无连字符形式）。
    pub uuid: String,
    /// 玩家名。
    pub name: String,
}

/// 封禁条目。
///
/// 直接对应服务器目录下 `banned-players.json` 的元素：读取该文件即可得到
/// 封禁列表，不要求服务器处于运行状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BanEntryDto {
    /// 玩家 UUID（无连字符形式）。
    pub uuid: String,
    /// 玩家名。
    pub name: String,
    /// 封禁原因。
    #[serde(default)]
    pub reason: String,
    /// 执行封禁的操作者（玩家名或 `Server`）。
    #[serde(default)]
    pub source: String,
    /// 封禁创建时间（RFC 3339，UTC，例如 `2026-01-01T00:00:00+00:00`）。
    ///
    /// `banned-players.json` 原始格式为 `yyyy-MM-dd HH:mm:ss Z`，由服务层
    /// 归一化为 UTC 的 RFC 3339 后对外返回；无法解析时保留原值。
    #[serde(default)]
    pub created: String,
    /// 封禁到期时间（RFC 3339，UTC；`forever` 表示永久封禁，原样透传）。
    #[serde(default)]
    pub expires: String,
}

/// OP 条目。
///
/// 直接对应服务器目录下 `ops.json` 的元素：读取该文件即可得到完整 OP 名单
/// （含离线玩家）与真实等级，不要求服务器处于运行状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct OpEntryDto {
    /// 玩家 UUID（无连字符形式）。
    pub uuid: String,
    /// 玩家名。
    pub name: String,
    /// OP 等级（0-4）。
    #[serde(default)]
    pub level: i32,
    /// 是否绕过玩家数量上限。
    #[serde(default, alias = "bypassesPlayerLimit")]
    pub bypasses_player_limit: bool,
}

/// 一名在线玩家的实时位置。
///
/// 坐标与游戏内 F3 一致，单位为方块：`x` 东正、`z` 南正、`y` 上正。
/// 地图是俯视图，只用到 `(x, z)`；`y` 留给悬浮提示。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlayerPosition {
    /// 玩家名。
    pub name: String,
    /// X 坐标（东为正）。
    pub x: f64,
    /// Y 坐标（上为正）。
    pub y: f64,
    /// Z 坐标（南为正）。
    pub z: f64,
    /// 水平朝向，单位度。
    ///
    /// Minecraft 的约定：0 指向正南（+Z），90 指向正西（-X），
    /// 与数学上的极角方向相反，前端画箭头时不能直接当弧度用。
    pub yaw: f32,
    /// 该玩家所在维度。
    pub dimension: Dimension,
}

/// 某个维度下所有在线玩家的位置快照。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlayerPositions {
    /// 目标实例 ID。
    pub instance_id: String,
    /// 本次查询的维度。
    pub dimension: Dimension,
    /// 快照生成时间（Unix 秒）。
    pub captured_at_unix_secs: u64,
    /// 按玩家名升序排列的位置列表。
    pub players: Vec<PlayerPosition>,
}
