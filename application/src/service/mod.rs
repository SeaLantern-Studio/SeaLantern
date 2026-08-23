//! 应用层服务实现模块。
//!
//! 存放各类宿主能力的默认实现（如 [`CoreInstanceService`]、[`CoreSystemService`]、
//! [`CoreServerService`]、[`CoreDownloadService`]、[`CoreCronTaskService`]、
//! [`CoreJavaService`]、[`CoreServerCatalogService`]、[`CoreProvisioningService`]、
//! [`CoreOnlineTunnelService`]、[`CoreUpdateInstallService`]），实现
//! `interface` 的能力端口，由 `services` 装配层组装进全局容器。

mod catalog;
mod command_capture;
mod console;
mod cron;
mod download;
mod instance;
mod java;
mod log_recorder;
mod network_settings;
mod online_tunnel;
mod player;
mod provisioning;
mod proxy_monitoring;
mod server;
mod settings;
mod system;
mod update;
mod update_install;

pub use catalog::CoreServerCatalogService;
pub use command_capture::{CaptureError, capture_command_output};
pub use console::CoreConsoleService;
pub use cron::CoreCronTaskService;
pub use download::CoreDownloadService;
pub use instance::CoreInstanceService;
pub use java::CoreJavaService;
pub use log_recorder::{LogEvent, LogRecorder, subscribe_log_events};
pub use online_tunnel::CoreOnlineTunnelService;
pub use player::CorePlayerService;
pub use provisioning::CoreProvisioningService;
pub use proxy_monitoring::ProxyMonitoringService;
pub use server::CoreServerService;
pub use settings::CoreSettingsService;
pub use system::CoreSystemService;
pub use update::CoreUpdateCheckService;
pub use update_install::CoreUpdateInstallService;
