//! 世界地图渲染服务实现。
//!
//! 负责按实例定位世界目录（读 `server.properties` 的 `level-name`，不硬编码
//! `world`），调度 `feature::world_map` 的同步渲染逻辑，并维护磁盘缓存。
//!
//! 缓存的命中判据不是「文件谁更新」，而是一整组指纹全对上：请求身份
//! （实例目录哈希 / 世界 / 维度 / region）、源 `.mca` 的修改时间与大小、
//! 以及 PNG 的校验和。任何一项对不上就重画 —— 多花一次渲染，换的是绝不把
//! 别人的图、或者上一版的地形交出去。
//!
//! region 解析与 PNG 编码都是同步且耗时的操作，统一经 `spawn_blocking`
//! 调度到阻塞线程池，不占用异步运行时核心线程。
//!
//! 并发相关的约定：
//! - 同一 region 同时只允许一个渲染，其余请求排队后拿缓存结果；
//! - 缓存先写临时文件再 `rename` 替换（临时名带进程号），读侧永远看到完整文件；
//! - 渲染前后各取一次源指纹，变了就重画，避免把「跨存盘的拼接图」写进缓存。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::Semaphore;

use sealantern_contract::{
    Dimension, PlayerPositionServiceError, WorldMapImage, WorldMapInfo, WorldMapRegion,
    WorldMapRegionList, WorldMapRequest, WorldMapServiceError,
};
use sealantern_core::instance::InstanceId;
use sealantern_feature::config::ServerPropertiesManager;
use sealantern_feature::world_map::{REGION_PIXELS, RenderInfo, WorldMapError, render_region_png};
use sealantern_infra::platform::get_app_data_dir;

use crate::port::{InstanceService, WorldMapService};
use crate::service::player_position::{RconEndpoint, flush_world, own_rcon_endpoint};
use crate::service::{CoreInstanceService, CoreServerService};

/// 一块 region 的 chunk 总数（32 × 32），用于计算覆盖率。
const CHUNKS_PER_REGION: usize = 32 * 32;

/// `gates` 超过这个条数就回收一轮已经没人持有的锁。
///
/// 256 是拍出来的：一屏最多同时看几十块 region，留一个数量级的余量。
const GATE_CLEANUP_THRESHOLD: usize = 256;

/// flush 失败后的重试间隔。
///
/// 比成功窗口短得多：失败往往是一时的（RCON 刚断开、服务端在重启），
/// 拖满 30 秒会让地图用户白看半分钟旧数据。
const FLUSH_RETRY: Duration = Duration::from_secs(5);

/// 两次 `save-all flush` 的最小间隔。
///
/// 一轮刷新会并发拉 3 块 region，不节流就是连发三次 `save-all`，白白卡服务器。
/// 取 30 秒：比前端 15 秒的刷新周期宽一倍，够快又不至于把服务端按住。
const FLUSH_INTERVAL: Duration = Duration::from_secs(30);

/// 一块 region 最多渲染几次。
///
/// 正常情况下一次就够。重画的原因只有一个：渲染途中服务端存了盘，`.mca` 变了，
/// 这张图可能是「半新半旧」几块 chunk 拼起来的。两次还没稳定就放弃缓存
/// （图照给，但下次请求会重画），比拿一张拼接图当好图存下来强。
const RENDER_ATTEMPTS: usize = 2;

/// 地图缓存占用的磁盘上限。
///
/// 一块 region 约 160KB，512MB 够存三千多块，覆盖几十人服一个季度的活动范围。
/// 超了就按「最久没被写过」淘汰 —— 已经删掉的实例和老世界自然排在最前面。
const CACHE_BUDGET_BYTES: u64 = 512 * 1024 * 1024;

/// 两次缓存清理的最小间隔（只在刚渲染完时才可能触发）。
const PRUNE_INTERVAL: Duration = Duration::from_secs(3600);

/// 累计新写入多少字节之后强制清一次缓存。
///
/// 只按 [`PRUNE_INTERVAL`] 清理的话，一小时内连着翻几千块 region 就能把缓存
/// 撑到预算之上，而且在这之前一块都不回收。按写入量触发，溢出量最多是
/// `预算 + 这个数`，跟浏览速度无关。
const PRUNE_WRITE_TRIGGER_BYTES: u64 = 64 * 1024 * 1024;

/// 同时最多几块 region 在渲染。
///
/// 一块 region 就是一次同步解析加 PNG 编码，占满阻塞线程池很容易。前端那边
/// 也只有 3 个并发取图，这里放宽没意义；真正需要防的是「能访问 API 的人不停
/// 请求不同坐标」——每块 region 有各自的锁，锁不住并发度。
const RENDER_CONCURRENCY: usize = 3;

/// 残留的 `.part` 临时文件多久之后可以删。
///
/// 渲染中途崩进程会留下它们；一小时足够长，不会误删正在写的那个。
const STALE_PART_AGE: Duration = Duration::from_secs(3600);

/// 世界地图渲染服务。
pub struct CoreWorldMapService {
    instance: Arc<CoreInstanceService>,
    /// 只用来判断「占着同一个 RCON 端口的那个实例是不是正在运行」——
    /// 落盘走的是 RCON，连错了就是把 `save-all` 打到别人的服上。
    server: Arc<CoreServerService>,
    cache_root: PathBuf,
    /// 每个「实例 / 世界 / region」一把异步锁，用于合并并发渲染。
    ///
    /// 存的是弱引用：region 的数量没有上限（一个大地图几千块），把每把锁都
    /// 永久留下等于随用户翻阅地图一直涨内存。渲染结束、锁没别的持有者之后
    /// 会被 `retain` 顺手清掉。
    gates: Mutex<HashMap<String, Weak<AsyncMutex<()>>>>,
    /// 上次让服务器落盘的结果（按实例分）：`(时间, 这个结果挡多久)`。
    ///
    /// 成功和失败挡的时长不一样 —— 成功挡满 [`FLUSH_INTERVAL`]，失败只挡一个短
    /// 窗口。失败如果也算满窗口，RCON 只是临时抽风就会导致接下来 30 秒都拿旧图；
    /// 完全不挡，服务器没开 RCON 时每个地图请求又都要等一次连接超时。
    last_flush: Mutex<HashMap<String, (Instant, Duration)>>,
    /// 上次清理磁盘缓存的时间，避免每渲染一块就遍历一遍整棵目录树。
    last_prune: Mutex<Option<Instant>>,
    /// 上次清理之后新写进缓存的字节数，到量就绕过 [`PRUNE_INTERVAL`] 清一次。
    written_since_prune: AtomicU64,
    /// 渲染并发上限（见 [`RENDER_CONCURRENCY`]）。
    render_slots: Arc<Semaphore>,
}

/// 一次落盘尝试的结果。
///
/// 不用 `Result` 是因为「失败」分三档，挡窗口的时长完全不同：配置写死的不开
/// RCON、端口被别人占着，这两种短时间内不会变，挡满窗口；连不上、密码不对
/// 是临时状态，只挡一个短窗口。
enum FlushOutcome {
    /// 成功落盘。
    Done,
    /// 服务器没开 RCON（`enable-rcon` 不是 true 或密码为空）。
    Disabled,
    /// RCON 端口被另一个正在运行的实例占着，落盘会把 `save-all` 打到别人家。
    Occupied(String),
    /// 临时失败：连不上、密码不对、端口归属查不出来。
    Transient(PlayerPositionServiceError),
}

impl CoreWorldMapService {
    /// 从应用数据目录下的 `map-cache` 建立缓存。
    pub fn new(instance: Arc<CoreInstanceService>, server: Arc<CoreServerService>) -> Self {
        Self::with_cache_root(instance, server, get_app_data_dir().join("map-cache"))
    }

