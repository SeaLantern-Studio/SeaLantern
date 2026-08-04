pub mod client;
pub mod error;
pub mod proxy;
pub mod request;

pub use client::{ClientConfig, NetClient, RemoteFileInfo, RetryPolicy, TimeoutPolicy};
pub use error::NetError;
pub use proxy::{
    EffectiveProxy, ProxyConfigError, ProxyController, ProxyMode, ProxyMonitor, ProxySettings,
    ProxyUpdate, SystemProxyProvider, SystemProxySnapshot,
};
pub use request::RequestBuilder;

#[cfg(feature = "blocking")]
pub use client::NetBlockingClient;
