//! 应用层服务实现模块。
//!
//! 存放各类宿主能力的默认实现（如 [`CoreInstanceService`]、[`CoreSystemService`]、
//! [`CoreServerService`]、[`CoreDownloadService`]、[`CoreCronTaskService`]），实现
//! `interface` 的能力端口，由 `services` 装配层组装进全局容器。

mod cron;
mod download;
mod instance;
mod network_settings;
mod provisioning;
mod proxy_monitoring;
mod server;
mod settings;
mod system;
mod update;

pub use cron::CoreCronTaskService;
pub use download::CoreDownloadService;
pub use instance::CoreInstanceService;
pub use provisioning::CoreProvisioningService;
pub use proxy_monitoring::ProxyMonitoringService;
pub use server::CoreServerService;
pub use settings::CoreSettingsService;
pub use system::CoreSystemService;
pub use update::CoreUpdateCheckService;