    /// 指定缓存根目录（便于测试注入）。
    pub fn with_cache_root(
        instance: Arc<CoreInstanceService>,
        server: Arc<CoreServerService>,
        cache_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            instance,
            server,
            cache_root: cache_root.into(),
            gates: Mutex::new(HashMap::new()),
            last_flush: Mutex::new(HashMap::new()),
            last_prune: Mutex::new(None),
            written_since_prune: AtomicU64::new(0),
            render_slots: Arc::new(Semaphore::new(RENDER_CONCURRENCY)),
        }
    }

    /// 清空整个地图缓存目录。
    ///
    /// 删的是可再生的派生数据，最多让下一次打开地图重渲染一遍。正在渲染的
    /// region 可能撞上删除（临时文件写不进去），那种情况渲染会失败并记日志，
    /// 下次请求重来即可。
    pub fn clear_cache(&self) {
        match std::fs::remove_dir_all(&self.cache_root) {
            Ok(()) => tracing::debug!(
                target: "sealantern.application.world_map",
                path = %self.cache_root.display(),
                "已清空地图缓存"
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => tracing::warn!(
                target: "sealantern.application.world_map",
                path = %self.cache_root.display(),
                error = %error,
                "清空地图缓存失败，旧配色可能仍然可见"
            ),
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

        // 表满之前先回收一轮没人持有的锁。回收之后同一个 region 会拿到一把新锁，
        // 但那把旧锁已经没有持有者了，不存在两个请求各持一把锁同时渲染的情况
        if gates.len() >= GATE_CLEANUP_THRESHOLD {
            gates.retain(|_, gate| gate.strong_count() > 0);
        }

        if let Some(existing) = gates.get(&key).and_then(Weak::upgrade) {
            return existing;
        }
        let gate = Arc::new(AsyncMutex::new(()));
        gates.insert(key, Arc::downgrade(&gate));
        gate
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
            let level = read_level_name(&root)?;
            let region_dir = locate_region_dir(&root, &level, dimension);
            Ok::<_, WorldMapServiceError>((level, region_dir))
        })
        .await
        .map_err(|_| WorldMapServiceError::OperationFailed)??;

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
    ///
    /// 三条时序上的约束：
    /// - 同一实例的 flush 串行，否则并发请求各发一次 `save-all`，而且后到的那个
    ///   会在第一次还没落盘时就去读磁盘；
    /// - 时间戳在 **flush 成功之后** 才记。先记再刷的话，flush 失败会把窗口
    ///   白白占掉，接下来半分钟都读旧数据；
    /// - 失败也记一个短窗口，避免服务器没开 RCON 时每个请求都卡一次超时。
    async fn flush_world_if_due(&self, server_dir: &Path, instance_id: &str) {
        let gate = self.instance_gate(instance_id);
        let _guard = gate.lock().await;

        // 排队等锁期间，前一个请求可能刚刷完，进来先确认一次
        if self.flush_is_throttled(instance_id) {
            return;
        }

        match self.flush_once(server_dir, instance_id).await {
            FlushOutcome::Done => {
                self.record_flush(instance_id, FLUSH_INTERVAL);
                tracing::debug!(
                    target: "sealantern.application.world_map",
                    instance_id,
                    "已让服务器把区块刷到磁盘"
                );
            }
            // 没开 RCON 是配置里写死的，短时间内不会变，挡满窗口免得
            // 每个请求都去读一次 server.properties
            FlushOutcome::Disabled => {
                self.record_flush(instance_id, FLUSH_INTERVAL);
                tracing::debug!(
                    target: "sealantern.application.world_map",
                    instance_id,
                    "服务器没开 RCON，按现有磁盘内容渲染"
                );
            }
            // 端口被别人占着也不会几秒内就变（得有人先去停服），同样挡满窗口
            FlushOutcome::Occupied(owner) => {
                self.record_flush(instance_id, FLUSH_INTERVAL);
                tracing::warn!(
                    target: "sealantern.application.world_map",
                    instance_id,
                    owner = %owner,
                    "RCON 端口被另一个正在运行的实例占用，跳过落盘"
                );
            }
            // 连不上、密码不对、归属查不出来：都是短时间可能恢复的状态，
            // 只挡一个短窗口，免得服务器恢复了地图还贴着旧图
            FlushOutcome::Transient(error) => {
                self.record_flush(instance_id, FLUSH_RETRY);
                let quiet = matches!(error, PlayerPositionServiceError::RconUnreachable);
                if quiet {
                    tracing::debug!(
                        target: "sealantern.application.world_map",
                        instance_id,
                        error = %error,
                        "服务器没在跑，按现有磁盘内容渲染"
                    );
                } else {
                    tracing::warn!(
                        target: "sealantern.application.world_map",
                        instance_id,
                        error = %error,
                        "save-all flush 失败，地图可能落后于服务器内存里的数据"
                    );
                }
            }
        }
    }

    /// 走一次落盘：先确认 RCON 端点确实归本实例，再发命令。
    async fn flush_once(&self, server_dir: &Path, instance_id: &str) -> FlushOutcome {
        let Ok(id) = InstanceId::new(instance_id) else {
            return FlushOutcome::Transient(PlayerPositionServiceError::InstanceNotFound);
        };

        let endpoint = match own_rcon_endpoint(&self.instance, &self.server, &id, server_dir).await
        {
            Ok(endpoint) => endpoint,
            Err(PlayerPositionServiceError::RconDisabled) => return FlushOutcome::Disabled,
            Err(error) => return FlushOutcome::Transient(error),
        };

        match endpoint {
            RconEndpoint::Own(target) => match flush_world(&target).await {
                Ok(()) => FlushOutcome::Done,
                Err(PlayerPositionServiceError::RconDisabled) => FlushOutcome::Disabled,
                Err(error) => FlushOutcome::Transient(error),
            },
            RconEndpoint::OccupiedBy(owner) => FlushOutcome::Occupied(owner),
            // 归属查不出来时宁可不落盘：`save-all flush` 会改服务端状态，
            // 打到别人家就是替别人的服存了一次盘
            RconEndpoint::Unknown => {
                FlushOutcome::Transient(PlayerPositionServiceError::OperationFailed)
            }
        }
    }

    /// 这个实例是否还在落盘冷却期里。
    fn flush_is_throttled(&self, instance_id: &str) -> bool {
        let last = match self.last_flush.lock() {
            Ok(last) => last,
            Err(poisoned) => poisoned.into_inner(),
        };
        last.get(instance_id)
            .is_some_and(|(at, window)| at.elapsed() < *window)
    }

    /// 记下这次 flush 的结果和它该挡多久。
    fn record_flush(&self, instance_id: &str, window: Duration) {
        let mut last = match self.last_flush.lock() {
            Ok(last) => last,
            Err(poisoned) => poisoned.into_inner(),
        };
        last.insert(instance_id.to_string(), (Instant::now(), window));
    }

    /// 按实例取一把 flush 锁，保证同一实例同时只有一次落盘在跑。
    fn instance_gate(&self, instance_id: &str) -> Arc<AsyncMutex<()>> {
        let mut gates = match self.gates.lock() {
            Ok(gates) => gates,
            Err(poisoned) => poisoned.into_inner(),
        };
        let key = format!("flush/{instance_id}");
        if gates.len() >= GATE_CLEANUP_THRESHOLD {
            gates.retain(|_, gate| gate.strong_count() > 0);
        }
        if let Some(existing) = gates.get(&key).and_then(Weak::upgrade) {
            return existing;
        }
        let gate = Arc::new(AsyncMutex::new(()));
        gates.insert(key, Arc::downgrade(&gate));
        gate
    }

    /// 渲染或复用缓存，返回完整图片。
    async fn load(&self, request: &WorldMapRequest) -> Result<WorldMapImage, WorldMapServiceError> {
        let dimension = request.dimension;
        let instance_id = request.instance_id.clone();
        let (level_name, region_dir, server_dir) =
            self.resolve_region_dir(&instance_id, dimension).await?;
        let (region_x, region_z) = (request.region_x, request.region_z);

        // 缓存目录带上「实例 + 实际目录」的指纹：单用实例 ID 会让
        // `sanitise` 折成同一串的两个实例共用缓存，也会让「ID 不变、目录改指」
        // 的实例读到上一个世界的图
        let instance_dir = instance_dir_key(&instance_id, &server_dir);
        let key = cache_key(&instance_dir, &level_name, dimension, region_x, region_z);
        let (png_path, meta_path) = cache_paths(
            &self.cache_root,
            &instance_dir,
            &level_name,
            dimension,
            region_x,
            region_z,
        );
        let source = region_dir.join(format!("r.{region_x}.{region_z}.mca"));

        // 同一 region 的并发请求在这里排队，后到的直接吃前一个写好的缓存
        let gate = self.gate(&instance_id, &level_name, dimension, region_x, region_z);
        let _guard = gate.lock().await;

        // 先落盘再看缓存：不然服务端不写 `.mca`，缓存永远判定「比源文件新」，
        // 地图就永远停在第一次渲染的样子
        self.flush_world_if_due(&server_dir, &instance_id).await;

        let (cached_png, cached_meta, cached_src, cached_key) =
            (png_path.clone(), meta_path.clone(), source.clone(), key.clone());
        let cached = tokio::task::spawn_blocking(move || {
            read_cache(&cached_png, &cached_meta, &cached_src, &cached_key)
        })
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
        let mut rendered = None;
        let mut png: Vec<u8> = Vec::new();
        let mut stamp = None;
        let mut stable = false;
        for attempt in 1..=RENDER_ATTEMPTS {
            // 渲染前后的源指纹：服务端在这中间存了盘，就说明这张图是跨版本的
            // 拼接结果，宁可重画一次也不写进缓存
            let before = source_stamp(&source);

            let (render_dir, render_out, render_tmp) =
                (region_dir.clone(), png_path.clone(), tmp_png.clone());
            // 占一个渲染名额：并发度不够时在这里排队，而不是把阻塞线程池塞满
            let slot = Arc::clone(&self.render_slots)
                .acquire_owned()
                .await
                .map_err(|_| WorldMapServiceError::OperationFailed)?;
            let result = tokio::task::spawn_blocking(move || {
                render_atomically(&render_dir, region_x, region_z, &render_tmp, &render_out)
            })
            .await;
            drop(slot);
            let result = result
                .map_err(|_| WorldMapServiceError::OperationFailed)?
                .map_err(|error| render_error(error, &instance_id, region_x, region_z))?;

            png = tokio::fs::read(&png_path)
                .await
                .map_err(|_| WorldMapServiceError::OperationFailed)?;
            let after = source_stamp(&source);
            rendered = Some(result);
            stamp = after;
            stable = before.is_some() && before == after;
            if stable {
                break;
            }
            tracing::debug!(
                target: "sealantern.application.world_map",
                instance_id = %instance_id,
                region_x,
                region_z,
                attempt,
                "渲染期间源 region 被改过，重画一次"
            );
        }
        let rendered = rendered.expect("至少渲染一次");
        let png_hash = checksum(&png);

        let info = WorldMapInfo {
            instance_id: instance_id.clone(),
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

        // 摘要和 PNG 一起存：`png_hash` 保证「摘要描述的确实是这张图」，
        // 另一个进程在我们之后覆盖了 PNG 时，这份摘要会被判失效而不是误导前端
        match (stable, stamp) {
            (true, Some(stamp)) => {
                let entry = CacheEntry {
                    key,
                    source_mtime_nanos: stamp.mtime_nanos,
                    source_bytes: stamp.bytes,
                    png_hash,
                    info: info.clone(),
                };
                match serde_json::to_vec(&entry) {
                    Ok(meta_json) => {
                        let meta_tmp =
                            meta_path.with_extension(format!("{}.json.part", std::process::id()));
                        if let Err(error) = write_atomically(&meta_path, &meta_tmp, meta_json).await
                        {
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
            }
            _ => tracing::debug!(
                target: "sealantern.application.world_map",
                instance_id = %instance_id,
                region_x,
                region_z,
                "源 region 一直在变，这次不写缓存"
            ),
        }

        // 新图刚落地，记一笔写入量，再看看缓存是不是该清了
        self.written_since_prune
            .fetch_add(png.len() as u64, Ordering::Relaxed);
        self.prune_cache_if_needed().await;

        Ok(WorldMapImage { info, png })
    }

    /// 缓存该清的时候清一次，按「写入量」和「时间」双重节流。
    ///
    /// 只按小时节流的话，一小时内连着翻几千块 region 会一路把缓存撑到预算
    /// 之上，而且在下一小时到来之前一块都不回收；只看写入量又会让空闲的实例
    /// 永远不清理。两个条件各管一半。
    async fn prune_cache_if_needed(&self) {
        let overgrown =
            self.written_since_prune.load(Ordering::Relaxed) >= PRUNE_WRITE_TRIGGER_BYTES;
        {
            let mut last = match self.last_prune.lock() {
                Ok(last) => last,
                Err(poisoned) => poisoned.into_inner(),
            };
            if !overgrown && last.is_some_and(|at: Instant| at.elapsed() < PRUNE_INTERVAL) {
                return;
            }
            *last = Some(Instant::now());
        }
        self.written_since_prune.store(0, Ordering::Relaxed);

        let root = self.cache_root.clone();
        let removed = tokio::task::spawn_blocking(move || {
            prune_cache(&root, CACHE_BUDGET_BYTES, SystemTime::now())
        })
        .await;
        match removed {
            Ok(Ok(0)) => {}
            Ok(Ok(bytes)) => tracing::info!(
                target: "sealantern.application.world_map",
                bytes,
                "地图缓存超出预算，已淘汰最久未用的部分"
            ),
            Ok(Err(error)) => tracing::warn!(
                target: "sealantern.application.world_map",
                error = %error,
                "清理地图缓存失败"
            ),
            Err(_) => {}
        }
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

/// 缓存文件路径：`<根>/<实例指纹>/<世界>/<维度>/r.<x>.<z>.{png,json}`。
///
/// 世界名进路径是必要的：`level-name` 改掉之后再请求同一坐标，旧缓存不能命中。
/// 维度同理：三界的 `r.0.0.mca` 是完全不同的文件。
fn cache_paths(
    cache_root: &Path,
    instance_dir: &str,
    level_name: &str,
    dimension: Dimension,
    x: isize,
    z: isize,
) -> (PathBuf, PathBuf) {
    let dir = cache_root
        .join(instance_dir)
        .join(level_dir_key(level_name))
        .join(dimension.as_str());
    (dir.join(format!("r.{x}.{z}.png")), dir.join(format!("r.{x}.{z}.json")))
}

/// 这份缓存属于哪个请求。
///
/// 只用来「认亲」：读缓存时拿它和当前请求比一遍，对不上就当没命中。
/// 目录名虽然也算得出来，但 `sanitise` 是有损的，不能当身份用。
fn cache_key(
    instance_dir: &str,
    level_name: &str,
    dimension: Dimension,
    x: isize,
    z: isize,
) -> String {
    format!("{instance_dir}/{level_name}/{}/r.{x}.{z}", dimension.as_str())
}

/// 缓存目录名里的实例标识：可读前缀 + 「原始 ID + 实际目录」的哈希。
///
/// 两个原因不能只用 ID：
/// - `sanitise` 会把 `a/b`、`a:b`、`a b` 折成同一个 `a_b`，不同实例共用缓存；
/// - ID 不变但目录被改指（`update_instance_path`），新世界会读到旧世界的图。
///
/// 大小写只在 Windows 上折叠：Windows 不区分大小写，同一个目录换个写法不该
/// 被当成两个实例；类 Unix 的 `A` 和 `a` 是两个货真价实的目录，折了会让它们
/// 互相冒用缓存 —— 正好是这条哈希想防的事。
fn instance_dir_key(instance_id: &str, server_dir: &Path) -> String {
    let canonical = std::fs::canonicalize(server_dir).unwrap_or_else(|_| server_dir.to_path_buf());
    let canonical_text = canonical.to_string_lossy();
    let path_key = if cfg!(windows) {
        canonical_text.to_lowercase()
    } else {
        canonical_text.into_owned()
    };

    let mut hash = FNV_OFFSET_BASIS;
    for byte in instance_id.as_bytes() {
        hash = fnv_step(hash, *byte);
    }
    // 分隔符，避免 `("ab", "c")` 和 `("a", "bc")` 撞在同一串上
    hash = fnv_step(hash, 0xff);
    for byte in path_key.bytes() {
        hash = fnv_step(hash, byte);
    }
    format!("{}-{hash:016x}", sanitise(instance_id))
}

/// 缓存目录里的世界名：可读前缀 + 原始名字的哈希。
///
/// `sanitise` 是有损的：`a b`、`a_b`、`a:b` 都折成 `a_b`。只拿它当目录名，
/// 改一次 `level-name` 就能让两个世界共用同一份 PNG，而 [`gate`] 是按原始
/// 名字上锁的 —— 两把不同的锁同时往同一组文件里写，互相覆盖临时文件、
/// 让一张图配上另一份摘要。
///
/// [`gate`]: CoreWorldMapService::gate
fn level_dir_key(level_name: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in level_name.as_bytes() {
        hash = fnv_step(hash, *byte);
    }
    format!("{}-{hash:016x}", sanitise(level_name))
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
    for entry in entries {
        // 单个条目读不出来不该让整张列表失败，但也不能悄悄跳过一个真实存在的
        // region —— 那样用户会以为地图缺了一块
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                tracing::warn!(
                    target: "sealantern.application.world_map",
                    dir = %region_dir.display(),
                    error = %error,
                    "跳过读不出来的目录项"
                );
                continue;
            }
        };
        let path = entry.path();
        // 渲染期间的 `.part` 临时文件要排除，只认正经的 .mca
        if path.extension().and_then(|ext| ext.to_str()) != Some("mca") {
            continue;
        }
        let Some((region_x, region_z)) = parse_region_file_name(&path) else {
            continue;
        };
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(error) => {
                tracing::warn!(
                    target: "sealantern.application.world_map",
                    path = %path.display(),
                    error = %error,
                    "读不到 region 的元数据，列表里先跳过"
                );
                continue;
            }
        };
        regions.push(WorldMapRegion {
            region_x,
            region_z,
            bytes: metadata.len(),
            modified_at_unix_millis: unix_millis(metadata.modified().ok()),
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

/// 读取 `level-name`；键缺失或名字不像目录名时退回默认值 `world`。
///
/// 名字里带分隔符、盘符或 `..` 的配置不认：`level-name` 会拼进文件路径，
/// 与其去猜用户意图，不如退回默认世界。
///
/// 读**不动**这份文件（权限、编码、格式坏了）是另一回事：这时候回退到 `world`
/// 等于赌「用户没改过世界名」。赌错的表现是给用户一张看着正常、其实是别的
/// 世界的地图 —— 比直接报错糟糕得多，所以这里如实把错误往上抛。
fn read_level_name(server_dir: &Path) -> Result<String, WorldMapServiceError> {
    match ServerPropertiesManager::new(server_dir).read() {
        Ok(properties) => Ok(properties
            .raw
            .get("level-name")
            .map(|value| value.trim().to_string())
            .filter(|value| is_plain_level_name(value))
            .unwrap_or_else(default_level_name)),
        Err(error) => {
            tracing::warn!(
                target: "sealantern.application.world_map",
                dir = %server_dir.display(),
                error = %error,
                "读不了 server.properties，无法确定世界名，放弃渲染"
            );
            Err(WorldMapServiceError::OperationFailed)
        }
    }
}

/// 世界名是否是合法的单层目录名。
///
/// 按路径组件判断，而不是「不含分隔符」：Windows 上 `D:..` 既没有斜杠也不是
/// `..`，但它带着盘符前缀和父目录组件，`join` 上去会切到别的盘并往上跳。
/// 只认「恰好一个普通组件」能一次挡掉前缀、根、`..` 和分隔符。
fn is_plain_level_name(name: &str) -> bool {
    let mut components = Path::new(name).components();
    matches!(components.next(), Some(std::path::Component::Normal(_)))
        && components.next().is_none()
}

/// 定位世界的 `region` 目录。
///
/// 服务端可能带 `--universe worlds` 启动，此时世界位于 `<目录>/worlds/<level>`
/// 而不是 `<目录>/<level>`。两套目录同时存在很常见（换过启动方式就会留下一份），
/// 该用哪份见 [`world_roots`]。
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
    let roots = world_roots(server_dir, level_name);
    let mut candidates: Vec<PathBuf> = Vec::new();

    // 1. 原版 / Paper：<世界>/<DIM-x>/region（主世界没有 DIM 这一层）。
    //    维度子目录是「这个世界的下界/末地」最硬的证据，排在最前面
    for root in &roots {
        candidates.push(match dimension.vanilla_subdir() {
            Some(subdir) => root.join(subdir).join("region"),
            None => root.join("region"),
        });
    }

    // 2. Bukkit：独立世界目录 world_nether / world_the_end
    if let Some(suffix) = dimension.bukkit_suffix() {
        for base in [server_dir.to_path_buf(), server_dir.join("worlds")] {
            let root = base.join(format!("{level_name}{suffix}"));
            if let Some(subdir) = dimension.vanilla_subdir() {
                candidates.push(root.join(subdir).join("region"));
            }
            candidates.push(root.join("region"));
        }
    }

    // 3. `level-name` 直接写成了某个维度的独立世界（`level-name=world_nether`），
    //    region 就挂在这个世界目录下。这一刻的 `root/region` 确实是下界地形，
    //    不是「回落到主世界」。
    //
    //    但光看名字后缀不够：一个恰好叫 `foo_nether` 的普通主世界会因此把
    //    自己的地形当成下界地图交出去。所以要一个旁证 —— 同名主世界 `foo`
    //    确实存在，说明这个目录是它的维度伴生世界
    if let Some(suffix) = dimension.bukkit_suffix()
        && let Some(base) = level_name.strip_suffix(suffix)
        && !base.is_empty()
        && has_world_root(server_dir, base)
    {
        for root in &roots {
            candidates.push(root.join("region"));
        }
    }

    candidates.into_iter().find(|path| path.is_dir())
}

/// `<name>` 这个世界的 `region` 目录存在吗（两套布局都算）。
fn has_world_root(server_dir: &Path, name: &str) -> bool {
    server_dir.join(name).join("region").is_dir()
        || server_dir.join("worlds").join(name).join("region").is_dir()
}

/// 两套布局的世界根目录，按「哪个是当前活动的世界」排序。
///
/// 判断依据是 `level.dat` 的修改时间：服务端每次启动和自动保存都会写它，
/// 所以活动世界的这份文件不可能比废弃的那份旧。一个典型的坑是服务器从默认
/// 布局换到 `--universe worlds` 之后，老的 `<level>/region` 还留在原地 ——
/// 只按「标准布局优先」查，地图会一直画那份没人再动过的旧地形。
///
/// 两边的 `level.dat` 都读不到时间时，退回默认布局优先。
fn world_roots(server_dir: &Path, level_name: &str) -> [PathBuf; 2] {
    let standard = server_dir.join(level_name);
    let universe = server_dir.join("worlds").join(level_name);
    let written_at = |root: &Path| {
        std::fs::metadata(root.join("level.dat"))
            .and_then(|metadata| metadata.modified())
            .ok()
    };
    match (written_at(&standard), written_at(&universe)) {
        (Some(standard_at), Some(universe_at)) if universe_at > standard_at => [universe, standard],
        _ => [standard, universe],
    }
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

/// 系统时间转 Unix 毫秒；取不到时返回 0。
///
/// region 清单用它而不是秒：一组连续存盘完全可能落在同一秒里，秒级时间戳
/// 看不出变化，前端就一直贴着旧图不重拉。
fn unix_millis(time: Option<SystemTime>) -> u64 {
    time.and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

/// FNV-1a 64 位偏移基准。
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a 的素数。
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv_step(hash: u64, byte: u8) -> u64 {
    (hash ^ byte as u64).wrapping_mul(FNV_PRIME)
}

/// 字节串校验和。
///
/// 不用密码学哈希：这里只需要「这张 PNG 还是不是摘要描述的那张」，
/// 碰撞概率 2^-64 足够，FNV 又比 SHA 便宜一个数量级。
fn checksum(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash = fnv_step(hash, *byte);
    }
    hash
}

/// 源 `.mca` 的指纹。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SourceStamp {
    /// 修改时间（Unix 纳秒）。
    mtime_nanos: u64,
    /// 文件大小。
    bytes: u64,
}

/// 读源文件指纹；文件不存在或读不到元数据时返回 `None`。
fn source_stamp(path: &Path) -> Option<SourceStamp> {
    let metadata = std::fs::metadata(path).ok()?;
    Some(SourceStamp {
        mtime_nanos: unix_nanos(metadata.modified().ok()),
        bytes: metadata.len(),
    })
}

fn unix_nanos(time: Option<SystemTime>) -> u64 {
    time.and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0)
}

/// 一份磁盘缓存：摘要 + 判断这份摘要在当前还成不成立所需的全部依据。
///
/// 单独存 `WorldMapInfo` 不够：PNG 和摘要是两个文件，中间任何一步失败、
/// 或者另一个进程正好覆盖了 PNG，留下的都是一份「看着合法、其实对不上」的
/// 缓存。把源指纹和 PNG 校验和一起写进摘要，读的时候逐项核对，
/// 对不上就当没命中 —— 最差多渲染一次，不会把别人的图送给用户。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CacheEntry {
    /// 请求指纹（见 [`cache_key`]）。
    key: String,
    /// 渲染时源 `.mca` 的修改时间（Unix 纳秒）。
    source_mtime_nanos: u64,
    /// 渲染时源 `.mca` 的大小。
    source_bytes: u64,
    /// 这次渲染写出的 PNG 的校验和。
    png_hash: u64,
    /// 给前端看的摘要。
    info: WorldMapInfo,
}

/// 读取缓存：请求指纹、源指纹、PNG 校验和三者都对上才算命中。
fn read_cache(
    png_path: &Path,
    meta_path: &Path,
    source: &Path,
    key: &str,
) -> Option<(Vec<u8>, WorldMapInfo)> {
    let stamp = source_stamp(source)?;
    let meta_raw = std::fs::read(meta_path).ok()?;
    let entry: CacheEntry = serde_json::from_slice(&meta_raw).ok()?;

    // 换了实例 / 世界 / 维度 / region，或者是更早版本写下的旧格式摘要
    if entry.key != key {
        return None;
    }
    // 源在渲染之后又被存过盘（或者文件被换掉）
    if entry.source_mtime_nanos != stamp.mtime_nanos || entry.source_bytes != stamp.bytes {
        return None;
    }

    let png = std::fs::read(png_path).ok()?;
    // PNG 不是这份摘要生成的那张：多半是另一个进程后来重画并覆盖了它
    if checksum(&png) != entry.png_hash {
        return None;
    }

    Some((png, entry.info))
}

/// 清理磁盘缓存：删掉陈旧的 `.part`，总量超预算时按最久未写入淘汰。
///
/// 返回删掉的字节数。整个目录树扫一遍，所以调用方要节流（见
/// [`CoreWorldMapService::prune_cache_if_due`]）。
///
/// 「最久未写入」用的是文件修改时间，而不是记一张访问表：地图缓存的价值
/// 本来就随世界变化衰减，已删实例、老世界的文件自然躺在最前面先被淘汰。
fn prune_cache(root: &Path, budget: u64, now: SystemTime) -> std::io::Result<u64> {
    let mut files: Vec<(PathBuf, u64, SystemTime)> = Vec::new();
    collect_cache_files(root, &mut files)?;

    let mut removed = 0u64;
    let mut keep: Vec<&(PathBuf, u64, SystemTime)> = Vec::with_capacity(files.len());
    for entry in &files {
        // 临时文件一律不参与预算和淘汰。正在渲染的那个 `.part` 一旦被当成
        // 「最久未写入」删掉，渲染写完 rename 就找不到源文件，整个请求失败。
        // 只有放了很久的（渲染中途崩进程留下的）才清掉
        if is_temporary(&entry.0) {
            let stale = now
                .duration_since(entry.2)
                .map(|age| age >= STALE_PART_AGE)
                .unwrap_or(false);
            if stale && std::fs::remove_file(&entry.0).is_ok() {
                removed += entry.1;
            }
            continue;
        }
        keep.push(entry);
    }

    let mut total: u64 = keep.iter().map(|(_, bytes, _)| *bytes).sum();
    if total <= budget {
        return Ok(removed);
    }

    // 最旧的先走，删到预算以内就停
    keep.sort_by_key(|entry| entry.2);
    for (path, bytes, _) in keep {
        if total <= budget {
            break;
        }
        if std::fs::remove_file(path).is_ok() {
            total -= bytes;
            removed += bytes;
        }
    }
    Ok(removed)
}

/// 这个文件是不是渲染中的临时文件。
///
/// 临时名是 `<正式名>.<进程号>.png.part` / `.json.part`，所以后缀判断要看
/// 整串末尾：`extension()` 对前者是 `part`，对 `r.0.0.mca` 这类正式文件是
/// `mca`，但也存在 `r.0.0.png.part` 这种带两截的写法，统一按结尾判最省事。
fn is_temporary(path: &Path) -> bool {
    path.to_string_lossy().ends_with(".part")
}

/// 递归收集缓存目录下的所有文件；目录不存在视为空。
///
/// 单个条目读不出来只跳过它：清理是尽力而为的维护动作，不该因为一个
/// 权限异常的文件就把整轮清理作废。
fn collect_cache_files(
    dir: &Path,
    out: &mut Vec<(PathBuf, u64, SystemTime)>,
) -> std::io::Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.is_dir() {
            let _ = collect_cache_files(&path, out);
        } else {
            let mtime = metadata.modified().unwrap_or(UNIX_EPOCH);
            out.push((path, metadata.len(), mtime));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// 临时目录里装配一对可用的「实例服务 + 服务器进程服务」。
    ///
    /// 地图服务两样都要：实例服务定位世界目录，进程服务判断 RCON 端口是被
    /// 哪个**运行中**实例占着（落盘前必须确认，否则 `save-all` 会打到别人家）。
    async fn test_services(dir: &Path) -> (Arc<CoreInstanceService>, Arc<CoreServerService>) {
        let manager = sealantern_feature::config::SettingsManager::load(dir.join("settings.json"))
            .await
            .expect("设置管理器");
        let settings = Arc::new(crate::service::CoreSettingsService::with_manager(manager));
        let instance = Arc::new(CoreInstanceService::new(settings.clone()));
        let server = Arc::new(CoreServerService::new(instance.clone(), settings));
        (instance, server)
    }

    #[test]
    fn level_name_falls_back_to_world() {
        let dir = tempfile::tempdir().expect("临时目录");
        // 文件缺失是常态：原版默认世界就叫 `world`，这时候回退是对的
        assert_eq!(read_level_name(dir.path()).expect("读世界名"), "world");

        fs::write(dir.path().join("server.properties"), "level-name=\n").expect("写入");
        assert_eq!(read_level_name(dir.path()).expect("读世界名"), "world");
    }

    /// `server.properties` 读不动时不能假装世界叫 `world`。
    ///
    /// 那种情况下旁边只要存在一个 `world/region`，地图就会拿它当真图交给用户，
    /// 而用户真正配的世界可能完全是另一个。
    #[test]
    fn level_name_fails_when_properties_unreadable() {
        let dir = tempfile::tempdir().expect("临时目录");
        fs::create_dir_all(dir.path().join("world/region")).expect("建目录");
        // 非 UTF-8 内容：读成字符串就失败，属于「文件在但读不动」
        fs::write(dir.path().join("server.properties"), [0xff, 0xfe, 0x00]).expect("写入");

        assert_eq!(read_level_name(dir.path()), Err(WorldMapServiceError::OperationFailed));
    }

    #[test]
    fn level_name_reads_custom_value() {
        let dir = tempfile::tempdir().expect("临时目录");
        fs::write(
            dir.path().join("server.properties"),
            "server-port=25566\nlevel-name=ThePitWorld\n",
        )
        .expect("写入");

        assert_eq!(read_level_name(dir.path()).expect("读世界名"), "ThePitWorld");
    }

    #[test]
    fn cache_paths_are_scoped_by_instance_and_level() {
        let root = Path::new("cache");
        let (png, meta) = cache_paths(root, "abc-0000", "world", Dimension::Overworld, 0, -1);

        assert_eq!(png.file_name().unwrap(), "r.0.-1.png");
        assert_eq!(meta.file_name().unwrap(), "r.0.-1.json");
        assert_eq!(png.parent().unwrap().file_name().unwrap(), "overworld");
        // 世界那一层是「可读前缀 + 哈希」，前缀取自世界名
        assert!(
            png.parent()
                .unwrap()
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("world-")
        );

        let (other, _) = cache_paths(root, "abc-0000", "ThePitWorld", Dimension::Overworld, 0, -1);
        assert_ne!(png, other, "换世界名后不能命中旧缓存");
    }

    /// `sanitise` 会把不同世界名折成同一串，缓存目录必须仍然分得开。
    ///
    /// 分不开的后果不只是「命中旧图」：两个世界的锁是按原始名字申请的，
    /// 共享输出路径意味着两把不同的锁同时往同一组文件里写。
    #[test]
    fn cache_paths_separate_level_names_that_sanitise_alike() {
        let root = Path::new("cache");
        let (a, _) = cache_paths(root, "abc", "a b", Dimension::Overworld, 0, 0);
        let (b, _) = cache_paths(root, "abc", "a_b", Dimension::Overworld, 0, 0);
        let (c, _) = cache_paths(root, "abc", "a:b", Dimension::Overworld, 0, 0);

        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
        // 前缀仍然可读，方便人眼在缓存目录里找
        assert!(a.to_string_lossy().contains("a_b-"));
    }

    /// 缓存目录名要能区分「`sanitise` 后撞名」的实例，也要能跟着目录变。
    #[test]
    fn instance_dir_key_separates_lookalike_instances_and_repointed_dirs() {
        let dir = tempfile::tempdir().expect("临时目录");
        let one = dir.path().join("server-a");
        let two = dir.path().join("server-b");
        fs::create_dir_all(&one).expect("建目录");
        fs::create_dir_all(&two).expect("建目录");

        // `a/b` 与 `a:b` 经过字符过滤都会变成 `a_b`，必须靠哈希分开
        assert_ne!(instance_dir_key("a/b", &one), instance_dir_key("a:b", &one));
        // ID 不变、目录改指：缓存也得跟着换
        assert_ne!(instance_dir_key("same", &one), instance_dir_key("same", &two));
        // 同一对输入要稳定，否则每请求一次都算未命中
        assert_eq!(instance_dir_key("same", &one), instance_dir_key("same", &one));
    }

    #[test]
    fn instance_dir_key_is_a_single_safe_path_segment() {
        let dir = tempfile::tempdir().expect("临时目录");
        let key = instance_dir_key("../evil", dir.path());

        assert!(!key.contains('/') && !key.contains('\\') && !key.contains(".."), "{key}");
        assert!(key.starts_with("___evil-"), "{key}");
    }

    /// 超预算时按最久未写入淘汰，且不碰预算内的文件。
    #[test]
    fn prune_cache_evicts_the_oldest_files_over_budget() {
        let dir = tempfile::tempdir().expect("临时目录");
        let old_dir = dir.path().join("i-old/world/overworld");
        let new_dir = dir.path().join("i-new/world/overworld");
        fs::create_dir_all(&old_dir).expect("建目录");
        fs::create_dir_all(&new_dir).expect("建目录");

        let old_png = old_dir.join("r.0.0.png");
        let new_png = new_dir.join("r.0.0.png");
        fs::write(&old_png, vec![0u8; 800]).expect("写旧缓存");
        fs::write(&new_png, vec![0u8; 800]).expect("写新缓存");

        // 把旧文件的时间推到一天前
        let yesterday = SystemTime::now() - std::time::Duration::from_secs(86_400);
        fs::OpenOptions::new()
            .write(true)
            .open(&old_png)
            .expect("打开旧缓存")
            .set_modified(yesterday)
            .expect("调整时间");

        // 预算只够放一份，最旧的先走
        let removed = prune_cache(dir.path(), 1_000, SystemTime::now()).expect("清理缓存");
        assert_eq!(removed, 800);
        assert!(!old_png.exists(), "最久未写入的应当被淘汰");
        assert!(new_png.exists(), "预算内最新的那份要留着");
    }

    #[test]
    fn prune_cache_keeps_everything_under_budget() {
        let dir = tempfile::tempdir().expect("临时目录");
        let cache = dir.path().join("i/world/overworld");
        fs::create_dir_all(&cache).expect("建目录");
        let png = cache.join("r.0.0.png");
        fs::write(&png, vec![0u8; 100]).expect("写缓存");

        assert_eq!(prune_cache(dir.path(), 1_000, SystemTime::now()).expect("清理"), 0);
        assert!(png.exists());
    }

    /// 崩溃残留的 `.part` 只在足够旧时才清，正在写的那个不能动。
    #[test]
    fn prune_cache_removes_only_stale_part_files() {
        let dir = tempfile::tempdir().expect("临时目录");
        fs::create_dir_all(dir.path()).expect("建目录");
        let fresh = dir.path().join("r.0.0.4242.png.part");
        let stale = dir.path().join("r.0.1.4242.png.part");
        fs::write(&fresh, b"half").expect("写临时文件");
        fs::write(&stale, b"half").expect("写临时文件");

        let old = SystemTime::now() - std::time::Duration::from_secs(7_200);
        fs::OpenOptions::new()
            .write(true)
            .open(&stale)
            .expect("打开临时文件")
            .set_modified(old)
            .expect("调整时间");

        prune_cache(dir.path(), u64::MAX, SystemTime::now()).expect("清理");
        assert!(fresh.exists(), "刚写的临时文件还在用");
        assert!(!stale.exists(), "两小时前的残留该清掉");
    }

    /// 超预算淘汰不能顺手删掉正在渲染的那份 `.part`。
    ///
    /// 渲染是先写 `.part`、再 `rename` 到正式名：中途被清理删掉的话，
    /// `rename` 找不到源文件，整个请求直接失败。临时文件比正式文件旧是常态
    /// （正式文件每次命中缓存都会被读，但不会被重写），所以按「最久未写入」
    /// 排序它一定排在最前面 —— 必须整个排除在候选之外。
    #[test]
    fn prune_cache_never_evicts_an_in_flight_part_file() {
        let dir = tempfile::tempdir().expect("临时目录");
        let cache = dir.path().join("i/world/overworld");
        fs::create_dir_all(&cache).expect("建目录");
        let done = cache.join("r.0.0.png");
        let part = cache.join("r.0.1.4242.png.part");
        fs::write(&done, vec![0u8; 100]).expect("写缓存");
        fs::write(&part, vec![0u8; 900]).expect("写临时文件");

        // 临时文件更旧，按 LRU 它排第一；预算只够放下一份
        let old = SystemTime::now() - std::time::Duration::from_secs(60);
        fs::OpenOptions::new()
            .write(true)
            .open(&part)
            .expect("打开临时文件")
            .set_modified(old)
            .expect("调整时间");

        prune_cache(dir.path(), 100, SystemTime::now()).expect("清理");
        assert!(part.exists(), "正在写的临时文件不能被淘汰");
        assert!(done.exists(), "临时文件不占预算，正式文件反而留下了");
    }

    /// 覆盖已存在的缓存文件必须能成功。
    ///
    /// 评审连着两轮怀疑「Windows 上 rename 到已存在的目标会失败」。std 在
    /// Windows 上走的是 `MoveFileExW(MOVEFILE_REPLACE_EXISTING)`，本来就是覆盖
    /// 语义；这条测试把这个假设钉住，将来底层实现真变了也能第一时间发现。
    #[tokio::test]
    async fn write_atomically_replaces_existing_target() {
        let dir = tempfile::tempdir().expect("临时目录");
        let target = dir.path().join("r.0.0.json");
        let tmp = dir.path().join("r.0.0.4242.json.part");
        fs::write(&target, b"old").expect("写旧缓存");

        write_atomically(&target, &tmp, b"new".to_vec())
            .await
            .expect("覆盖写已有目标");
        assert_eq!(fs::read(&target).expect("读目标"), b"new");
        assert!(!tmp.exists(), "临时文件不该留下");

        // 渲染路径用的是同步 rename，同样是覆盖语义
        let sync_tmp = dir.path().join("r.0.1.4242.png.part");
        fs::write(&sync_tmp, b"png").expect("写临时文件");
        std::fs::rename(&sync_tmp, &target).expect("同步 rename 覆盖已有目标");
        assert_eq!(fs::read(&target).expect("读目标"), b"png");
    }

    /// 写入量到量时必须清一次，哪怕距上次清理还不到一小时。
    #[tokio::test]
    async fn prune_is_forced_by_written_volume() {
        let dir = tempfile::tempdir().expect("临时目录");
        let (instance, server) = test_services(dir.path()).await;
        let service =
            CoreWorldMapService::with_cache_root(instance, server, dir.path().join("cache"));

        // 假装刚清过：按时间这一轮不该清
        *service.last_prune.lock().expect("锁") = Some(Instant::now());
        service
            .written_since_prune
            .store(PRUNE_WRITE_TRIGGER_BYTES, Ordering::Relaxed);

        service.prune_cache_if_needed().await;

        assert_eq!(
            service.written_since_prune.load(Ordering::Relaxed),
            0,
            "写入量到量就该清一次，并把计数归零"
        );
    }

    /// 没到量、又在小时窗口里，就别去遍历整棵缓存目录。
    #[tokio::test]
    async fn prune_waits_for_the_hour_when_written_volume_is_low() {
        let dir = tempfile::tempdir().expect("临时目录");
        let (instance, server) = test_services(dir.path()).await;
        let service =
            CoreWorldMapService::with_cache_root(instance, server, dir.path().join("cache"));

        *service.last_prune.lock().expect("锁") = Some(Instant::now());
        service.written_since_prune.store(1, Ordering::Relaxed);

        service.prune_cache_if_needed().await;

        assert_eq!(
            service.written_since_prune.load(Ordering::Relaxed),
            1,
            "没触发清理时不该把写入量清零"
        );
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
        // 世界名是配置里来的，仍要过滤；实例目录已经是算好的指纹
        let (png, _) = cache_paths(root, "___evil-0123", "my world", Dimension::Overworld, 1, 2);
        let text = png.to_string_lossy().replace('\\', "/");

        assert!(
            text.starts_with("cache/___evil-0123/my_world-"),
            "世界名要过滤成一个安全的可读前缀，实际：{text}"
        );
        assert!(text.ends_with("/overworld/r.1.2.png"), "实际：{text}");
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

    /// 在临时目录里摆一份「源 + PNG + 摘要」齐全的缓存。
    fn seed_cache(dir: &Path, key: &str, png_bytes: &[u8]) -> (PathBuf, PathBuf, PathBuf) {
        let source = dir.join("r.0.0.mca");
        let png = dir.join("r.0.0.png");
        let meta = dir.join("r.0.0.json");

        fs::write(&source, b"mca").expect("写源文件");
        fs::write(&png, png_bytes).expect("写缓存");
        let stamp = source_stamp(&source).expect("源指纹");
        let entry = CacheEntry {
            key: key.to_string(),
            source_mtime_nanos: stamp.mtime_nanos,
            source_bytes: stamp.bytes,
            png_hash: checksum(png_bytes),
            info: sample_info(),
        };
        fs::write(&meta, serde_json::to_vec(&entry).expect("序列化摘要")).expect("写摘要");
        (source, png, meta)
    }

    #[test]
    fn cache_hit_when_key_and_source_match() {
        let dir = tempfile::tempdir().expect("临时目录");
        let key = "i-1/world/overworld/r.0.0";
        let (source, png, meta) = seed_cache(dir.path(), key, b"png-bytes");

        let hit = read_cache(&png, &meta, &source, key).expect("应命中缓存");
        assert_eq!(hit.0, b"png-bytes");
        assert_eq!(hit.1.rendered_chunks, 841);
    }

    #[test]
    fn cache_miss_when_source_changed_after_render() {
        let dir = tempfile::tempdir().expect("临时目录");
        let key = "i-1/world/overworld/r.0.0";
        let (source, png, meta) = seed_cache(dir.path(), key, b"png-bytes");

        // 渲染之后服务端又存了一次盘
        fs::write(&source, b"mca-after-save").expect("改写源文件");
        assert!(read_cache(&png, &meta, &source, key).is_none());
    }

    /// 摘要是别的请求（撞名的实例、换了目录的实例）写下的，不能拿来用。
    #[test]
    fn cache_miss_when_key_differs() {
        let dir = tempfile::tempdir().expect("临时目录");
        let (source, png, meta) = seed_cache(dir.path(), "i-1/world/overworld/r.0.0", b"png-bytes");

        assert!(read_cache(&png, &meta, &source, "i-2/world/overworld/r.0.0").is_none());
    }

    /// 另一个进程覆盖了 PNG，旧摘要不能跟着这张新图一起被交出去。
    #[test]
    fn cache_miss_when_png_was_replaced_under_the_old_summary() {
        let dir = tempfile::tempdir().expect("临时目录");
        let key = "i-1/world/overworld/r.0.0";
        let (source, png, meta) = seed_cache(dir.path(), key, b"png-bytes");

        fs::write(&png, b"another-process-png").expect("覆盖 PNG");
        assert!(read_cache(&png, &meta, &source, key).is_none());
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

        assert!(read_cache(&png, &meta, &source, "k").is_none());
    }

    /// 旧版本写下的纯 `WorldMapInfo` 摘要认不出来，应当按未命中处理。
    #[test]
    fn cache_ignores_legacy_meta_without_a_key() {
        let dir = tempfile::tempdir().expect("临时目录");
        let source = dir.path().join("r.0.0.mca");
        let png = dir.path().join("r.0.0.png");
        let meta = dir.path().join("r.0.0.json");

        fs::write(&source, b"mca").expect("写源文件");
        fs::write(&png, b"png-bytes").expect("写缓存");
        fs::write(&meta, serde_json::to_vec(&sample_info()).expect("序列化摘要")).expect("写摘要");

        assert!(read_cache(&png, &meta, &source, "k").is_none());
    }

    #[test]
    fn level_name_rejects_path_traversal() {
        let dir = tempfile::tempdir().expect("临时目录");
        for value in ["../escape", "a/b", "a\\b", "..", ".", ""] {
            fs::write(dir.path().join("server.properties"), format!("level-name={value}\n"))
                .expect("写入");
            assert_eq!(read_level_name(dir.path()).expect("读世界名"), "world", "value={value}");
        }
    }

    /// Windows 上 `D:..` 是个没斜杠、也不是 `..` 的名字，但它是「D 盘的当前
    /// 目录的上一级」——`join` 上去会换盘并跳出服务器目录。
    #[cfg(windows)]
    #[test]
    fn level_name_rejects_drive_relative_names() {
        let dir = tempfile::tempdir().expect("临时目录");
        for value in ["D:..", "C:\\Windows", "D:", "\\\\server\\share"] {
            fs::write(dir.path().join("server.properties"), format!("level-name={value}\n"))
                .expect("写入");
            assert_eq!(read_level_name(dir.path()).expect("读世界名"), "world", "value={value}");
        }
    }

    #[tokio::test]
    async fn gate_is_shared_per_region_and_level() {
        let dir = tempfile::tempdir().expect("临时目录");
        let (instance, server) = test_services(dir.path()).await;
        let service =
            CoreWorldMapService::with_cache_root(instance, server, dir.path().join("cache"));

        let same_a = service.gate("i", "world", Dimension::Overworld, 0, 0);
        let same_b = service.gate("i", "world", Dimension::Overworld, 0, 0);
        let other_region = service.gate("i", "world", Dimension::Overworld, 0, 1);
        let other_dimension = service.gate("i", "world", Dimension::Nether, 0, 0);

        assert!(Arc::ptr_eq(&same_a, &same_b), "同一 region 必须复用同一把锁");
        assert!(!Arc::ptr_eq(&same_a, &other_region), "不同 region 不应互相阻塞");
        assert!(!Arc::ptr_eq(&same_a, &other_dimension), "不同维度不应互相阻塞");
    }

    /// 节流必须真的省掉那次 RCON 往返，而不只是不更新时间戳。
    ///
    /// 起一个只数连接数的假 RCON 服务端：连上就断开，让 `save-all flush`
    /// 立刻失败（走 [`FLUSH_RETRY`] 短窗口）。窗口内再请求一次，连接数不能涨 ——
    /// 光比对 `last_flush` 的话，一个「节流了但还是发了命令」的回归能照样通过。
    #[tokio::test]
    async fn flush_is_throttled_per_instance() {
        let dir = tempfile::tempdir().expect("临时目录");
        let (instance, server) = test_services(dir.path()).await;
        let service =
            CoreWorldMapService::with_cache_root(instance, server, dir.path().join("cache"));

        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("绑定假 RCON 端口");
        let port = listener.local_addr().expect("端口").port();
        let attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = Arc::clone(&attempts);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if stream.is_err() {
                    break;
                }
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                // 直接关掉：客户端握手读到 EOF，按连接失败处理
            }
        });
        fs::write(
            dir.path().join("server.properties"),
            format!("enable-rcon=true\nrcon.password=p\nrcon.port={port}\n"),
        )
        .expect("写配置");

        service.flush_world_if_due(dir.path(), "a").await;
        let first = *service
            .last_flush
            .lock()
            .expect("锁")
            .get("a")
            .expect("应记录时间");
        assert_eq!(
            attempts.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "第一次应该真的去连 RCON"
        );

        // 节流窗口内再来一次：不能重连，也不能重新计时间
        service.flush_world_if_due(dir.path(), "a").await;
        let second = *service
            .last_flush
            .lock()
            .expect("锁")
            .get("a")
            .expect("应记录时间");
        assert_eq!(first, second, "节流窗口内不应重置落盘时间");
        assert_eq!(
            attempts.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "节流窗口内不该再发一次 save-all"
        );

        // 另一个实例有自己的计时，不该被 a 的窗口挡掉
        service.flush_world_if_due(dir.path(), "b").await;
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 2, "换实例要重新落盘");
        assert!(service.last_flush.lock().expect("锁").contains_key("b"));
    }

    #[test]
    fn locate_region_dir_supports_universe_layout() {
        // `--universe worlds` 启动时世界在 <目录>/worlds/<level>/region
        let dir = tempfile::tempdir().expect("临时目录");
        fs::write(dir.path().join("server.properties"), "level-name=world\n").expect("写入");
        fs::create_dir_all(dir.path().join("worlds/world/region")).expect("建目录");

        let level = read_level_name(dir.path()).expect("读世界名");
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

        // 两边都读不到 level.dat 时间时按默认布局兜底
        assert_eq!(
            locate_region_dir(dir.path(), "world", Dimension::Overworld),
            Some(dir.path().join("world/region"))
        );
    }

    /// 换过启动方式之后，废弃的老世界目录通常还躺在原地。
    ///
    /// 谁在活动不能看目录名，得看谁还在被写：`level.dat` 新的那份才是活的。
    #[test]
    fn locate_region_dir_follows_the_freshly_written_world() {
        let dir = tempfile::tempdir().expect("临时目录");
        fs::create_dir_all(dir.path().join("world/region")).expect("建旧目录");
        fs::create_dir_all(dir.path().join("worlds/world/region")).expect("建新目录");
        fs::write(dir.path().join("world/level.dat"), b"old").expect("写旧 level.dat");
        fs::write(dir.path().join("worlds/world/level.dat"), b"new").expect("写新 level.dat");

        let old = SystemTime::now() - std::time::Duration::from_secs(86_400);
        fs::OpenOptions::new()
            .write(true)
            .open(dir.path().join("world/level.dat"))
            .expect("打开")
            .set_modified(old)
            .expect("调整时间");

        assert_eq!(
            locate_region_dir(dir.path(), "world", Dimension::Overworld),
            Some(dir.path().join("worlds/world/region")),
            "活动世界在 worlds/ 下，不能画老目录的地形"
        );
    }

    /// 名字恰好以 `_nether` 结尾的**普通主世界**，不能把自己的地形当下界地图。
    #[test]
    fn locate_region_dir_ignores_dimension_suffix_without_a_base_world() {
        let dir = tempfile::tempdir().expect("临时目录");
        fs::create_dir_all(dir.path().join("foo_nether/region")).expect("建目录");

        assert_eq!(
            locate_region_dir(dir.path(), "foo_nether", Dimension::Nether),
            None,
            "没有同名主世界作旁证时，不能把 root/region 当下界"
        );
        assert_eq!(
            locate_region_dir(dir.path(), "foo_nether", Dimension::Overworld),
            Some(dir.path().join("foo_nether/region"))
        );
    }

    /// `level-name` 直接指向拆分布局里的维度世界时，`root/region` 就是该维度。
    #[test]
    fn locate_region_dir_accepts_dimension_world_with_a_base_sibling() {
        let dir = tempfile::tempdir().expect("临时目录");
        fs::create_dir_all(dir.path().join("world/region")).expect("建主世界");
        fs::create_dir_all(dir.path().join("world_nether/region")).expect("建下界");

        assert_eq!(
            locate_region_dir(dir.path(), "world_nether", Dimension::Nether),
            Some(dir.path().join("world_nether/region"))
        );
        // 但同一个目录对主世界来说仍是主世界自己
        assert_eq!(
            locate_region_dir(dir.path(), "world", Dimension::Overworld),
            Some(dir.path().join("world/region"))
        );
    }

    /// 有原版 `DIM-1` 时以它为准，名字后缀不参与判断。
    #[test]
    fn locate_region_dir_prefers_vanilla_subdir_over_dimension_suffix() {
        let dir = tempfile::tempdir().expect("临时目录");
        fs::create_dir_all(dir.path().join("foo_nether/region")).expect("建目录");
        fs::create_dir_all(dir.path().join("foo_nether/DIM-1/region")).expect("建目录");

        assert_eq!(
            locate_region_dir(dir.path(), "foo_nether", Dimension::Nether),
            Some(dir.path().join("foo_nether/DIM-1/region"))
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
