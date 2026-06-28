use super::{LocalHelperControlState, LocalHelperStatusSnapshot};
use crate::services::server::runtime::i18n::{runtime_t1, runtime_t2};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub(in super::super) enum LocalHelperRequest {
    Status { auth_token: String },
    Send { auth_token: String, command: String },
    Stop { auth_token: String },
    ForceStop { auth_token: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub(in super::super) struct LocalHelperResponse {
    pub(in super::super) ok: bool,
    pub(in super::super) snapshot: Option<LocalHelperStatusSnapshot>,
    pub(in super::super) error: Option<String>,
}

impl LocalHelperRequest {
    pub(in super::super) fn auth_token(&self) -> &str {
        match self {
            Self::Status { auth_token }
            | Self::Send { auth_token, .. }
            | Self::Stop { auth_token }
            | Self::ForceStop { auth_token } => auth_token,
        }
    }
}

pub(in super::super) fn read_request(stream: &TcpStream) -> Result<LocalHelperRequest, String> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| {
        runtime_t1("server.runtime.local_helper.stream_clone_failed", e.to_string())
    })?);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| {
        runtime_t1("server.runtime.local_helper.request_read_failed", e.to_string())
    })?;
    serde_json::from_str(&line)
        .map_err(|e| runtime_t1("server.runtime.local_helper.request_parse_failed", e.to_string()))
}

pub(in super::super) fn write_response(
    stream: &mut TcpStream,
    response: &LocalHelperResponse,
) -> Result<(), String> {
    let payload = serde_json::to_string(response).map_err(|e| {
        runtime_t1("server.runtime.local_helper.response_serialize_failed", e.to_string())
    })?;
    writeln!(stream, "{}", payload)
        .map_err(|e| runtime_t1("server.runtime.local_helper.response_write_failed", e.to_string()))
}

pub(super) fn send_request(
    state: &LocalHelperControlState,
    request: LocalHelperRequest,
) -> Result<LocalHelperResponse, String> {
    let control_port = state.control_port;
    let mut stream = TcpStream::connect(("127.0.0.1", control_port)).map_err(|e| {
        runtime_t2(
            "server.runtime.local_helper.connect_failed",
            format!("helper_pid={} port={}", state.helper_pid, control_port),
            e.to_string(),
        )
    })?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| {
            runtime_t1("server.runtime.local_helper.read_timeout_set_failed", e.to_string())
        })?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| {
            runtime_t1("server.runtime.local_helper.write_timeout_set_failed", e.to_string())
        })?;

    let payload = serde_json::to_string(&request).map_err(|e| {
        runtime_t1("server.runtime.local_helper.request_serialize_failed", e.to_string())
    })?;
    writeln!(stream, "{}", payload).map_err(|e| {
        runtime_t1("server.runtime.local_helper.request_write_failed", e.to_string())
    })?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| {
        runtime_t1("server.runtime.local_helper.response_read_failed", e.to_string())
    })?;

    serde_json::from_str(&line)
        .map_err(|e| runtime_t1("server.runtime.local_helper.response_parse_failed", e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{LocalHelperRequest, LocalHelperResponse};
    use crate::services::server::runtime::local_helper::LocalHelperStatusSnapshot;

    #[test]
    fn helper_request_auth_token_returns_token_for_each_variant() {
        let token = "secret".to_string();

        let status = LocalHelperRequest::Status { auth_token: token.clone() };
        assert_eq!(status.auth_token(), "secret");

        let send = LocalHelperRequest::Send {
            auth_token: token.clone(),
            command: "say hi".to_string(),
        };
        assert_eq!(send.auth_token(), "secret");

        let stop = LocalHelperRequest::Stop { auth_token: token.clone() };
        assert_eq!(stop.auth_token(), "secret");

        let force_stop = LocalHelperRequest::ForceStop { auth_token: token };
        assert_eq!(force_stop.auth_token(), "secret");
    }

    #[test]
    fn helper_response_round_trips_snapshot_payload() {
        let response = LocalHelperResponse {
            ok: true,
            snapshot: Some(LocalHelperStatusSnapshot {
                running: true,
                pid: Some(42),
                exit_code: None,
                detail_message: "runtime=local running=true source=helper pid=42".to_string(),
                error_message: None,
            }),
            error: None,
        };

        let encoded = serde_json::to_string(&response).expect("response should serialize");
        let decoded: LocalHelperResponse =
            serde_json::from_str(&encoded).expect("response should deserialize");

        assert!(decoded.ok);
        assert_eq!(decoded.snapshot.as_ref().and_then(|snapshot| snapshot.pid), Some(42));
        assert_eq!(decoded.error, None);
    }
}
