//! `sealantern-contract` 公共数据契约 crate。
//!
//! 定义跨宿主共享的 DTO、可序列化错误和配置模型。
//! 服务能力端口位于 `sealantern-application::port`，具体实现位于
//! `sealantern-feature` / `sealantern-application`，因此本 crate 不依赖任何项目实现层。

#![forbid(unsafe_code)]

/// 服务器核心下载目录相关模型。
pub mod catalog;
/// 服务器控制台日志相关模型。
pub mod console;
/// 服务器定时任务相关模型。
pub mod cron;
/// 下载任务管理相关模型。
pub mod download;
/// 接口契约错误类型。
pub mod error;
/// Java 检测结果和安装信息模型。
pub mod java;
/// 在线隧道相关模型。
pub mod online;
/// 网络代理配置模型。
pub mod proxy;
/// 服务器进程管理相关模型。
pub mod server;
/// 设置信息相关模型。
pub mod settings;
/// 系统资源信息相关模型。
pub mod system;
/// 应用更新检查相关模型。
pub mod update;

/// 服务器核心下载链接模型。
pub use catalog::DownloadLink;
/// 服务器控制台日志行模型。
pub use console::ConsoleLogLine;
/// 服务器控制台日志错误枚举。
pub use error::ConsoleServiceError;
/// 服务器定时任务错误枚举。
pub use error::CronTaskServiceError;
/// 下载任务管理错误枚举。
pub use error::DownloadServiceError;
/// 服务器实例管理错误枚举。
pub use error::InstanceServiceError;
/// Java 检测与校验错误枚举。
pub use error::JavaServiceError;
/// 在线隧道服务错误枚举。
pub use error::OnlineTunnelServiceError;
/// 服务端检查与实例供给计划失败类别。
pub use error::ProvisioningServiceError;
/// 服务器核心下载目录错误枚举。
pub use error::ServerCatalogServiceError;
/// 服务器进程管理错误枚举。
pub use error::ServerServiceError;
/// 设置信息服务错误枚举。
pub use error::SettingsServiceError;
/// 系统资源信息服务错误枚举。
pub use error::SystemServiceError;
/// 应用更新检查错误枚举。
pub use error::UpdateCheckServiceError;
/// 应用更新安装错误枚举。
pub use error::UpdateInstallServiceError;
/// Java 检测结果和安装信息模型。
pub use java::{JavaDetectionReport, JavaDiscoveryError, JavaInfo};
/// 在线隧道模型。
pub use online::{
    OnlineTunnelConnection, OnlineTunnelEvent, OnlineTunnelHostRequest, OnlineTunnelJoinRequest,
    OnlineTunnelMode, OnlineTunnelStatus,
};
/// 网络代理配置模型。
pub use proxy::{ProxyConfigError, ProxyMode, ProxySettings};
/// 设置模型。
pub use settings::{
    AppSettings, CURRENT_CONFIG_VERSION, DEFAULT_ACRYLIC_BLUR_LEVEL, NullablePatch,
    PartialAppSettings, SettingsEntry, SettingsEntryType, SettingsGroup, SettingsGroupInfo,
    SettingsOption, SettingsOverview, SettingsValidationError, UpdateResult,
};
/// 更新模型。
pub use update::{PendingUpdate, UpdateInfo, UpdateSource};
