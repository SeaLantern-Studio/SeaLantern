use super::{ProxyConfigError, ProxyMode, ProxyRoutes, ProxySettings, SystemProxySnapshot};
use crate::observability;

/// Proxy that an HTTP client should use after policy resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectiveProxy {
    Direct,
    Routes(ProxyRoutes),
}

impl EffectiveProxy {
    /// Creates a proxy decision that applies the URL to all HTTP client traffic.
    pub fn proxy(proxy_url: impl Into<String>) -> Self {
        Self::Routes(ProxyRoutes::all(proxy_url))
    }

    /// Creates a decision from independently resolved HTTP and HTTPS routes.
    pub fn routes(routes: ProxyRoutes) -> Self {
        Self::Routes(routes)
    }

    pub fn routes_ref(&self) -> Option<&ProxyRoutes> {
        match self {
            Self::Direct => None,
            Self::Routes(routes) => Some(routes),
        }
    }
}

/// Result of applying settings or a system network-change event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyUpdate {
    pub previous: EffectiveProxy,
    pub current: EffectiveProxy,
}

impl ProxyUpdate {
    /// Returns whether a caller needs to rebuild its HTTP client.
    pub fn changed(&self) -> bool {
        self.previous != self.current
    }
}

/// Resolves proxy policy and holds its current in-memory decision.
///
/// Configuration files and OS event loops remain outside this type. A host
/// supplies snapshots through [`Self::handle_system_proxy_change`].
#[derive(Debug, Clone)]
pub struct ProxyController {
    settings: ProxySettings,
    effective_proxy: EffectiveProxy,
}

impl ProxyController {
    /// Creates a controller from application settings and the current system proxy.
    pub fn new(
        settings: ProxySettings,
        system_proxy: SystemProxySnapshot,
    ) -> Result<Self, ProxyConfigError> {
        settings
            .validate()
            .inspect_err(|error| observability::proxy_settings_invalid(error))?;
        let effective_proxy = resolve(&settings, &system_proxy);
        observability::proxy_decision_updated("initial", settings.mode.as_str(), true);

        Ok(Self { settings, effective_proxy })
    }

    /// Returns the settings currently owned by this controller.
    pub fn settings(&self) -> &ProxySettings {
        &self.settings
    }

    /// Returns the proxy currently selected for newly built HTTP clients.
    pub fn effective_proxy(&self) -> &EffectiveProxy {
        &self.effective_proxy
    }

    /// Replaces settings and resolves them against the current system snapshot.
    pub fn update_settings(
        &mut self,
        settings: ProxySettings,
        system_proxy: SystemProxySnapshot,
    ) -> Result<ProxyUpdate, ProxyConfigError> {
        settings
            .validate()
            .inspect_err(|error| observability::proxy_settings_invalid(error))?;
        self.settings = settings;
        let update = self.replace_effective_proxy(resolve(&self.settings, &system_proxy));
        observability::proxy_decision_updated(
            "settings",
            self.settings.mode.as_str(),
            update.changed(),
        );
        Ok(update)
    }

    /// Applies an operating-system network change.
    ///
    /// Only adaptive settings follow later system changes. Preserve, manual,
    /// and disabled modes intentionally retain their existing decision.
    pub fn handle_system_proxy_change(&mut self, system_proxy: SystemProxySnapshot) -> ProxyUpdate {
        let next = match self.settings.mode {
            ProxyMode::Adaptive => resolve(&self.settings, &system_proxy),
            ProxyMode::Preserve | ProxyMode::Manual { .. } | ProxyMode::Disabled => {
                self.effective_proxy.clone()
            }
        };

        let update = self.replace_effective_proxy(next);
        observability::proxy_decision_updated(
            "system",
            self.settings.mode.as_str(),
            update.changed(),
        );
        update
    }

    fn replace_effective_proxy(&mut self, current: EffectiveProxy) -> ProxyUpdate {
        let previous = std::mem::replace(&mut self.effective_proxy, current.clone());
        ProxyUpdate { previous, current }
    }
}

fn resolve(settings: &ProxySettings, system_proxy: &SystemProxySnapshot) -> EffectiveProxy {
    match &settings.mode {
        ProxyMode::Adaptive | ProxyMode::Preserve => {
            let routes = system_proxy.routes().clone();
            if routes.http_proxy().is_some() || routes.https_proxy().is_some() {
                EffectiveProxy::routes(routes)
            } else {
                EffectiveProxy::Direct
            }
        }
        ProxyMode::Manual { proxy_url } => EffectiveProxy::proxy(proxy_url),
        ProxyMode::Disabled => EffectiveProxy::Direct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST_PROXY: &str = "http://127.0.0.1:7890";
    const SECOND_PROXY: &str = "http://127.0.0.1:7891";

    #[test]
    fn adaptive_mode_follows_system_proxy_changes() {
        let mut controller =
            ProxyController::new(ProxySettings::default(), SystemProxySnapshot::proxy(FIRST_PROXY))
                .unwrap();

        let update =
            controller.handle_system_proxy_change(SystemProxySnapshot::proxy(SECOND_PROXY));

        assert!(update.changed());
        assert_eq!(
            controller
                .effective_proxy()
                .routes_ref()
                .unwrap()
                .http_proxy(),
            Some(SECOND_PROXY)
        );
    }

    #[test]
    fn preserve_mode_keeps_the_initial_system_proxy() {
        let mut controller = ProxyController::new(
            ProxySettings { mode: ProxyMode::Preserve },
            SystemProxySnapshot::proxy(FIRST_PROXY),
        )
        .unwrap();

        let update =
            controller.handle_system_proxy_change(SystemProxySnapshot::proxy(SECOND_PROXY));

        assert!(!update.changed());
        assert_eq!(
            controller
                .effective_proxy()
                .routes_ref()
                .unwrap()
                .http_proxy(),
            Some(FIRST_PROXY)
        );
    }

    #[test]
    fn manual_mode_ignores_system_proxy_changes() {
        let mut controller = ProxyController::new(
            ProxySettings {
                mode: ProxyMode::Manual { proxy_url: FIRST_PROXY.into() },
            },
            SystemProxySnapshot::direct(),
        )
        .unwrap();

        let update =
            controller.handle_system_proxy_change(SystemProxySnapshot::proxy(SECOND_PROXY));

        assert!(!update.changed());
        assert_eq!(
            controller
                .effective_proxy()
                .routes_ref()
                .unwrap()
                .http_proxy(),
            Some(FIRST_PROXY)
        );
    }

    #[test]
    fn disabled_mode_forces_direct_connections() {
        let controller = ProxyController::new(
            ProxySettings { mode: ProxyMode::Disabled },
            SystemProxySnapshot::proxy(FIRST_PROXY),
        )
        .unwrap();

        assert_eq!(controller.effective_proxy(), &EffectiveProxy::Direct);
    }

    #[test]
    fn changing_to_preserve_uses_the_current_snapshot_once() {
        let mut controller =
            ProxyController::new(ProxySettings::default(), SystemProxySnapshot::direct()).unwrap();

        let update = controller
            .update_settings(
                ProxySettings { mode: ProxyMode::Preserve },
                SystemProxySnapshot::proxy(FIRST_PROXY),
            )
            .unwrap();

        assert!(update.changed());
        assert_eq!(update.current.routes_ref().unwrap().http_proxy(), Some(FIRST_PROXY));
    }
}
