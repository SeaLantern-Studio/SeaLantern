//! 应用层服务端口。
//!
//! 这里定义面向业务能力的异步 trait；端口使用的 DTO 和错误由
//! `sealantern-contract` 提供，具体实现由应用层装配 `feature` 能力完成。

pub mod backup;
pub mod catalog;
pub mod console;
pub mod cron;
pub mod download;
pub mod instance;
pub mod java;
pub mod online;
pub mod provisioning;
pub mod server;
pub mod server_config;
pub mod settings;
pub mod system;
pub mod update;

pub use backup::BackupService;
pub use catalog::ServerCatalogService;
pub use console::ConsoleService;
pub use cron::CronTaskService;
pub use download::DownloadService;
pub use instance::InstanceService;
pub use java::JavaService;
pub use online::OnlineTunnelService;
pub use provisioning::ProvisioningService;
pub use server::ServerService;
pub use server_config::ServerConfigService;
pub use settings::SettingsService;
pub use system::SystemService;
pub use update::{UpdateCheckService, UpdateInstallService};
