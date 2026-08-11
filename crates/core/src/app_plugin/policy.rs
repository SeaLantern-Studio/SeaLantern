use super::{
    capability, ApprovalMode, GrantKind, PolicyDecision, PolicyDenialReason, RiskLevel,
    ScopeBinding, TrustSource,
};

/// 应用层完成授权、审批查询后交给纯策略求值器的事实集合。
#[derive(Debug, Clone, Copy)]
pub struct PolicyFacts<'a> {
    pub capability_id: &'a str,
    pub scope: Option<&'a ScopeBinding>,
    pub declared: bool,
    pub trust_source: TrustSource,
    pub grant: Option<GrantKind>,
    pub session_approved: bool,
    pub single_use_approved: bool,
}

/// 按固定顺序求值，确保高风险边界、作用域和信任检查不能被授权记录绕过。
pub fn evaluate(facts: PolicyFacts<'_>) -> PolicyDecision {
    let Some(descriptor) = capability(facts.capability_id) else {
        return PolicyDecision::Deny(PolicyDenialReason::UnknownCapability);
    };

    if !descriptor.enabled || descriptor.risk >= RiskLevel::H1 {
        return PolicyDecision::Deny(PolicyDenialReason::UnsupportedRiskBoundary);
    }
    if descriptor.declaration_required && !facts.declared {
        return PolicyDecision::Deny(PolicyDenialReason::CapabilityNotDeclared);
    }
    match (descriptor.scope, facts.scope) {
        (Some(_), None) => return PolicyDecision::Deny(PolicyDenialReason::ScopeRequired),
        (Some(expected), Some(actual)) if expected != actual.kind => {
            return PolicyDecision::Deny(PolicyDenialReason::ScopeNotAllowed);
        }
        (None, Some(_)) => return PolicyDecision::Deny(PolicyDenialReason::ScopeNotAllowed),
        _ => {}
    }
    if descriptor.risk >= RiskLevel::L2 && !facts.trust_source.permits_scoped_capabilities() {
        return PolicyDecision::Deny(PolicyDenialReason::ExplicitTrustRequired);
    }
    match descriptor.grant {
        GrantKind::None => {}
        GrantKind::Persistent if facts.grant != Some(GrantKind::Persistent) => {
            return PolicyDecision::Deny(PolicyDenialReason::PersistentGrantRequired);
        }
        GrantKind::Session if facts.grant != Some(GrantKind::Session) => {
            return PolicyDecision::Deny(PolicyDenialReason::SessionGrantRequired);
        }
        GrantKind::Persistent | GrantKind::Session => {}
    }
    match descriptor.approval {
        ApprovalMode::None => {}
        ApprovalMode::PerSession if !facts.session_approved => {
            return PolicyDecision::Deny(PolicyDenialReason::SessionApprovalRequired);
        }
        ApprovalMode::PerCall if !facts.single_use_approved => {
            return PolicyDecision::Deny(PolicyDenialReason::SingleUseApprovalRequired);
        }
        ApprovalMode::PerSession | ApprovalMode::PerCall => {}
    }
    PolicyDecision::Allow
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_plugin::ScopeKind;

    fn scope(kind: ScopeKind) -> ScopeBinding {
        ScopeBinding::new(kind, "alpha").unwrap()
    }

    #[test]
    fn l0_log_is_available_without_manifest_declaration() {
        assert_eq!(
            evaluate(PolicyFacts {
                capability_id: "plugin.log.emit",
                scope: None,
                declared: false,
                trust_source: TrustSource::UntrustedLocal,
                grant: None,
                session_approved: false,
                single_use_approved: false,
            }),
            PolicyDecision::Allow
        );
    }

    #[test]
    fn m2_requires_scope_trust_session_grant_and_single_use_approval() {
        let server_scope = scope(ScopeKind::ServerInstance);
        let base = PolicyFacts {
            capability_id: "server.lifecycle.restart",
            scope: Some(&server_scope),
            declared: true,
            trust_source: TrustSource::LocallyTrusted,
            grant: Some(GrantKind::Session),
            session_approved: false,
            single_use_approved: true,
        };
        assert_eq!(evaluate(base), PolicyDecision::Allow);

        assert_eq!(
            evaluate(PolicyFacts {
                trust_source: TrustSource::UntrustedLocal,
                ..base
            }),
            PolicyDecision::Deny(PolicyDenialReason::ExplicitTrustRequired)
        );
        assert_eq!(
            evaluate(PolicyFacts {
                grant: Some(GrantKind::Persistent),
                ..base
            }),
            PolicyDecision::Deny(PolicyDenialReason::SessionGrantRequired)
        );
        assert_eq!(
            evaluate(PolicyFacts { single_use_approved: false, ..base }),
            PolicyDecision::Deny(PolicyDenialReason::SingleUseApprovalRequired)
        );
    }

    #[test]
    fn disabled_high_risk_capability_cannot_be_overridden_by_facts() {
        let scope = scope(ScopeKind::ApprovedExecutable);
        assert_eq!(
            evaluate(PolicyFacts {
                capability_id: "process.execute",
                scope: Some(&scope),
                declared: true,
                trust_source: TrustSource::BuiltIn,
                grant: Some(GrantKind::Persistent),
                session_approved: true,
                single_use_approved: true,
            }),
            PolicyDecision::Deny(PolicyDenialReason::UnsupportedRiskBoundary)
        );
    }

    #[test]
    fn scope_kind_is_checked_before_grants() {
        let wrong_scope = scope(ScopeKind::NetworkOrigin);
        assert_eq!(
            evaluate(PolicyFacts {
                capability_id: "server.logs.read",
                scope: Some(&wrong_scope),
                declared: true,
                trust_source: TrustSource::LocallyTrusted,
                grant: Some(GrantKind::Persistent),
                session_approved: true,
                single_use_approved: true,
            }),
            PolicyDecision::Deny(PolicyDenialReason::ScopeNotAllowed)
        );
    }
}
