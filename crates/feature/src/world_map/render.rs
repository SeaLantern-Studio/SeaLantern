//! 把单个 region 渲染成俯视 PNG。
//!
//! 上色以**地表方块**为主，群系只用来给草方块和树叶做色调（和 Minecraft
//! 自带地图的做法一致）。纯群系配色对建筑图无效：练习服/竞技场世界通常整张
//! 图只有一种群系（实测 ThePit 世界 256 列全是 Ocean），按群系上色只会得到
//! 一片纯色。
//!
//! 一个 region 固定输出 512×512 像素（32 chunk × 16 方块）。
//!
//! ## 立体感怎么来的
//!
//! 俯视图没有透视，全靠明暗和颜色层次。这里做两件事：
//!
//! - **连续山体阴影**：光源固定在西北，按「本列高度 − 北邻/西邻高度」连续缩放
//!   亮度。fastanvil 自带的 `TopShadeRenderer` 只有三档（比北邻高 / 相等 / 低），
//!   一格台阶和十格悬崖会被压成同一种灰，所以这里自己采样、自己着色。
//! - **水体按深度调色**：浅水偏青、透出海底，深水接近纯深蓝。原来整片海是一个
//!   死蓝色，看不出浅滩和海沟的区别。
//!
//! 跨 chunk 的阴影靠两条边缘缓存（北边一块的最后一行、西边一块的最后一列），
//! 否则每个 chunk 边界会出现一道直边。
//!
//! 只渲染 1.18+（`sections`）格式的 chunk。更早的 chunk 会被跳过并留空：
//! fastanvil 0.32 的 pre-1.13 路径在 `init_default_block` 里对半砖数据值
//! 用了 `unreachable!()`（`data_value & 0b1000` 只可能得 0 或 8，却去匹配 0/1），
//! 碰到旧区块会直接 panic，所以不能交给它处理。

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use fastanvil::biome::Biome;
use fastanvil::{Chunk, HeightMode, JavaChunk, Region, Rgba};

use super::biome_colors::biome_colours;
use super::palette::TexturePalette;
use super::{WorldMapError, is_finished};

/// 一个 region 的边长（方块数）。
pub const REGION_PIXELS: u32 = 512;

/// 每个 region 每个方向的 chunk 数。
const CHUNKS_PER_SIDE: usize = 32;

/// 水与海草等水体方块的统一色。
const WATER: Rgba = [58, 102, 175, 255];

/// 统计水深时最多往下看多少格。
///
/// 再深的海（深海沟能到 40 格以上）多算也没意义：水色到十几格就已经接近纯深蓝，
/// 徒增每列几十次方块读取。
const MAX_WATER_DEPTH: usize = 16;

/// 无法识别时的兜底色。
const UNKNOWN: Rgba = [120, 120, 120, 255];

/// 渲染结果摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderInfo {
    /// 输出图片边长，恒为 `REGION_PIXELS`。
    pub pixels: u32,
    /// 输出文件字节数。
    pub bytes: u64,
    /// 实际渲染的 chunk 数（1.18+ 已生成）。
    pub rendered_chunks: usize,
    /// 因不是 1.18+ 格式而跳过的 chunk 数（含 1.13~1.17 与 1.12 及更早）。
    pub unsupported_chunks: usize,
    /// 因区块数据无法解析而跳过的 chunk 数。
    ///
    /// 和 [`Self::unsupported_chunks`] 分开统计：那个是版本问题（数据本身完好，
    /// 只是格式太旧），这个是数据损坏（多数情况下是服务端正在写这块 region）。
    /// 混在一起会让「图上有洞」被误判成版本限制，从而掩盖真正的问题。
    pub corrupt_chunks: usize,
    /// 因未完全生成而跳过的 chunk 数。
    pub unfinished_chunks: usize,
}

/// 一列地表的采样结果。
struct SurfaceColumn {
    /// 这一列在俯视图上的颜色。
    colour: Rgba,
    /// 地表高度（`surface_height`，最高的阻挡移动方块之上那一格）。
    ///
    /// 山体阴影靠它算坡向，所以必须整列都要，不能只留颜色。
    height: isize,
}

/// 采一列：取最高的阻挡移动方块，遇到水就往下一路数到海底。
///
/// 用 `surface_height` 而不是从世界顶端往下扫，是因为 `motion_blocking` 高度图
/// 本身就把花草、火把这类不挡路的方块算作空气了 —— 否则每格草都会盖在地形上，
/// 整张图变成一片绿色噪点。
fn sample_column<C: Chunk + ?Sized>(
    chunk: &C,
    x: usize,
    z: usize,
    y_min: isize,
    palette: &TexturePalette,
) -> SurfaceColumn {
    let height = chunk.surface_height(x, z, HeightMode::Trust);
    let mut y = (height - 1).max(y_min);

    let Some(top) = chunk.block(x, y, z) else {
        return SurfaceColumn { colour: [0, 0, 0, 0], height };
    };

    // 地表这一格的群系决定整列的着色：水面和海底的群系偶尔会不一样（河海交界），
    // 取水面上看到的那一层
    let biome = chunk.biome(x, y, z);

    if !is_water_like(top.name()) {
        return SurfaceColumn {
            colour: block_colour(top.name(), biome, palette),
            height,
        };
    }

    // 水面：往下数水深，顺便拿到海底的颜色
    let mut depth = 1usize;
    y -= 1;
    while y >= y_min && depth < MAX_WATER_DEPTH {
        match chunk.block(x, y, z) {
            Some(block) if is_water_like(block.name()) => {
                depth += 1;
                y -= 1;
            }
            Some(block) => {
                let floor = block_colour(block.name(), biome, palette);
                return SurfaceColumn {
                    colour: water_colour(floor, depth, biome),
                    height,
                };
            }
            None => break,
        }
    }

    // 到底都没见到海底（深海，或高度图本身不可信）：按纯水色收场
    SurfaceColumn {
        colour: water_colour(WATER, depth, biome),
        height,
    }
}

