//! `extra` 内跨模块复用的稳定数据模型。
//!
//! 领域服务、错误和仅供实现使用的传输结构继续由各自模块维护。

mod app;
mod app_update;
mod download_link;
mod java;
mod server;
mod task;

pub use app::{AppSettings, SettingsGroup, CURRENT_CONFIG_VERSION};
pub use app_update::{PartialAppSettings, UpdateResult};
pub use download_link::{BaseDownloadLinks, DownloadLink, TypeDownloadLinks};
pub use java::JavaInfo;
pub use server::InstanceList;
pub use task::{TaskProgressResponse, TaskStatus};
