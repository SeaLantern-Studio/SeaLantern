//! 世界地图渲染服务端口。

use async_trait::async_trait;
use sealantern_contract::{
    Dimension, WorldMapImage, WorldMapInfo, WorldMapRegionList, WorldMapRequest,
    WorldMapServiceError,
};

/// 把服务器世界的一个 region 渲染成俯视 PNG 的宿主能力端口。
///
/// 实现方负责按实例定位世界目录、调度 region 解析与图片渲染，并维护磁盘缓存；
/// 传输层只消费本契约，不关心 fastanvil、PNG 编码或缓存布局。
#[async_trait]
pub trait WorldMapService: Send + Sync {
    /// 渲染指定 region；缓存仍然有效时直接返回缓存内容。
    ///
    /// `info.from_cache` 表示本次是否命中缓存。同一 region 的并发请求由实现方
    /// 串行化，调用方不需要自己去重。
    async fn render(&self, request: WorldMapRequest)
    -> Result<WorldMapImage, WorldMapServiceError>;

    /// 只取渲染摘要，不返回图片字节；缓存未命中时会触发生成。
    async fn info(&self, request: WorldMapRequest) -> Result<WorldMapInfo, WorldMapServiceError>;

    /// 列出世界里全部可渲染的 region 文件。
    ///
    /// 只做目录扫描，不解析 region 内容，因此可以随时调用。
    async fn regions(
        &self,
        instance_id: &str,
        dimension: Dimension,
    ) -> Result<WorldMapRegionList, WorldMapServiceError>;
}
