use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sealantern_core::app_plugin::{
    evaluate, GrantKind, PolicyDecision, PolicyDenialReason, PolicyFacts, ScopeBinding, TrustSource,
};
use sealantern_infra::persistence::{Migration, PersistenceError, SqlValue, SqliteDatabase};
use tokio::sync::Mutex;
use uuid::Uuid;

const TOKEN_TTL: Duration = Duration::from_secs(5 * 60);
const MAX_AUDIT_ROWS: i64 = 10_000;

/// 插件授权状态服务的错误。
#[derive(Debug)]
pub enum PluginPolicyError {
    Persistence(PersistenceError),
    InvalidInput(&'static str),
    Denied(PolicyDenialReason),
    TokenExpired,
    TokenUnknown,
}

impl std::fmt::Display for PluginPolicyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Persistence(error) => {
                write!(formatter, "plugin policy persistence failed: {error}")
            }
            Self::InvalidInput(subject) => {
                write!(formatter, "invalid plugin policy input: {subject}")
            }
            Self::Denied(reason) => write!(formatter, "plugin policy denied: {}", reason.as_str()),
            Self::TokenExpired => formatter.write_str("plugin approval token expired"),
            Self::TokenUnknown => {
                formatter.write_str("plugin approval token is unknown or already used")
            }
        }
    }
}

impl std::error::Error for PluginPolicyError {}

impl From<PersistenceError> for PluginPolicyError {
    fn from(error: PersistenceError) -> Self {
        Self::Persistence(error)
    }
}

/// 一条脱敏后的插件审计事件。
#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct AuditEntry {
    pub id: i64,
    pub plugin_id: String,
    pub capability_id: String,
    pub scope: Option<ScopeBinding>,
    pub outcome: String,
    pub detail: String,
    pub created_at_unix_secs: i64,
}

/// 会话授权记录。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SessionGrant {
    pub session_id: String,
    pub plugin_id: String,
    pub capability_id: String,
    pub scope: Option<ScopeBinding>,
}

/// 已签发的会话审批状态；token 只在内存中保存。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct SessionApproval {
    pub session_id: String,
    pub plugin_id: String,
    pub capability_id: String,
    pub scope: Option<ScopeBinding>,
}

#[derive(Debug, Clone)]
struct ApprovalToken {
    session_id: String,
    plugin_id: String,
    capability_id: String,
    scope: Option<ScopeBinding>,
    expires_at: SystemTime,
}

