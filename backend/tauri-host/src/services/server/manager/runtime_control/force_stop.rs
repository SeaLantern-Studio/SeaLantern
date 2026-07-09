use super::super::common::{current_timestamp_millis, current_timestamp_secs};
use super::{ForceStopPreparation, ServerManager};
use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::manager::i18n::{manager_t, manager_t1};
use crate::services::server::runtime;

/// 生成强停确认信息
pub(super) fn prepare_force_stop_server(
    manager: &ServerManager,
    id: &str,
) -> Result<ForceStopPreparation, String> {
    let server = manager.find_server_clone(id)?;

    let resolved_runtime = runtime::resolve_runtime(&server)?;
    let preparation = resolved_runtime.prepare_force_stop_with_manager(manager, &server)?;
    if !preparation.supported {
        return Err(manager_t1(
            "server.manager.force_stop_not_supported",
            server.runtime_kind.clone(),
        ));
    }

    if server.runtime_kind == "local" {
        let _ = manager.ensure_local_runtime(&server)?;
    }

    let expires_at = current_timestamp_secs().saturating_add(15);
    let token = format!("{}-{}", id, current_timestamp_millis());

    let mut pending = manager
        .pending_force_stop_tokens
        .lock()
        .map_err(|_| "pending_force_stop_tokens lock poisoned".to_string())?;
    pending.insert(id.to_string(), (token.clone(), expires_at));
    drop(pending);

    let _ = server_log_pipeline::append_sealantern_log(
        id,
        "[Sea Lantern] 已创建强制关停确认，会在确认后执行",
    );

    Ok(ForceStopPreparation { token, expires_at })
}

/// 校验确认口令并强制终止服务器进程
pub(super) fn force_stop_server(
    manager: &ServerManager,
    id: &str,
    confirmation_token: &str,
) -> Result<(), String> {
    let server = manager.find_server_clone(id)?;

    let resolved_runtime = runtime::resolve_runtime(&server)?;

    validate_force_stop_confirmation(manager, id, confirmation_token)?;

    if server.runtime_kind == "local" {
        let _ = manager.ensure_local_runtime(&server)?;
    }

    resolved_runtime.force_stop_with_manager(manager, &server)
}

fn validate_force_stop_confirmation(
    manager: &ServerManager,
    id: &str,
    confirmation_token: &str,
) -> Result<(), String> {
    let mut pending = manager
        .pending_force_stop_tokens
        .lock()
        .map_err(|_| "pending_force_stop_tokens lock poisoned".to_string())?;

    let Some((expected_token, expires_at)) = pending.get(id).cloned() else {
        return Err(manager_t("server.manager.force_stop_confirmation_missing"));
    };

    let now = current_timestamp_secs();
    if now > expires_at {
        pending.remove(id);
        return Err(manager_t("server.manager.force_stop_confirmation_expired"));
    }

    if expected_token != confirmation_token {
        return Err(manager_t("server.manager.force_stop_confirmation_invalid"));
    }

    pending.remove(id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_force_stop_confirmation;
    use crate::services::server::manager::ServerManager;

    #[test]
    fn validate_force_stop_confirmation_rejects_missing_token() {
        let manager = ServerManager::new();

        let err = validate_force_stop_confirmation(&manager, "docker-1", "token")
            .expect_err("missing token should be rejected");

        assert!(err.contains("缺少强制关停确认"));
    }

    #[test]
    fn validate_force_stop_confirmation_rejects_wrong_token_without_consuming_it() {
        let manager = ServerManager::new();
        manager
            .pending_force_stop_tokens
            .lock()
            .unwrap()
            .insert("docker-1".to_string(), ("expected".to_string(), u64::MAX));

        let err = validate_force_stop_confirmation(&manager, "docker-1", "wrong")
            .expect_err("wrong token should be rejected");
        assert!(err.contains("无效"));

        let pending = manager.pending_force_stop_tokens.lock().unwrap();
        assert!(pending.contains_key("docker-1"));
    }

    #[test]
    fn validate_force_stop_confirmation_consumes_valid_token() {
        let manager = ServerManager::new();
        manager
            .pending_force_stop_tokens
            .lock()
            .unwrap()
            .insert("docker-1".to_string(), ("expected".to_string(), u64::MAX));

        validate_force_stop_confirmation(&manager, "docker-1", "expected")
            .expect("valid token should pass");

        let pending = manager.pending_force_stop_tokens.lock().unwrap();
        assert!(!pending.contains_key("docker-1"));
    }

    #[test]
    fn validate_force_stop_confirmation_removes_expired_token() {
        let manager = ServerManager::new();
        manager
            .pending_force_stop_tokens
            .lock()
            .unwrap()
            .insert("docker-1".to_string(), ("expected".to_string(), 0));

        let err = validate_force_stop_confirmation(&manager, "docker-1", "expected")
            .expect_err("expired token should be rejected");
        assert!(err.contains("已过期"));

        let pending = manager.pending_force_stop_tokens.lock().unwrap();
        assert!(!pending.contains_key("docker-1"));
    }
}
