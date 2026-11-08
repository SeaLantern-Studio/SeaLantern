//! 世界地图渲染服务实现。
//!
//! 负责按实例定位世界目录（读 `server.properties` 的 `level-name`，不硬编码
//! `world`），调度 `feature::world_map` 的同步渲染逻辑，并维护磁盘缓存：
//! 缓存文件比源 `.mca` 新时直接复用，否则重新渲染。
//!
//! region 解析与 PNG 编码都是同步且耗时的操作，统一经 `spawn_blocking`
//! 调度到阻塞线程池，不占用异步运行时核心线程。
//!
//! 两个并发相关的约定：
//! - 同一 region 同时只允许一个渲染，其余请求排队后拿缓存结果；
//! - 缓存先写临时文件再 `rename` 替换，读侧永远看到完整文件。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use tokio::sync::Mutex as AsyncMutex;

use sealantern_contract::{
    Dimension, WorldMapImage, WorldMapInfo, WorldMapRegion, WorldMapRegionList, WorldMapRequest,
    WorldMapServiceError,
};
use sealantern_core::instance::InstanceId;
use sealantern_feature::config::ServerPropertiesManager;
use sealantern_feature::world_map::{REGION_PIXELS, RenderInfo, WorldMapError, render_region_png};
use sealantern_infra::platform::get_app_data_dir;

use crate::port::{InstanceService, WorldMapService};
use crate::service::CoreInstanceService;
use crate::service::player_position::flush_world;

/// 一块 region 的 chunk 总数（32 × 32），用于计算覆盖率。
const CHUNKS_PER_REGION: usize = 32 * 32;

/// 两次 `save-all flush` 的最小间隔。
///
/// 一轮刷新会并发拉 3 块 region，不节流就是连发三次 `save-all`，白白卡服务器。
/// 取 30 秒：比前端 15 秒的刷新周期宽一倍，够快又不至于把服务端按住。
const FLUSH_INTERVAL: Duration = Duration::from_secs(30);

/// 世界地图渲染服务。
pub struct CoreWorldMapService {
    instance: Arc<CoreInstanceService>,
    cache_root: PathBuf,
    /// 每个「实例 / 世界 / region」一把异步锁，用于合并并发渲染。
    gates: Mutex<HashMap<String, Arc<AsyncMutex<()>>>>,
    /// 上次让服务器落盘的时间（按实例分），用于给 `save-all flush` 节流。
    last_flush: Mutex<HashMap<String, Instant>>,
}

impl CoreWorldMapService {
    /// 从应用数据目录下的 `map-cache` 建立缓存。
    pub fn new(instance: Arc<CoreInstanceService>) -> Self {
        Self::with_cache_root(instance, get_app_data_dir().join("map-cache"))
    }