#[derive(Default)]
struct SessionState {
    grants: HashSet<SessionGrantKey>,
    approvals: HashSet<SessionApprovalKey>,
    tokens: HashMap<String, ApprovalToken>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SessionGrantKey {
    plugin_id: String,
    capability_id: String,
    scope: Option<ScopeBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SessionApprovalKey {
    plugin_id: String,
    capability_id: String,
    scope: Option<ScopeBinding>,
}

/// 插件策略的 SQLite 状态和进程内会话状态。
#[derive(Clone)]
pub struct PluginPolicyStore {
    database: SqliteDatabase,
    sessions: Arc<Mutex<HashMap<String, SessionState>>>,
}

impl PluginPolicyStore {
    /// 打开状态库并执行版本化迁移。
    pub async fn open(path: impl Into<PathBuf>) -> Result<Self, PluginPolicyError> {
        let database = SqliteDatabase::open(path).await?;
        database
            .migrate(vec![Migration {
                version: 1,
                name: "create plugin policy state",
                sql: "CREATE TABLE IF NOT EXISTS plugin_policy_grants (\
                    plugin_id TEXT NOT NULL,\
                    capability_id TEXT NOT NULL,\
                    scope_kind TEXT NOT NULL,\
                    scope_value TEXT NOT NULL,\
                    granted_at INTEGER NOT NULL,\
                    revoked_at INTEGER,\
                    PRIMARY KEY (plugin_id, capability_id, scope_kind, scope_value)\
                 );\
                 CREATE TABLE IF NOT EXISTS plugin_policy_trust (\
                    plugin_id TEXT PRIMARY KEY NOT NULL,\
                    trust_source TEXT NOT NULL,\
                    updated_at INTEGER NOT NULL\
                 );\
                 CREATE TABLE IF NOT EXISTS plugin_policy_plugins (\
                    plugin_id TEXT PRIMARY KEY NOT NULL,\
                    enabled INTEGER NOT NULL DEFAULT 0,\
                    updated_at INTEGER NOT NULL\
                 );\
                 CREATE TABLE IF NOT EXISTS plugin_policy_audit (\
                    id INTEGER PRIMARY KEY AUTOINCREMENT,\
                    plugin_id TEXT NOT NULL,\
                    capability_id TEXT NOT NULL,\
                    scope_kind TEXT,\
                    scope_value TEXT,\
                    outcome TEXT NOT NULL,\
                    detail TEXT NOT NULL,\
                    created_at INTEGER NOT NULL\
                 );",
            }])
            .await?;
        Ok(Self {
            database,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn database_path(&self) -> &Path {
        self.database.path()
    }

    /// 写入或恢复一个持久能力授权。
    pub async fn grant_persistent(
        &self,
        plugin_id: &str,
        capability_id: &str,
        scope: Option<&ScopeBinding>,
    ) -> Result<(), PluginPolicyError> {
        validate_ids(plugin_id, capability_id)?;
        let (kind, value) = scope_columns(scope);
        self.database
            .execute(
                "INSERT INTO plugin_policy_grants(plugin_id, capability_id, scope_kind, scope_value, granted_at, revoked_at) \
                 VALUES(?1, ?2, ?3, ?4, ?5, NULL) \
                 ON CONFLICT(plugin_id, capability_id, scope_kind, scope_value) \
                 DO UPDATE SET granted_at=excluded.granted_at, revoked_at=NULL",
                params(vec![
                    text(plugin_id),
                    text(capability_id),
                    text(&kind),
                    text(&value),
                    SqlValue::Integer(now()),
                ]),
            )
            .await?;
        Ok(())
    }

    pub async fn revoke_persistent(
        &self,
        plugin_id: &str,
        capability_id: &str,
        scope: Option<&ScopeBinding>,
    ) -> Result<(), PluginPolicyError> {
        validate_ids(plugin_id, capability_id)?;
        let (kind, value) = scope_columns(scope);
        self.database
            .execute(
                "UPDATE plugin_policy_grants SET revoked_at=?5 WHERE plugin_id=?1 AND capability_id=?2 \
                 AND scope_kind = ?3 AND scope_value = ?4",
                params(vec![
                    text(plugin_id),
                    text(capability_id),
                    text(&kind),
                    text(&value),
                    SqlValue::Integer(now()),
                ]),
            )
            .await?;
        Ok(())
    }

    pub async fn set_trust(
        &self,
        plugin_id: &str,
        source: TrustSource,
    ) -> Result<(), PluginPolicyError> {
        validate_plugin_id(plugin_id)?;
        self.database
            .execute(
                "INSERT INTO plugin_policy_trust(plugin_id, trust_source, updated_at) VALUES(?1, ?2, ?3) \
                 ON CONFLICT(plugin_id) DO UPDATE SET trust_source=excluded.trust_source, updated_at=excluded.updated_at",
                params(vec![text(plugin_id), text(trust_name(source)), SqlValue::Integer(now())]),
            )
            .await?;
        Ok(())
    }

    pub async fn trust_source(&self, plugin_id: &str) -> Result<TrustSource, PluginPolicyError> {
        validate_plugin_id(plugin_id)?;
        let rows = self
            .database
            .query(
                "SELECT trust_source FROM plugin_policy_trust WHERE plugin_id=?1",
                params(vec![text(plugin_id)]),
                |row| row.get::<_, String>(0),
            )
            .await?;
        rows.first()
            .map(|value| parse_trust(value))
            .transpose()
            .map(|value| value.unwrap_or(TrustSource::UntrustedLocal))
    }

    pub async fn set_enabled(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<(), PluginPolicyError> {
        validate_plugin_id(plugin_id)?;
        self.database
            .execute(
                "INSERT INTO plugin_policy_plugins(plugin_id, enabled, updated_at) VALUES(?1, ?2, ?3) \
                 ON CONFLICT(plugin_id) DO UPDATE SET enabled=excluded.enabled, updated_at=excluded.updated_at",
                params(vec![
                    text(plugin_id),
                    SqlValue::Integer(i64::from(enabled)),
                    SqlValue::Integer(now()),
                ]),
            )
            .await?;
        Ok(())
    }

    pub async fn is_enabled(&self, plugin_id: &str) -> Result<bool, PluginPolicyError> {
        validate_plugin_id(plugin_id)?;
        let rows = self
            .database
            .query(
                "SELECT enabled FROM plugin_policy_plugins WHERE plugin_id=?1",
                params(vec![text(plugin_id)]),
                |row| row.get::<_, i64>(0),
            )
            .await?;
        Ok(rows.first().copied().unwrap_or(0) != 0)
    }

    /// 为当前会话授予临时能力。
    pub async fn grant_session(&self, grant: SessionGrant) -> Result<(), PluginPolicyError> {
        validate_ids(&grant.plugin_id, &grant.capability_id)?;
        let mut sessions = self.sessions.lock().await;
        sessions
            .entry(grant.session_id)
            .or_default()
            .grants
            .insert(SessionGrantKey {
                plugin_id: grant.plugin_id,
                capability_id: grant.capability_id,
                scope: grant.scope,
            });
        Ok(())
    }

    pub async fn approve_session(
        &self,
        approval: SessionApproval,
    ) -> Result<(), PluginPolicyError> {
        validate_ids(&approval.plugin_id, &approval.capability_id)?;
        let mut sessions = self.sessions.lock().await;
        let session = sessions.entry(approval.session_id).or_default();
        let key = SessionApprovalKey {
            plugin_id: approval.plugin_id,
            capability_id: approval.capability_id,
            scope: approval.scope.clone(),
        };
        session.approvals.insert(key.clone());
        Ok(())
    }

    /// 签发一个与具体能力和 scope 绑定的单次审批 token。
    pub async fn issue_single_use_token(
        &self,
        session_id: &str,
        plugin_id: &str,
        capability_id: &str,
        scope: Option<ScopeBinding>,
    ) -> Result<String, PluginPolicyError> {
        validate_ids(plugin_id, capability_id)?;
        let token = Uuid::new_v4().to_string();
        let mut sessions = self.sessions.lock().await;
        sessions
            .entry(session_id.to_owned())
            .or_default()
            .tokens
            .insert(
                token.clone(),
                ApprovalToken {
                    session_id: session_id.to_owned(),
                    plugin_id: plugin_id.to_owned(),
                    capability_id: capability_id.to_owned(),
                    scope,
                    expires_at: SystemTime::now() + TOKEN_TTL,
                },
            );
        Ok(token)
    }

    /// 消费单次 token；不匹配、过期或重复使用均失败。
    pub async fn consume_single_use_token(
        &self,
        session_id: &str,
        token: &str,
        plugin_id: &str,
        capability_id: &str,
        scope: Option<&ScopeBinding>,
    ) -> Result<(), PluginPolicyError> {
        let mut sessions = self.sessions.lock().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or(PluginPolicyError::TokenUnknown)?;
        let approval = session
            .tokens
            .remove(token)
            .ok_or(PluginPolicyError::TokenUnknown)?;
        if approval.expires_at <= SystemTime::now() {
            return Err(PluginPolicyError::TokenExpired);
        }
        if approval.session_id != session_id
            || approval.plugin_id != plugin_id
            || approval.capability_id != capability_id
            || approval.scope.as_ref() != scope
        {
            return Err(PluginPolicyError::TokenUnknown);
        }
        Ok(())
    }

    /// 将授权事实交给 core 策略求值器，并返回可记录的结果。
    pub async fn evaluate(
        &self,
        session_id: Option<&str>,
        plugin_id: &str,
        capability_id: &str,
        scope: Option<&ScopeBinding>,
        declared: bool,
        single_use_approved: bool,
    ) -> Result<PolicyDecision, PluginPolicyError> {
        if !self.is_enabled(plugin_id).await? {
            return Ok(PolicyDecision::Deny(PolicyDenialReason::PluginNotEnabled));
        }
        let trust = self.trust_source(plugin_id).await?;
        let persistent = self
            .has_persistent_grant(plugin_id, capability_id, scope)
            .await?;
        let session = if let Some(session_id) = session_id {
            let sessions = self.sessions.lock().await;
            sessions.get(session_id).map(|state| {
                let key = SessionGrantKey {
                    plugin_id: plugin_id.to_owned(),
                    capability_id: capability_id.to_owned(),
                    scope: scope.cloned(),
                };
                let approval = SessionApprovalKey {
                    plugin_id: plugin_id.to_owned(),
                    capability_id: capability_id.to_owned(),
                    scope: scope.cloned(),
                };
                (state.grants.contains(&key), state.approvals.contains(&approval))
            })
        } else {
            None
        };
        let descriptor = sealantern_core::app_plugin::capability(capability_id);
        let grant = match descriptor.map(|value| value.grant) {
            Some(GrantKind::Persistent) if persistent => Some(GrantKind::Persistent),
            Some(GrantKind::Session) if session.is_some_and(|value| value.0) => {
                Some(GrantKind::Session)
            }
            _ => None,
        };
        let session_approved = session.is_some_and(|value| value.1);
        let decision = evaluate(PolicyFacts {
            capability_id,
            scope,
            declared,
            trust_source: trust,
            grant,
            session_approved,
            single_use_approved,
        });
        self.audit(
            plugin_id,
            capability_id,
            scope,
            match decision {
                PolicyDecision::Allow => "allow",
                PolicyDecision::Deny(_) => "deny",
            },
            match decision {
                PolicyDecision::Allow => "policy accepted",
                PolicyDecision::Deny(reason) => reason.as_str(),
            },
        )
        .await?;
        Ok(decision)
    }

    pub async fn audit_entries(&self, limit: u32) -> Result<Vec<AuditEntry>, PluginPolicyError> {
        let limit = i64::from(limit.clamp(1, 500));
        self.database
            .query(
                "SELECT id, plugin_id, capability_id, scope_kind, scope_value, outcome, detail, created_at \
                 FROM plugin_policy_audit ORDER BY id DESC LIMIT ?1",
                params(vec![SqlValue::Integer(limit)]),
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, String>(6)?,
                        row.get::<_, i64>(7)?,
                    ))
                },
            )
            .await?
            .into_iter()
            .map(|(id, plugin_id, capability_id, kind, value, outcome, detail, created_at)| {
                let scope = match (kind, value) {
                    (Some(kind), Some(value)) if kind.is_empty() && value.is_empty() => None,
                    (Some(kind), Some(value)) => Some(
                        ScopeBinding::new(parse_scope_kind(&kind)?, value)
                            .map_err(|_| PluginPolicyError::InvalidInput("stored audit scope"))?,
                    ),
                    (None, None) => None,
                    _ => return Err(PluginPolicyError::InvalidInput("stored audit scope")),
                };
                Ok(AuditEntry {
                    id,
                    plugin_id,
                    capability_id,
                    scope,
                    outcome,
                    detail,
                    created_at_unix_secs: created_at,
                })
            })
            .collect()
    }

    async fn has_persistent_grant(
        &self,
        plugin_id: &str,
        capability_id: &str,
        scope: Option<&ScopeBinding>,
    ) -> Result<bool, PluginPolicyError> {
        let (kind, value) = scope_columns(scope);
        let rows = self
            .database
            .query(
                "SELECT 1 FROM plugin_policy_grants WHERE plugin_id=?1 AND capability_id=?2 \
                 AND scope_kind = ?3 AND scope_value = ?4 AND revoked_at IS NULL LIMIT 1",
                params(vec![text(plugin_id), text(capability_id), text(&kind), text(&value)]),
                |_| Ok(()),
            )
            .await?;
        Ok(!rows.is_empty())
    }

    async fn audit(
        &self,
        plugin_id: &str,
        capability_id: &str,
        scope: Option<&ScopeBinding>,
        outcome: &str,
        detail: &str,
    ) -> Result<(), PluginPolicyError> {
        let (kind, value) = scope_columns(scope);
        let detail = redact_detail(detail);
        self.database
            .execute(
                "INSERT INTO plugin_policy_audit(plugin_id, capability_id, scope_kind, scope_value, outcome, detail, created_at) \
                 VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params(vec![
                    text(plugin_id),
                    text(capability_id),
                    text(&kind),
                    text(&value),
                    text(outcome),
                    text(&detail),
                    SqlValue::Integer(now()),
                ]),
            )
            .await?;
        self.database
            .execute(
                "DELETE FROM plugin_policy_audit WHERE id NOT IN (SELECT id FROM plugin_policy_audit ORDER BY id DESC LIMIT ?1)",
                params(vec![SqlValue::Integer(MAX_AUDIT_ROWS)]),
            )
            .await?;
        Ok(())
    }
}

fn validate_ids(plugin_id: &str, capability_id: &str) -> Result<(), PluginPolicyError> {
    validate_plugin_id(plugin_id)?;
    sealantern_core::app_plugin::CapabilityId::new(capability_id)
        .map(|_| ())
        .map_err(|_| PluginPolicyError::InvalidInput("capability_id"))
}

fn validate_plugin_id(plugin_id: &str) -> Result<(), PluginPolicyError> {
    if plugin_id.is_empty()
        || plugin_id.len() > 128
        || !plugin_id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
    {
        return Err(PluginPolicyError::InvalidInput("plugin_id"));
    }
    Ok(())
}

fn scope_columns(scope: Option<&ScopeBinding>) -> (String, String) {
    scope.map_or_else(
        || (String::new(), String::new()),
        |scope| (scope_kind_name(scope.kind).to_owned(), scope.value.clone()),
    )
}

fn scope_kind_name(value: sealantern_core::app_plugin::ScopeKind) -> &'static str {
    use sealantern_core::app_plugin::ScopeKind;
    match value {
        ScopeKind::PluginData => "plugin_data",
        ScopeKind::PluginBundle => "plugin_bundle",
        ScopeKind::ServerInstance => "server_instance",
        ScopeKind::AppGlobal => "app_global",
        ScopeKind::NetworkOrigin => "network_origin",
        ScopeKind::UiExtension => "ui_extension",
        ScopeKind::HostElement => "host_element",
        ScopeKind::MarketArtifact => "market_artifact",
        ScopeKind::ApprovedExecutable => "approved_executable",
    }
}

