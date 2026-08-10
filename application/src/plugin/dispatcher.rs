use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use sealantern_core::app_plugin::{
    capability, CapabilityDispatchError, CapabilityDispatcher, CapabilityInvocation,
    ExecutionPrincipal, PolicyDecision, ScopeKind,
};
use sealantern_extra::market::{
    Fetcher, MarketError, MarketSource, ResourceInfo, SearchResult, Version,
};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::Mutex;

use super::PluginPolicyStore;

const MARKET_PAGE_SIZE_LIMIT: u32 = 100;
const RATE_WINDOW: Duration = Duration::from_secs(60);

/// 只读市场能力的宿主端口。
#[async_trait]
pub trait MarketGateway: Send + Sync {
    async fn search(
        &self,
        source: MarketSource,
        query: &str,
        page: u32,
        page_size: u32,
    ) -> Result<SearchResult, MarketError>;
    async fn resource(&self, source: MarketSource, id: &str) -> Result<ResourceInfo, MarketError>;
    async fn versions(&self, source: MarketSource, id: &str) -> Result<Vec<Version>, MarketError>;
}

/// 基于既有市场抓取器的只读网关。
pub struct DefaultMarketGateway {
    modrinth: Arc<dyn Fetcher>,
    spiget: Arc<dyn Fetcher>,
}

impl DefaultMarketGateway {
    pub fn new() -> Result<Self, String> {
        let client = sealantern_infra::net::NetClient::from_config(&Default::default())
            .map_err(|error| error.to_string())?;
        Ok(Self {
            modrinth: Arc::new(sealantern_extra::market::ModrinthFetcher::new(client.clone())),
            spiget: Arc::new(sealantern_extra::market::SpigetFetcher::new(client)),
        })
    }

    fn fetcher(&self, source: MarketSource) -> &Arc<dyn Fetcher> {
        match source {
            MarketSource::Modrinth => &self.modrinth,
            MarketSource::Spiget => &self.spiget,
        }
    }
}

#[async_trait]
impl MarketGateway for DefaultMarketGateway {
    async fn search(
        &self,
        source: MarketSource,
        query: &str,
        page: u32,
        page_size: u32,
    ) -> Result<SearchResult, MarketError> {
        self.fetcher(source).search(query, page, page_size).await
    }

    async fn resource(&self, source: MarketSource, id: &str) -> Result<ResourceInfo, MarketError> {
        self.fetcher(source).get_resource(id).await
    }

    async fn versions(&self, source: MarketSource, id: &str) -> Result<Vec<Version>, MarketError> {
        self.fetcher(source).get_resource_versions(id).await
    }
}

/// 所有 Lua 宿主能力调用的策略、限流与执行边界。
pub struct CoreCapabilityDispatcher {
    policy: Arc<PluginPolicyStore>,
    market: Arc<dyn MarketGateway>,
    calls: Mutex<HashMap<(String, String), VecDeque<Instant>>>,
}

impl CoreCapabilityDispatcher {
    pub fn new(policy: Arc<PluginPolicyStore>, market: Arc<dyn MarketGateway>) -> Self {
        Self {
            policy,
            market,
            calls: Mutex::new(HashMap::new()),
        }
    }

    async fn check_rate_limit(
        &self,
        plugin_id: &str,
        capability_id: &str,
    ) -> Result<(), CapabilityDispatchError> {
        let Some(limit) =
            capability(capability_id).and_then(|descriptor| descriptor.max_calls_per_minute)
        else {
            return Ok(());
        };
        let now = Instant::now();
        let mut calls = self.calls.lock().await;
        let entries = calls
            .entry((plugin_id.to_owned(), capability_id.to_owned()))
            .or_default();
        while entries
            .front()
            .is_some_and(|timestamp| now.duration_since(*timestamp) >= RATE_WINDOW)
        {
            entries.pop_front();
        }
        if entries.len() >= limit as usize {
            return Err(CapabilityDispatchError::Unavailable("capability rate limit exceeded"));
        }
        entries.push_back(now);
        Ok(())
    }