    /// 指定缓存根目录（便于测试注入）。
    pub fn with_cache_root(
        instance: Arc<CoreInstanceService>,
        cache_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            instance,
            cache_root: cache_root.into(),
            gates: Mutex::new(HashMap::new()),
            last_flush: Mutex::new(HashMap::new()),
        }
    }

    /// 取某个 region 的渲染闸门；同一 key 始终拿到同一把锁。
    fn gate(
        &self,
        instance_id: &str,
        level_name: &str,
        dimension: Dimension,
        x: isize,
        z: isize,
    ) -> Arc<AsyncMutex<()>> {
        let key = format!("{instance_id}/{level_name}/{}/r.{x}.{z}", dimension.as_str());
        let mut gates = match self.gates.lock() {
            Ok(gates) => gates,
            Err(poisoned) => poisoned.into_inner(),
        };
        gates.entry(key).or_default().clone()
    }

    /// 定位实例某个维度的世界 region 目录，返回（世界名, region 目录, 实例目录）。
    async fn resolve_region_dir(
        &self,
        instance_id: &str,
        dimension: Dimension,
    ) -> Result<(String, PathBuf, PathBuf), WorldMapServiceError> {
        let id =
            InstanceId::new(instance_id).map_err(|_| WorldMapServiceError::InstanceNotFound)?;
        let instance = self
            .instance
            .find(&id)
            .await
            .map_err(|error| {
                tracing::warn!(
                    target: "sealantern.application.world_map",
                    instance_id,
                    error = %error,
                    "读取实例记录失败"
                );
                WorldMapServiceError::OperationFailed
            })?
            .ok_or(WorldMapServiceError::InstanceNotFound)?;

        let root = instance.directory.clone();
        let server_dir = root.clone();
        let (level, region_dir) = tokio::task::spawn_blocking(move || {
            let level = read_level_name(&root);
            let region_dir = locate_region_dir(&root, &level, dimension);
            (level, region_dir)
        })
        .await
        .map_err(|_| WorldMapServiceError::OperationFailed)?;

        let Some(region_dir) = region_dir else {
            tracing::warn!(
                target: "sealantern.application.world_map",
                instance_id,
                level_name = %level,
                dimension = dimension.as_str(),
                "世界 region 目录不存在"
            );
            return Err(WorldMapServiceError::WorldNotFound);
        };

        Ok((level, region_dir, server_dir))
    }

    /// 让服务器把内存里的区块刷进 `.mca`，并按 [`FLUSH_INTERVAL`] 节流。
    ///
    /// 失败不报错：服务器没运行、没开 RCON 都是正常情况，按现有磁盘内容渲染就好。
    /// 但必须在读缓存**之前**做——只比 `.mca` 的修改时间的话，服务端不落盘，
    /// 缓存就永远「比源文件新」，地图会一直停在第一次渲染的样子。
    async fn flush_world_if_due(&self, server_dir: &Path, instance_id: &str) {
        {
            let mut last = match self.last_flush.lock() {
                Ok(last) => last,
                Err(poisoned) => poisoned.into_inner(),
            };
            let now = Instant::now();
            if let Some(previous) = last.get(instance_id)
                && now.duration_since(*previous) < FLUSH_INTERVAL
            {
                return;
            }
            last.insert(instance_id.to_string(), now);
        }

        match flush_world(server_dir.to_path_buf()).await {
            Ok(()) => tracing::debug!(
                target: "sealantern.application.world_map",
                instance_id,
                "已让服务器把区块刷到磁盘"
            ),
            Err(error) => tracing::debug!(
                target: "sealantern.application.world_map",
                instance_id,
                error = %error,
                "跳过 save-all flush，按现有磁盘内容渲染"
            ),
        }
    }

    /// 渲染或复用缓存，返回完整图片。
    async fn load(&self, request: &WorldMapRequest) -> Result<WorldMapImage, WorldMapServiceError> {
        let dimension = request.dimension;
        let (level_name, region_dir, server_dir) = self
            .resolve_region_dir(&request.instance_id, dimension)
            .await?;
        let (region_x, region_z) = (request.region_x, request.region_z);
        let (png_path, meta_path) = cache_paths(
            &self.cache_root,
            &request.instance_id,
            &level_name,
            dimension,
            region_x,
            region_z,
        );
        let source = region_dir.join(format!("r.{region_x}.{region_z}.mca"));

        // 同一 region 的并发请求在这里排队，后到的直接吃前一个写好的缓存
        let gate = self.gate(&request.instance_id, &level_name, dimension, region_x, region_z);
        let _guard = gate.lock().await;

        // 先落盘再看缓存：不然服务端不写 `.mca`，缓存永远判定「比源文件新」，
        // 地图就永远停在第一次渲染的样子
        self.flush_world_if_due(&server_dir, &request.instance_id)
            .await;

        let (cached_png, cached_meta, cached_src) =
            (png_path.clone(), meta_path.clone(), source.clone());
        let cached =
            tokio::task::spawn_blocking(move || read_cache(&cached_png, &cached_meta, &cached_src))
                .await
                .map_err(|_| WorldMapServiceError::OperationFailed)?;
        if let Some((png, mut info)) = cached {
            info.from_cache = true;
            return Ok(WorldMapImage { info, png });
        }

        // 先渲染到临时文件再原子替换，避免读侧读到写了一半的 PNG。
        // 临时名带进程号：桌面端和 Web 端可能同时渲染同一块 region，
        // 共用一个 `.part` 名字会互相踩。
        let tmp_png = png_path.with_extension(format!("{}.png.part", std::process::id()));
        let (render_dir, render_out) = (region_dir, png_path.clone());
        let rendered = tokio::task::spawn_blocking(move || {
            render_atomically(&render_dir, region_x, region_z, &tmp_png, &render_out)
        })
        .await
        .map_err(|_| WorldMapServiceError::OperationFailed)?
        .map_err(|error| render_error(error, &request.instance_id, region_x, region_z))?;

        let info = WorldMapInfo {
            instance_id: request.instance_id.clone(),
            level_name,
            dimension,
            region_x,
            region_z,
            pixels: REGION_PIXELS,
            rendered_chunks: rendered.rendered_chunks,
            unsupported_chunks: rendered.unsupported_chunks,
            corrupt_chunks: rendered.corrupt_chunks,
            unfinished_chunks: rendered.unfinished_chunks,
            coverage: rendered.rendered_chunks as f32 / CHUNKS_PER_REGION as f32,
            from_cache: false,
            generated_at_unix_secs: unix_secs(Some(SystemTime::now())),
        };

        let png = tokio::fs::read(&png_path)
            .await
            .map_err(|_| WorldMapServiceError::OperationFailed)?;

        // 摘要写失败只影响下次能否命中缓存，不影响本次结果
        match serde_json::to_vec(&info) {
            Ok(meta_json) => {
                let meta_tmp =
                    meta_path.with_extension(format!("{}.json.part", std::process::id()));
                if let Err(error) = write_atomically(&meta_path, &meta_tmp, meta_json).await {
                    tracing::warn!(
                        target: "sealantern.application.world_map",
                        error = %error,
                        "写入地图缓存摘要失败"
                    );
                }
            }
            Err(error) => {
                tracing::warn!(
                    target: "sealantern.application.world_map",
                    error = %error,
                    "序列化地图缓存摘要失败"
                );
            }
        }

        Ok(WorldMapImage { info, png })
    }
}

