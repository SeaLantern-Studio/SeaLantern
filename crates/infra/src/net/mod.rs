//! 网络基础设施。
//!
//! 四个关注点各自独立成模块：
//!
//! - [`client`]：统一的 `reqwest::Client` 构造入口，避免客户端在各处分散创建；
//! - [`request`]：可链式构建的请求与指数退避重试；
//! - [`proxy`]：用户设置与系统代理快照折叠成有效代理策略；
//! - `plugin`：给第三方插件用的受控出站网络，带 SSRF 防护与资源上限；
//! - `runtime`：进程级全局客户端与代理状态，带世代号以支持无锁读取。
//!
//! `client` / `error` / `request` 与 `proxy` 目前是 `pub mod`，使调用方既可
//! 走下方重导出，也可直接引用子模块路径；`plugin` 与 `runtime` 私有，只能
//! 经重导出使用。

pub mod client;
pub mod error;
mod plugin;
pub mod proxy;
pub mod request;
mod runtime;

pub use client::{ClientConfig, NetClient, RemoteFileInfo, RetryPolicy, TimeoutPolicy};
pub use error::NetError;
pub use plugin::{
    AllowedNetworkTarget, NetworkOrigin, PluginHttpMethod, PluginNetworkAddressPolicy,
    PluginNetworkClient, PluginNetworkCredentials, PluginNetworkError, PluginNetworkExecution,
    PluginNetworkExecutor, PluginNetworkLimits, PluginNetworkRequest, PluginNetworkResponse,
    PluginNetworkScope, PluginNetworkTrace, PluginRequestHeaders, PluginTransportErrorKind,
    PluginTransportStage, ResolvedNetworkTarget,
};
pub use proxy::{
    EffectiveProxy, ProxyConfigError, ProxyController, ProxyMode, ProxyMonitor, ProxySettings,
    ProxyUpdate, SystemProxyProvider, SystemProxySnapshot,
};
pub use request::RequestBuilder;
pub use runtime::{
    ClientProvider, NetworkCommitError, NetworkUpdate, PreparedNetworkUpdate, apply_proxy_settings,
    apply_system_proxy_snapshot, commit_prepared_proxy_update, global_client,
    global_client_provider, prepare_proxy_settings,
};

#[cfg(feature = "blocking")]
pub use client::NetBlockingClient;
