use serde::{Deserialize, Serialize};

use crate::dimension::Dimension;

/// 渲染一块世界地图的请求。
///
/// 一个 region 覆盖 512×512 方块，坐标对应 `.mca` 文件名里的 `r.<x>.<z>`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorldMapRequest {
    /// 目标实例 ID。
    pub instance_id: String,
    /// region 的 X 坐标，默认 0。
    #[serde(default)]
    pub region_x: isize,
    /// region 的 Z 坐标，默认 0。
    #[serde(default)]
    pub region_z: isize,
    /// 目标维度，默认主世界。
    #[serde(default)]
    pub dimension: Dimension,
}

/// 一次世界地图渲染的结果摘要。
///
/// `coverage` 与各个 chunk 计数都来自真实数据，不做美化：没渲染的部分就是 0，
/// 调用方据此决定是否提示「该区域含旧版本区块」或「该区域有区块读取失败」。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorldMapInfo {
    /// 目标实例 ID。
    pub instance_id: String,
    /// 世界名（`server.properties` 的 `level-name`）。
    pub level_name: String,
    /// 该 region 所属维度。
    pub dimension: Dimension,
    /// region 的 X 坐标。
    pub region_x: isize,
    /// region 的 Z 坐标。
    pub region_z: isize,
    /// 输出图片边长，恒为 512。
    pub pixels: u32,
    /// 已渲染的 chunk 数（1.18+ 且已生成）。
    pub rendered_chunks: usize,
    /// 因不是 1.18+ 格式而跳过的 chunk 数。
    ///
    /// 覆盖 1.13~1.17 与 1.12 及更早两种旧格式：1.18 之前的区块结构与当前渲染器
    /// 不兼容，一律跳过。这张摘要不细分版本，需要逐版本统计请用探查接口。
    pub unsupported_chunks: usize,
    /// 因区块数据无法解析而跳过的 chunk 数。
    ///
    /// 与 [`Self::unsupported_chunks`] 区分：那个是版本问题，数据本身完好；
    /// 这个是数据损坏，常见于服务端正在写这块 region。两者混在一起会把
    /// 「图上有洞」误判成版本限制。
    pub corrupt_chunks: usize,
    /// 因未完全生成而跳过的 chunk 数。
    pub unfinished_chunks: usize,
    /// 已渲染 chunk 占整块 region 的比例，取值 0.0~1.0。
    pub coverage: f32,
    /// 是否直接命中磁盘缓存。
    pub from_cache: bool,
    /// 图片生成时间（Unix 秒）。
    pub generated_at_unix_secs: u64,
}

/// 渲染结果：摘要 + PNG 字节。
///
/// 刻意不实现 `Serialize`：图片走原始字节（HTTP 直接回 `image/png`，
/// 桌面端走 IPC `ArrayBuffer`），编进 JSON 会让体积膨胀数倍。
#[derive(Debug, Clone, PartialEq)]
pub struct WorldMapImage {
    /// 渲染摘要。
    pub info: WorldMapInfo,
    /// PNG 文件内容。
    pub png: Vec<u8>,
}

/// 一个已存在的 region 文件（`r.<x>.<z>.mca`）的概要。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorldMapRegion {
    /// region 的 X 坐标。
    pub region_x: isize,
    /// region 的 Z 坐标。
    pub region_z: isize,
    /// `.mca` 文件大小（字节）。
    pub bytes: u64,
    /// `.mca` 最后修改时间（Unix 秒）；读不到时为 0。
    pub modified_at_unix_secs: u64,
}

/// 某个实例的世界里所有可渲染的 region。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WorldMapRegionList {
    /// 目标实例 ID。
    pub instance_id: String,
    /// 世界名（`server.properties` 的 `level-name`）。
    pub level_name: String,
    /// 该清单对应的维度。
    pub dimension: Dimension,
    /// 已按 `(region_x, region_z)` 升序排列的 region 列表。
    pub regions: Vec<WorldMapRegion>,
}