fn parse_scope_kind(
    value: &str,
) -> Result<sealantern_core::app_plugin::ScopeKind, PluginPolicyError> {
    use sealantern_core::app_plugin::ScopeKind;
    match value {
        "plugin_data" => Ok(ScopeKind::PluginData),
        "plugin_bundle" => Ok(ScopeKind::PluginBundle),
        "server_instance" => Ok(ScopeKind::ServerInstance),
        "app_global" => Ok(ScopeKind::AppGlobal),
        "network_origin" => Ok(ScopeKind::NetworkOrigin),
        "ui_extension" => Ok(ScopeKind::UiExtension),
        "host_element" => Ok(ScopeKind::HostElement),
        "market_artifact" => Ok(ScopeKind::MarketArtifact),
        "approved_executable" => Ok(ScopeKind::ApprovedExecutable),
        _ => Err(PluginPolicyError::InvalidInput("stored scope kind")),
    }
}

fn trust_name(value: TrustSource) -> &'static str {
    match value {
        TrustSource::UntrustedLocal => "untrusted_local",
        TrustSource::StandardMarketplace => "standard_marketplace",
        TrustSource::VerifiedPublisher => "verified_publisher",
        TrustSource::LocallyTrusted => "locally_trusted",
        TrustSource::BuiltIn => "built_in",
    }
}

