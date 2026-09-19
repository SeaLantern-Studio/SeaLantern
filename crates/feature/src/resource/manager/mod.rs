//! 实例资源管理器：管理服务器实例中的模组 / 插件 / 数据包。
//!
//! # 职责
//!
//! - 扫描实例 `mods/`、`plugins/` 等资源目录（[`scan`]）；
//! - 增删资源文件并维护清单（[`install`]、[`remove`]）；
//! - 启用 / 禁用资源（[`set_enabled`]）；
//! - 清单持久化与对账（[`manifest`]、[`reconcile`]、[`sync`]）。
//!
//! # 模块分层
//!
//! - 纯逻辑（零 IO、可独立单测）：[`layout`]、[`naming`]、[`path`]、[`reconcile`]；
//! - 文件系统交互：[`scan`]、[`manifest`]、[`operations`]。
//!
//! 资源清单位于实例根目录的 `.sealantern/resources.json`，随实例目录
//! 一起复制与备份。

mod error;
mod layout;
mod manifest;
mod models;
mod naming;
mod operations;
mod path;
mod reconcile;
mod scan;

pub use error::ResourceManagerError;
pub use layout::{LayoutError, ResourceTarget, ResourceTargets, datapack_target, resource_targets};
pub use models::{
    InstanceExtension, InstanceExtensionError, InstanceExtensionKind, MANIFEST_SCHEMA_VERSION,
    ManagedResource, ReconcileReport, ReconciledResource, ResourceManifest, ResourceProvenance,
    ResourceSource, ResourceState,
};
pub use operations::{install, list, remove, set_enabled, sync};
pub use path::is_single_normal_component;
