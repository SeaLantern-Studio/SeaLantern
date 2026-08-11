use crate::rpc::axum::HttpRpcAccessResolver;
use crate::rpc::{RpcAccess, RpcError, RpcPermission, RpcResult};
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};

const PLUGIN_INVOKE_PERMISSION: &str = "plugin.v2.invoke";

/// Server 插件 RPC 的 bearer token 认证解析器。
#[derive(Clone)]
pub struct PluginRpcTokenResolver {
    token: Option<String>,
    permission: RpcPermission,
}

impl PluginRpcTokenResolver {
    pub fn from_env() -> Self {
        Self {
            token: std::env::var("SEALANTERN_PLUGIN_RPC_TOKEN")
                .ok()
                .filter(|token| token.len() >= 32),
            permission: plugin_invoke_permission(),
        }
    }

    pub fn with_token(token: impl Into<String>) -> Self {
        Self {
            token: Some(token.into()),
            permission: plugin_invoke_permission(),
        }
    }

    fn authenticated(&self, headers: &HeaderMap) -> bool {
        let Some(expected) = self.token.as_deref() else {
            return false;
        };
        let Some(value) = headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
        else {
            return false;
        };
        let Some(provided) = value.strip_prefix("Bearer ") else {
            return false;
        };
        provided.len() == expected.len()
            && provided
                .as_bytes()
                .iter()
                .zip(expected.as_bytes())
                .fold(0u8, |different, (left, right)| different | (left ^ right))
                == 0
    }
}

fn plugin_invoke_permission() -> RpcPermission {
    RpcPermission::new(PLUGIN_INVOKE_PERMISSION)
}

impl HttpRpcAccessResolver for PluginRpcTokenResolver {
    fn resolve(&self, headers: &HeaderMap) -> RpcResult<RpcAccess> {
        if !self.authenticated(headers) {
            return Err(RpcError::permission_denied());
        }
        Ok(RpcAccess::allow([self.permission]))
    }

    fn rejection_status(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_resolver_denies_missing_invalid_and_short_tokens() {
        let resolver = PluginRpcTokenResolver::with_token("a".repeat(32));
        assert!(resolver.resolve(&HeaderMap::new()).is_err());
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, "Bearer wrong".parse().unwrap());
        assert!(resolver.resolve(&headers).is_err());
        headers.insert(AUTHORIZATION, format!("Bearer {}", "a".repeat(32)).parse().unwrap());
        assert!(resolver.resolve(&headers).is_ok());
    }
}
