use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use sealantern_core::app_plugin::{CapabilityDispatchError, CapabilityInvocation};
use sealantern_extra::app_plugin::{
    AsyncPluginManager, PluginInfo, PluginLoader, PluginManagerConfig,
};
use sealantern_infra::platform::get_app_data_dir;

use super::{
    CoreCapabilityDispatcher, DefaultMarketGateway, PluginPolicyError, PluginPolicyStore,
    PluginReadHost,
};

/// 应用插件生命周期的宿主入口。
#[async_trait]
pub trait PluginService: Send + Sync {
    async fn discover(&self) -> Result<Vec<PathBuf>, PluginServiceError>;
    async fn load(&self, plugin_dir: &Path) -> Result<PluginInfo, PluginServiceError>;
    async fn enable(&self, plugin_id: &str) -> Result<(), PluginServiceError>;
    async fn disable(&self, plugin_id: &str) -> Result<(), PluginServiceError>;
    async fn unload(&self, plugin_id: &str) -> Result<(), PluginServiceError>;
    async fn plugins(&self) -> Result<Vec<PluginInfo>, PluginServiceError>;
    async fn invoke(
        &self,
        invocation: CapabilityInvocation,
    ) -> Result<serde_json::Value, PluginServiceError>;
}

/// 应用插件服务的可恢复错误。
#[derive(Debug)]
pub enum PluginServiceError {
    Runtime(sealantern_extra::app_plugin::AppPluginError),
    Policy(PluginPolicyError),
    Dispatch(CapabilityDispatchError),
    Initialization(String),
}

impl std::fmt::Display for PluginServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(formatter, "plugin runtime failed: {error}"),
            Self::Policy(error) => write!(formatter, "plugin policy state failed: {error}"),
            Self::Dispatch(error) => {
                write!(formatter, "plugin capability dispatch failed: {error}")
            }
            Self::Initialization(error) => {
                write!(formatter, "plugin service initialization failed: {error}")
            }
        }
    }
}

impl std::error::Error for PluginServiceError {}

impl From<sealantern_extra::app_plugin::AppPluginError> for PluginServiceError {
    fn from(error: sealantern_extra::app_plugin::AppPluginError) -> Self {
        Self::Runtime(error)
    }
}

impl From<PluginPolicyError> for PluginServiceError {
    fn from(error: PluginPolicyError) -> Self {
        Self::Policy(error)
    }
}

/// 将插件运行时和安全策略状态组合为单一宿主服务。
pub struct CorePluginService {
    runtime: AsyncPluginManager,
    policy: Arc<PluginPolicyStore>,
}

impl CorePluginService {
    /// 使用应用数据目录中的插件目录、私有数据和策略数据库构造服务。
    pub async fn open_default() -> Result<Self, PluginServiceError> {
        let root = get_app_data_dir().join("plugins");
        Self::open(&root, root.join("data"), root.join("plugin-state.sqlite")).await
    }

    /// 使用明确的目录构造服务，适用于宿主配置和测试注入。
    pub async fn open(
        plugins_dir: impl Into<PathBuf>,
        data_dir: impl Into<PathBuf>,
        state_path: impl Into<PathBuf>,
    ) -> Result<Self, PluginServiceError> {
        Self::open_with_read_host(plugins_dir, data_dir, state_path, None).await
    }

    /// 使用明确的只读宿主能力构造服务。
    pub async fn open_with_read_host(
        plugins_dir: impl Into<PathBuf>,
        data_dir: impl Into<PathBuf>,
        state_path: impl Into<PathBuf>,
        read_host: Option<Arc<dyn PluginReadHost>>,
    ) -> Result<Self, PluginServiceError> {
        let policy = Arc::new(PluginPolicyStore::open(state_path).await?);
        let market =
            Arc::new(DefaultMarketGateway::new().map_err(PluginServiceError::Initialization)?);
        let dispatcher = CoreCapabilityDispatcher::new(policy.clone(), market);
        let dispatcher = match read_host {
            Some(host) => dispatcher.with_read_host(host),
            None => dispatcher,
        };
        let dispatcher = Arc::new(dispatcher);
        Ok(Self {
            runtime: AsyncPluginManager::new(
                PluginManagerConfig::new(plugins_dir, data_dir).with_dispatcher(dispatcher),
            ),
            policy,
        })
    }

    pub fn policy(&self) -> &Arc<PluginPolicyStore> {
        &self.policy
    }
}

#[async_trait]
impl PluginService for CorePluginService {
    async fn discover(&self) -> Result<Vec<PathBuf>, PluginServiceError> {
        self.runtime.discover().await.map_err(Into::into)
    }

    async fn load(&self, plugin_dir: &Path) -> Result<PluginInfo, PluginServiceError> {
        // 先清除上次进程留下的启用状态，再允许 entry/on_load 执行能力调用。
        let manifest =
            PluginLoader::load_manifest(plugin_dir).map_err(PluginServiceError::Runtime)?;
        self.policy.set_enabled(&manifest.id, false).await?;
        let info = self.runtime.load(plugin_dir).await?;
        Ok(info)
    }

    async fn enable(&self, plugin_id: &str) -> Result<(), PluginServiceError> {
        self.policy.set_enabled(plugin_id, true).await?;
        if let Err(error) = self.runtime.enable(plugin_id).await {
            if let Err(rollback_error) = self.policy.set_enabled(plugin_id, false).await {
                tracing::error!(
                    target: "sealantern.application.plugin",
                    plugin_id,
                    error = %rollback_error,
                    "plugin enable rollback could not be persisted"
                );
            }
            tracing::error!(
                target: "sealantern.application.plugin",
                plugin_id,
                error = %error,
                "plugin enable lifecycle failed"
            );
            return Err(PluginServiceError::Runtime(error));
        }
        Ok(())
    }

