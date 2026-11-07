//! 把单个 region 渲染成俯视 PNG。
//!
//! 上色以**地表方块**为主，群系只用来给草方块和树叶做色调（和 Minecraft
//! 自带地图的做法一致）。纯群系配色对建筑图无效：练习服/竞技场世界通常整张
//! 图只有一种群系（实测 ThePit 世界 256 列全是 Ocean），按群系上色只会得到
//! 一片纯色。
//!
//! 一个 region 固定输出 512×512 像素（32 chunk × 16 方块）。
//!
//! 只渲染 1.18+（`sections`）格式的 chunk。更早的 chunk 会被跳过并留空：
//! fastanvil 0.32 的 pre-1.13 路径在 `init_default_block` 里对半砖数据值
//! 用了 `unreachable!()`（`data_value & 0b1000` 只可能得 0 或 8，却去匹配 0/1），
//! 碰到旧区块会直接 panic，所以不能交给它处理。

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use fastanvil::biome::Biome;
use fastanvil::{Block, Chunk, HeightMode, JavaChunk, Palette, Region, Rgba, TopShadeRenderer};

use super::{WorldMapError, is_finished};

/// 一个 region 的边长（方块数）。
pub const REGION_PIXELS: u32 = 512;

/// 每个 region 每个方向的 chunk 数。
const CHUNKS_PER_SIDE: usize = 32;

/// 水与海草等水体方块的统一色。
const WATER: Rgba = [58, 102, 175, 255];

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
    /// 因不是 1.18+ 格式而跳过的 chunk 数。
    pub unsupported_chunks: usize,
    /// 因未完全生成而跳过的 chunk 数。
    pub unfinished_chunks: usize,
}

/// 按地表方块上色、用群系给植被做色调的调色板。
pub struct TerrainPalette;

impl Palette for TerrainPalette {
    fn pick(&self, block: &Block, biome: Option<Biome>) -> Rgba {
        block_colour(block.name(), biome)
    }
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
fn block_colour(name: &str, biome: Option<Biome>) -> Rgba {
    if is_water_like(name) {
        return WATER;
    }

    let short = name.strip_prefix("minecraft:").unwrap_or(name);

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
pub fn render_region_png(
    region_dir: &Path,
    x: isize,
    z: isize,
    out_path: &Path,
) -> Result<RenderInfo, WorldMapError> {
    let mut region = open_region(region_dir, x, z)?;
    let renderer = TopShadeRenderer::new(&TerrainPalette, HeightMode::Trust);

    let side = CHUNKS_PER_SIDE * 16;
    let mut pixels = vec![[0u8, 0u8, 0u8, 0u8]; side * side];
    // 缓存上一行 chunk，供渲染器做跨 chunk 的北向阴影
    let mut north_cache: [Option<JavaChunk>; CHUNKS_PER_SIDE] = Default::default();

    let mut rendered_chunks = 0usize;
    let mut unsupported_chunks = 0usize;
    let mut unfinished_chunks = 0usize;

    for cz in 0..CHUNKS_PER_SIDE {
        for (cx, north_slot) in north_cache.iter_mut().enumerate() {
            let Some(data) = region
                .read_chunk(cx, cz)
                .map_err(|e| WorldMapError::Region(e.to_string()))?
            else {
                *north_slot = None;
                continue;
            };

            let chunk = match JavaChunk::from_bytes(&data) {
                Ok(chunk) => chunk,
                Err(_) => {
                    unsupported_chunks += 1;
                    *north_slot = None;
                    continue;
                }
            };

            // 只有 1.18+ 格式能安全交给渲染器，旧格式会 panic（见模块注释）
            if !matches!(chunk, JavaChunk::Post18(_)) {
                unsupported_chunks += 1;
                *north_slot = None;
                continue;
            }

            if !is_finished(&chunk.status()) {
                unfinished_chunks += 1;
                *north_slot = None;
                continue;
            }

            let tile = renderer.render(&chunk, north_slot.as_ref());
            *north_slot = Some(chunk);
            rendered_chunks += 1;

            let row_offset = cz * 16;
            let col_offset = cx * 16;
            for row in 0..16 {
                let dest = (row_offset + row) * side + col_offset;
                pixels[dest..dest + 16].copy_from_slice(&tile[row * 16..row * 16 + 16]);
            }
        }
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

    let file_bytes = std::fs::metadata(out_path).map(|m| m.len()).unwrap_or(0);

    Ok(RenderInfo {
        pixels: REGION_PIXELS,
        bytes: file_bytes,
        rendered_chunks,
        unsupported_chunks,
        unfinished_chunks,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 需要真实 region 目录时才跑：设置 `SEALANTERN_TEST_REGION_DIR`。
    #[test]
    fn render_real_region_to_png() {
        let Ok(dir) = std::env::var("SEALANTERN_TEST_REGION_DIR") else {
            eprintln!("跳过：未设置 SEALANTERN_TEST_REGION_DIR");
            return;
        };

        let out = std::env::temp_dir().join("sealantern_world_map_test.png");
        let info = render_region_png(Path::new(&dir), 0, 0, &out).expect("渲染 region 失败");

        println!("输出: {} ({:?})", out.display(), info);

        assert_eq!(info.pixels, REGION_PIXELS);
        assert!(info.rendered_chunks > 0, "应该有 1.18+ chunk 被渲染");
        assert!(info.bytes > 10_000, "PNG 体积过小，可能是全空图：{} 字节", info.bytes);
    }

    /// 不同方块要产出不同颜色，避免像纯群系配色那样整张图一个色。
    #[test]
    fn block_colours_are_distinct() {
        let green = block_colour("minecraft:green_terracotta", None);
        let lime = block_colour("minecraft:lime_terracotta", None);
        let clay = block_colour("minecraft:clay", None);
        let stone = block_colour("minecraft:stone", None);

        assert_ne!(green, lime);
        assert_ne!(clay, stone);
        assert_ne!(green, clay);
        // 水方块固定走水体色
        assert_eq!(block_colour("minecraft:water", None), WATER);
    }
}
