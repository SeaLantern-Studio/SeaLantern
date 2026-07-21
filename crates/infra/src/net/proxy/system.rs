/// Snapshot of the proxy currently reported by the operating system.
///
/// A missing URL means direct connections. Platform adapters may obtain this
/// value from Windows, macOS, Linux, or another host-specific source.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SystemProxySnapshot {
    proxy_url: Option<String>,
}

impl SystemProxySnapshot {
    /// Creates a snapshot that requests direct connections.
    pub const fn direct() -> Self {
        Self { proxy_url: None }
    }

    /// Creates a snapshot with one proxy applied to all HTTP client traffic.
    pub fn proxy(proxy_url: impl Into<String>) -> Self {
        Self { proxy_url: Some(proxy_url.into()) }
    }

    /// Returns the reported proxy URL, if any.
    pub fn proxy_url(&self) -> Option<&str> {
        self.proxy_url.as_deref()
    }
}

/// Supplies the current system proxy without coupling network policy to an OS.
pub trait SystemProxyProvider {
    type Error: std::error::Error + Send + Sync + 'static;

    fn current_system_proxy(&self) -> Result<SystemProxySnapshot, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_snapshot_has_no_proxy() {
        assert_eq!(SystemProxySnapshot::direct().proxy_url(), None);
    }

    #[test]
    fn proxy_snapshot_exposes_its_url() {
        let snapshot = SystemProxySnapshot::proxy("http://127.0.0.1:7890");
        assert_eq!(snapshot.proxy_url(), Some("http://127.0.0.1:7890"));
    }
}