#[async_trait]
impl WorldMapService for CoreWorldMapService {
    async fn render(
        &self,
        request: WorldMapRequest,
    ) -> Result<WorldMapImage, WorldMapServiceError> {
        self.load(&request).await
    }

    async fn info(&self, request: WorldMapRequest) -> Result<WorldMapInfo, WorldMapServiceError> {
        self.load(&request).await.map(|image| image.info)
    }

    async fn regions(
        &self,
        instance_id: &str,
        dimension: Dimension,
    ) -> Result<WorldMapRegionList, WorldMapServiceError> {
        let (level_name, region_dir, server_dir) =
            self.resolve_region_dir(instance_id, dimension).await?;
        // 清单里的 mtime 是前端判断「哪块地形变了、要不要重新取图」的唯一依据，
        // 不先落盘的话前端永远认为没变化，连图都不会重拉
        self.flush_world_if_due(&server_dir, instance_id).await;
        let instance_id = instance_id.to_string();
        tokio::task::spawn_blocking(move || {
            scan_regions(&region_dir, instance_id, level_name, dimension)
        })
        .await
        .map_err(|_| WorldMapServiceError::OperationFailed)?
    }
}

/// 缓存文件路径：`<根>/<实例>/<世界>/<维度>/r.<x>.<z>.{png,json}`。
///
/// 世界名进路径是必要的：`level-name` 改掉之后再请求同一坐标，旧缓存不能命中。
/// 维度同理：三界的 `r.0.0.mca` 是完全不同的文件。
fn cache_paths(
    cache_root: &Path,
    instance_id: &str,
    level_name: &str,
    dimension: Dimension,
    x: isize,
    z: isize,
) -> (PathBuf, PathBuf) {
    let dir = cache_root
        .join(sanitise(instance_id))
        .join(sanitise(level_name))
        .join(dimension.as_str());
    (dir.join(format!("r.{x}.{z}.png")), dir.join(format!("r.{x}.{z}.json")))
}

