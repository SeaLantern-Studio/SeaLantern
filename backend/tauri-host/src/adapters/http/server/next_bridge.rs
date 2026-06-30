use super::state::{ApiErrorDetail, ApiResponse, AppState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const NEXT_BRIDGE_PURPOSE: &str = "next_bridge";
const NEXT_BROWSER_SESSION_PURPOSE: &str = "next_browser_session";
const DEFAULT_NEXT_TARGET_PATH: &str = "/settings";
const NEXT_BRIDGE_TTL_SECS: u64 = 60;
const NEXT_BROWSER_SESSION_TTL_SECS: u64 = 60 * 60 * 12;

#[derive(Clone)]
struct NextBridgeTicket {
    target_bearer: String,
    purpose: &'static str,
    expires_at: u64,
}

#[derive(Clone)]
struct BrowserSessionTicket {
    target_bearer: String,
    purpose: &'static str,
    expires_at: u64,
}

#[derive(Deserialize)]
pub(super) struct IssueNextBridgeTokenRequest {
    #[serde(default)]
    target_path: Option<String>,
}

#[derive(Serialize)]
struct IssueNextBridgeTokenResponse {
    bridge_token: String,
    expires_at: u64,
    purpose: &'static str,
    target_path: String,
}

#[derive(Deserialize)]
pub(super) struct ExchangeNextBridgeTokenRequest {
    bridge_token: String,
}

#[derive(Serialize)]
struct ExchangeNextBridgeTokenResponse {
    token: String,
    expires_at: u64,
    purpose: &'static str,
}

#[derive(Debug)]
enum BridgeExchangeError {
    Unauthorized,
}

fn bridge_registry() -> &'static Mutex<HashMap<String, NextBridgeTicket>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, NextBridgeTicket>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn browser_session_registry() -> &'static Mutex<HashMap<String, BrowserSessionTicket>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, BrowserSessionTicket>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_registry<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn sanitize_next_target_path(raw: Option<&str>) -> Result<String, String> {
    let trimmed = raw.map(str::trim).filter(|value| !value.is_empty());
    let path = trimmed.unwrap_or(DEFAULT_NEXT_TARGET_PATH);

    if !path.starts_with('/') || path.starts_with("//") {
        return Err("next bridge target path must stay within the current origin".to_string());
    }

    Ok(path.to_string())
}

fn prune_bridge_registry(registry: &mut HashMap<String, NextBridgeTicket>, now: u64) {
    registry.retain(|_, ticket| ticket.expires_at >= now);
}

fn prune_browser_session_registry(registry: &mut HashMap<String, BrowserSessionTicket>, now: u64) {
    registry.retain(|_, ticket| ticket.expires_at >= now);
}

fn issue_next_bridge_ticket_for_bearer_at(
    target_bearer: &str,
    target_path: &str,
    issued_at: u64,
) -> IssueNextBridgeTokenResponse {
    let bridge_token = format!("next-bridge-{}", Uuid::new_v4().simple());
    let expires_at = issued_at.saturating_add(NEXT_BRIDGE_TTL_SECS);
    let ticket = NextBridgeTicket {
        target_bearer: target_bearer.to_string(),
        purpose: NEXT_BRIDGE_PURPOSE,
        expires_at,
    };

    let mut registry = lock_registry(bridge_registry());
    prune_bridge_registry(&mut registry, issued_at);
    registry.insert(bridge_token.clone(), ticket);

    IssueNextBridgeTokenResponse {
        bridge_token,
        expires_at,
        purpose: NEXT_BRIDGE_PURPOSE,
        target_path: target_path.to_string(),
    }
}

fn mint_browser_session_token_at(
    target_bearer: &str,
    issued_at: u64,
) -> ExchangeNextBridgeTokenResponse {
    let token = format!("next-session-{}", Uuid::new_v4().simple());
    let expires_at = issued_at.saturating_add(NEXT_BROWSER_SESSION_TTL_SECS);
    let ticket = BrowserSessionTicket {
        target_bearer: target_bearer.to_string(),
        purpose: NEXT_BROWSER_SESSION_PURPOSE,
        expires_at,
    };

    let mut registry = lock_registry(browser_session_registry());
    prune_browser_session_registry(&mut registry, issued_at);
    registry.insert(token.clone(), ticket);

    ExchangeNextBridgeTokenResponse {
        token,
        expires_at,
        purpose: NEXT_BROWSER_SESSION_PURPOSE,
    }
}

fn exchange_next_bridge_token_at(
    bridge_token: &str,
    current_time: u64,
) -> Result<ExchangeNextBridgeTokenResponse, BridgeExchangeError> {
    let ticket = {
        let mut registry = lock_registry(bridge_registry());
        prune_bridge_registry(&mut registry, current_time);
        registry.remove(bridge_token)
    }
    .ok_or(BridgeExchangeError::Unauthorized)?;

    if ticket.expires_at < current_time || ticket.purpose != NEXT_BRIDGE_PURPOSE {
        return Err(BridgeExchangeError::Unauthorized);
    }

    Ok(mint_browser_session_token_at(&ticket.target_bearer, current_time))
}

fn error_response(
    status: StatusCode,
    message: impl Into<String>,
    code: &str,
    error_kind: &str,
) -> Response {
    let message = message.into();
    (
        status,
        Json(ApiResponse::error_with_detail(
            message.clone(),
            ApiErrorDetail {
                code: code.to_string(),
                message,
                args: None,
                error_kind: Some(error_kind.to_string()),
            },
        )),
    )
        .into_response()
}

pub(super) async fn issue_next_bridge_token(
    State(state): State<AppState>,
    Json(payload): Json<IssueNextBridgeTokenRequest>,
) -> impl IntoResponse {
    let target_path = match sanitize_next_target_path(payload.target_path.as_deref()) {
        Ok(path) => path,
        Err(error) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                error,
                "common.message_unknown_error",
                "invalid_request",
            )
        }
    };

    let issued =
        issue_next_bridge_ticket_for_bearer_at(&state.config.auth_token, &target_path, now_secs());

    (StatusCode::OK, Json(ApiResponse::success(serde_json::json!(issued)))).into_response()
}

