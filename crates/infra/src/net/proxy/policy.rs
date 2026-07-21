use super::{ProxyConfigError, ProxyMode, ProxyRoutes, ProxySettings, SystemProxySnapshot};
use crate::observability;

/// 策略解析后 HTTP 客户端应使用的代理。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectiveProxy {
    Direct,
    Routes(ProxyRoutes),
}

impl EffectiveProxy {
    /// 创建一个代理决策，将 URL 应用于所有 HTTP 客户端流量。
    pub fn proxy(proxy_url: impl Into<String>) -> Self {
        Self::Routes(ProxyRoutes::all(proxy_url))
    }

    /// 从独立解析的 HTTP 和 HTTPS 路由创建一个决策。
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

/// 应用设置或系统网络变更事件的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyUpdate {
    pub previous: EffectiveProxy,
    pub current: EffectiveProxy,
}

impl ProxyUpdate {
    /// 返回调用者是否需要重建其 HTTP 客户端。
    pub fn changed(&self) -> bool {
        self.previous != self.current
    }
}

/// 解析代理策略并持有当前内存中的决策。
///
/// 配置文件和操作系统事件循环保持在此类型之外。宿主通过
/// [`Self::handle_system_proxy_change`] 提供快照。
#[derive(Debug, Clone)]
pub struct ProxyController {
    settings: ProxySettings,
    effective_proxy: EffectiveProxy,
}

impl ProxyController {
    /// 从应用程序设置和当前系统代理创建一个控制器。
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

    /// 返回此控制器当前拥有的设置。
    pub fn settings(&self) -> &ProxySettings {
        &self.settings
    }

    /// 返回当前为新建 HTTP 客户端选择的代理。
    pub fn effective_proxy(&self) -> &EffectiveProxy {
        &self.effective_proxy
    }

    /// 替换设置并根据当前系统快照重新解析。
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

    /// 应用操作系统网络变更。
    ///
    /// 只有自适应模式会跟随后续系统变化。保留、手动和禁用模式
    /// 有意保留其现有决策。
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
