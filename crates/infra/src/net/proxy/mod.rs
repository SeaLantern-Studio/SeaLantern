//! 出站网络客户端的代理策略。
//!
//! 此模块拥有代理设置及其内存解析逻辑。它不读取应用程序配置文件，
//! 也不检测操作系统变化。这些输入由应用程序配置和平台适配器提供。

mod config;
mod monitor;
mod policy;
mod system;

pub use config::{ProxyConfigError, ProxyMode, ProxySettings};
pub use monitor::ProxyMonitor;
pub use policy::{EffectiveProxy, ProxyController, ProxyUpdate};
pub use system::{read_system_proxy, ProxyRoutes, SystemProxyProvider, SystemProxySnapshot};