pub(super) async fn exchange_next_bridge_token(
    Json(payload): Json<ExchangeNextBridgeTokenRequest>,
) -> impl IntoResponse {
    let bridge_token = payload.bridge_token.trim();
    if bridge_token.is_empty() {
        return error_response(
            StatusCode::BAD_REQUEST,
            "bridge token is required",
            "common.message_unknown_error",
            "invalid_request",
        );
    }

    match exchange_next_bridge_token_at(bridge_token, now_secs()) {
        Ok(exchanged) => (StatusCode::OK, Json(ApiResponse::success(serde_json::json!(exchanged))))
            .into_response(),
        Err(BridgeExchangeError::Unauthorized) => error_response(
            StatusCode::UNAUTHORIZED,
            "Unauthorized",
            "common.message_unauthorized",
            "unauthorized",
        ),
    }
}

pub(super) fn is_valid_browser_session_token(token: &str, expected_bearer: &str) -> bool {
    is_valid_browser_session_token_at(token, expected_bearer, now_secs())
}

fn is_valid_browser_session_token_at(
    token: &str,
    expected_bearer: &str,
    current_time: u64,
) -> bool {
    if token.trim().is_empty() {
        return false;
    }

    let mut registry = lock_registry(browser_session_registry());
    prune_browser_session_registry(&mut registry, current_time);

    registry.get(token).is_some_and(|ticket| {
        ticket.purpose == NEXT_BROWSER_SESSION_PURPOSE
            && ticket.expires_at >= current_time
            && ticket.target_bearer == expected_bearer
    })
}

#[cfg(test)]
mod tests {
    use super::{
        bridge_registry, browser_session_registry, exchange_next_bridge_token_at,
        is_valid_browser_session_token_at, issue_next_bridge_ticket_for_bearer_at, lock_registry,
        sanitize_next_target_path, NEXT_BRIDGE_TTL_SECS, NEXT_BROWSER_SESSION_TTL_SECS,
    };
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    static TEST_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn clear_registries() {
        lock_registry(bridge_registry()).clear();
        lock_registry(browser_session_registry()).clear();
    }

    #[test]
    fn sanitize_next_target_path_defaults_to_settings() {
        assert_eq!(sanitize_next_target_path(None).unwrap(), "/settings");
        assert!(sanitize_next_target_path(Some("https://example.com/settings")).is_err());
        assert!(sanitize_next_target_path(Some("//example.com/settings")).is_err());
    }

    #[test]
    fn exchanged_bridge_token_is_consumed_and_mints_browser_session_token() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_registries();

        let issued = issue_next_bridge_ticket_for_bearer_at("test-token", "/settings", 100);
        let exchanged = exchange_next_bridge_token_at(&issued.bridge_token, 120)
            .expect("bridge token should exchange successfully");

        assert!(is_valid_browser_session_token_at(&exchanged.token, "test-token", 120,));
        assert!(exchange_next_bridge_token_at(&issued.bridge_token, 121).is_err());
        assert!(!is_valid_browser_session_token_at(
            &exchanged.token,
            "test-token",
            120 + NEXT_BROWSER_SESSION_TTL_SECS + 1,
        ));
    }

    #[test]
    fn expired_bridge_token_is_rejected() {
        let _guard = TEST_LOCK.lock().unwrap();
        clear_registries();

        let issued = issue_next_bridge_ticket_for_bearer_at("test-token", "/settings", 200);
        let result =
            exchange_next_bridge_token_at(&issued.bridge_token, 200 + NEXT_BRIDGE_TTL_SECS + 1);

        assert!(result.is_err());
    }
}