    async fn dispatch_market(
        &self,
        capability_id: &str,
        scope: Option<&sealantern_core::app_plugin::ScopeBinding>,
        payload: &Value,
    ) -> Result<Value, CapabilityDispatchError> {
        match capability_id {
            "market.search" => {
                let request: MarketSearchRequest = serde_json::from_value(payload.clone())
                    .map_err(|_| {
                        CapabilityDispatchError::InvalidRequest("market search payload")
                    })?;
                let page = request.page.unwrap_or(1);
                let page_size = request.page_size.unwrap_or(20);
                if request.query.trim().is_empty()
                    || page == 0
                    || page_size == 0
                    || page_size > MARKET_PAGE_SIZE_LIMIT
                {
                    return Err(CapabilityDispatchError::InvalidRequest(
                        "market search pagination",
                    ));
                }
                let result = self
                    .market
                    .search(request.source, &request.query, page, page_size)
                    .await
                    .map_err(|error| market_error("search", error))?;
                serde_json::to_value(result)
                    .map_err(|_| CapabilityDispatchError::Failed("market response encoding"))
            }
            "market.resource.read" => {
                let (source, id) = market_scope(scope)?;
                let result = self
                    .market
                    .resource(source, id)
                    .await
                    .map_err(|error| market_error("resource", error))?;
                serde_json::to_value(result)
                    .map_err(|_| CapabilityDispatchError::Failed("market response encoding"))
            }
            "market.versions.read" => {
                let (source, id) = market_scope(scope)?;
                let result = self
                    .market
                    .versions(source, id)
                    .await
                    .map_err(|error| market_error("versions", error))?;
                serde_json::to_value(result)
                    .map_err(|_| CapabilityDispatchError::Failed("market response encoding"))
            }
            _ => Err(CapabilityDispatchError::Unavailable(
                "capability implementation is not available",
            )),
        }
    }
}

