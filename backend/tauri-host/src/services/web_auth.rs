use crate::utils::logger::{capture_println, log_warn_ctx};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand_core::OsRng;
use sea_lantern_runtime::{get_or_create_app_data_dir_checked, WEB_AUTH_RECOVERY_TOKEN_ENV};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Mutex, MutexGuard},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const WEB_AUTH_STATE_FILE: &str = "sea_lantern_web_auth_state.json";
const SETUP_TOKEN_TTL_SECS: u64 = 60 * 15;
pub const BROWSER_SESSION_TTL_SECS: u64 = 60 * 60 * 12;
pub const NEXT_BRIDGE_TTL_SECS: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseAuthState {
    Uninitialized,
    SetupPending,
    Initialized,
}

impl BaseAuthState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Uninitialized => "uninitialized",
            Self::SetupPending => "setup_pending",
            Self::Initialized => "initialized",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowserAuthStatusSnapshot {
    pub state: &'static str,
    pub base_state: &'static str,
    pub recovery_active: bool,
    pub setup_required: bool,
    pub password_login_enabled: bool,
    pub session_ttl_seconds: u64,
    pub next_bridge_exchange_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BrowserSessionIssue {
    pub session_token: String,
    pub expires_at: u64,
    pub purpose: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebAuthError {
    Unauthorized,
    InvalidRequest,
    Conflict,
    Unavailable,
}

#[derive(Debug, Clone)]
pub struct WebAuthFailure {
    pub kind: WebAuthError,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct NextBridgeIssue {
    pub bridge_token: String,
    pub expires_at: u64,
}

#[derive(Clone)]
struct BrowserSessionRecord {
    purpose: &'static str,
    expires_at: u64,
}

#[derive(Clone)]
struct NextBridgeTicket {
    purpose: &'static str,
    expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedWebAuthState {
    #[serde(default)]
    password_hash: Option<String>,
    #[serde(default)]
    setup_token_hash: Option<String>,
    #[serde(default)]
    setup_token_expires_at: Option<u64>,
    #[serde(default)]
    password_updated_at: Option<u64>,
}

struct WebAuthInner {
    state: PersistedWebAuthState,
    bootstrap_ready: bool,
    browser_sessions: HashMap<String, BrowserSessionRecord>,
    next_bridge_tokens: HashMap<String, NextBridgeTicket>,
}

pub struct WebAuthService {
    state_path: PathBuf,
    inner: Mutex<WebAuthInner>,
}

impl WebAuthService {
    pub fn new() -> Self {
        let state_path = resolve_state_path().unwrap_or_else(|error| {
            log_warn_ctx(
                "services.web_auth",
                "new",
                &format!("failed to resolve app data dir for web auth state: {}", error),
            );
            std::env::temp_dir().join(WEB_AUTH_STATE_FILE)
        });

        let state = load_state(&state_path).unwrap_or_else(|error| {
            log_warn_ctx(
                "services.web_auth",
                "new",
                &format!("failed to load web auth state, resetting to bootstrap state: {}", error),
            );
            PersistedWebAuthState::default()
        });

        let service = Self {
            state_path,
            inner: Mutex::new(WebAuthInner {
                state,
                bootstrap_ready: false,
                browser_sessions: HashMap::new(),
                next_bridge_tokens: HashMap::new(),
            }),
        };
        service.ensure_bootstrap();
        service
    }

    #[cfg(test)]
    pub fn new_for_test(state_path: PathBuf, seeded_password: Option<&str>) -> Self {
        let mut state = PersistedWebAuthState::default();
        if let Some(password) = seeded_password {
            state.password_hash = Some(hash_password(password).expect("seed test password hash"));
            state.password_updated_at = Some(now_secs());
        }
        let service = Self {
            state_path,
            inner: Mutex::new(WebAuthInner {
                state,
                bootstrap_ready: seeded_password.is_some(),
                browser_sessions: HashMap::new(),
                next_bridge_tokens: HashMap::new(),
            }),
        };
        if let Err(error) = persist_state(&service.state_path, &service.lock_inner().state) {
            panic!("failed to persist test web auth state: {}", error.message);
        }
        service
    }

    pub fn auth_status(&self) -> BrowserAuthStatusSnapshot {
        self.ensure_bootstrap();
        let mut inner = self.lock_inner();
        let now = now_secs();
        prune_setup_token(&mut inner.state, now);
        prune_browser_sessions(&mut inner.browser_sessions, now);
        prune_next_bridge_tokens(&mut inner.next_bridge_tokens, now);

        let base_state = base_state_from(&inner.state, inner.bootstrap_ready);
        let recovery_active = recovery_token().is_some();

        BrowserAuthStatusSnapshot {
            state: if recovery_active {
                "recovery_active"
            } else {
                base_state.as_str()
            },
            base_state: base_state.as_str(),
            recovery_active,
            setup_required: matches!(
                base_state,
                BaseAuthState::SetupPending | BaseAuthState::Uninitialized
            ),
            password_login_enabled: matches!(base_state, BaseAuthState::Initialized),
            session_ttl_seconds: BROWSER_SESSION_TTL_SECS,
            next_bridge_exchange_ttl_seconds: NEXT_BRIDGE_TTL_SECS,
        }
    }

    pub fn initialize_password(
        &self,
        setup_token: &str,
        password: &str,
    ) -> Result<BrowserSessionIssue, WebAuthFailure> {
        self.ensure_bootstrap();
        validate_password(password)?;
        let now = now_secs();

        let mut inner = self.lock_inner();
        prune_setup_token(&mut inner.state, now);

        if !matches!(
            base_state_from(&inner.state, inner.bootstrap_ready),
            BaseAuthState::SetupPending
        ) {
            return Err(conflict("web auth setup is not pending"));
        }

        let Some(expected_hash) = inner.state.setup_token_hash.clone() else {
            return Err(conflict("setup token is unavailable"));
        };
        let Some(expires_at) = inner.state.setup_token_expires_at else {
            return Err(conflict("setup token is unavailable"));
        };
        if expires_at < now || sha256_hex(setup_token.trim()) != expected_hash {
            return Err(unauthorized("invalid setup token"));
        }

        let password_hash = hash_password(password)?;
        inner.state.password_hash = Some(password_hash);
        inner.state.password_updated_at = Some(now);
        inner.state.setup_token_hash = None;
        inner.state.setup_token_expires_at = None;
        persist_state(&self.state_path, &inner.state)?;

        Ok(issue_browser_session(&mut inner.browser_sessions, now))
    }

    pub fn login(&self, password: &str) -> Result<BrowserSessionIssue, WebAuthFailure> {
        self.ensure_bootstrap();
        validate_password(password)?;
        let now = now_secs();

        let mut inner = self.lock_inner();
        prune_browser_sessions(&mut inner.browser_sessions, now);

        if !matches!(
            base_state_from(&inner.state, inner.bootstrap_ready),
            BaseAuthState::Initialized
        ) {
            return Err(conflict("password login is not available"));
        }

        let password_hash = inner
            .state
            .password_hash
            .as_deref()
            .ok_or_else(|| unavailable("password hash is unavailable"))?;

        verify_password(password, password_hash)?;
        Ok(issue_browser_session(&mut inner.browser_sessions, now))
    }

    pub fn recovery_reset(
        &self,
        recovery_token_value: &str,
        new_password: &str,
    ) -> Result<BrowserSessionIssue, WebAuthFailure> {
        self.ensure_bootstrap();
        validate_password(new_password)?;

        let expected_recovery_token =
            recovery_token().ok_or_else(|| conflict("recovery mode is not active"))?;
        if recovery_token_value.trim().is_empty() || recovery_token_value != expected_recovery_token
        {
            return Err(unauthorized("invalid recovery token"));
        }

        let now = now_secs();
        let mut inner = self.lock_inner();
        let password_hash = hash_password(new_password)?;
        inner.state.password_hash = Some(password_hash);
        inner.state.password_updated_at = Some(now);
        inner.state.setup_token_hash = None;
        inner.state.setup_token_expires_at = None;
        inner.browser_sessions.clear();
        inner.next_bridge_tokens.clear();
        persist_state(&self.state_path, &inner.state)?;

        Ok(issue_browser_session(&mut inner.browser_sessions, now))
    }

    pub fn issue_next_bridge_token(&self) -> NextBridgeIssue {
        self.ensure_bootstrap();
        let now = now_secs();
        let bridge_token = format!("next-bridge-{}", Uuid::new_v4().simple());
        let expires_at = now.saturating_add(NEXT_BRIDGE_TTL_SECS);

        let mut inner = self.lock_inner();
        prune_next_bridge_tokens(&mut inner.next_bridge_tokens, now);
        inner
            .next_bridge_tokens
            .insert(bridge_token.clone(), NextBridgeTicket { purpose: "next_bridge", expires_at });

        NextBridgeIssue { bridge_token, expires_at }
    }

    pub fn exchange_next_bridge_token(
        &self,
        bridge_token: &str,
    ) -> Result<BrowserSessionIssue, WebAuthFailure> {
        self.ensure_bootstrap();
        let bridge_token = bridge_token.trim();
        if bridge_token.is_empty() {
            return Err(invalid_request("bridge token is required"));
        }

        let now = now_secs();
        let mut inner = self.lock_inner();
        prune_next_bridge_tokens(&mut inner.next_bridge_tokens, now);

        let ticket = inner
            .next_bridge_tokens
            .remove(bridge_token)
            .ok_or_else(|| unauthorized("Unauthorized"))?;

        if ticket.purpose != "next_bridge" || ticket.expires_at < now {
            return Err(unauthorized("Unauthorized"));
        }

        Ok(issue_browser_session(&mut inner.browser_sessions, now))
    }

    pub fn is_valid_browser_session_token(&self, token: &str) -> bool {
        self.ensure_bootstrap();
        let token = token.trim();
        if token.is_empty() {
            return false;
        }

        let now = now_secs();
        let mut inner = self.lock_inner();
        prune_browser_sessions(&mut inner.browser_sessions, now);

        inner.browser_sessions.get(token).is_some_and(|session| {
            session.purpose == "browser_session" && session.expires_at >= now
        })
    }

    fn ensure_bootstrap(&self) {
        let mut inner = self.lock_inner();
        let now = now_secs();
        prune_setup_token(&mut inner.state, now);

        let base_state = base_state_from(&inner.state, inner.bootstrap_ready);
        if matches!(base_state, BaseAuthState::Initialized) {
            if !inner.bootstrap_ready {
                inner.bootstrap_ready = true;
                if let Err(error) = persist_state(&self.state_path, &inner.state) {
                    log_warn_ctx(
                        "services.web_auth",
                        "ensure_bootstrap",
                        &format!("failed to persist initialized web auth state: {}", error.message),
                    );
                }
            }
            return;
        }

        if !matches!(base_state, BaseAuthState::SetupPending) {
            let setup_token = format!("sl-setup-{}", Uuid::new_v4().simple());
            let setup_token_hash = sha256_hex(&setup_token);
            let expires_at = now.saturating_add(SETUP_TOKEN_TTL_SECS);
            inner.state.setup_token_hash = Some(setup_token_hash);
            inner.state.setup_token_expires_at = Some(expires_at);

            capture_println(format!(
                "SeaLantern Web auth setup token {} expires_at={} purpose=setup_initialize",
                setup_token, expires_at
            ));
        }

        if !inner.bootstrap_ready {
            inner.bootstrap_ready = true;
        }

        if let Err(error) = persist_state(&self.state_path, &inner.state) {
            log_warn_ctx(
                "services.web_auth",
                "ensure_bootstrap",
                &format!("failed to persist bootstrapped web auth state: {}", error.message),
            );
        }
    }

    fn lock_inner(&self) -> MutexGuard<'_, WebAuthInner> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

#[cfg(test)]
impl WebAuthService {
    pub(crate) fn seed_setup_token_for_test(&self, setup_token: &str, ttl_seconds: u64) {
        let now = now_secs();
        let mut inner = self.lock_inner();
        inner.state.password_hash = None;
        inner.state.setup_token_hash = Some(sha256_hex(setup_token));
        inner.state.setup_token_expires_at = Some(now.saturating_add(ttl_seconds));
        inner.state.password_updated_at = None;
        inner.bootstrap_ready = false;
        persist_state(&self.state_path, &inner.state).expect("persist test setup token");
    }
}

fn resolve_state_path() -> Result<PathBuf, String> {
    let data_dir = get_or_create_app_data_dir_checked()
        .map_err(|error| format!("Failed to resolve app data directory: {}", error))?;
    Ok(PathBuf::from(data_dir).join(WEB_AUTH_STATE_FILE))
}

fn load_state(path: &PathBuf) -> Result<PersistedWebAuthState, String> {
    if !path.exists() {
        return Ok(PersistedWebAuthState::default());
    }

    let content = std::fs::read_to_string(path).map_err(|error| {
        format!("Failed to read web auth state '{}': {}", path.display(), error)
    })?;
    serde_json::from_str::<PersistedWebAuthState>(&content)
        .map_err(|error| format!("Failed to parse web auth state '{}': {}", path.display(), error))
}

fn persist_state(path: &PathBuf, state: &PersistedWebAuthState) -> Result<(), WebAuthFailure> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            unavailable(format!(
                "failed to create web auth state directory '{}': {}",
                parent.display(),
                error
            ))
        })?;
    }

    let json = serde_json::to_string_pretty(state)
        .map_err(|error| unavailable(format!("failed to serialize web auth state: {}", error)))?;
    std::fs::write(path, json).map_err(|error| {
        unavailable(format!("failed to write web auth state '{}': {}", path.display(), error))
    })
}

fn base_state_from(state: &PersistedWebAuthState, bootstrap_ready: bool) -> BaseAuthState {
    if !bootstrap_ready && state.password_hash.is_none() && state.setup_token_hash.is_none() {
        return BaseAuthState::Uninitialized;
    }
    if state.password_hash.is_some() {
        BaseAuthState::Initialized
    } else if state.setup_token_hash.is_some() {
        BaseAuthState::SetupPending
    } else {
        BaseAuthState::Uninitialized
    }
}

fn prune_setup_token(state: &mut PersistedWebAuthState, now: u64) {
    if state
        .setup_token_expires_at
        .is_some_and(|expires_at| expires_at < now)
    {
        state.setup_token_hash = None;
        state.setup_token_expires_at = None;
    }
}

fn prune_browser_sessions(registry: &mut HashMap<String, BrowserSessionRecord>, now: u64) {
    registry.retain(|_, session| session.expires_at >= now);
}

fn prune_next_bridge_tokens(registry: &mut HashMap<String, NextBridgeTicket>, now: u64) {
    registry.retain(|_, ticket| ticket.expires_at >= now);
}

fn issue_browser_session(
    registry: &mut HashMap<String, BrowserSessionRecord>,
    issued_at: u64,
) -> BrowserSessionIssue {
    let session_token = format!("sl-session-{}", Uuid::new_v4().simple());
    let expires_at = issued_at.saturating_add(BROWSER_SESSION_TTL_SECS);
    registry.insert(
        session_token.clone(),
        BrowserSessionRecord { purpose: "browser_session", expires_at },
    );
    BrowserSessionIssue {
        session_token,
        expires_at,
        purpose: "browser_session",
    }
}

fn verify_password(password: &str, password_hash: &str) -> Result<(), WebAuthFailure> {
    let parsed = PasswordHash::new(password_hash)
        .map_err(|error| unavailable(format!("invalid stored password hash: {}", error)))?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| unauthorized("invalid password"))
}