/// 按水深给水面调色：浅水偏青、能透出海底，深水接近纯深蓝。
///
/// 基准色是该群系自己的水色（沼泽偏绿、暖水偏青、冻洋偏灰蓝），不是写死的一个蓝 ——
/// 原来整片海是同一个颜色，看不出浅滩和海沟，也看不出沼泽和海洋的区别。
fn water_colour(floor: Rgba, depth: usize, biome: Option<Biome>) -> Rgba {
    let base = biome_colours(biome).water;

    // 浅水在群系水色上加亮，深水统一压暗。两个方向都用同一个系数，这样沼泽那种
    //「偏绿的水」压暗后还是绿的，不会因为蓝通道被特殊照顾而反超
    let shallow = base.map(|channel| (channel as f32 * 1.08 + 14.0).min(255.0));
    let deep = base.map(|channel| channel as f32 * 0.45);

    let span = (MAX_WATER_DEPTH - 1).max(1) as f32;
    let deepness = (depth.saturating_sub(1) as f32 / span).min(1.0);

    let mut out = [0u8; 4];
    for channel in 0..3 {
        let water = shallow[channel] + (deep[channel] - shallow[channel]) * deepness;
        // 水越浅，海底的颜色透出来越多。比例不能太高，否则浅滩会被海底的
        // 沙色、石色冲淡成灰白，看着像干涸的河床
        let see_floor = (1.0 - deepness) * 0.30;
        let value = water * (1.0 - see_floor) + floor[channel] as f32 * see_floor;
        out[channel] = value.clamp(0.0, 255.0) as u8;
    }
    out[3] = 255;
    out
}

/// 按西北方向的光源给像素加减亮度。
///
/// 原来的实现只有三档（比北邻高 / 相等 / 低 → 255 / 220 / 180），一格高的台阶
/// 和十格高的悬崖会被压成同一种灰，看起来就是一块块色斑。改成按高度差连续缩放，
/// 坡面越陡越亮或越暗，山脊和沟壑才看得见。
fn apply_shade(colour: Rgba, height: isize, north: isize, west: isize) -> Rgba {
    if colour[3] == 0 {
        return colour;
    }
    let slope = ((height - north) + (height - west)) as f32;
    let factor = (1.0 + slope * 0.055).clamp(0.68, 1.30);
    [
        (colour[0] as f32 * factor).clamp(0.0, 255.0) as u8,
        (colour[1] as f32 * factor).clamp(0.0, 255.0) as u8,
        (colour[2] as f32 * factor).clamp(0.0, 255.0) as u8,
        colour[3],
    ]
}

/// 这些方块在画面上属于水体。
fn is_water_like(name: &str) -> bool {
    matches!(
        name,
        "minecraft:water"
            | "minecraft:bubble_column"
            | "minecraft:kelp"
            | "minecraft:kelp_plant"
            | "minecraft:sea_grass"
            | "minecraft:tall_seagrass"
    )
}

/// 16 种染料色，用于羊毛、陶瓦、混凝土等成组方块。
fn dye_colour(name: &str) -> Option<Rgba> {
    Some(match name {
        "white" => [233, 236, 236, 255],
        "light_gray" => [155, 155, 148, 255],
        "gray" => [62, 68, 71, 255],
        "black" => [29, 29, 33, 255],
        "brown" => [96, 60, 32, 255],
        "red" => [142, 33, 33, 255],
        "orange" => [216, 127, 51, 255],
        "yellow" => [249, 198, 40, 255],
        "lime" => [112, 185, 26, 255],
        "green" => [85, 110, 28, 255],
        "cyan" => [21, 137, 145, 255],
        "light_blue" => [58, 175, 217, 255],
        "blue" => [53, 57, 157, 255],
        "purple" => [121, 42, 172, 255],
        "magenta" => [189, 68, 179, 255],
        "pink" => [238, 141, 172, 255],
        _ => return None,
    })
}

/// 木材色，用于木板、原木、台阶等成套方块。
fn wood_colour(name: &str) -> Option<Rgba> {
    Some(match name {
        "oak" => [162, 130, 78, 255],
        "spruce" => [104, 78, 47, 255],
        "birch" => [196, 180, 123, 255],
        "jungle" => [150, 110, 80, 255],
        "acacia" => [172, 92, 50, 255],
        "dark_oak" => [60, 40, 20, 255],
        "mangrove" => [117, 54, 48, 255],
        "cherry" => [224, 168, 160, 255],
        "bamboo" => [193, 173, 80, 255],
        "crimson" => [101, 48, 70, 255],
        "warped" => [43, 104, 99, 255],
        "pale_oak" => [225, 219, 201, 255],
        _ => return None,
    })
}

/// 按比例压暗一个通道，`add` 用于把压暗后的颜色抬回可见范围。
fn scale(v: u8, numerator: u16, denominator: u16, add: u16) -> u8 {
    (((v as u16) * numerator) / denominator + add).min(255) as u8
}

