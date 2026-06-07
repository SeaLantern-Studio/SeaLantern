use super::protocol::{read_request, write_response, LocalHelperRequest, LocalHelperResponse};
use super::snapshot::snapshot_from_manager;
use super::{LocalHelperStatusSnapshot, LocalRuntimeState};
use crate::models::server::ServerInstance;
use crate::services::server::manager::ServerManager;
use crate::services::server::runtime::i18n::runtime_t;
use crate::services::server::runtime::local::LocalServerRuntime;
use crate::services::server::runtime::ServerRuntime;
use crate::utils::logger;
use std::net::TcpStream;

pub(super) fn handle_connection(
    manager: &ServerManager,
    server: &ServerInstance,
    state: &LocalRuntimeState,
    mut stream: TcpStream,
) -> Result<bool, String> {
    let request = read_request(&stream)?;

    if request.auth_token() != state.auth_token.as_str() {
        write_response(&mut stream, &auth_failed_response())?;
        return Ok(false);
    }

    let outcome = match request {
        LocalHelperRequest::Status { .. } => DispatchOutcome::stay_running(status_response(
            helper_status_snapshot(manager, server, state)?,
        )),
        LocalHelperRequest::Send { command, .. } => DispatchOutcome::stay_running(
            command_response(manager.send_command(&server.id, &command)),
        ),
        LocalHelperRequest::Stop { .. } => DispatchOutcome::request_stop_async(server),
        LocalHelperRequest::ForceStop { .. } => DispatchOutcome::from_control_result(
            LocalServerRuntime.force_stop_with_manager(manager, server),
        ),
    };

    write_response(&mut stream, &outcome.response)?;
    Ok(outcome.should_exit)
}

fn helper_status_snapshot(
    manager: &ServerManager,
    server: &ServerInstance,
    state: &LocalRuntimeState,
) -> Result<LocalHelperStatusSnapshot, String> {
    if state.running && state.child_pid.is_none() {
        return Ok(LocalHelperStatusSnapshot {
            running: true,
            pid: None,
            exit_code: None,
            detail_message: "runtime=local running=true source=helper startup=preparing"
                .to_string(),
            error_message: None,
        });
    }

    snapshot_from_manager(manager, server.id.as_str())
}

struct DispatchOutcome {
    response: LocalHelperResponse,
    should_exit: bool,
}

impl DispatchOutcome {
    fn stay_running(response: LocalHelperResponse) -> Self {
        Self { response, should_exit: false }
    }

    fn request_stop_async(server: &ServerInstance) -> Self {
        let server_id = server.id.clone();
        std::thread::spawn(move || {
            let manager = crate::services::global::server_manager();
            if let Err(error) = manager.stop_server(&server_id) {
                logger::log_user_action_error(
                    "server.runtime.local_helper",
                    "stop_async",
                    &format!("server_id={}", server_id),
                    &error,
                );
            }
        });

        Self {
            response: LocalHelperResponse { ok: true, snapshot: None, error: None },
            should_exit: true,
        }
    }

    fn from_control_result(result: Result<(), String>) -> Self {
        let response = command_response(result);
        let should_exit = response.ok;
        Self { response, should_exit }
    }
}

fn auth_failed_response() -> LocalHelperResponse {
    LocalHelperResponse {
        ok: false,
        snapshot: None,
        error: Some(runtime_t("server.runtime.local_helper.auth_failed")),
    }
}

fn status_response(snapshot: super::LocalHelperStatusSnapshot) -> LocalHelperResponse {
    LocalHelperResponse {
        ok: true,
        snapshot: Some(snapshot),
        error: None,
    }
}

fn command_response(result: Result<(), String>) -> LocalHelperResponse {
    match result {
        Ok(()) => LocalHelperResponse { ok: true, snapshot: None, error: None },
        Err(err) => LocalHelperResponse {
            ok: false,
            snapshot: None,
            error: Some(err),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{auth_failed_response, command_response, status_response, DispatchOutcome};
    use crate::services::server::runtime::i18n::runtime_t;
    use crate::services::server::runtime::local_helper::LocalHelperStatusSnapshot;

    #[test]
    fn auth_failed_response_returns_expected_error_payload() {
        let response = auth_failed_response();

        assert!(!response.ok);
        assert_eq!(response.snapshot, None);
        assert_eq!(
            response.error.as_deref(),
            Some(runtime_t("server.runtime.local_helper.auth_failed").as_str())
        );
    }

    #[test]
    fn status_response_wraps_snapshot_without_exit_signal() {
        let snapshot = LocalHelperStatusSnapshot {
            running: true,
            pid: Some(42),
            exit_code: None,
            detail_message: "runtime=local running=true source=helper pid=42".to_string(),
            error_message: None,
        };

        let outcome = DispatchOutcome::stay_running(status_response(snapshot.clone()));

        assert!(!outcome.should_exit);
        assert!(outcome.response.ok);
        assert_eq!(outcome.response.snapshot, Some(snapshot));
        assert_eq!(outcome.response.error, None);
    }

    #[test]
    fn control_result_only_requests_exit_on_success() {
        let success = DispatchOutcome::from_control_result(Ok(()));
        assert!(success.response.ok);
        assert!(success.should_exit);

        let failure = DispatchOutcome::from_control_result(Err("stop failed".to_string()));
        assert!(!failure.response.ok);
        assert!(!failure.should_exit);
        assert_eq!(failure.response.error.as_deref(), Some("stop failed"));
    }

    #[test]
    fn command_response_maps_errors_without_snapshot() {
        let response = command_response(Err("send failed".to_string()));

        assert!(!response.ok);
        assert_eq!(response.snapshot, None);
        assert_eq!(response.error.as_deref(), Some("send failed"));
    }
}
