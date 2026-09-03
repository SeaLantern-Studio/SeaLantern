//! 插件授权状态、审批令牌和审计记录。

mod dispatcher;
mod policy;
mod service;

pub use dispatcher::{
    ApplicationPluginReadHost, CoreCapabilityDispatcher, DefaultMarketGateway, MarketGateway,
    PluginReadHost,
};
pub use policy::{AuditEntry, PluginPolicyError, PluginPolicyStore, SessionApproval, SessionGrant};
pub use service::{CorePluginService, PluginService, PluginServiceError};