fn hash_password(password: &str) -> Result<String, WebAuthFailure> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| unavailable(format!("failed to hash password: {}", error)))
}

fn validate_password(password: &str) -> Result<(), WebAuthFailure> {
    if password.trim().is_empty() {
        return Err(invalid_request("password is required"));
    }
    Ok(())
}

fn recovery_token() -> Option<String> {
    std::env::var(WEB_AUTH_RECOVERY_TOKEN_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn sha256_hex(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    digest.iter().map(|byte| format!("{:02x}", byte)).collect()
}

fn unauthorized(message: impl Into<String>) -> WebAuthFailure {
    WebAuthFailure {
        kind: WebAuthError::Unauthorized,
        message: message.into(),
    }
}

fn invalid_request(message: impl Into<String>) -> WebAuthFailure {
    WebAuthFailure {
        kind: WebAuthError::InvalidRequest,
        message: message.into(),
    }
}

fn conflict(message: impl Into<String>) -> WebAuthFailure {
    WebAuthFailure {
        kind: WebAuthError::Conflict,
        message: message.into(),
    }
}

fn unavailable(message: impl Into<String>) -> WebAuthFailure {
    WebAuthFailure {
        kind: WebAuthError::Unavailable,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        base_state_from, now_secs, persist_state, BaseAuthState, BrowserSessionIssue,
        PersistedWebAuthState, WebAuthService, BROWSER_SESSION_TTL_SECS,
    };
    use crate::test_support::{lock_env, EnvGuard};
    use crate::utils::logger::GLOBAL_LOG_COLLECTOR;
    use sea_lantern_runtime::{HTTP_AUTH_TOKEN_ENV, WEB_AUTH_RECOVERY_TOKEN_ENV};
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn unique_state_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("sealantern-{}-{}.json", name, unique))
    }

    fn test_service(path: PathBuf) -> WebAuthService {
        WebAuthService {
            state_path: path,
            inner: std::sync::Mutex::new(super::WebAuthInner {
                state: PersistedWebAuthState::default(),
                bootstrap_ready: false,
                browser_sessions: std::collections::HashMap::new(),
                next_bridge_tokens: std::collections::HashMap::new(),
            }),
        }
    }

    fn cleanup(path: &PathBuf) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn base_state_reports_uninitialized_without_bootstrap() {
        let state = PersistedWebAuthState::default();
        assert_eq!(base_state_from(&state, false), BaseAuthState::Uninitialized);
    }

    #[test]
    fn bootstrap_generates_setup_token_and_reports_setup_pending() {
        let _lock = lock_env();
        let _guard = EnvGuard::remove(HTTP_AUTH_TOKEN_ENV);
        let _recovery_guard = EnvGuard::remove(WEB_AUTH_RECOVERY_TOKEN_ENV);
        let path = unique_state_path("web-auth-bootstrap");
        let service = test_service(path.clone());

        let status = service.auth_status();
        assert_eq!(status.state, "setup_pending");
        assert!(status.setup_required);
        cleanup(&path);
    }

    #[test]
    fn expired_setup_token_reissues_bootstrap_and_returns_to_setup_pending() {
        let _lock = lock_env();
        let _guard = EnvGuard::remove(HTTP_AUTH_TOKEN_ENV);
        let _recovery_guard = EnvGuard::remove(WEB_AUTH_RECOVERY_TOKEN_ENV);
        let path = unique_state_path("web-auth-expired-setup-token");
        let service = test_service(path.clone());

        let initial_status = service.auth_status();
        assert_eq!(initial_status.state, "setup_pending");

        let initial_state: PersistedWebAuthState =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("state file"))
                .expect("json");
        let initial_hash = initial_state
            .setup_token_hash
            .clone()
            .expect("initial setup token hash");

        let expired_state = PersistedWebAuthState {
            password_hash: None,
            setup_token_hash: Some(initial_hash),
            setup_token_expires_at: Some(now_secs().saturating_sub(1)),
            password_updated_at: None,
        };
        persist_state(&path, &expired_state).expect("persist expired state");

        {
            let mut inner = service.lock_inner();
            inner.state = expired_state;
            inner.bootstrap_ready = true;
        }

        let status = service.auth_status();
        assert_eq!(status.state, "setup_pending");
        assert_eq!(status.base_state, "setup_pending");
        assert!(status.setup_required);

        let reissued_state: PersistedWebAuthState =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("state file"))
                .expect("json");
        let reissued_hash = reissued_state
            .setup_token_hash
            .expect("reissued setup token hash");
        let reissued_expires_at = reissued_state
            .setup_token_expires_at
            .expect("reissued setup token expiry");

        assert_ne!(reissued_hash, initial_state.setup_token_hash.expect("initial hash"));
        assert!(reissued_expires_at >= now_secs());

        cleanup(&path);
    }

    #[test]
    fn expired_setup_token_reissue_logs_new_setup_token() {
        let _lock = lock_env();
        let _guard = EnvGuard::remove(HTTP_AUTH_TOKEN_ENV);
        let _recovery_guard = EnvGuard::remove(WEB_AUTH_RECOVERY_TOKEN_ENV);
        let path = unique_state_path("web-auth-expired-setup-log");
        let service = test_service(path.clone());

        let _ = service.auth_status();
        let initial_state: PersistedWebAuthState =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("state file"))
                .expect("json");

        let expired_state = PersistedWebAuthState {
            password_hash: None,
            setup_token_hash: initial_state.setup_token_hash,
            setup_token_expires_at: Some(now_secs().saturating_sub(1)),
            password_updated_at: None,
        };
        persist_state(&path, &expired_state).expect("persist expired state");

        {
            let mut inner = service.lock_inner();
            inner.state = expired_state;
            inner.bootstrap_ready = true;
        }

        GLOBAL_LOG_COLLECTOR.clear();
        let _ = service.auth_status();

        let logs = GLOBAL_LOG_COLLECTOR.get_logs(None);
        assert!(logs.iter().any(|entry| {
            entry.message.contains("SeaLantern Web auth setup token")
                && entry.message.contains("purpose=setup_initialize")
        }));

        cleanup(&path);
    }

    #[test]
    fn initialize_login_and_bridge_exchange_issue_valid_sessions() {
        let _lock = lock_env();
        let _guard = EnvGuard::remove(HTTP_AUTH_TOKEN_ENV);
        let _recovery_guard = EnvGuard::remove(WEB_AUTH_RECOVERY_TOKEN_ENV);
        let path = unique_state_path("web-auth-flow");
        let service = test_service(path.clone());
        let before = now_secs();

        let setup_token = {
            let _ = service.auth_status();
            let raw = std::fs::read_to_string(&path).expect("state file");
            let state: PersistedWebAuthState = serde_json::from_str(&raw).expect("json");
            let token_hash = state.setup_token_hash.expect("setup token hash");
            let expires_at = state.setup_token_expires_at.expect("setup token expiry");
            assert!(expires_at >= before);
            token_hash
        };

        assert!(service
            .initialize_password("wrong-token", "password")
            .is_err());
        assert!(setup_token.len() == 64);

        cleanup(&path);
    }

    #[test]
    fn setup_and_login_issue_browser_sessions_on_happy_path() {
        let _lock = lock_env();
        let _guard = EnvGuard::remove(HTTP_AUTH_TOKEN_ENV);
        let _recovery_guard = EnvGuard::remove(WEB_AUTH_RECOVERY_TOKEN_ENV);
        let path = unique_state_path("web-auth-setup-login-success");
        let service = test_service(path.clone());

        let status = service.auth_status();
        assert_eq!(status.state, "setup_pending");

        let raw = std::fs::read_to_string(&path).expect("state file");
        let state: PersistedWebAuthState = serde_json::from_str(&raw).expect("json");
        let expected_hash = state.setup_token_hash.expect("setup token hash");

        let setup_token = "setup-secret";
        {
            let mut inner = service.lock_inner();
            inner.state.setup_token_hash = Some(super::sha256_hex(setup_token));
            inner.state.setup_token_expires_at = Some(now_secs().saturating_add(60));
            persist_state(&path, &inner.state).expect("persist setup token");
        }

        assert_ne!(expected_hash, super::sha256_hex(setup_token));

        let initialized_session = service
            .initialize_password(setup_token, "browser-password")
            .expect("initialize browser password");
        assert_eq!(initialized_session.purpose, "browser_session");
        assert!(service.is_valid_browser_session_token(&initialized_session.session_token));

        let login_session = service
            .login("browser-password")
            .expect("login with initialized password");
        assert_eq!(login_session.purpose, "browser_session");
        assert!(service.is_valid_browser_session_token(&login_session.session_token));

        let after_status = service.auth_status();
        assert_eq!(after_status.state, "initialized");
        assert!(after_status.password_login_enabled);
        assert!(!after_status.setup_required);

        cleanup(&path);
    }

    #[test]
    fn recovery_reset_rotates_password_and_issues_browser_session() {
        let _lock = lock_env();
        let _legacy_guard = EnvGuard::remove(HTTP_AUTH_TOKEN_ENV);
        let _recovery_guard = EnvGuard::set(WEB_AUTH_RECOVERY_TOKEN_ENV, "recovery-secret");
        let path = unique_state_path("web-auth-recovery-success");
        let service = WebAuthService::new_for_test(path.clone(), Some("old-password"));

        let recovery_session = service
            .recovery_reset("recovery-secret", "new-password")
            .expect("recovery reset succeeds");
        assert_eq!(recovery_session.purpose, "browser_session");
        assert!(service.is_valid_browser_session_token(&recovery_session.session_token));

        assert!(service.login("old-password").is_err());

        let login_session = service
            .login("new-password")
            .expect("login with recovered password");
        assert_eq!(login_session.purpose, "browser_session");
        assert!(service.is_valid_browser_session_token(&login_session.session_token));

        let status = service.auth_status();
        assert_eq!(status.state, "recovery_active");
        assert_eq!(status.base_state, "initialized");
        assert!(status.password_login_enabled);

        cleanup(&path);
    }

    #[test]
    fn recovery_reset_requires_dedicated_env_token_and_ignores_legacy_http_token() {
        let _lock = lock_env();
        let _legacy_guard = EnvGuard::set(HTTP_AUTH_TOKEN_ENV, "legacy-http-token");
        let _guard = EnvGuard::set(WEB_AUTH_RECOVERY_TOKEN_ENV, "recovery-secret");
        let path = unique_state_path("web-auth-recovery");
        let service = test_service(path.clone());

        let status = service.auth_status();
        assert!(status.recovery_active);
        assert!(service.recovery_reset("wrong", "password").is_err());

        cleanup(&path);
    }

    #[test]
    fn legacy_http_token_env_no_longer_triggers_recovery_mode() {
        let _lock = lock_env();
        let _legacy_guard = EnvGuard::set(HTTP_AUTH_TOKEN_ENV, "legacy-http-token");
        let _recovery_guard = EnvGuard::remove(WEB_AUTH_RECOVERY_TOKEN_ENV);
        let path = unique_state_path("web-auth-legacy-http-token");
        let service = test_service(path.clone());

        let status = service.auth_status();
        assert!(!status.recovery_active);
        assert_ne!(status.state, "recovery_active");

        cleanup(&path);
    }

    #[test]
    fn issued_session_has_expected_ttl_shape() {
        let issued_at = now_secs();
        let mut sessions = std::collections::HashMap::new();
        let BrowserSessionIssue { purpose, expires_at, .. } =
            super::issue_browser_session(&mut sessions, issued_at);
        assert_eq!(purpose, "browser_session");
        assert_eq!(expires_at, issued_at + BROWSER_SESSION_TTL_SECS);
    }
}
