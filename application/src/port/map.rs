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
    /// 除目录扫描外，还有一项**会改服务端状态**的副作用：按节流（约 30 秒一次）
    /// 让运行中的服务器执行 `save-all flush`，把内存里的区块刷进 `.mca`。
    /// 清单里的 `modified_at_unix_millis` 是前端判断「哪块地形变了」的唯一依据，
    /// 不先落盘就永远不会变。
    ///
    /// 因此调用方不要以远高于节流窗口的频率轮询；只想看某一块图时用
    /// [`render`](Self::render)，那条路径的落盘节流是一样的，但至少会带回图片。
    async fn regions(
        &self,
        instance_id: &str,
        dimension: Dimension,
    ) -> Result<WorldMapRegionList, WorldMapServiceError>;
}