/// 群系给植被的色调：只影响草和树叶，冷/干地区偏白偏黄。
fn vegetation_tint(biome: Option<Biome>) -> Rgba {
    use Biome::*;

    match biome {
        Some(
            SnowyPlains | SnowyTundra | FrozenOcean | FrozenRiver | SnowyBeach | SnowyTaiga
            | SnowyTaigaHills | SnowyTaigaMountains | SnowyMountains | FrozenPeaks | JaggedPeaks
            | SnowySlopes | Grove | IceSpikes,
        ) => [168, 190, 172, 255],
        Some(
            Desert
            | DesertHills
            | DesertLakes
            | Badlands
            | BadlandsPlateau
            | ErodedBadlands
            | WoodedBadlands
            | WoodedBadlandsPlateau
            | ModifiedBadlandsPlateau
            | ModifiedWoodedBadlandsPlateau
            | Savanna
            | SavannaPlateau
            | ShatteredSavanna
            | ShatteredSavannaPlateau
            | WindsweptSavanna,
        ) => [176, 168, 96, 255],
        Some(Swamp | SwampHills | MangroveSwamp) => [108, 122, 76, 255],
        _ => [120, 172, 88, 255],
    }
}

/// 方块名到颜色的映射。
///
/// 先查贴图平均色（见 [`texture_colour`]），对不上再退回手写调色板。手写表留着是
/// 为了两类情况：模组方块不在原版贴图表里；以及 `*_stairs`、`*_wall` 这类衍生方块
/// 的贴图名推不出来时，至少还有个像样的颜色，不会掉进灰色兜底。
fn block_colour(name: &str, biome: Option<Biome>, palette: &TexturePalette) -> Rgba {
    if is_water_like(name) {
        return WATER;
    }

    let short = name.strip_prefix("minecraft:").unwrap_or(name);

    if let Some(colour) = texture_colour(short, biome, palette) {
        return colour;
    }

    legacy_block_colour(short, biome)
}

/// 按贴图平均色 + 群系染色求一个方块的颜色。
///
/// 这是和 Minecraft 自带地图拉开差距的地方：自带地图每种方块只有 4 档明暗，
/// 这里直接用 1.21 官方贴图的平均色，沙子是米黄、恶地是橙红、雪是近乎纯白，
/// 一千多个方块不用一条条手写。
fn texture_colour(block: &str, biome: Option<Biome>, palette: &TexturePalette) -> Option<Rgba> {
    let tint = biome_colours(biome);

    // 草方块和草状植物：贴图是灰度的，颜色全由群系决定
    if is_grass_like(block) {
        let base = palette.lookup("grass_block_top")?;
        // 群系定义里有显式 grass_color 就用它（恶地、沼泽这类），否则按气候分档
        let colour = tint.grass.unwrap_or_else(|| climate_tint(biome).0);
        return Some(apply_tint(colour, base));
    }

    // 树叶：同上，但用 foliage 色
    if is_foliage_like(block) {
        let base = palette
            .lookup(block)
            .or_else(|| palette.lookup("oak_leaves"))?;
        // 樱花、苍白橡树这类树叶贴图本身就有颜色，再染一遍会变成脏绿
        if !is_greyscale(base) {
            return Some(base);
        }
        let colour = tint.foliage.unwrap_or_else(|| climate_tint(biome).1);
        return Some(apply_tint(colour, base));
    }

    let texture = texture_name(block)?;
    palette.lookup(texture)
}

/// 把方块名翻成贴图名。
///
/// 顺序是有讲究的：台阶、楼梯要先剥后缀（它们的贴图名和方块名对不上），
/// 原木、砂岩这类要看顶面，最后才试同名贴图。
fn texture_name(block: &str) -> Option<&str> {
    // 台阶、楼梯、墙、栅栏这类是基础材质的变形，贴图要往基础材质上找
    for suffix in [
        "_stairs",
        "_slab",
        "_wall",
        "_fence_gate",
        "_fence",
        "_pressure_plate",
        "_button",
        "_trapdoor",
        "_door",
    ] {
        if let Some(base) = block.strip_suffix(suffix) {
            if let Some(texture) = texture_name_of_base(base) {
                return Some(texture);
            }
        }
    }

    texture_name_of_base(block)
}

/// 基础材质本身（`stone`、`oak_planks`）对应的贴图名。
fn texture_name_of_base(base: &str) -> Option<&str> {
    // 名字和贴图对不上的几个，直接列表
    let direct = match base {
        "grass_block" => "grass_block_top",
        "dirt_path" => "dirt_path_top",
        "mycelium" => "mycelium_top",
        "podzol" => "podzol_top",
        "crimson_nylium" => "crimson_nylium",
        "warped_nylium" => "warped_nylium",
        "farmland" => "farmland",
        "snow_block" => "snow",
        "short_grass" => "short_grass",
        "tall_grass" => "tall_grass",
        // 木材名单独出现时按木板算（`oak_stairs` 剥完是 `oak`）
        "oak" => "oak_planks",
        "spruce" => "spruce_planks",
        "birch" => "birch_planks",
        "jungle" => "jungle_planks",
        "acacia" => "acacia_planks",
        "dark_oak" => "dark_oak_planks",
        "mangrove" => "mangrove_planks",
        "cherry" => "cherry_planks",
        "bamboo" => "bamboo_planks",
        "crimson" => "crimson_planks",
        "warped" => "warped_planks",
        "pale_oak" => "pale_oak_planks",
        _ => "",
    };
    if !direct.is_empty() {
        return Some(direct);
    }

    // 具体贴图存在与否由调用方查表决定，这里只给出候选顺序
    Some(base)
}

