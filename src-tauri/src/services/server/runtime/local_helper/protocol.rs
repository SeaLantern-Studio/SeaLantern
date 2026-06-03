use super::state::{LocalHelperStatusSnapshot, LocalRuntimeState};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub(super) enum LocalHelperRequest {
    Status { auth_token: String },
    Send { auth_token: String, command: String },
    Stop { auth_token: String },
    ForceStop { auth_token: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct LocalHelperResponse {
    pub(super) ok: bool,
    pub(super) snapshot: Option<LocalHelperStatusSnapshot>,
    pub(super) error: Option<String>,
}

impl LocalHelperRequest {
    pub(super) fn auth_token(&self) -> &str {
        match self {
            Self::Status { auth_token }
            | Self::Send { auth_token, .. }
            | Self::Stop { auth_token }
            | Self::ForceStop { auth_token } => auth_token,
        }
    }
}

pub(super) fn read_request(stream: &TcpStream) -> Result<LocalHelperRequest, String> {
    let mut reader = BufReader::new(
        stream
            .try_clone()
            .map_err(|e| format!("复制 helper 连接失败: {}", e))?,
    );
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("读取 helper 请求失败: {}", e))?;
    serde_json::from_str(&line).map_err(|e| format!("解析 helper 请求失败: {}", e))
}

pub(super) fn write_response(
    stream: &mut TcpStream,
    response: &LocalHelperResponse,
) -> Result<(), String> {
    let payload =
        serde_json::to_string(response).map_err(|e| format!("序列化 helper 响应失败: {}", e))?;
    writeln!(stream, "{}", payload).map_err(|e| format!("写入 helper 响应失败: {}", e))
}

pub(super) fn send_request(
    state: &LocalRuntimeState,
    request: LocalHelperRequest,
) -> Result<LocalHelperResponse, String> {
    let control_port = state
        .control_port
        .ok_or_else(|| "本地 runtime helper 当前未暴露控制端口".to_string())?;
    let mut stream = TcpStream::connect(("127.0.0.1", control_port)).map_err(|e| {
        format!(
            "连接本地 runtime helper 失败: helper_pid={} port={} error={}",
            state.helper_pid, control_port, e
        )
    })?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("设置 helper 读取超时失败: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("设置 helper 写入超时失败: {}", e))?;

    let payload =
        serde_json::to_string(&request).map_err(|e| format!("序列化 helper 请求失败: {}", e))?;
    writeln!(stream, "{}", payload).map_err(|e| format!("写入 helper 请求失败: {}", e))?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("读取 helper 响应失败: {}", e))?;

    serde_json::from_str(&line).map_err(|e| format!("解析 helper 响应失败: {}", e))
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
