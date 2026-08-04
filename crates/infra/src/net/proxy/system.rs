/// 从操作系统配置解析的静态代理路由。
///
/// 平台适配器必须在构造此值之前解析 PAC 或其他动态配置。
/// 网络层从不执行外部脚本。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyRoutes {
    http_proxy: Option<String>,
    https_proxy: Option<String>,
    no_proxy: Vec<String>,
}

impl ProxyRoutes {
    /// 为 HTTP 和 HTTPS 流量创建直连路由。
    pub const fn direct() -> Self {
        Self {
            http_proxy: None,
            https_proxy: None,
            no_proxy: Vec::new(),
        }
    }

    /// 创建将对 HTTP 和 HTTPS 流量应用同一代理的路由。
    pub fn all(proxy_url: impl Into<String>) -> Self {
        let proxy_url = proxy_url.into();
        Self {
            http_proxy: Some(proxy_url.clone()),
            https_proxy: Some(proxy_url),
            no_proxy: Vec::new(),
        }
    }

    /// 创建具有独立 HTTP 和 HTTPS 代理端点的路由。
    pub fn split(http_proxy: Option<String>, https_proxy: Option<String>) -> Self {
        Self {
            http_proxy,
            https_proxy,
            no_proxy: Vec::new(),
        }
    }

    /// 添加绕过已配置代理的主机、域名、IP 或 CIDR 条目。
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

/// 操作系统当前报告的静态代理路由快照。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemProxySnapshot {
    routes: ProxyRoutes,
}

impl SystemProxySnapshot {
    /// 创建一个请求直连的快照。
    pub const fn direct() -> Self {
        Self { routes: ProxyRoutes::direct() }
    }

    /// 创建一个将同一代理应用于 HTTP 和 HTTPS 流量的快照。
    pub fn proxy(proxy_url: impl Into<String>) -> Self {
        Self::from_routes(ProxyRoutes::all(proxy_url))
    }

    /// 创建具有独立 HTTP 和 HTTPS 路由的快照。
    pub fn split(http_proxy: Option<String>, https_proxy: Option<String>) -> Self {
        Self::from_routes(ProxyRoutes::split(http_proxy, https_proxy))
    }

    /// 从平台解析的路由创建一个快照。
    pub fn from_routes(routes: ProxyRoutes) -> Self {
        Self { routes }
    }

    pub fn routes(&self) -> &ProxyRoutes {
        &self.routes
    }
}

/// 提供当前系统代理，而不将网络策略与操作系统耦合。
pub trait SystemProxyProvider {
    type Error: std::error::Error + Send + Sync + 'static;

    fn current_system_proxy(&self) -> Result<SystemProxySnapshot, Self::Error>;
}

/// 读取平台代理快照并记录稳定的失败事件。
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
