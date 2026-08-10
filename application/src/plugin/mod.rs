//! 插件授权状态、审批令牌和审计记录。

mod policy;

pub use policy::{AuditEntry, PluginPolicyError, PluginPolicyStore, SessionApproval, SessionGrant};