/// 实例 ID 与世界名会进路径，按保守字符集过滤，避免路径穿越。
fn sanitise(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "_".to_string()
    } else {
        cleaned
    }
}

/// 同步渲染到临时文件并原子替换目标文件。
fn render_atomically(
    region_dir: &Path,
    x: isize,
    z: isize,
    tmp: &Path,
    out: &Path,
) -> Result<RenderInfo, WorldMapError> {
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|source| WorldMapError::Output {
            path: parent.display().to_string(),
            source,
        })?;
    }

    let rendered = match render_region_png(region_dir, x, z, tmp) {
        Ok(rendered) => rendered,
        Err(error) => {
            // 渲染中途失败会留下半截文件，清掉，避免下次被当成完整缓存
            let _ = std::fs::remove_file(tmp);
            return Err(error);
        }
    };
    if let Err(source) = std::fs::rename(tmp, out) {
        let _ = std::fs::remove_file(tmp);
        return Err(WorldMapError::Output { path: out.display().to_string(), source });
    }
    Ok(rendered)
}

/// 把渲染错误收敛成契约错误，并记录底层原因。
fn render_error(
    error: WorldMapError,
    instance_id: &str,
    x: isize,
    z: isize,
) -> WorldMapServiceError {
    match error {
        WorldMapError::MissingRegion { .. } => WorldMapServiceError::RegionNotFound,
        other => {
            tracing::warn!(
                target: "sealantern.application.world_map",
                instance_id,
                region_x = x,
                region_z = z,
                error = %other,
                "世界地图渲染失败"
            );
            WorldMapServiceError::RenderFailed
        }
    }
}

/// 原子写入：先写 `.part`，再 `rename` 到目标路径。
async fn write_atomically(target: &Path, tmp: &Path, bytes: Vec<u8>) -> std::io::Result<()> {
    tokio::fs::write(tmp, bytes).await?;
    match tokio::fs::rename(tmp, target).await {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = tokio::fs::remove_file(tmp).await;
            Err(error)
        }
    }
}

/// 扫描 region 目录，列出所有 `r.<x>.<z>.mca`。
fn scan_regions(
    region_dir: &Path,
    instance_id: String,
    level_name: String,
    dimension: Dimension,
) -> Result<WorldMapRegionList, WorldMapServiceError> {
    let entries = std::fs::read_dir(region_dir).map_err(|error| {
        tracing::warn!(
            target: "sealantern.application.world_map",
            dir = %region_dir.display(),
            error = %error,
            "读取 region 目录失败"
        );
        // 目录消失算「世界不存在」，权限等其他失败才算内部错误
        match error.kind() {
            std::io::ErrorKind::NotFound => WorldMapServiceError::WorldNotFound,
            _ => WorldMapServiceError::OperationFailed,
        }
    })?;

    let mut regions = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        // 渲染期间的 `.part` 临时文件要排除，只认正经的 .mca
        if path.extension().and_then(|ext| ext.to_str()) != Some("mca") {
            continue;
        }
        let Some((region_x, region_z)) = parse_region_file_name(&path) else {
            continue;
        };
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        regions.push(WorldMapRegion {
            region_x,
            region_z,
            bytes: metadata.len(),
            modified_at_unix_secs: unix_secs(metadata.modified().ok()),
        });
    }

    regions.sort_by_key(|region| (region.region_x, region.region_z));
    Ok(WorldMapRegionList {
        instance_id,
        level_name,
        dimension,
        regions,
    })
}

/// 从 `r.<x>.<z>.mca` 解析坐标；不匹配的文件名返回 `None`。
fn parse_region_file_name(path: &Path) -> Option<(isize, isize)> {
    let name = path.file_name()?.to_str()?;
    let rest = name.strip_prefix("r.")?.strip_suffix(".mca")?;
    let (x, z) = rest.split_once('.')?;
    Some((x.parse().ok()?, z.parse().ok()?))
}

