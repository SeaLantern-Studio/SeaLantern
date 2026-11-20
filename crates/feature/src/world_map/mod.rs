//! 世界地图：从 Anvil region 文件读取地形与群系，并渲染成俯视 PNG。
//!
//! 两个能力：
//! - [`probe_region`]：统计 chunk 数量、格式版本、群系覆盖与地表高度，用于
//!   验证解析链路与排查数据问题；
//! - [`render_region_png`]：把整块 region 画成 512×512 的 PNG。
//!
//! 这里不做任何宿主相关处理（不碰缓存、不碰实例目录），保持可单元测试。
//!
//! ## 与正在运行的服务器并发
//!
//! 只读 `.mca`，不加文件锁。如果服务端此刻正在写同一块 region，读到的数据可能
//! 半新半旧，极端情况下单个 chunk 根本解不出来。缓解手段有三层：
//!
//! - 调用方在渲染前先发 `save-all flush`，并按实例节流（见应用层的
//!   `flush_world_if_due`），把「脏区块还没落盘」这个最大的窗口关掉；
//! - 解析失败的 chunk 单独记进 `corrupt_chunks`，不会被说成「格式太旧」；
//! - 有损坏时打 `warn`，带上坐标和原因，便于事后判断是不是撞上了保存。
//!
//! 彻底解决要么给 region 加文件锁（服务端不配合，拿不到锁），要么每次渲染前
//! 把整个 `.mca` 复制一份再读（几 MB 到上百 MB 的额外 IO，而地图只是个只读
//! 可视化）。成本与收益不匹配，所以接受这个残余风险并让它可见。

use std::fs::File;
use std::path::Path;

use fastanvil::{Chunk, HeightMode, JavaChunk, Region};

mod render;

pub use render::{REGION_PIXELS, RenderInfo, render_region_png};

/// region 文件探查结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionProbe {
    /// 文件中实际存在的 chunk 数量。
    pub chunks: usize,
    /// 因未完全生成而跳过统计的 chunk 数量。
    pub unfinished_chunks: usize,
    /// 1.18+ 格式（`sections` 结构）的 chunk 数量。
    pub post18_chunks: usize,
    /// 1.13~1.17 格式的 chunk 数量。
    pub pre18_chunks: usize,
    /// 1.12 及更早格式的 chunk 数量。
    pub pre13_chunks: usize,
    /// 能读出群系的列数，共 `已完成 chunk 数 × 256`。
    pub biome_columns: usize,
    /// 地表高度区间；没读到任何列时为 `None`。
    pub surface_range: Option<(isize, isize)>,
}

/// 世界地图相关错误。
#[derive(Debug, thiserror::Error)]
pub enum WorldMapError {
    #[error("无法打开 region 文件 {path}: {source}")]
    Open {
        path: String,
        /// 底层 IO 错误。
        source: std::io::Error,
    },
    #[error("region 结构无法解析: {0}")]
    Region(String),
    #[error("chunk NBT 无法解析: {0}")]
    Chunk(String),
    #[error("渲染 region ({x}, {z}) 失败: {reason}")]
    Render {
        /// region 的 X 坐标。
        x: isize,
        /// region 的 Z 坐标。
        z: isize,
        /// 失败原因。
        reason: String,
    },
    #[error("region r.{x}.{z}.mca 不存在")]
    MissingRegion {
        /// region 的 X 坐标。
        x: isize,
        /// region 的 Z 坐标。
        z: isize,
    },
    #[error("无法写出图片 {path}: {source}")]
    Output {
        /// 输出文件路径。
        path: String,
        /// 底层 IO 错误。
        source: std::io::Error,
    },
}

