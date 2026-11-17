//! 群系给植被和水的着色，取自 1.21.11 的群系定义，由 `tools/gen_biome_colors.py` 生成。
//!
//! 只列「和默认值不同」的群系：`grass` / `foliage` 为 `None` 表示该群系没有
//! 显式指定，调用方按温度降雨就近取档；`water` 几乎所有群系都是同一个蓝，
//! 少数（沼泽、冻洋）才不一样。

use fastanvil::biome::Biome;

/// 没显式指定水色时的默认值，也是绝大多数群系的取值。
pub(crate) const DEFAULT_WATER: [u8; 3] = [0x3F, 0x76, 0xE4];

/// 一个群系的植被与水颜色。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BiomeColours {
    /// 草和草状植物的颜色。`None` 表示要按气候档位取。
    pub(crate) grass: Option<[u8; 3]>,
    /// 树叶的颜色。`None` 同上。
    pub(crate) foliage: Option<[u8; 3]>,
    /// 该群系的水色。
    pub(crate) water: [u8; 3],
}

/// 默认：三个都交给调用方决定。
const FALLBACK: BiomeColours = BiomeColours {
    grass: None,
    foliage: None,
    water: DEFAULT_WATER,
};

/// 查一个群系的着色。不认识的群系返回 `FALLBACK`。
pub(crate) fn biome_colours(biome: Option<Biome>) -> BiomeColours {
    use Biome::*;

    match biome {
        Some(Badlands) => BiomeColours {
            grass: Some([144, 129, 77]),
            foliage: Some([158, 129, 77]),
            water: DEFAULT_WATER,
        },
        Some(ColdOcean) => BiomeColours {
            grass: None,
            foliage: None,
            water: [61, 87, 214],
        },
        Some(DeepColdOcean) => BiomeColours {
            grass: None,
            foliage: None,
            water: [61, 87, 214],
        },
        Some(DeepFrozenOcean) => BiomeColours {
            grass: None,
            foliage: None,
            water: [57, 56, 201],
        },
        Some(DeepLukewarmOcean) => BiomeColours {
            grass: None,
            foliage: None,
            water: [69, 173, 242],
        },
        Some(ErodedBadlands) => BiomeColours {
            grass: Some([144, 129, 77]),
            foliage: Some([158, 129, 77]),
            water: DEFAULT_WATER,
        },
        Some(FrozenOcean) => BiomeColours {
            grass: None,
            foliage: None,
            water: [57, 56, 201],
        },
        Some(FrozenRiver) => BiomeColours {
            grass: None,
            foliage: None,
            water: [57, 56, 201],
        },
        Some(LukewarmOcean) => BiomeColours {
            grass: None,
            foliage: None,
            water: [69, 173, 242],
        },
        Some(MangroveSwamp) => BiomeColours {
            grass: Some([141, 177, 39]),
            foliage: Some([141, 177, 39]),
            water: [58, 122, 106],
        },
        Some(Meadow) => BiomeColours {
            grass: None,
            foliage: None,
            water: [14, 78, 207],
        },
        Some(SnowyBeach) => BiomeColours {
            grass: None,
            foliage: None,
            water: [61, 87, 214],
        },
        Some(SnowyTaiga) => BiomeColours {
            grass: None,
            foliage: None,
            water: [61, 87, 214],
        },
        Some(Swamp) => BiomeColours {
            grass: Some([106, 112, 57]),
            foliage: Some([106, 112, 57]),
            water: [97, 123, 100],
        },
        Some(WarmOcean) => BiomeColours {
            grass: None,
            foliage: None,
            water: [67, 213, 238],
        },
        Some(WoodedBadlands) => BiomeColours {
            grass: Some([144, 129, 77]),
            foliage: Some([158, 129, 77]),
            water: DEFAULT_WATER,
        },
        _ => FALLBACK,
    }
}