/// 这张贴图是不是灰度图。
///
/// 灰度贴图（草、树叶、水）的颜色在游戏里由群系决定，贴图只提供明暗纹理；
/// 直接拿平均色当结果会得到一片灰，必须先染色。彩色贴图（樱花树叶、
/// 各色陶瓦）本来就有自己的颜色，不能染。
fn is_greyscale(texture: Rgba) -> bool {
    let max = texture[0].max(texture[1]).max(texture[2]);
    let min = texture[0].min(texture[1]).min(texture[2]);
    max - min < 24
}

/// 用群系色给灰度贴图上色，保留贴图的明暗纹理。
///
/// 系数按「平均亮度 150」归一化 —— 草方块顶面平均 147、橡树树叶 151，都在这个
/// 数附近，除下来系数接近 1，结果基本就是群系色本身，只带一点贴图的明暗起伏。
fn apply_tint(tint: [u8; 3], base: Rgba) -> Rgba {
    let luma = (base[0] as u32 * 30 + base[1] as u32 * 59 + base[2] as u32 * 11) / 100;
    let factor = (luma as f32 / 150.0).clamp(0.82, 1.18);
    [
        (tint[0] as f32 * factor).clamp(0.0, 255.0) as u8,
        (tint[1] as f32 * factor).clamp(0.0, 255.0) as u8,
        (tint[2] as f32 * factor).clamp(0.0, 255.0) as u8,
        255,
    ]
}

/// 草方块、草、蕨这类吃 grass 染色的方块。
fn is_grass_like(block: &str) -> bool {
    matches!(
        block,
        "grass_block" | "short_grass" | "grass" | "tall_grass" | "fern" | "large_fern"
    )
}

/// 吃 foliage 染色的方块。
fn is_foliage_like(block: &str) -> bool {
    block.ends_with("_leaves") || block == "vine"
}

/// 气候档位到植被色的对照。
///
/// 数据取自原版各群系的实际草地色和树叶色（wiki 的 biome color 表），第一列温度、
/// 第二列降雨，用来找最近的档位。
///
/// 为什么不用官方的 `grass.png` 色图：那张图的坐标语义在 1.13 之后就没有可靠的
/// 公开定义了。按「(1 − 温度, 1 − 降雨)」去查，平原/沙漠/丛林都能对上，但森林
/// （温度 0.7、降雨 0.8）会落进右上的空白区，压回对角线后取到的是边界浅绿，
/// 于是整片森林渲染成 `#91B386` 这种饱和度只有 25% 的灰绿 —— 一眼就不像游戏里
/// 的草地。宁愿用分档的固定色，也不要一个对不上号的颜色计算。
const CLIMATE_TINTS: &[(f32, f32, [u8; 3], [u8; 3])] = &[
    // 温度, 降雨, 草色, 叶色
    (0.00, 0.50, [0x80, 0xB4, 0x97], [0x60, 0xA1, 0x7B]), // 雪原、冰刺
    (0.25, 0.80, [0x86, 0xB7, 0x83], [0x68, 0xA4, 0x64]), // 针叶林
    (0.30, 0.30, [0x8A, 0xB6, 0x89], [0x6D, 0xA3, 0x6B]), // 山地、风袭丘陵
    (0.70, 0.30, [0x91, 0xBD, 0x59], [0x77, 0xAB, 0x2F]), // 平原、草甸
    (0.70, 0.80, [0x79, 0xC0, 0x5A], [0x59, 0xAE, 0x30]), // 森林、桦木林
    (0.95, 0.90, [0x59, 0xC9, 0x3C], [0x30, 0xBB, 0x0B]), // 丛林
    (1.50, 0.00, [0xBF, 0xB7, 0x55], [0xAE, 0xA4, 0x2A]), // 沙漠、恶地
    (0.80, 0.90, [0x6A, 0x70, 0x39], [0x4F, 0x5A, 0x27]), // 沼泽
];

/// 按温度和降雨找最接近的气候档位，返回草色和叶色。
///
/// 温度权重更高：同一个温度带里降雨只让颜色偏深一点，而跨温度带是整个色系
/// 变化（雪原白绿 → 沙漠黄褐）。
fn climate_tint(biome: Option<Biome>) -> ([u8; 3], [u8; 3]) {
    let Some(climate) = biome.map(|biome| biome.climate()) else {
        // 群系缺失时按温带草原处理，这是最不会出错的中间值
        return ([0x91, 0xBD, 0x59], [0x77, 0xAB, 0x2F]);
    };
    let (temperature, rainfall) = (climate.temperature as f32, climate.rainfall as f32);

    let mut best = CLIMATE_TINTS[0];
    let mut best_distance = f32::MAX;
    for entry in CLIMATE_TINTS {
        let (t, r) = (entry.0, entry.1);
        let distance = (temperature - t).powi(2) * 2.0 + (rainfall - r).powi(2);
        if distance < best_distance {
            best_distance = distance;
            best = *entry;
        }
    }
    (best.2, best.3)
}