/// 只有完全生成的 chunk 才参与统计，与 fastanvil 渲染器的判断保持一致。
///
/// `spawn` 也在白名单里，看着违反直觉（它早于 `heightmaps` 与 `full`），但实测
/// 无害，理由有三条：
/// - 1.18+ 的阶段顺序是 `features → initialize_light → light → spawn → heightmaps
///   → full`，方块数据在 `features` 阶段就写完了，到 `spawn` 时地形已经完整；
/// - 缺 heightmap 不会把高度算成 0：fastanvil 的 `HeightMode::Trust` 在
///   `motion_blocking` 缺失时会 fall through 到按方块重算（`chunk.rs` 里
///   `Trust` 分支未命中就继续走 calc 逻辑）；
/// - fastanvil 自己的 `TopShadeRenderer` 用的就是同一份白名单
///   （`OK_STATUSES`，见 `fastanvil-0.32.0/src/render.rs`），这里不单方面收紧。
///
/// 换句话说：`spawn` 的区块画出来是对的，只是慢一点（要重算高度图）。
pub(crate) fn is_finished(status: &str) -> bool {
    matches!(
        status,
        "full"
            | "spawn"
            | "postprocessed"
            | "fullchunk"
            | "minecraft:full"
            | "minecraft:spawn"
            | "minecraft:postprocessed"
            | "minecraft:fullchunk"
    )
}

/// 打开单个 region 文件并统计 chunk、群系与地表高度。
///
/// 只读不写，不修改任何世界数据。
pub fn probe_region(path: &Path) -> Result<RegionProbe, WorldMapError> {
    let file = File::open(path)
        .map_err(|source| WorldMapError::Open { path: path.display().to_string(), source })?;
    let mut region = Region::from_stream(file).map_err(|e| WorldMapError::Region(e.to_string()))?;

    let mut chunks = 0usize;
    let mut unfinished_chunks = 0usize;
    let mut post18_chunks = 0usize;
    let mut pre18_chunks = 0usize;
    let mut pre13_chunks = 0usize;
    let mut biome_columns = 0usize;
    let mut surface_range: Option<(isize, isize)> = None;

    for entry in region.iter() {
        let chunk_data = entry.map_err(|e| WorldMapError::Region(e.to_string()))?;
        let chunk = JavaChunk::from_bytes(&chunk_data.data)
            .map_err(|e| WorldMapError::Chunk(e.to_string()))?;
        chunks += 1;

        match &chunk {
            JavaChunk::Post18(_) => post18_chunks += 1,
            JavaChunk::Pre18(_) => pre18_chunks += 1,
            JavaChunk::Pre13(_) => pre13_chunks += 1,
        }

        if !is_finished(&chunk.status()) {
            unfinished_chunks += 1;
            continue;
        }

        // 逐列取地表高度与群系：地表方块在 air_height - 1，并钳制到世界底，
        // 与 fastanvil 渲染器 drill_for_colour 的取值方式保持一致
        let y_min = chunk.y_range().start;
        for z in 0..16usize {
            for x in 0..16usize {
                let surface = chunk.surface_height(x, z, HeightMode::Trust);
                surface_range = Some(match surface_range {
                    Some((low, high)) => (low.min(surface), high.max(surface)),
                    None => (surface, surface),
                });

                if chunk.biome(x, (surface - 1).max(y_min), z).is_some() {
                    biome_columns += 1;
                }
            }
        }
    }

    Ok(RegionProbe {
        chunks,
        unfinished_chunks,
        post18_chunks,
        pre18_chunks,
        pre13_chunks,
        biome_columns,
        surface_range,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 需要真实 region 文件时才跑。
    ///
    /// ```text
    /// SEALANTERN_TEST_REGION=/path/to/r.0.0.mca \
    ///   cargo test -p sealantern-feature -- --ignored
    /// ```
    ///
    /// 标 `#[ignore]` 而不是「环境变量没设就 return」：后者会让 `cargo test`
    /// 把没跑过的用例报成通过，被测代码坏掉也看不出来。
    #[test]
    #[ignore = "需要 SEALANTERN_TEST_REGION 指向一个真实的 .mca 文件"]
    fn probe_real_region_file() {
        let path = std::env::var("SEALANTERN_TEST_REGION")
            .expect("跑这个测试需要设置 SEALANTERN_TEST_REGION 指向一个真实的 .mca");

        let probe = probe_region(Path::new(&path)).expect("探查 region 失败");
        println!("{probe:#?}");

        assert!(probe.chunks > 0, "region 里应该至少有 1 个 chunk");
        assert!(probe.surface_range.is_some(), "至少应有一列读到地表高度");
        let (low, high) = probe.surface_range.unwrap();
        assert!(low >= -64 && high <= 320, "1.18+ 地表高度应落在 -64..320，实际 {low}..{high}");
    }
}