/// 读取 `level-name`；文件缺失、键为空或名字不像目录名时退回默认值 `world`。
///
/// 名字里带分隔符或 `..` 的配置不认：`level-name` 会拼进文件路径，
/// 与其去猜用户意图，不如退回默认世界。
fn read_level_name(server_dir: &Path) -> String {
    match ServerPropertiesManager::new(server_dir).read() {
        Ok(properties) => properties
            .raw
            .get("level-name")
            .map(|value| value.trim().to_string())
            .filter(|value| is_plain_level_name(value))
            .unwrap_or_else(default_level_name),
        Err(_) => default_level_name(),
    }
}

/// 世界名是否是合法的单层目录名。
fn is_plain_level_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\'])
}

/// 定位世界的 `region` 目录。
///
/// 服务端可能带 `--universe worlds` 启动，此时世界位于 `<目录>/worlds/<level>`
/// 而不是 `<目录>/<level>`。两种布局都试，优先标准布局。
///
/// 维度上的差异有两条线：
/// - 原版与 Paper 把下界/末地放在 `<世界>/DIM-1`、`<世界>/DIM1` 下；
/// - 部分服务端（以及 `level-name` 直接配成 `world_nether` 的场景）会把它们
///   拆成独立世界目录 `world_nether` / `world_the_end`，region 直接挂在该目录下。
///
/// 关键约束：**下界/末地绝不允许回落到 `<世界>/region`**，那是主世界的地形，
/// 拿它当下界地图等于给用户一张错图。所以候选路径严格按维度生成，
/// 不做「随便找得到就用」的兜底。
fn locate_region_dir(server_dir: &Path, level_name: &str, dimension: Dimension) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    let world_roots = [server_dir.join(level_name), server_dir.join("worlds").join(level_name)];

    // 1. 原版 / Paper：<世界>/<DIM-x>/region（主世界没有 DIM 这一层）
    for root in &world_roots {
        candidates.push(match dimension.vanilla_subdir() {
            Some(subdir) => root.join(subdir).join("region"),
            None => root.join("region"),
        });
    }

    // 2. Bukkit：独立世界目录 world_nether / world_the_end
    if let Some(suffix) = dimension.bukkit_suffix() {
        for root in [
            server_dir.join(format!("{level_name}{suffix}")),
            server_dir
                .join("worlds")
                .join(format!("{level_name}{suffix}")),
        ] {
            if let Some(subdir) = dimension.vanilla_subdir() {
                candidates.push(root.join(subdir).join("region"));
            }
            candidates.push(root.join("region"));
        }
    }

    candidates.into_iter().find(|path| path.is_dir())
}

/// Minecraft 默认世界名。
fn default_level_name() -> String {
    "world".to_string()
}

