//! 世界地图 Tauri 命令。
//!
//! 桌面端没有 HTTP 服务，因此图片以 IPC 原始字节返回：前端拿到的是
//! `ArrayBuffer`，自行转成 `Blob` + object URL 即可，不走 JSON 数组
//! （512×512 的 PNG 编成 JSON 会膨胀数倍）。
//!
//! 渲染与缓存都由应用层服务负责，命令只做参数整形。

use std::sync::Arc;

use sealantern_application::port::{PlayerPositionService, WorldMapService};
use sealantern_application::service::{CorePlayerPositionService, CoreWorldMapService};
use sealantern_application::services::AppServices;
use sealantern_contract::{
    Dimension, PlayerPositionServiceError, PlayerPositions, WorldMapInfo, WorldMapRegionList,
    WorldMapRequest, WorldMapServiceError,
};
use tauri::State;
use tauri::ipc::Response;

fn world_map_service(services: &AppServices) -> Arc<CoreWorldMapService> {
    services.world_map().clone()
}

fn player_position_service(services: &AppServices) -> Arc<CorePlayerPositionService> {
    services.player_position().clone()
}

/// 组装契约请求，区域坐标缺省为 0、维度缺省主世界。
fn request(
    instance_id: String,
    region_x: Option<isize>,
    region_z: Option<isize>,
    dimension: Option<Dimension>,
) -> WorldMapRequest {
    WorldMapRequest {
        instance_id,
        region_x: region_x.unwrap_or(0),
        region_z: region_z.unwrap_or(0),
        dimension: dimension.unwrap_or_default(),
    }
}

/// 渲染指定 region，返回 PNG 原始字节（前端按 `ArrayBuffer` 接收）。
#[tauri::command(rename_all = "snake_case")]
pub async fn render_world_map(
    services: State<'_, AppServices>,
    instance_id: String,
    region_x: Option<isize>,
    region_z: Option<isize>,
    dimension: Option<Dimension>,
) -> Result<Response, WorldMapServiceError> {
    let image = world_map_service(&services)
        .render(request(instance_id, region_x, region_z, dimension))
        .await?;
    Ok(Response::new(image.png))
}

/// 只取渲染摘要；缓存未命中时会触发一次完整渲染。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_world_map_info(
    services: State<'_, AppServices>,
    instance_id: String,
    region_x: Option<isize>,
    region_z: Option<isize>,
    dimension: Option<Dimension>,
) -> Result<WorldMapInfo, WorldMapServiceError> {
    world_map_service(&services)
        .info(request(instance_id, region_x, region_z, dimension))
        .await
}

/// 列出实例某个维度里全部可渲染 region（只扫目录，不解析内容）。
#[tauri::command(rename_all = "snake_case")]
pub async fn list_world_map_regions(
    services: State<'_, AppServices>,
    instance_id: String,
    dimension: Option<Dimension>,
) -> Result<WorldMapRegionList, WorldMapServiceError> {
    world_map_service(&services)
        .regions(&instance_id, dimension.unwrap_or_default())
        .await
}

/// 查询某维度在线玩家的实时坐标（走 RCON）。
#[tauri::command(rename_all = "snake_case")]
pub async fn get_map_player_positions(
    services: State<'_, AppServices>,
    instance_id: String,
    dimension: Option<Dimension>,
) -> Result<PlayerPositions, PlayerPositionServiceError> {
    player_position_service(&services)
        .positions(&instance_id, dimension.unwrap_or_default())
        .await
}

/// 切换世界地图的贴图配色来源。
///
/// `path` 传资源包 zip 或解包后的目录，传 `None` 恢复内置配色。返回实际覆盖的
/// 贴图张数 —— 为 0 说明这个包不改方块贴图（只动模型、音效、GUI 的包就属于
/// 这种），地图不会有什么变化。
///
/// 读包和解几百张 PNG 是重活，扔到阻塞线程池，别占异步运行时。
#[tauri::command(rename_all = "snake_case")]
pub async fn set_world_map_resource_pack(
    services: State<'_, AppServices>,
    path: Option<String>,
) -> Result<usize, String> {
    let service = world_map_service(&services);
    tokio::task::spawn_blocking(move || {
        service.set_resource_pack(path.as_deref().map(std::path::Path::new))
    })
    .await
    .map_err(|error| format!("切换贴图配色失败: {error}"))?
    .map_err(|error| error.to_string())
}