    async fn disable(&self, plugin_id: &str) -> Result<(), PluginServiceError> {
        self.runtime.disable(plugin_id).await?;
        self.policy.set_enabled(plugin_id, false).await?;
        Ok(())
    }

    async fn unload(&self, plugin_id: &str) -> Result<(), PluginServiceError> {
        self.runtime.unload(plugin_id).await?;
        self.policy.set_enabled(plugin_id, false).await?;
        Ok(())
    }

    async fn plugins(&self) -> Result<Vec<PluginInfo>, PluginServiceError> {
        self.runtime.plugins().await.map_err(Into::into)
    }

    async fn invoke(
        &self,
        mut invocation: CapabilityInvocation,
    ) -> Result<serde_json::Value, PluginServiceError> {
        let sealantern_core::app_plugin::ExecutionPrincipal::Plugin(plugin_id) =
            &invocation.principal
        else {
            return Err(PluginServiceError::Dispatch(CapabilityDispatchError::InvalidRequest(
                "plugin principal",
            )));
        };
        let plugin = self.runtime.plugin(plugin_id).await?.ok_or_else(|| {
            PluginServiceError::Dispatch(CapabilityDispatchError::InvalidRequest("loaded plugin"))
        })?;
        invocation.declared = plugin.manifest.capabilities.iter().any(|capability| {
            capability.id == invocation.capability.as_str()
                && capability.scope.as_ref() == invocation.scope.as_ref()
        });
        self.runtime
            .invoke(invocation)
            .await
            .map_err(PluginServiceError::Dispatch)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use sealantern_core::app_plugin::{
        CapabilityId, ExecutionPrincipal, ScopeBinding, ScopeKind, TrustSource,
    };

    fn manifest() -> &'static str {
        r#"{
            "apiVersion": 2,
            "id": "example.plugin",
            "name": "Example Plugin",
            "version": "1.0.0",
            "main": "main.lua",
            "capabilities": []
        }"#
    }

    #[tokio::test]
    async fn service_keeps_persisted_enabled_state_in_sync_with_lifecycle() {
        let root = tempfile::tempdir().expect("temporary root should be created");
        let plugin_dir = root.path().join("plugins").join("example.plugin");
        fs::create_dir_all(&plugin_dir).expect("plugin directory should be created");
        fs::write(plugin_dir.join("manifest.json"), manifest())
            .expect("manifest should be written");
        fs::write(
            plugin_dir.join("main.lua"),
            "function on_load() end function on_enable() end function on_disable() end",
        )
        .expect("script should be written");
        let service = CorePluginService::open(
            root.path().join("plugins"),
            root.path().join("data"),
            root.path().join("plugin-state.sqlite"),
        )
        .await
        .expect("service should open");

        service
            .policy()
            .set_enabled("example.plugin", true)
            .await
            .expect("stale enabled state should be writable");
        service.load(&plugin_dir).await.expect("plugin should load");
        assert!(!service.policy().is_enabled("example.plugin").await.unwrap());
        service
            .enable("example.plugin")
            .await
            .expect("plugin should enable");
        assert!(service.policy().is_enabled("example.plugin").await.unwrap());
        service
            .disable("example.plugin")
            .await
            .expect("plugin should disable");
        assert!(!service.policy().is_enabled("example.plugin").await.unwrap());
    }

    #[tokio::test]
    async fn invoke_recomputes_manifest_declaration_at_the_host_boundary() {
        let root = tempfile::tempdir().expect("temporary root should be created");
        let plugin_dir = root.path().join("plugins").join("example.plugin");
        fs::create_dir_all(&plugin_dir).expect("plugin directory should be created");
        fs::write(plugin_dir.join("manifest.json"), manifest())
            .expect("manifest should be written");
        fs::write(
            plugin_dir.join("main.lua"),
            "function on_load() end function on_enable() end function on_disable() end",
        )
        .expect("script should be written");
        let service = CorePluginService::open(
            root.path().join("plugins"),
            root.path().join("data"),
            root.path().join("plugin-state.sqlite"),
        )
        .await
        .expect("service should open");

        service.load(&plugin_dir).await.expect("plugin should load");
        service
            .enable("example.plugin")
            .await
            .expect("plugin should enable");
        let scope = ScopeBinding::new(ScopeKind::AppGlobal, "host").unwrap();
        service
            .policy()
            .grant_persistent("example.plugin", "host.system.facts", Some(&scope))
            .await
            .expect("grant should persist");

        let error = service
            .invoke(CapabilityInvocation {
                principal: ExecutionPrincipal::Plugin("example.plugin".to_string()),
                trust_source: TrustSource::BuiltIn,
                capability: CapabilityId::new("host.system.facts").unwrap(),
                scope: Some(scope),
                declared: true,
                session_id: None,
                payload: serde_json::Value::Null,
                approval_token: None,
                request_id: "service-test-1".to_string(),
            })
            .await
            .expect_err("undeclared capability must be denied");

        assert!(matches!(
            error,
            PluginServiceError::Dispatch(CapabilityDispatchError::Denied(
                sealantern_core::app_plugin::PolicyDenialReason::CapabilityNotDeclared
            ))
        ));
    }
}
