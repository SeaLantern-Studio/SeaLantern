//! 应用层主错误类型。
//!
//! 各领域子模块定义应用层主错误：承接 `core/extra/infra` 的底层错误（保留
//! source 细节供日志排查），并可向 `interface::error` 的契约错误转换，供
//! tauri / server 等宿主统一消费。

/// 配置管理领域错误。
pub mod config;
/// 服务器控制台日志领域错误。
pub mod console;
/// 服务器定时任务领域错误。
pub mod cron;
/// 下载任务管理领域错误。
pub mod download;
/// 实例管理领域错误。
pub mod instance;
/// 插件管理领域错误。
pub mod plugin;
/// 服务器管理领域错误。
pub mod server;
/// 设置信息服务领域错误。
pub mod settings;
/// 系统资源信息领域错误。
pub mod system;
/// 应用更新检查领域错误。
pub mod update;

pub use config::ConfigError;
pub use console::ConsoleError;
pub use cron::CronTaskError;
pub use download::DownloadError;
pub use instance::InstanceError;
pub use plugin::PluginError;
pub use server::ServerError;
pub use settings::SettingsError;
pub use system::SystemError;
pub use update::UpdateCheckError;
