/// Static proxy routes resolved from operating-system configuration.
///
/// Platform adapters must resolve PAC or other dynamic configuration before
/// constructing this value. The network layer never executes external scripts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyRoutes {
    http_proxy: Option<String>,
    https_proxy: Option<String>,
    no_proxy: Vec<String>,
}

impl ProxyRoutes {
    /// Creates direct routes for both HTTP and HTTPS traffic.
    pub const fn direct() -> Self {
        Self {
            http_proxy: None,
            https_proxy: None,
            no_proxy: Vec::new(),
        }
    }

    /// Creates routes that apply one proxy to both HTTP and HTTPS traffic.
    pub fn all(proxy_url: impl Into<String>) -> Self {
        let proxy_url = proxy_url.into();
        Self {
            http_proxy: Some(proxy_url.clone()),
            https_proxy: Some(proxy_url),
            no_proxy: Vec::new(),
        }
    }

    /// Creates routes with independent HTTP and HTTPS proxy endpoints.
    pub fn split(http_proxy: Option<String>, https_proxy: Option<String>) -> Self {
        Self {
            http_proxy,
            https_proxy,
            no_proxy: Vec::new(),
        }
    }

    /// Adds host, domain, IP, or CIDR entries that bypass configured proxies.
    pub fn with_no_proxy(mut self, no_proxy: Vec<String>) -> Self {
        self.no_proxy = no_proxy;
        self
    }

    pub fn http_proxy(&self) -> Option<&str> {
        self.http_proxy.as_deref()
    }

    pub fn https_proxy(&self) -> Option<&str> {
        self.https_proxy.as_deref()
    }

    pub fn no_proxy(&self) -> &[String] {
        &self.no_proxy
    }
}

/// Snapshot of static proxy routes currently reported by the operating system.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemProxySnapshot {
    routes: ProxyRoutes,
}

impl SystemProxySnapshot {
    /// Creates a snapshot that requests direct connections.
    pub const fn direct() -> Self {
        Self { routes: ProxyRoutes::direct() }
    }

    /// Creates a snapshot with one proxy applied to HTTP and HTTPS traffic.
    pub fn proxy(proxy_url: impl Into<String>) -> Self {
        Self::from_routes(ProxyRoutes::all(proxy_url))
    }

    /// Creates a snapshot with independent HTTP and HTTPS routes.
    pub fn split(http_proxy: Option<String>, https_proxy: Option<String>) -> Self {
        Self::from_routes(ProxyRoutes::split(http_proxy, https_proxy))
    }

    /// Creates a snapshot from platform-resolved routes.
    pub fn from_routes(routes: ProxyRoutes) -> Self {
        Self { routes }
    }

    pub fn routes(&self) -> &ProxyRoutes {
        &self.routes
    }
}

/// Supplies the current system proxy without coupling network policy to an OS.
pub trait SystemProxyProvider {
    type Error: std::error::Error + Send + Sync + 'static;

    fn current_system_proxy(&self) -> Result<SystemProxySnapshot, Self::Error>;
}

/// Reads a platform proxy snapshot and records a stable failure event.
pub fn read_system_proxy<P: SystemProxyProvider>(
    provider: &P,
) -> Result<SystemProxySnapshot, P::Error> {
    provider
        .current_system_proxy()
        .inspect_err(|_| crate::observability::system_proxy_read_failed())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_snapshot_has_no_routes() {
        let snapshot = SystemProxySnapshot::direct();
        let routes = snapshot.routes();
        assert_eq!(routes.http_proxy(), None);
        assert_eq!(routes.https_proxy(), None);
    }

    #[test]
    fn split_snapshot_preserves_routes_and_bypass_rules() {
        let snapshot = SystemProxySnapshot::split(
            Some("http://127.0.0.1:7890".into()),
            Some("http://127.0.0.1:7891".into()),
        );
        let routes = snapshot
            .routes()
            .clone()
            .with_no_proxy(vec!["localhost".into()]);

        assert_eq!(routes.http_proxy(), Some("http://127.0.0.1:7890"));
        assert_eq!(routes.https_proxy(), Some("http://127.0.0.1:7891"));
        assert_eq!(routes.no_proxy(), &["localhost".to_owned()]);
    }

    #[derive(Debug)]
    struct FailingProvider;

    impl SystemProxyProvider for FailingProvider {
        type Error = std::io::Error;

        fn current_system_proxy(&self) -> Result<SystemProxySnapshot, Self::Error> {
            Err(std::io::Error::other("system settings unavailable"))
        }
    }

    #[test]
    fn provider_errors_are_returned_to_the_caller() {
        let error = read_system_proxy(&FailingProvider).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::Other);
    }
}