/// 系统时间转 Unix 秒；取不到时返回 0。
fn unix_secs(time: Option<SystemTime>) -> u64 {
    time.and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

/// 读取缓存：只有 PNG 不早于源 `.mca` 且摘要可解析时才算命中。
fn read_cache(png_path: &Path, meta_path: &Path, source: &Path) -> Option<(Vec<u8>, WorldMapInfo)> {
    let png_time = modified_time(png_path)?;
    let source_time = modified_time(source)?;
    if png_time < source_time {
        return None;
    }

    let meta_raw = std::fs::read(meta_path).ok()?;
    let info: WorldMapInfo = serde_json::from_slice(&meta_raw).ok()?;
    let png = std::fs::read(png_path).ok()?;

    Some((png, info))
}

/// 文件修改时间。
fn modified_time(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// 临时目录里装配一个可用的实例服务（设置先于实例）。
    async fn test_instance(dir: &Path) -> CoreInstanceService {
        let manager = sealantern_feature::config::SettingsManager::load(dir.join("settings.json"))
            .await
            .expect("设置管理器");
        CoreInstanceService::with_settings(crate::service::CoreSettingsService::with_manager(
            manager,
        ))
    }

    #[test]
    fn level_name_falls_back_to_world() {
        let dir = tempfile::tempdir().expect("临时目录");
        assert_eq!(read_level_name(dir.path()), "world");

        fs::write(dir.path().join("server.properties"), "level-name=\n").expect("写入");
        assert_eq!(read_level_name(dir.path()), "world");
    }

    #[test]
    fn level_name_reads_custom_value() {
        let dir = tempfile::tempdir().expect("临时目录");
        fs::write(
            dir.path().join("server.properties"),
            "server-port=25566\nlevel-name=ThePitWorld\n",
        )
        .expect("写入");

        assert_eq!(read_level_name(dir.path()), "ThePitWorld");
    }

    #[test]
    fn cache_paths_are_scoped_by_instance_and_level() {
        let root = Path::new("cache");
        let (png, meta) = cache_paths(root, "abc", "world", Dimension::Overworld, 0, -1);

        assert_eq!(png, Path::new("cache/abc/world/overworld/r.0.-1.png"));
        assert_eq!(meta, Path::new("cache/abc/world/overworld/r.0.-1.json"));

        let (other, _) = cache_paths(root, "abc", "ThePitWorld", Dimension::Overworld, 0, -1);
        assert_ne!(png, other, "换世界名后不能命中旧缓存");
    }

    #[test]
    fn cache_paths_are_scoped_by_dimension() {
        let root = Path::new("cache");
        let (overworld, _) = cache_paths(root, "abc", "world", Dimension::Overworld, 0, 0);
        let (nether, _) = cache_paths(root, "abc", "world", Dimension::Nether, 0, 0);
        let (end, _) = cache_paths(root, "abc", "world", Dimension::End, 0, 0);

        assert_ne!(overworld, nether, "三界的 r.0.0 是不同文件，不能共用缓存");
        assert_ne!(nether, end);
    }

    #[test]
    fn cache_paths_sanitise_unsafe_names() {
        let root = Path::new("cache");
        let (png, _) = cache_paths(root, "../evil", "my world", Dimension::Overworld, 1, 2);

        assert_eq!(png, Path::new("cache/___evil/my_world/overworld/r.1.2.png"));
    }

    fn sample_info() -> WorldMapInfo {
        WorldMapInfo {
            instance_id: "instance-1".to_string(),
            level_name: "world".to_string(),
            dimension: Dimension::Overworld,
            region_x: 0,
            region_z: 0,
            pixels: REGION_PIXELS,
            rendered_chunks: 841,
            unsupported_chunks: 183,
            corrupt_chunks: 0,
            unfinished_chunks: 0,
            coverage: 841.0 / 1024.0,
            from_cache: false,
            generated_at_unix_secs: 1_700_000_000,
        }
    }

    #[test]
    fn cache_hit_when_png_is_not_older() {
        let dir = tempfile::tempdir().expect("临时目录");
        let source = dir.path().join("r.0.0.mca");
        let png = dir.path().join("r.0.0.png");
        let meta = dir.path().join("r.0.0.json");

        fs::write(&source, b"mca").expect("写源文件");
        fs::write(&png, b"png-bytes").expect("写缓存");
        fs::write(&meta, serde_json::to_vec(&sample_info()).expect("序列化摘要")).expect("写摘要");

        let hit = read_cache(&png, &meta, &source).expect("应命中缓存");
        assert_eq!(hit.0, b"png-bytes");
        assert_eq!(hit.1.rendered_chunks, 841);
    }

    #[test]
    fn cache_miss_when_source_is_newer() {
        let dir = tempfile::tempdir().expect("临时目录");
        let source = dir.path().join("r.0.0.mca");
        let png = dir.path().join("r.0.0.png");
        let meta = dir.path().join("r.0.0.json");

        fs::write(&source, b"mca").expect("写源文件");
        fs::write(&png, b"png-bytes").expect("写缓存");
        fs::write(&meta, serde_json::to_vec(&sample_info()).expect("序列化摘要")).expect("写摘要");

        // 把 PNG 的时间推到一小时前，模拟世界已更新过
        let earlier = SystemTime::now() - std::time::Duration::from_secs(3600);
        let file = fs::OpenOptions::new()
            .write(true)
            .open(&png)
            .expect("打开缓存");
        file.set_modified(earlier).expect("调整修改时间");

        assert!(read_cache(&png, &meta, &source).is_none());
    }

    #[test]
    fn cache_ignores_broken_meta() {
        let dir = tempfile::tempdir().expect("临时目录");
        let source = dir.path().join("r.0.0.mca");
        let png = dir.path().join("r.0.0.png");
        let meta = dir.path().join("r.0.0.json");

        fs::write(&source, b"mca").expect("写源文件");
        fs::write(&png, b"png-bytes").expect("写缓存");
        fs::write(&meta, b"not json").expect("写坏摘要");

        assert!(read_cache(&png, &meta, &source).is_none());
    }

    #[test]
    fn level_name_rejects_path_traversal() {
        let dir = tempfile::tempdir().expect("临时目录");
        for value in ["../escape", "a/b", "a\\b", "..", "."] {
            fs::write(dir.path().join("server.properties"), format!("level-name={value}\n"))
                .expect("写入");
            assert_eq!(read_level_name(dir.path()), "world", "value={value}");
        }
    }

    #[tokio::test]
    async fn gate_is_shared_per_region_and_level() {
        let dir = tempfile::tempdir().expect("临时目录");
        let instance = test_instance(dir.path()).await;
        let service =
            CoreWorldMapService::with_cache_root(Arc::new(instance), dir.path().join("cache"));

        let same_a = service.gate("i", "world", Dimension::Overworld, 0, 0);
        let same_b = service.gate("i", "world", Dimension::Overworld, 0, 0);
        let other_region = service.gate("i", "world", Dimension::Overworld, 0, 1);
        let other_dimension = service.gate("i", "world", Dimension::Nether, 0, 0);

        assert!(Arc::ptr_eq(&same_a, &same_b), "同一 region 必须复用同一把锁");
        assert!(!Arc::ptr_eq(&same_a, &other_region), "不同 region 不应互相阻塞");
        assert!(!Arc::ptr_eq(&same_a, &other_dimension), "不同维度不应互相阻塞");
    }

    #[tokio::test]
    async fn flush_is_throttled_per_instance() {
        let dir = tempfile::tempdir().expect("临时目录");
        let instance = test_instance(dir.path()).await;
        let service =
            CoreWorldMapService::with_cache_root(Arc::new(instance), dir.path().join("cache"));

        // 没有 server.properties，落盘必然失败——但绝不能让调用方炸
        service.flush_world_if_due(dir.path(), "a").await;
        let first = *service
            .last_flush
            .lock()
            .expect("锁")
            .get("a")
            .expect("应记录时间");

        // 节流窗口内再来一次，不能重新计时间
        service.flush_world_if_due(dir.path(), "a").await;
        let second = *service
            .last_flush
            .lock()
            .expect("锁")
            .get("a")
            .expect("应记录时间");
        assert_eq!(first, second, "节流窗口内不应重置落盘时间");

        // 另一个实例有自己的计时，不该被 a 的窗口挡掉
        service.flush_world_if_due(dir.path(), "b").await;
        assert!(service.last_flush.lock().expect("锁").contains_key("b"));
    }

    #[test]
    fn locate_region_dir_supports_universe_layout() {
        // `--universe worlds` 启动时世界在 <目录>/worlds/<level>/region
        let dir = tempfile::tempdir().expect("临时目录");
        fs::write(dir.path().join("server.properties"), "level-name=world\n").expect("写入");
        fs::create_dir_all(dir.path().join("worlds/world/region")).expect("建目录");

        let level = read_level_name(dir.path());
        assert_eq!(
            locate_region_dir(dir.path(), &level, Dimension::Overworld),
            Some(dir.path().join("worlds/world/region"))
        );
    }

    #[test]
    fn locate_region_dir_prefers_standard_layout() {
        let dir = tempfile::tempdir().expect("临时目录");
        fs::create_dir_all(dir.path().join("world/region")).expect("建目录");
        fs::create_dir_all(dir.path().join("worlds/world/region")).expect("建目录");

        assert_eq!(
            locate_region_dir(dir.path(), "world", Dimension::Overworld),
            Some(dir.path().join("world/region"))
        );
    }

    #[test]
    fn locate_region_dir_returns_none_without_world() {
        let dir = tempfile::tempdir().expect("临时目录");
        assert_eq!(locate_region_dir(dir.path(), "world", Dimension::Overworld), None);
    }

    #[test]
    fn locate_region_dir_finds_vanilla_dimension_subdirs() {
        // 原版 / Paper 默认布局：下界在 <世界>/DIM-1，末地在 <世界>/DIM1
        let dir = tempfile::tempdir().expect("临时目录");
        fs::create_dir_all(dir.path().join("world/DIM-1/region")).expect("建目录");
        fs::create_dir_all(dir.path().join("world/DIM1/region")).expect("建目录");

        assert_eq!(
            locate_region_dir(dir.path(), "world", Dimension::Nether),
            Some(dir.path().join("world/DIM-1/region"))
        );
        assert_eq!(
            locate_region_dir(dir.path(), "world", Dimension::End),
            Some(dir.path().join("world/DIM1/region"))
        );
        // 主世界没建 region，就应该老实返回 None，不能把下界目录当成主世界
        assert_eq!(locate_region_dir(dir.path(), "world", Dimension::Overworld), None);
    }

    #[test]
    fn locate_region_dir_never_falls_back_to_overworld() {
        // 只有主世界的 region 时，下界/末地必须是 None —— 拿主世界地形
        // 冒充下界地图比报「世界不存在」糟糕得多
        let dir = tempfile::tempdir().expect("临时目录");
        fs::create_dir_all(dir.path().join("world/region")).expect("建目录");

        assert_eq!(locate_region_dir(dir.path(), "world", Dimension::Nether), None);
        assert_eq!(locate_region_dir(dir.path(), "world", Dimension::End), None);
        assert_eq!(
            locate_region_dir(dir.path(), "world", Dimension::Overworld),
            Some(dir.path().join("world/region"))
        );
    }

    #[test]
    fn locate_region_dir_finds_bukkit_world_dirs() {
        // Bukkit 布局：独立世界目录，region 直接挂在下面
        let dir = tempfile::tempdir().expect("临时目录");
        fs::create_dir_all(dir.path().join("world/region")).expect("建目录");
        fs::create_dir_all(dir.path().join("world_nether/region")).expect("建目录");
        fs::create_dir_all(dir.path().join("world_the_end/region")).expect("建目录");

        assert_eq!(
            locate_region_dir(dir.path(), "world", Dimension::Nether),
            Some(dir.path().join("world_nether/region"))
        );
        assert_eq!(
            locate_region_dir(dir.path(), "world", Dimension::End),
            Some(dir.path().join("world_the_end/region"))
        );
    }

    #[test]
    fn parses_region_file_names() {
        assert_eq!(parse_region_file_name(Path::new("r.0.0.mca")), Some((0, 0)));
        assert_eq!(parse_region_file_name(Path::new("r.-1.2.mca")), Some((-1, 2)));
        assert_eq!(parse_region_file_name(Path::new("r.bad.mca")), None);
        assert_eq!(parse_region_file_name(Path::new("r.0.0.mca.part")), None);
        assert_eq!(parse_region_file_name(Path::new("level.dat")), None);
    }

    #[test]
    fn scan_regions_lists_only_mca_files_in_order() {
        let dir = tempfile::tempdir().expect("临时目录");
        for name in ["r.1.-2.mca", "r.0.0.mca", "r.0.0.mca.part", "session.lock"] {
            fs::write(dir.path().join(name), b"x").expect("写文件");
        }

        let list = scan_regions(
            dir.path(),
            "instance-1".to_string(),
            "world".to_string(),
            Dimension::Nether,
        )
        .expect("扫描 region");

        assert_eq!(list.instance_id, "instance-1");
        assert_eq!(list.level_name, "world");
        assert_eq!(list.dimension, Dimension::Nether);
        let coords: Vec<(isize, isize)> = list
            .regions
            .iter()
            .map(|region| (region.region_x, region.region_z))
            .collect();
        assert_eq!(coords, vec![(0, 0), (1, -2)]);
        assert!(list.regions.iter().all(|region| region.bytes == 1));
    }
}
