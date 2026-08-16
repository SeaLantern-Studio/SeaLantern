//! `sealantern-interface` 接口契约 crate。
//!
//! 定义宿主侧能力端口（trait）及其相关模型与契约错误，供 `server` 实现与上层复用，
//! 不依赖任何具体 RPC 传输。

#![forbid(unsafe_code)]

/// 服务器核心下载目录相关模型与服务端口。
pub mod catalog;
/// 服务器控制台日志相关模型与服务端口。
pub mod console;
/// 服务器定时任务相关模型与服务端口。
pub mod cron;
/// 下载任务管理相关模型与服务端口。
pub mod download;
/// 接口契约错误类型。
pub mod error;
/// 服务器实例记录相关模型与服务端口。
pub mod instance;
/// Java 检测与校验相关模型与服务端口。
pub mod java;
/// 在线隧道相关模型与服务端口。
pub mod online;
/// 服务端检查与实例供给计划相关服务端口。
pub mod provisioning;
/// 服务器进程管理相关模型与服务端口。
pub mod server;
/// 设置信息相关模型与服务端口。
pub mod settings;
/// 系统资源信息相关模型与服务端口。
pub mod system;
/// 应用更新检查相关模型与服务端口。
pub mod update;

/// 服务器核心下载目录服务端口。
pub use catalog::ServerCatalogService;
/// 服务器控制台日志服务端口。
pub use console::ConsoleService;
/// 服务器定时任务服务端口。
pub use cron::CronTaskService;
/// 下载任务管理服务端口。
pub use download::DownloadService;
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
/// 服务器实例记录管理服务端口。
pub use instance::InstanceService;
/// Java 检测与校验服务端口。
pub use java::JavaService;
/// 在线隧道模型与服务端口。
pub use online::{
    OnlineTunnelConnection, OnlineTunnelEvent, OnlineTunnelHostRequest, OnlineTunnelJoinRequest,
    OnlineTunnelMode, OnlineTunnelService, OnlineTunnelStatus,
};
/// 服务端检查与实例供给计划服务端口。
pub use provisioning::ProvisioningService;
/// 服务器进程管理服务端口。
pub use server::ServerService;
/// 设置信息服务端口。
pub use settings::SettingsService;
/// 系统资源信息服务端口。
pub use system::SystemService;
/// 应用更新检查服务端口。
pub use update::UpdateCheckService;
/// 应用更新安装服务端口。
pub use update::UpdateInstallService;
