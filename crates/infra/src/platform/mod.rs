//! 跨平台的系统级基础设施。
//!
//! 此模块只封装可移植的系统交互基础组件。应用策略、配置持久化和 UI 授权流程
//! 由上层负责；尤其是 CA 证书仅供 HTTP 客户端使用，不会修改操作系统信任库。

mod certificate;
mod elevation;
mod environment;
mod error;
mod locations;
mod system;

pub use certificate::{
    load_ca_certificate_bundle, parse_ca_certificate_bundle, CaCertificateBundle,
};
pub use elevation::{is_elevated, request_elevation, ElevationLaunch};
pub use environment::{Environment, EnvironmentError};
pub use error::PlatformError;
pub use locations::{get_app_data_dir, get_or_create_app_data_dir};
pub use system::{
    collect_disks, collect_networks, collect_process_usage, collect_resource_snapshot,
    collect_system_info, directory_size, path_disk_capacity, DiskUsage, NetworkUsage, ProcessUsage,
    ResourceSnapshot, SystemInfo,
};
