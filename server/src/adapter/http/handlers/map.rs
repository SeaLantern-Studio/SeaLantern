//! 世界地图 REST handler。
//!
//! 单块 region 暴露三个只读资源：
//! - `GET /api/instances/{id}/map` 直接返回 `image/png`；
//! - `GET /api/instances/{id}/map/info` 返回渲染摘要 JSON；
//! - `GET /api/instances/{id}/map/regions` 返回世界里的 region 清单。
//!
//! 另有一个实时资源：
//! - `GET /api/instances/{id}/map/players` 返回某维度在线玩家的坐标。
//!
//! 渲染、世界目录定位与磁盘缓存全部由应用层服务负责，这里只做参数解析和
//! 响应组装。三个资源共用同一份缓存，因此同时请求不会重复渲染。

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderName, HeaderValue, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use sealantern_application::port::{PlayerPositionService, WorldMapService};
use sealantern_contract::{
    Dimension, PlayerPositions, WorldMapInfo, WorldMapRegionList, WorldMapRequest,
};
use sealantern_core::instance::InstanceId;

use super::super::error::HttpError;
use super::super::state::AppState;

/// 渲染摘要是否来自缓存的响应头。
const HEADER_FROM_CACHE: HeaderName = HeaderName::from_static("x-map-from-cache");

/// 地图相关资源的查询参数。
#[derive(Debug, Default, Deserialize)]
pub struct MapQuery {
    /// region 的 X 坐标（缺省 0）。
    pub region_x: Option<isize>,
    /// region 的 Z 坐标（缺省 0）。
    pub region_z: Option<isize>,
    /// 目标维度（缺省主世界）。
    pub dimension: Option<Dimension>,
}

/// 校验路径参数里的实例 ID，非法输入视为客户端错误。
fn validate_id(raw: &str) -> Result<(), HttpError> {
    InstanceId::new(raw.to_owned())
        .map(|_| ())
        .map_err(|_| HttpError::bad_request("invalid_instance_id", "invalid instance id"))
}

/// 把路径参数与查询参数拼成契约请求。
fn to_request(instance_id: String, query: &MapQuery) -> WorldMapRequest {
    WorldMapRequest {
        instance_id,
        region_x: query.region_x.unwrap_or(0),
        region_z: query.region_z.unwrap_or(0),
        dimension: query.dimension.unwrap_or_default(),
    }
}

/// `GET /api/instances/{id}/map?region_x=&region_z=&dimension=` — 返回 region 的俯视 PNG。
pub async fn world_map_image(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MapQuery>,
) -> Result<Response, HttpError> {
    validate_id(&id)?;
    let image = state.world_map().render(to_request(id, &query)).await?;
    let cached = if image.info.from_cache {
        "true"
    } else {
        "false"
    };

    Ok((
        [
            (header::CONTENT_TYPE, HeaderValue::from_static("image/png")),
            // 新鲜度由服务端按 .mca 修改时间判断，浏览器不需要再缓存一层
            (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
            (HEADER_FROM_CACHE, HeaderValue::from_static(cached)),
        ],
        image.png,
    )
        .into_response())
}

/// `GET /api/instances/{id}/map/info?region_x=&region_z=&dimension=` — 返回渲染摘要。
///
/// 缓存未命中时会触发一次完整渲染，不会拿到「有摘要但没有图片」的中间态。
pub async fn world_map_info(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MapQuery>,
) -> Result<Json<WorldMapInfo>, HttpError> {
    validate_id(&id)?;
    state
        .world_map()
        .info(to_request(id, &query))
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `GET /api/instances/{id}/map/regions?dimension=` — 列出世界里全部可渲染的 region。
///
/// 只做目录扫描，不会触发渲染，可以放心在打开地图页时先调。
pub async fn world_map_regions(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MapQuery>,
) -> Result<Json<WorldMapRegionList>, HttpError> {
    validate_id(&id)?;
    state
        .world_map()
        .regions(&id, query.dimension.unwrap_or_default())
        .await
        .map(Json)
        .map_err(HttpError::from)
}

/// `GET /api/instances/{id}/map/players?dimension=` — 返回该维度在线玩家坐标。
///
/// 通过 RCON 实时查询，服务器未运行或未启用 RCON 时返回 4xx/5xx 而不是空列表，
/// 便于前端区分「没人」和「问不到」。
pub async fn world_map_players(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<MapQuery>,
) -> Result<Json<PlayerPositions>, HttpError> {
    validate_id(&id)?;
    state
        .player_position()
        .positions(&id, query.dimension.unwrap_or_default())
        .await
        .map(Json)
        .map_err(HttpError::from)
}
