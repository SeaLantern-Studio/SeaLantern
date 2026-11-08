//! 维度（主世界 / 下界 / 末地）。
//!
//! 地图与玩家位置都要区分维度：三者的 region 目录不同，坐标空间也各自独立，
//! 下界的 1 格对应主世界的 8 格，混在一起画会得到错误的地图。
//!
//! 序列化名与前端/HTTP 查询参数一致：`overworld` / `nether` / `end`。

use serde::{Deserialize, Serialize};

/// Minecraft 的维度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dimension {
    /// 主世界，对应游戏内 `minecraft:overworld`。
    #[default]
    Overworld,
    /// 下界，对应游戏内 `minecraft:the_nether`。
    Nether,
    /// 末地，对应游戏内 `minecraft:the_end`。
    End,
}

impl Dimension {
    /// 全部维度，供上层枚举。
    pub const ALL: [Self; 3] = [Self::Overworld, Self::Nether, Self::End];

    /// 契约层使用的序列化名（与 `serde` 输出一致）。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Overworld => "overworld",
            Self::Nether => "nether",
            Self::End => "end",
        }
    }

    /// 游戏内维度 ID，用作 `execute if dimension <id>` 的参数。
    pub fn vanilla_id(self) -> &'static str {
        match self {
            Self::Overworld => "minecraft:overworld",
            Self::Nether => "minecraft:the_nether",
            Self::End => "minecraft:the_end",
        }
    }

    /// 原版布局下相对世界根的子目录；主世界直接落在世界根上。
    ///
    /// 原版与 Paper 默认把下界/末地放在 `<世界>/DIM-1`、`<世界>/DIM1`。
    pub fn vanilla_subdir(self) -> Option<&'static str> {
        match self {
            Self::Overworld => None,
            Self::Nether => Some("DIM-1"),
            Self::End => Some("DIM1"),
        }
    }

    /// Bukkit 布局下世界目录名的后缀（`world_nether` / `world_the_end`）。
    ///
    /// 部分服务端（以及把 `level-name` 配成 `world_nether` 的场景）会把
    /// 下界/末地拆成独立世界目录，region 直接放在该目录下。
    pub fn bukkit_suffix(self) -> Option<&'static str> {
        match self {
            Self::Overworld => None,
            Self::Nether => Some("_nether"),
            Self::End => Some("_the_end"),
        }
    }

    /// 从游戏内维度 ID 反解，认不出时返回 `None`。
    ///
    /// 只认原版三界；数据包自定义维度不在支持范围。
    pub fn from_vanilla_id(value: &str) -> Option<Self> {
        match value.trim().trim_matches('"') {
            "minecraft:overworld" => Some(Self::Overworld),
            "minecraft:the_nether" => Some(Self::Nether),
            "minecraft:the_end" => Some(Self::End),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_as_short_snake_case() {
        assert_eq!(serde_json::to_string(&Dimension::Overworld).unwrap(), "\"overworld\"");
        assert_eq!(serde_json::to_string(&Dimension::Nether).unwrap(), "\"nether\"");
        assert_eq!(serde_json::to_string(&Dimension::End).unwrap(), "\"end\"");
    }

    #[test]
    fn round_trips_through_vanilla_id() {
        for dimension in Dimension::ALL {
            assert_eq!(Dimension::from_vanilla_id(dimension.vanilla_id()), Some(dimension));
        }
    }

    #[test]
    fn vanilla_id_accepts_quoted_form() {
        // `data get entity` 的返回值带双引号
        assert_eq!(Dimension::from_vanilla_id("\"minecraft:the_nether\""), Some(Dimension::Nether));
    }

    #[test]
    fn unknown_dimension_is_rejected() {
        assert_eq!(Dimension::from_vanilla_id("minecraft:overworld_custom"), None);
        assert_eq!(Dimension::from_vanilla_id(""), None);
    }

    #[test]
    fn default_is_overworld() {
        assert_eq!(Dimension::default(), Dimension::Overworld);
        // 契约字段依赖 default 保持向后兼容：老请求不带 dimension 时按主世界处理
        let request: crate::map::WorldMapRequest =
            serde_json::from_str(r#"{"instance_id":"abc"}"#).unwrap();
        assert_eq!(request.dimension, Dimension::Overworld);
    }
}
