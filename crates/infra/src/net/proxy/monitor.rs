use super::{ProxyController, ProxyUpdate, SystemProxySnapshot};

/// Accepts network-change snapshots from a platform adapter.
///
/// This type intentionally does not poll or subscribe to an operating system.
/// The composition root owns the platform-specific event source and forwards
/// each updated proxy snapshot here.
#[derive(Debug)]
pub struct ProxyMonitor {
    controller: ProxyController,
}

impl ProxyMonitor {
    /// Creates a monitor around a configured proxy controller.
    pub fn new(controller: ProxyController) -> Self {
        Self { controller }
    }

    /// Applies a system proxy snapshot received after a network change.
    pub fn network_changed(&mut self, system_proxy: SystemProxySnapshot) -> ProxyUpdate {
        self.controller.handle_system_proxy_change(system_proxy)
    }

    /// Returns the proxy controller so callers can apply user settings updates.
    pub fn controller(&self) -> &ProxyController {
        &self.controller
    }

    /// Returns mutable access for applying user settings updates.
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
            monitor.controller().effective_proxy().proxy_url(),
            Some("http://127.0.0.1:7890")
        );
    }
}
