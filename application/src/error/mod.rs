//! 应用层主错误类型。
//!
//! 各领域子模块定义应用层主错误：承接 `core/extra/infra` 的底层错误（保留
//! source 细节供日志排查），并可向 `interface::error` 的契约错误转换，供
//! tauri / server 等宿主统一消费。

/// 实例管理领域错误。
pub mod instance;
/// 服务器管理领域错误。
pub mod server;
/// 配置管理领域错误。
pub mod config;
/// 插件管理领域错误。
pub mod plugin;

pub use instance::InstanceError;
pub use server::ServerError;
pub use config::ConfigError;
pub use plugin::PluginError;
