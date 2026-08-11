use std::future::Future;

use sealantern_application::plugin::{PluginService, PluginServiceError};
use sealantern_application::services::AppServices;
use sealantern_core::app_plugin::{CapabilityDispatchError, CapabilityInvocation};
use serde_json::Value;

use crate::rpc::axum::{RpcAxumMethod, RpcHttpMethod};
use crate::rpc::{RpcContext, RpcError, RpcMethod, RpcMethodName, RpcPermission, RpcResult};

use crate::rpc::methods::PERMISSION_PLUGIN_V2_INVOKE;

/// 经 Server bearer 边界认证后调用一个已加载插件的能力。
///
/// Manifest 声明和插件来源信任不会从 HTTP 请求信任；应用服务会根据已加载插件及持久化
/// 策略状态重新计算这些事实。
pub struct InvokePluginCapability {
    services: AppServices,
}

impl InvokePluginCapability {
    pub fn new(services: AppServices) -> Self {
        Self { services }
    }
}

impl RpcMethod for InvokePluginCapability {
    const NAME: RpcMethodName = RpcMethodName::new("plugin.v2.invoke");
    const REQUIRED_PERMISSION: Option<RpcPermission> = Some(PERMISSION_PLUGIN_V2_INVOKE);

    type Request = CapabilityInvocation;
    type Response = Value;

    fn call(
        &self,
        _context: &RpcContext,
        request: Self::Request,
    ) -> impl Future<Output = RpcResult<Self::Response>> + Send {
        let services = self.services.clone();
        async move {
            let plugin = services.plugin().await.map_err(map_plugin_service_error)?;
            plugin
                .invoke(request)
                .await
                .map_err(map_plugin_service_error)
        }
    }
}

impl RpcAxumMethod for InvokePluginCapability {
    const HTTP_METHOD: RpcHttpMethod = RpcHttpMethod::Post;
}

fn map_plugin_service_error(error: PluginServiceError) -> RpcError {
    match error {
        PluginServiceError::Dispatch(CapabilityDispatchError::Denied(_)) => {
            RpcError::permission_denied()
        }
        PluginServiceError::Dispatch(CapabilityDispatchError::InvalidRequest(_)) => {
            RpcError::invalid_argument("invocation", "is not permitted for this plugin")
        }
        PluginServiceError::Dispatch(CapabilityDispatchError::Unavailable(_)) => {
            RpcError::unavailable("plugin capability")
        }
        PluginServiceError::Dispatch(CapabilityDispatchError::Failed(_)) => {
            RpcError::internal("the plugin capability invocation")
        }
        PluginServiceError::Runtime(_) => RpcError::conflict("plugin invocation"),
        PluginServiceError::Policy(_) | PluginServiceError::Initialization(_) => {
            RpcError::internal("the plugin capability invocation")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::RpcErrorCode;
    use sealantern_core::app_plugin::PolicyDenialReason;

    #[test]
    fn dispatch_errors_map_to_safe_rpc_contracts() {
        assert_eq!(
            map_plugin_service_error(PluginServiceError::Dispatch(
                CapabilityDispatchError::Denied(PolicyDenialReason::PluginNotEnabled,)
            ))
            .code(),
            RpcErrorCode::PermissionDenied
        );
        assert_eq!(
            map_plugin_service_error(PluginServiceError::Dispatch(
                CapabilityDispatchError::Unavailable("not configured"),
            ))
            .code(),
            RpcErrorCode::Unavailable
        );
    }
}
