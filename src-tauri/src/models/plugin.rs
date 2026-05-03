//! 插件模型入口
//!
//! 这里保留对外公开的插件类型，具体结构按职责拆到 `plugin_model/` 目录下

#[path = "plugin_model/dependencies.rs"]
mod dependencies;
#[path = "plugin_model/market.rs"]
mod market;
#[path = "plugin_model/manifest.rs"]
mod manifest;
#[path = "plugin_model/runtime.rs"]
mod runtime;

#[allow(unused_imports)] // 对外类型导出
/// 依赖和版本相关的公开类型
pub use dependencies::{PluginDependency, SemVer};

#[allow(unused_imports)] // 对外类型导出
/// 插件市场相关的公开类型
pub use market::{MarketAuthorInfo, MarketPluginInfo, PluginUpdateInfo};

#[allow(unused_imports)] // 对外类型导出
/// 插件清单相关的公开类型
pub use manifest::{
    PluginAuthor, PluginCommand, PluginContextMenu, PluginEngines, PluginLocaleEntry,
    PluginManifest, PluginPage, PluginPermission, PluginSettingField, PluginSettingOption,
    PluginSidebarConfig, PluginUiConfig, SidebarCategoryConfig, SidebarMode,
};

#[allow(unused_imports)] // 对外类型导出
/// 插件运行状态和安装结果相关的公开类型
pub use runtime::{
    BatchInstallError, BatchInstallResult, MissingDependency, PluginInfo, PluginInstallResult,
    PluginState,
};
