use super::{
    read_system_proxy, ProxyController, ProxyUpdate, SystemProxyProvider, SystemProxySnapshot,
};

/// 从平台适配器接收网络变更快照。
///
/// 此类型故意不轮询或订阅操作系统。组合根拥有平台特定的事件源，
/// 并将每个更新的代理快照转发到此。
#[derive(Debug)]
pub struct ProxyMonitor {
    controller: ProxyController,
}

impl ProxyMonitor {
    /// 围绕已配置的代理控制器创建一个监视器。
    pub fn new(controller: ProxyController) -> Self {
        Self { controller }
    }

    /// 应用网络变更后收到的系统代理快照。
    pub fn network_changed(&mut self, system_proxy: SystemProxySnapshot) -> ProxyUpdate {
        self.controller.handle_system_proxy_change(system_proxy)
    }

    /// 在平台网络变更通知后读取最新快照。
    pub fn refresh<P: SystemProxyProvider>(
        &mut self,
        provider: &P,
    ) -> Result<ProxyUpdate, P::Error> {
        Ok(self.network_changed(read_system_proxy(provider)?))
    }

    /// 返回代理控制器，以便调用者可以应用用户设置更新。
    pub fn controller(&self) -> &ProxyController {
        &self.controller
    }

    /// 返回可变访问权限以应用用户设置更新。
    pub fn controller_mut(&mut self) -> &mut ProxyController {
        &mut self.controller
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::proxy::{ProxySettings, SystemProxySnapshot};

    #[test]
    fn network_change_is_forwarded_to_the_controller() {
        let controller =
            ProxyController::new(ProxySettings::default(), SystemProxySnapshot::direct()).unwrap();
        let mut monitor = ProxyMonitor::new(controller);

        let update = monitor.network_changed(SystemProxySnapshot::proxy("http://127.0.0.1:7890"));

        assert!(update.changed());
        assert_eq!(
            monitor
                .controller()
                .effective_proxy()
                .routes_ref()
                .unwrap()
                .http_proxy(),
            Some("http://127.0.0.1:7890")
        );
    }

    #[derive(Debug)]
    struct FailingProvider;

    impl SystemProxyProvider for FailingProvider {
        type Error = std::io::Error;

        fn current_system_proxy(&self) -> Result<SystemProxySnapshot, Self::Error> {
            Err(std::io::Error::other("provider failed"))
        }
    }

    #[test]
    fn refresh_returns_provider_errors() {
        let controller =
            ProxyController::new(ProxySettings::default(), SystemProxySnapshot::direct()).unwrap();
        let mut monitor = ProxyMonitor::new(controller);

        let error = monitor.refresh(&FailingProvider).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::Other);
    }
}