/// 手写调色板，贴图查不到时的兜底。
fn legacy_block_colour(short: &str, biome: Option<Biome>) -> Rgba {
    // 草方块和树叶：底色 + 群系色调
    if short == "grass_block" || short == "grass" {
        return vegetation_tint(biome);
    }
    if short.ends_with("_leaves") || short == "azalea_leaves" || short == "vine" {
        let tint = vegetation_tint(biome);
        return [scale(tint[0], 3, 4, 0), scale(tint[1], 3, 4, 0), scale(tint[2], 3, 4, 0), 255];
    }

    // 成组方块：羊毛、陶瓦、混凝土、染色玻璃、地毯
    for (suffix, group) in [
        ("_terracotta", "terracotta"),
        ("_wool", "wool"),
        ("_concrete_powder", "concrete"),
        ("_concrete", "concrete"),
        ("_stained_glass", "glass"),
        ("_carpet", "wool"),
        ("_glazed_terracotta", "glazed"),
    ] {
        if let Some(colour_name) = short.strip_suffix(suffix)
            && let Some(dye) = dye_colour(colour_name)
        {
            return match group {
                // 陶瓦压暗一点，避免和羊毛糊在一起
                "terracotta" => {
                    [scale(dye[0], 3, 4, 40), scale(dye[1], 3, 4, 30), scale(dye[2], 3, 4, 0), 255]
                }
                "glazed" => {
                    [scale(dye[0], 4, 5, 20), scale(dye[1], 4, 5, 20), scale(dye[2], 4, 5, 20), 255]
                }
                _ => dye,
            };
        }
    }

    // 木材类：木板、原木、台阶、门、活板门、栅栏
    for suffix in [
        "_planks",
        "_log",
        "_wood",
        "_stairs",
        "_slab",
        "_trapdoor",
        "_door",
        "_fence",
        "_fence_gate",
        "_sign",
        "_button",
        "_pressure_plate",
    ] {
        if let Some(prefix) = short.strip_suffix(suffix) {
            for wood in [
                "dark_oak", "pale_oak", "spruce", "birch", "jungle", "acacia", "mangrove",
                "cherry", "bamboo", "crimson", "warped", "oak",
            ] {
                if (prefix == wood || prefix == format!("stripped_{wood}"))
                    && let Some(base) = wood_colour(wood)
                {
                    return base;
                }
            }
        }
    }

    match short {
        "stone" | "smooth_stone" => [125, 125, 125, 255],
        "cobblestone" | "mossy_cobblestone" | "cobbled_deepslate" => [110, 110, 110, 255],
        "andesite" | "polished_andesite" => [132, 134, 133, 255],
        "granite" | "polished_granite" => [149, 103, 85, 255],
        "diorite" | "polished_diorite" => [188, 188, 190, 255],
        "deepslate" | "tuff" => [80, 80, 85, 255],
        "calcite" => [223, 224, 220, 255],
        "gravel" => [136, 126, 126, 255],
        "dirt" | "coarse_dirt" | "rooted_dirt" => [134, 96, 67, 255],
        "podzol" => [92, 64, 35, 255],
        "mud" | "muddy_mangrove_roots" => [60, 57, 60, 255],
        "sand" | "suspicious_sand" => [219, 207, 163, 255],
        "red_sand" => [169, 88, 33, 255],
        "sandstone" | "smooth_sandstone" | "cut_sandstone" => [216, 203, 155, 255],
        "red_sandstone" => [181, 97, 31, 255],
        "clay" => [160, 166, 179, 255],
        "snow_block" | "snow" | "powder_snow" => [249, 254, 254, 255],
        "ice" => [145, 183, 235, 255],
        "packed_ice" => [141, 180, 250, 255],
        "blue_ice" => [116, 167, 253, 255],
        "lava" | "magma_block" => [207, 92, 20, 255],
        "obsidian" | "crying_obsidian" => [21, 18, 30, 255],
        "bedrock" => [85, 85, 85, 255],
        "netherrack" => [97, 38, 38, 255],
        "soul_sand" | "soul_soil" => [81, 62, 50, 255],
        "end_stone" => [219, 222, 158, 255],
        "quartz_block" | "smooth_quartz" => [235, 229, 222, 255],
        "prismarine" | "dark_prismarine" => [99, 156, 151, 255],
        "sea_lantern" => [190, 220, 215, 255],
        "glowstone" => [184, 145, 90, 255],
        "bookshelf" => [130, 105, 70, 255],
        _ => {
            if short.contains("glass") {
                [200, 215, 220, 255]
            } else if short.contains("copper") {
                [150, 105, 80, 255]
            } else if short.contains("iron") {
                [190, 190, 190, 255]
            } else if short.contains("gold") {
                [230, 190, 80, 255]
            } else {
                UNKNOWN
            }
        }
    }
}

/// 打开 `region/r.{x}.{z}.mca`；文件不存在时返回 [`WorldMapError::MissingRegion`]。
fn open_region(region_dir: &Path, x: isize, z: isize) -> Result<Region<File>, WorldMapError> {
    let path = region_dir.join(format!("r.{x}.{z}.mca"));
    let file = File::open(&path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            WorldMapError::MissingRegion { x, z }
        } else {
            WorldMapError::Open { path: path.display().to_string(), source }
        }
    })?;

    Region::from_stream(file).map_err(|e| WorldMapError::Region(e.to_string()))
}

