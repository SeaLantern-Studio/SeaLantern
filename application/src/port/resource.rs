//! 资源管理服务端口。

use async_trait::async_trait;

use sealantern_contract::ResourceServiceError;
use sealantern_feature::resource::manager::{
    InstanceExtension, ManagedResource, ReconcileReport, ResourceProvenance, ResourceTargets,
};
use sealantern_feature::resource::market::{
    MarketSource, ResourceInfo, ResourceType, SearchResult, Version,
};

/// 从市场安装所需的下载解析结果。
///
/// 前端拿到本结果后**自行驱动下载**（复用 TaskPill 的任务球机制），
/// 下载完成后把 `save_path` 连同 [`ResourceProvenance`] 交给
/// [`ResourceService::install`] 完成落盘。
///
/// # `save_path` 语义与清理责任
///
/// `save_path` 指向应用数据根目录 `market-tmp/<项目>-<版本>/` 下的唯一
/// 子目录路径，**文件名保持市场原名**（唯一化只作用于目录，因此安装进
/// 实例后 `mods/` 里就是 `sodium.jar`）。安装成功后由服务端清理该临时
/// 子目录；安装失败时保留，便于用户重试。前端无需自行清理，也不要改动路径。
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResolvedDownload {
    /// 资源文件的直接下载链接。
    pub url: String,
    /// 建议保存的文件名。
    pub filename: String,
    /// 后端建议的临时保存路径（下载完成后该文件被安装进实例）。
    pub save_path: String,
    /// 资源类型（决定安装到 `mods/` 还是 `plugins/`）。
    pub kind: ResourceType,
    /// 安装时携带的来源溯源。
    pub provenance: ResourceProvenance,
}

/// 资源管理宿主能力端口。
///
/// 覆盖实例资源管理（目标查询 / 扫描 / 增删 / 启停 / 对账）与市场查询
/// （搜索 / 详情 / 版本 / 下载解析）。下载动作由前端驱动，后端只做元数据
/// 解析与落盘。
///
/// # 宿主范围
///
/// 当前**仅 Tauri 宿主消费**（`src-tauri` 已注册全部资源命令）；`server`
/// crate 尚无对应 handler，契约错误 [`ResourceServiceError`] 目前只在该宿主
/// 暴露。将来 server 侧接入时可复用同一端口。
#[async_trait]
pub trait ResourceService: Send + Sync {
    /// 列出实例资源：扫描目录并对账（只读）。
    async fn list(&self, instance_id: &str) -> Result<ReconcileReport, ResourceServiceError>;

    /// 列出实例可管理的资源目录与主要种类（只读）。
    ///
    /// 资源列表为空时无法从条目反推该实例支持什么，前端需要本查询来决定
    /// 展示哪些分类。实例完全没有资源目录时返回
    /// [`ResourceServiceError::NoResourceDirs`]。
    async fn targets(&self, instance_id: &str) -> Result<ResourceTargets, ResourceServiceError>;

    /// 安装本地文件到实例（`source_path` → `mods/` 或 `plugins/`）。
    ///
    /// `provenance` 为**前端声明**、服务端不做来源验证：市场流程中该值由
    /// [`market_resolve_download`](Self::market_resolve_download) 生成后原样
    /// 回传，本地导入则为 `None`。它仅用于展示与溯源，不应作为可信依据。
    async fn install(
        &self,
        instance_id: &str,
        source_path: &str,
        kind: ResourceType,
        provenance: Option<ResourceProvenance>,
    ) -> Result<ManagedResource, ResourceServiceError>;
    /// 卸载实例资源（文件缺失时也清理账目）。
    ///
    /// `kind` 与 `file_name` 共同定位账目：`mods/` 与 `plugins/` 下的同名文件
    /// 是两条独立账目。
    async fn remove(
        &self,
        instance_id: &str,
        kind: ResourceType,
        file_name: &str,
    ) -> Result<(), ResourceServiceError>;

    /// 启用 / 禁用实例资源（`kind` 与 `file_name` 共同定位账目）。
    async fn set_enabled(
        &self,
        instance_id: &str,
        kind: ResourceType,
        file_name: &str,
        enabled: bool,
    ) -> Result<InstanceExtension, ResourceServiceError>;

    /// 对账并回写实例清单。
    async fn sync(&self, instance_id: &str) -> Result<ReconcileReport, ResourceServiceError>;

    /// 在市场中搜索资源。
    async fn market_search(
        &self,
        source: MarketSource,
        query: &str,
        page: u32,
        page_size: u32,
    ) -> Result<SearchResult, ResourceServiceError>;

    /// 获取市场资源详情。
    async fn market_resource(
        &self,
        source: MarketSource,
        id: &str,
    ) -> Result<ResourceInfo, ResourceServiceError>;

    /// 获取市场资源的版本列表。
    async fn market_versions(
        &self,
        source: MarketSource,
        id: &str,
    ) -> Result<Vec<Version>, ResourceServiceError>;

    /// 解析"从市场安装"的下载目标（不实际下载，由前端驱动下载）。
    async fn market_resolve_download(
        &self,
        source: MarketSource,
        project_id: &str,
        version_id: &str,
    ) -> Result<ResolvedDownload, ResourceServiceError>;
}
