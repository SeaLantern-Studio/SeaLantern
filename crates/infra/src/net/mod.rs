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
