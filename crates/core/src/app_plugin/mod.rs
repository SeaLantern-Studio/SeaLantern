//! 应用插件能力与安全决策的稳定领域契约。
//!
//! 本模块不执行插件代码、不访问持久化，也不依赖具体宿主。运行时负责构造调用，
//! 应用层负责提供授权事实并执行最终能力。

mod catalog;
mod policy;
mod types;

pub use catalog::{CapabilityDescriptor, capabilities, capability};
pub use policy::{PolicyFacts, evaluate};
pub use types::{
    ApprovalMode, AuditMode, CapabilityDispatchError, CapabilityDispatcher, CapabilityId,
    CapabilityIdError, CapabilityInvocation, ExecutionPrincipal, GrantKind, PolicyDecision,
    PolicyDenialReason, RiskLevel, ScopeBinding, ScopeBindingError, ScopeKind, TrustSource,
};