/// 把 `region/r.{x}.{z}.mca` 渲染成 512×512 PNG。
///
/// `region_dir` 是包含 `.mca` 文件的目录。不是 1.18+ 格式的 chunk 会被跳过，
/// 对应像素保持透明——宁可留空，也不画假数据。
///
/// # 输出路径的原子性由调用方负责
///
/// 本函数直接 `File::create(out_path)`，中途失败会留下半截文件，并发调用同一
/// 路径也会互相覆盖。这个约定是有意的：`crates/feature` 只管渲染，不碰缓存
/// 布局。应用层传进来的是 `<缓存名>.<pid>.png.part`，渲染成功后才 `rename` 到
/// 正式缓存路径，所以这里的覆盖影响不到已有图片。
///
/// 如果要在别的宿主里直接调用，请自己先写临时文件再替换目标。
pub fn render_region_png(
    region_dir: &Path,
    x: isize,
    z: isize,
    out_path: &Path,
) -> Result<RenderInfo, WorldMapError> {
    render_region_png_with(region_dir, x, z, out_path, &TexturePalette::builtin())
}

/// 同 [`render_region_png`]，但可以指定贴图颜色表。
///
/// 传 [`TexturePalette::from_resource_pack`] 出来的表，用户装了什么材质包，
/// 地图就跟着变。不传就用内置表。
pub fn render_region_png_with(
    region_dir: &Path,
    x: isize,
    z: isize,
    out_path: &Path,
    palette: &TexturePalette,
) -> Result<RenderInfo, WorldMapError> {
    let mut region = open_region(region_dir, x, z)?;

    let side = CHUNKS_PER_SIDE * 16;
    let mut pixels = vec![[0u8, 0u8, 0u8, 0u8]; side * side];
    // 每个 chunk 列记下「北边那块」最后一行的地表高度，跨 chunk 的阴影要靠它
    let mut north_heights: [Option<[isize; 16]>; CHUNKS_PER_SIDE] = [None; CHUNKS_PER_SIDE];

    let mut rendered_chunks = 0usize;
    let mut unsupported_chunks = 0usize;
    let mut corrupt_chunks = 0usize;
    let mut unfinished_chunks = 0usize;
    let mut first_corrupt: Option<String> = None;

    for cz in 0..CHUNKS_PER_SIDE {
        // 同一行里「西边那块」最右一列的地表高度
        let mut west_heights: Option<[isize; 16]> = None;
        for (cx, north_slot) in north_heights.iter_mut().enumerate() {
            let data = match region.read_chunk(cx, cz) {
                Ok(Some(data)) => data,
                // 位置表里没有这一项：区块根本不存在，不算损坏
                Ok(None) => {
                    *north_slot = None;
                    west_heights = None;
                    continue;
                }
                // 读都读不出来（扇区损坏、或服务端正在改这张位置表）：
                // 和 NBT 解析失败一样按损坏跳过，不能因为一个坏块让整块 region
                // 渲染失败 —— 那等于放弃其余正常区块
                Err(error) => {
                    corrupt_chunks += 1;
                    if first_corrupt.is_none() {
                        first_corrupt = Some(format!("({cx}, {cz}) 读取失败: {error}"));
                    }
                    *north_slot = None;
                    west_heights = None;
                    continue;
                }
            };

            let chunk = match JavaChunk::from_bytes(&data) {
                Ok(chunk) => chunk,
                Err(error) => {
                    // 解析失败是数据本身的问题（region 正被写入、或文件损坏），
                    // 与「版本太旧」是两回事，单独计数并留痕
                    corrupt_chunks += 1;
                    if first_corrupt.is_none() {
                        first_corrupt = Some(format!("({cx}, {cz}): {error}"));
                    }
                    *north_slot = None;
                    west_heights = None;
                    continue;
                }
            };

            // 只有 1.18+ 格式能安全交给渲染器，旧格式会 panic（见模块注释）
            if !matches!(chunk, JavaChunk::Post18(_)) {
                unsupported_chunks += 1;
                *north_slot = None;
                west_heights = None;
                continue;
            }

            if !is_finished(&chunk.status()) {
                unfinished_chunks += 1;
                *north_slot = None;
                west_heights = None;
                continue;
            }

            let y_min = chunk.y_range().start;
            let north_edge = *north_slot;
            let row_offset = cz * 16;
            let col_offset = cx * 16;
            let mut heights = [[y_min; 16]; 16];

            for row in 0..16 {
                for column in 0..16 {
                    let sampled = sample_column(&chunk, column, row, y_min, palette);
                    // 北 / 西两个邻居的高度：chunk 边界上取自上一块的边缘
                    let north = if row > 0 {
                        heights[row - 1][column]
                    } else {
                        north_edge.map_or(sampled.height, |edge| edge[column])
                    };
                    let west = if column > 0 {
                        heights[row][column - 1]
                    } else {
                        west_heights.map_or(sampled.height, |edge| edge[row])
                    };
                    heights[row][column] = sampled.height;
                    pixels[(row_offset + row) * side + col_offset + column] =
                        apply_shade(sampled.colour, sampled.height, north, west);
                }
            }

            *north_slot = Some(heights[15]);
            west_heights = Some(std::array::from_fn(|row| heights[row][15]));
            rendered_chunks += 1;
        }
    }

    if corrupt_chunks > 0 {
        tracing::warn!(
            target: "sealantern.feature.world_map",
            x,
            z,
            corrupt_chunks,
            first_error = first_corrupt.as_deref().unwrap_or_default(),
            "region 里有区块无法解析，已按损坏跳过"
        );
    }

    let mut bytes = Vec::with_capacity(pixels.len() * 4);
    for pixel in &pixels {
        bytes.extend_from_slice(pixel);
    }

    let file = File::create(out_path).map_err(|source| WorldMapError::Output {
        path: out_path.display().to_string(),
        source,
    })?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), REGION_PIXELS, REGION_PIXELS);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| WorldMapError::Render {
        x,
        z,
        reason: format!("写 PNG 头失败: {e}"),
    })?;
    writer
        .write_image_data(&bytes)
        .map_err(|e| WorldMapError::Render {
            x,
            z,
            reason: format!("写 PNG 数据失败: {e}"),
        })?;
    writer.finish().map_err(|e| WorldMapError::Render {
        x,
        z,
        reason: format!("收尾 PNG 失败: {e}"),
    })?;

    // 读不到输出文件的大小说明文件没落成（被删、权限变了、磁盘出问题），
    // 这时候报成功等于给调用方一个 `bytes == 0` 的假摘要
    let file_bytes = std::fs::metadata(out_path)
        .map_err(|source| WorldMapError::Output {
            path: out_path.display().to_string(),
            source,
        })?
        .len();

    Ok(RenderInfo {
        pixels: REGION_PIXELS,
        bytes: file_bytes,
        rendered_chunks,
        unsupported_chunks,
        corrupt_chunks,
        unfinished_chunks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 需要真实 region 目录时才跑。
    ///
    /// ```text
    /// SEALANTERN_TEST_REGION_DIR=/path/to/world/region \
    ///   cargo test -p sealantern-feature -- --ignored
    /// ```
    ///
    /// 标 `#[ignore]` 而不是「环境变量没设就 return」，理由同 `probe_real_region_file`：
    /// 静默 return 会把没执行的用例报成通过。
    #[test]
    #[ignore = "需要 SEALANTERN_TEST_REGION_DIR 指向真实的 region 目录"]
    fn render_real_region_to_png() {
        let dir = std::env::var("SEALANTERN_TEST_REGION_DIR")
            .expect("跑这个测试需要设置 SEALANTERN_TEST_REGION_DIR 指向真实的 region 目录");

        let out = std::env::temp_dir().join("sealantern_world_map_test.png");
        let info = render_region_png(Path::new(&dir), 0, 0, &out).expect("渲染 region 失败");

        println!("输出: {} ({:?})", out.display(), info);

        assert_eq!(info.pixels, REGION_PIXELS);
        assert!(info.rendered_chunks > 0, "应该有 1.18+ chunk 被渲染");
        assert!(info.bytes > 10_000, "PNG 体积过小，可能是全空图：{} 字节", info.bytes);
    }

    /// 造一个 region 文件：8KB 位置表 + 一个「未压缩」标记但内容是垃圾的 chunk。
    ///
    /// 压缩类型写 3（未压缩），fastanvil 就不解压、直接把原始字节交出来，失败点
    /// 因此落在 NBT 解析上 —— 正是「数据损坏」，而不是「格式太旧」。
    fn write_region_with_one_corrupt_chunk(path: &Path) {
        let mut file = vec![0u8; 8192];
        // 位置表第 0 项：起始扇区 2（跳过 8KB 头部）、占 1 个扇区
        file[2] = 2;
        file[3] = 1;

        let garbage = b"not a valid chunk payload";
        // 4 字节大端长度（含压缩类型那 1 字节）+ 压缩类型 3（未压缩）
        file.extend_from_slice(&((garbage.len() as u32 + 1).to_be_bytes()));
        file.push(3);
        file.extend_from_slice(garbage);

        let pad = (4096 - (file.len() - 8192) % 4096) % 4096;
        file.resize(file.len() + pad, 0);

        std::fs::write(path, &file).expect("写 region 文件");
    }

    /// 损坏的区块要单独计数，不能混进「不是 1.18+ 格式」那一栏。
    #[test]
    fn corrupt_chunk_is_not_counted_as_unsupported_format() {
        let dir = tempfile::tempdir().expect("临时目录");
        write_region_with_one_corrupt_chunk(&dir.path().join("r.0.0.mca"));

        let out = dir.path().join("r.0.0.png");
        let info =
            render_region_png(dir.path(), 0, 0, &out).expect("单个坏块不该让整块 region 渲染失败");

        assert_eq!(info.corrupt_chunks, 1, "应当记一次损坏");
        assert_eq!(info.unsupported_chunks, 0, "损坏不能被算成「格式太旧」");
        assert_eq!(info.rendered_chunks, 0);
        assert_eq!(info.pixels, REGION_PIXELS);
    }

    /// 位置表指向文件末尾之外：`from_stream` 能过，但 `read_chunk` 会失败。
    fn write_region_with_unreadable_chunk(path: &Path) {
        let mut file = vec![0u8; 8192];
        // 第 0 项指向第 3 个扇区，而文件只有 2 个扇区
        file[2] = 3;
        file[3] = 1;
        std::fs::write(path, &file).expect("写 region 文件");
    }

    /// 单个区块读失败不能中止整块 region 的渲染，要按损坏计数后继续。
    #[test]
    fn unreadable_chunk_is_counted_as_corrupt() {
        let dir = tempfile::tempdir().expect("临时目录");
        write_region_with_unreadable_chunk(&dir.path().join("r.0.0.mca"));

        let out = dir.path().join("r.0.0.png");
        let info = render_region_png(dir.path(), 0, 0, &out).expect("读失败不应让整块 region 报错");

        assert_eq!(info.corrupt_chunks, 1, "读失败要记成损坏");
        assert_eq!(info.unsupported_chunks, 0);
        assert_eq!(info.rendered_chunks, 0);
    }

    /// 不同方块要产出不同颜色，避免像纯群系配色那样整张图一个色。
    #[test]
    fn block_colours_are_distinct() {
        let green = block_colour("minecraft:green_terracotta", None, &palette());
        let lime = block_colour("minecraft:lime_terracotta", None, &palette());
        let clay = block_colour("minecraft:clay", None, &palette());
        let stone = block_colour("minecraft:stone", None, &palette());

        assert_ne!(green, lime);
        assert_ne!(clay, stone);
        assert_ne!(green, clay);
        // 水方块固定走水体色
        assert_eq!(block_colour("minecraft:water", None, &palette()), WATER);
    }

    /// 平地不该被着色改动，否则整张图会整体偏亮或偏暗。
    #[test]
    fn flat_ground_keeps_its_colour() {
        let base = [100u8, 120u8, 90u8, 255];
        assert_eq!(apply_shade(base, 64, 64, 64), base);
    }

    /// 测试统一用内置表，不依赖外部资源包。
    fn palette() -> TexturePalette {
        TexturePalette::builtin()
    }

    /// 朝光的坡面变亮、背光的变暗，且方向由西北两侧共同决定。
    #[test]
    fn shading_follows_the_slope() {
        let base = [100u8, 100u8, 100u8, 255];

        let uphill = apply_shade(base, 70, 68, 68);
        let downhill = apply_shade(base, 66, 68, 68);
        assert!(uphill[0] > base[0], "比西北邻居高应该更亮: {uphill:?}");
        assert!(downhill[0] < base[0], "比西北邻居低应该更暗: {downhill:?}");

        // 只在一个方向上高起来也要有变化，但幅度小于两侧都高
        let north_only = apply_shade(base, 70, 68, 70);
        assert!(north_only[0] > base[0] && north_only[0] < uphill[0], "{north_only:?}");
    }

    /// 越深的水越暗，浅水还能透出海底。
    #[test]
    fn deeper_water_is_darker() {
        let sand = [219u8, 207u8, 163u8, 255];
        let shallow = water_colour(sand, 1, None);
        let middle = water_colour(sand, MAX_WATER_DEPTH / 2, None);
        let deep = water_colour(sand, MAX_WATER_DEPTH, None);

        assert!(shallow[0] > middle[0] && middle[0] > deep[0], "{shallow:?} {middle:?} {deep:?}");
        assert!(shallow[1] > middle[1] && middle[1] > deep[1]);
        assert!(shallow[2] > middle[2] && middle[2] > deep[2]);
        // 水深超上限时不能越界
        assert_eq!(water_colour(sand, MAX_WATER_DEPTH * 4, None), deep);
    }

    /// 水色要跟着群系走：沼泽偏绿、暖水偏青，不是全图一个蓝。
    #[test]
    fn water_colour_follows_the_biome() {
        let floor = [100u8, 100u8, 100u8, 255];
        let ocean = water_colour(floor, MAX_WATER_DEPTH, Some(Biome::Ocean));
        let swamp = water_colour(floor, MAX_WATER_DEPTH, Some(Biome::Swamp));
        let warm = water_colour(floor, MAX_WATER_DEPTH, Some(Biome::WarmOcean));

        assert!(swamp[1] > swamp[2], "沼泽水应该偏绿: {swamp:?}");
        assert!(warm[2] > ocean[2], "暖水应该比普通海更亮: {warm:?} {ocean:?}");
    }

    /// 贴图色表能查到具体方块：沙子是米黄，不是灰。
    #[test]
    fn texture_colours_are_looked_up() {
        let sand = palette().lookup("sand").expect("沙子的贴图色应该在表里");
        assert!(sand[0] > 200 && sand[1] > 190, "沙应该是米黄: {sand:?}");
        assert!(sand[0] > sand[2], "米黄的红通道要高于蓝: {sand:?}");

        let snow = palette().lookup("snow").expect("雪的贴图色应该在表里");
        assert!(snow[0] > 240 && snow[1] > 240 && snow[2] > 240, "{snow:?}");

        assert!(palette().lookup("no_such_block_at_all").is_none());
    }

    /// 草方块的颜色来自群系：平原偏绿，沙漠偏黄。
    #[test]
    fn grass_follows_the_biome() {
        let plains = block_colour("minecraft:grass_block", Some(Biome::Plains), &palette());
        let desert = block_colour("minecraft:grass_block", Some(Biome::Desert), &palette());

        assert!(plains[1] > plains[0], "平原草应该偏绿: {plains:?}");
        assert!(desert[0] > desert[2], "沙漠草应该偏黄褐: {desert:?}");
        assert!(desert[0] > plains[0], "沙漠草的黄要压过平原的绿: {desert:?} {plains:?}");
    }

    /// 衍生方块要借基础材质的贴图，连手写表都没有的才允许是灰。
    #[test]
    fn unknown_texture_falls_back_to_the_palette() {
        let stairs = block_colour("minecraft:oak_stairs", None, &palette());
        let planks = block_colour("minecraft:oak_planks", None, &palette());
        assert_eq!(stairs, planks, "橡木台阶该和木板同色: {stairs:?} {planks:?}");
        assert_ne!(stairs, UNKNOWN, "橡木台阶不该掉进灰色兜底: {stairs:?}");

        // 模组方块在贴图表和手写表里都没有，这时才允许是灰
        assert_eq!(block_colour("minecraft:some_modded_block", None, &palette()), UNKNOWN);
    }
}
