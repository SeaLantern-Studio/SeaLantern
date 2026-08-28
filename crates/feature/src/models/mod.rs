//! `feature` 内跨模块复用的稳定数据模型。
//!
//! 领域服务、错误和仅供实现使用的传输结构继续由各自模块维护。

pub(crate) mod compatibility;
mod download_link;
mod server;
mod task;

pub use download_link::{BaseDownloadLinks, DownloadLink, TypeDownloadLinks};
pub use sealantern_contract::java::JavaInfo;
pub use sealantern_contract::settings::{
    AppSettings, CURRENT_CONFIG_VERSION, DEFAULT_ACRYLIC_BLUR_LEVEL, SettingsGroup,
    SettingsValidationError,
};
pub use sealantern_contract::settings::{NullablePatch, PartialAppSettings, UpdateResult};
pub use server::InstanceList;
pub(crate) use server::LegacyServerInstance;
pub use task::{TaskProgressResponse, TaskStatus};

#[allow(deprecated)]
pub use compatibility::ServerInstance;

#[deprecated(note = "请使用 crate::download_link::LinkManager")]
pub use crate::download_link::LinkManager;