#[async_trait]
impl CapabilityDispatcher for CoreCapabilityDispatcher {
    async fn invoke(
        &self,
        invocation: CapabilityInvocation,
    ) -> Result<Value, CapabilityDispatchError> {
        let ExecutionPrincipal::Plugin(plugin_id) = &invocation.principal else {
            return Err(CapabilityDispatchError::InvalidRequest("plugin principal"));
        };
        let decision = self
            .policy
            .evaluate(
                None,
                plugin_id,
                invocation.capability.as_str(),
                invocation.scope.as_ref(),
                invocation.declared,
                false,
            )
            .await
            .map_err(|error| {
                tracing::error!(
                    target: "sealantern.application.plugin",
                    plugin_id,
                    capability_id = invocation.capability.as_str(),
                    error = %error,
                    "plugin capability policy evaluation failed"
                );
                CapabilityDispatchError::Failed("plugin policy evaluation")
            })?;
        if let PolicyDecision::Deny(reason) = decision {
            return Err(CapabilityDispatchError::Denied(reason));
        }
        self.check_rate_limit(plugin_id, invocation.capability.as_str())
            .await?;
        self.dispatch_market(
            invocation.capability.as_str(),
            invocation.scope.as_ref(),
            &invocation.payload,
        )
        .await
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MarketSearchRequest {
    source: MarketSource,
    query: String,
    page: Option<u32>,
    page_size: Option<u32>,
}

fn market_scope(
    scope: Option<&sealantern_core::app_plugin::ScopeBinding>,
) -> Result<(MarketSource, &str), CapabilityDispatchError> {
    let scope = scope.ok_or(CapabilityDispatchError::InvalidRequest("market scope"))?;
    if scope.kind != ScopeKind::MarketArtifact {
        return Err(CapabilityDispatchError::InvalidRequest("market scope kind"));
    }
    let (source, id) = scope
        .value
        .split_once(':')
        .ok_or(CapabilityDispatchError::InvalidRequest("market scope value"))?;
    let source = match source {
        "modrinth" => MarketSource::Modrinth,
        "spiget" => MarketSource::Spiget,
        _ => return Err(CapabilityDispatchError::InvalidRequest("market source")),
    };
    (!id.is_empty())
        .then_some((source, id))
        .ok_or(CapabilityDispatchError::InvalidRequest("market resource id"))
}

fn market_error(operation: &'static str, error: MarketError) -> CapabilityDispatchError {
    tracing::warn!(
        target: "sealantern.application.plugin",
        operation,
        error = %error,
        "plugin market capability failed"
    );
    match error {
        MarketError::NotFound { .. } => {
            CapabilityDispatchError::Unavailable("market resource not found")
        }
        _ => CapabilityDispatchError::Failed("market request failed"),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use sealantern_core::app_plugin::{CapabilityId, ScopeBinding, TrustSource};

    struct FakeMarket {
        searches: AtomicUsize,
    }

    #[async_trait]
    impl MarketGateway for FakeMarket {
        async fn search(
            &self,
            _source: MarketSource,
            _query: &str,
            page: u32,
            page_size: u32,
        ) -> Result<SearchResult, MarketError> {
            self.searches.fetch_add(1, Ordering::Relaxed);
            Ok(SearchResult {
                resources: vec![],
                total: 0,
                offset: u64::from(page - 1) * u64::from(page_size),
                limit: u64::from(page_size),
            })
        }

        async fn resource(&self, _: MarketSource, _: &str) -> Result<ResourceInfo, MarketError> {
            Err(MarketError::NotFound { resource: "test".to_string() })
        }

        async fn versions(&self, _: MarketSource, _: &str) -> Result<Vec<Version>, MarketError> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn market_search_requires_enabled_plugin_and_declared_capability() {
        let root = tempfile::tempdir().unwrap();
        let policy = Arc::new(
            PluginPolicyStore::open(root.path().join("state.sqlite"))
                .await
                .unwrap(),
        );
        policy.set_enabled("example.plugin", true).await.unwrap();
        let market = Arc::new(FakeMarket { searches: AtomicUsize::new(0) });
        let dispatcher = CoreCapabilityDispatcher::new(policy, market.clone());
        let invocation = CapabilityInvocation {
            principal: ExecutionPrincipal::Plugin("example.plugin".to_string()),
            trust_source: TrustSource::UntrustedLocal,
            capability: CapabilityId::new("market.search").unwrap(),
            scope: None,
            declared: true,
            payload: serde_json::json!({"source":"Modrinth", "query":"paper", "page":1, "pageSize":20}),
            approval_token: None,
            request_id: "request-1".to_string(),
        };
        assert!(dispatcher.invoke(invocation).await.is_ok());
        assert_eq!(market.searches.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn market_resource_requires_matching_artifact_scope() {
        let root = tempfile::tempdir().unwrap();
        let policy = Arc::new(
            PluginPolicyStore::open(root.path().join("state.sqlite"))
                .await
                .unwrap(),
        );
        policy.set_enabled("example.plugin", true).await.unwrap();
        let dispatcher = CoreCapabilityDispatcher::new(
            policy,
            Arc::new(FakeMarket { searches: AtomicUsize::new(0) }),
        );
        let invocation = CapabilityInvocation {
            principal: ExecutionPrincipal::Plugin("example.plugin".to_string()),
            trust_source: TrustSource::UntrustedLocal,
            capability: CapabilityId::new("market.resource.read").unwrap(),
            scope: Some(ScopeBinding::new(ScopeKind::MarketArtifact, "modrinth:paper").unwrap()),
            declared: false,
            payload: Value::Null,
            approval_token: None,
            request_id: "request-2".to_string(),
        };
        assert!(matches!(
            dispatcher.invoke(invocation).await,
            Err(CapabilityDispatchError::Denied(
                sealantern_core::app_plugin::PolicyDenialReason::CapabilityNotDeclared
            ))
        ));
    }
}
