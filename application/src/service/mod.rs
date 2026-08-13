//! 应用层服务实现模块。
//!
//! 存放各类宿主能力的默认实现（如 [`CoreInstanceService`]、[`CoreSystemService`]、
//! [`CoreServerService`]、[`CoreDownloadService`]、[`CoreCronTaskService`]、
//! [`CoreJavaService`]、[`CoreServerCatalogService`]、[`CoreProvisioningService`]、
//! [`CoreOnlineTunnelService`]、[`CoreUpdateInstallService`]），实现
//! `interface` 的能力端口，由 `services` 装配层组装进全局容器。

mod catalog;
mod console;
mod cron;
mod download;
mod instance;
mod java;
mod network_settings;
mod online_tunnel;
mod provisioning;
mod proxy_monitoring;
mod server;
mod server_log;
mod settings;
mod system;
mod update;
mod update_install;

pub use catalog::CoreServerCatalogService;
pub use console::CoreConsoleService;
pub use cron::CoreCronTaskService;
pub use download::CoreDownloadService;
pub use instance::CoreInstanceService;
pub use java::CoreJavaService;
pub use online_tunnel::CoreOnlineTunnelService;
pub use provisioning::CoreProvisioningService;
pub use proxy_monitoring::ProxyMonitoringService;
pub use server::CoreServerService;
pub use server_log::{open_log_database, read_logs, LogLine, LogSource, LOG_DATABASE_FILE};
pub use settings::CoreSettingsService;
pub use system::CoreSystemService;
pub use update::CoreUpdateCheckService;
pub use update_install::CoreUpdateInstallService;