fn parse_trust(value: &str) -> Result<TrustSource, PluginPolicyError> {
    match value {
        "untrusted_local" => Ok(TrustSource::UntrustedLocal),
        "standard_marketplace" => Ok(TrustSource::StandardMarketplace),
        "verified_publisher" => Ok(TrustSource::VerifiedPublisher),
        "locally_trusted" => Ok(TrustSource::LocallyTrusted),
        "built_in" => Ok(TrustSource::BuiltIn),
        _ => Err(PluginPolicyError::InvalidInput("stored trust source")),
    }
}

fn redact_detail(value: &str) -> String {
    ["password", "token", "secret", "authorization", "cookie", "private_key"]
        .iter()
        .fold(value.to_owned(), |value, key| {
            value
                .split_once('=')
                .filter(|(left, _)| left.trim().eq_ignore_ascii_case(key))
                .map_or(value.clone(), |(left, _)| format!("{left}=<redacted>"))
        })
}

fn text(value: &str) -> SqlValue {
    SqlValue::Text(value.to_owned())
}

fn params(values: Vec<SqlValue>) -> impl IntoIterator<Item = SqlValue> + Send + 'static {
    values
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use sealantern_core::app_plugin::{CapabilityId, ScopeKind};

    fn path(label: &str) -> PathBuf {
        std::env::temp_dir()
            .join(format!("sealantern-plugin-policy-{label}-{}", Uuid::new_v4()))
            .join("plugin-state.sqlite")
    }

    fn scope() -> ScopeBinding {
        ScopeBinding::new(ScopeKind::ServerInstance, "server-a").unwrap()
    }

    #[tokio::test]
    async fn persists_grants_trust_and_enabled_state() {
        let database = path("persist");
        let store = PluginPolicyStore::open(&database).await.unwrap();
        store
            .grant_persistent("example.plugin", "server.status.read", Some(&scope()))
            .await
            .unwrap();
        store
            .set_trust("example.plugin", TrustSource::LocallyTrusted)
            .await
            .unwrap();
        store.set_enabled("example.plugin", true).await.unwrap();
        drop(store);
        let restored = PluginPolicyStore::open(&database).await.unwrap();
        assert_eq!(
            restored.trust_source("example.plugin").await.unwrap(),
            TrustSource::LocallyTrusted
        );
        assert!(restored.is_enabled("example.plugin").await.unwrap());
        assert_eq!(
            restored
                .evaluate(None, "example.plugin", "server.status.read", Some(&scope()), true, false)
                .await
                .unwrap(),
            PolicyDecision::Allow
        );
        cleanup(database);
    }

    #[tokio::test]
    async fn single_use_token_is_bound_and_consumed_once() {
        let database = path("token");
        let store = PluginPolicyStore::open(&database).await.unwrap();
        let token = store
            .issue_single_use_token(
                "session-1",
                "example.plugin",
                "server.console.send",
                Some(scope()),
            )
            .await
            .unwrap();
        store
            .consume_single_use_token(
                "session-1",
                &token,
                "example.plugin",
                "server.console.send",
                Some(&scope()),
            )
            .await
            .unwrap();
        assert!(matches!(
            store
                .consume_single_use_token(
                    "session-1",
                    &token,
                    "example.plugin",
                    "server.console.send",
                    Some(&scope())
                )
                .await,
            Err(PluginPolicyError::TokenUnknown)
        ));
        cleanup(database);
    }

    #[tokio::test]
    async fn audit_is_redacted_and_bounded_query_is_returned() {
        let database = path("audit");
        let store = PluginPolicyStore::open(&database).await.unwrap();
        store
            .audit("example.plugin", "plugin.log.emit", None, "deny", "token=super-secret")
            .await
            .unwrap();
        let entries = store.audit_entries(500).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].detail, "token=<redacted>");
        assert!(CapabilityId::new("plugin.log.emit").is_ok());
        cleanup(database);
    }

    fn cleanup(path: PathBuf) {
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}
