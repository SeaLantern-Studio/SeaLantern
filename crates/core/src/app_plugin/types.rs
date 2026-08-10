use std::fmt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 插件能力的风险等级，顺序同时表示风险递增关系。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RiskLevel {
    L0,
    L1,
    L2,
    M1,
    M2,
    H1,
    H2,
}

/// 能力授权的生命周期。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantKind {
    None,
    Session,
    Persistent,
}

/// 用户确认要求。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalMode {
    None,
    PerSession,
    PerCall,
}

/// 能力调用的审计要求。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditMode {
    None,
    Default,
    Required,
}

/// 执行请求的身份主体。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum ExecutionPrincipal {
    Plugin(String),
    AgentSession(String),
    BuiltInHost,
}

impl ExecutionPrincipal {
    /// 返回适合持久化和审计的稳定主体类型。
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Plugin(_) => "plugin",
            Self::AgentSession(_) => "agent_session",
            Self::BuiltInHost => "built_in_host",
        }
    }

    /// 返回主体标识；内建宿主使用固定标识，避免空值分支扩散。
    pub fn id(&self) -> &str {
        match self {
            Self::Plugin(id) | Self::AgentSession(id) => id,
            Self::BuiltInHost => "host",
        }
    }
}

/// 插件包来源的宿主判定结果，不能从 manifest 反序列化后直接信任。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustSource {
    UntrustedLocal,
    StandardMarketplace,
    VerifiedPublisher,
    LocallyTrusted,
    BuiltIn,
}

impl TrustSource {
    /// L2/M2 只接受宿主或用户明确建立的信任。
    pub const fn permits_scoped_capabilities(self) -> bool {
        matches!(self, Self::VerifiedPublisher | Self::LocallyTrusted | Self::BuiltIn)
    }
}

/// 能力资源的作用域类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    PluginData,
    PluginBundle,
    ServerInstance,
    AppGlobal,
    NetworkOrigin,
    UiExtension,
    HostElement,
    MarketArtifact,
    ApprovedExecutable,
}

/// 经过结构化解析的具体作用域绑定。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScopeBinding {
    pub kind: ScopeKind,
    pub value: String,
}

impl ScopeBinding {
    pub fn new(kind: ScopeKind, value: impl Into<String>) -> Result<Self, ScopeBindingError> {
        let value = value.into();
        if value.is_empty() || value.len() > 2048 {
            return Err(ScopeBindingError::InvalidLength);
        }
        if value.chars().any(char::is_control) {
            return Err(ScopeBindingError::ControlCharacter);
        }
        Ok(Self { kind, value })
    }
}

/// 作用域绑定校验错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeBindingError {
    InvalidLength,
    ControlCharacter,
}

impl fmt::Display for ScopeBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength => formatter.write_str("scope value must contain 1 to 2048 bytes"),
            Self::ControlCharacter => {
                formatter.write_str("scope value contains control characters")
            }
        }
    }
}

impl std::error::Error for ScopeBindingError {}

/// 经过语法校验的点分能力标识。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CapabilityId(String);

impl CapabilityId {
    pub fn new(value: impl Into<String>) -> Result<Self, CapabilityIdError> {
        let value = value.into();
        if !valid_capability_id(&value) {
            return Err(CapabilityIdError);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityIdError;

impl fmt::Display for CapabilityIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("capability ID must be lowercase ASCII dot-separated segments")
    }
}

impl std::error::Error for CapabilityIdError {}

fn valid_capability_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.split('.').all(|segment| {
            !segment.is_empty()
                && segment.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'_' | b'-')
                })
        })
}

/// 已完成 manifest 解析的能力调用。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityInvocation {
    pub principal: ExecutionPrincipal,
    pub trust_source: TrustSource,
    pub capability: CapabilityId,
    pub scope: Option<ScopeBinding>,
    pub declared: bool,
    #[serde(default)]
    pub payload: Value,
    pub approval_token: Option<String>,
    pub request_id: String,
}

/// 宿主能力执行端口，由应用层实现并注入插件运行时。
#[async_trait]
pub trait CapabilityDispatcher: Send + Sync {
    async fn invoke(
        &self,
        invocation: CapabilityInvocation,
    ) -> Result<Value, CapabilityDispatchError>;
}

/// 能力执行向运行时公开的稳定错误分类。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityDispatchError {
    Denied(PolicyDenialReason),
    InvalidRequest(&'static str),
    Unavailable(&'static str),
    Failed(&'static str),
}

impl fmt::Display for CapabilityDispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Denied(reason) => write!(formatter, "capability denied: {}", reason.as_str()),
            Self::InvalidRequest(subject) => {
                write!(formatter, "invalid capability request: {subject}")
            }
            Self::Unavailable(subject) => write!(formatter, "capability unavailable: {subject}"),
            Self::Failed(subject) => write!(formatter, "capability failed: {subject}"),
        }
    }
}

impl std::error::Error for CapabilityDispatchError {}

/// 策略拒绝的机器可读原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDenialReason {
    PluginNotEnabled,
    UnknownCapability,
    CapabilityNotDeclared,
    ScopeRequired,
    ScopeNotAllowed,
    UnsupportedRiskBoundary,
    ExplicitTrustRequired,
    PersistentGrantRequired,
    SessionGrantRequired,
    SessionApprovalRequired,
    SingleUseApprovalRequired,
}

impl PolicyDenialReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PluginNotEnabled => "plugin_not_enabled",
            Self::UnknownCapability => "unknown_capability",
            Self::CapabilityNotDeclared => "capability_not_declared",
            Self::ScopeRequired => "scope_required",
            Self::ScopeNotAllowed => "scope_not_allowed",
            Self::UnsupportedRiskBoundary => "unsupported_risk_boundary",
            Self::ExplicitTrustRequired => "explicit_trust_required",
            Self::PersistentGrantRequired => "persistent_grant_required",
            Self::SessionGrantRequired => "session_grant_required",
            Self::SessionApprovalRequired => "session_approval_required",
            Self::SingleUseApprovalRequired => "single_use_approval_required",
        }
    }
}

/// 纯策略求值结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny(PolicyDenialReason),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_ids_require_lowercase_dot_segments() {
        assert!(CapabilityId::new("server.status.read").is_ok());
        for value in ["Server.status", "server..read", ".server", "server/read", ""] {
            assert!(CapabilityId::new(value).is_err(), "{value} should be rejected");
        }
    }

    #[test]
    fn scope_values_reject_control_characters() {
        assert!(ScopeBinding::new(ScopeKind::ServerInstance, "alpha").is_ok());
        assert!(ScopeBinding::new(ScopeKind::ServerInstance, "alpha\nsecret").is_err());
    }
}
