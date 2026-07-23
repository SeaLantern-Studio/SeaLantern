//! 服务器控制台命令 RPC 方法。

use std::fmt;
use std::future::Future;

use crate::observability;
use crate::rpc::service::{ConsoleCommandService, ConsoleCommandServiceError};
use crate::rpc::{RpcContext, RpcError, RpcMethod, RpcMethodName, RpcResult};

const MAX_INSTANCE_ID_LENGTH: usize = 128;
const MAX_COMMAND_CHAR_COUNT: usize = 32_767;

/// 经过边界校验的单行服务器控制台命令。
///
/// 命令正文可能包含密码、令牌或玩家输入，因此该类型不会派生 `Debug` 或 `Clone`。
pub struct ConsoleCommandRequest {
    instance_id: String,
    command: String,
}

impl ConsoleCommandRequest {
    /// 构建受约束的控制台命令请求。
    ///
    /// 只接受单行、无控制字符的命令，避免一条 RPC 请求被扩展为多次终端写入。实例 ID 仅
    /// 允许日志安全字符，避免未经验证的标识进入追踪字段。
    pub fn new(instance_id: impl Into<String>, command: impl Into<String>) -> RpcResult<Self> {
        let instance_id = instance_id.into();
        validate_instance_id(&instance_id)?;

        let command = command.into();
        validate_command(&command)?;

        Ok(Self { instance_id, command })
    }

    /// 返回已经校验的服务器实例标识。
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// 返回已经校验的单行控制台命令。
    pub fn command(&self) -> &str {
        &self.command
    }

    /// 返回命令的 Unicode 字符数量，供脱敏可观测性使用。
    pub fn command_char_count(&self) -> usize {
        self.command.chars().count()
    }
}

impl fmt::Debug for ConsoleCommandRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConsoleCommandRequest")
            .field("instance_id", &self.instance_id)
            .field("command_char_count", &self.command_char_count())
            .finish()
    }
}

/// 将已验证请求交给受管服务器控制台的 RPC 方法。
pub struct SendConsoleCommand<S> {
    service: S,
}

impl<S> SendConsoleCommand<S> {
    /// 使用宿主提供的受管控制台能力创建方法实例。
    pub const fn new(service: S) -> Self {
        Self { service }
    }
}

impl<S> RpcMethod for SendConsoleCommand<S>
where
    S: ConsoleCommandService,
{
    const NAME: RpcMethodName = RpcMethodName::new("server.console.send");

    type Request = ConsoleCommandRequest;
    type Response = ();

    fn call(
        &self,
        _context: &RpcContext,
        request: Self::Request,
    ) -> impl Future<Output = RpcResult<Self::Response>> + Send {
        let service = &self.service;
        async move {
            let result = service
                .send_console_command(request.instance_id(), request.command())
                .map_err(map_service_error);
            observability::console_command_dispatched(
                request.instance_id(),
                request.command_char_count(),
                result.is_ok(),
            );
            result
        }
    }
}

fn validate_instance_id(instance_id: &str) -> RpcResult<()> {
    if instance_id.is_empty() || instance_id.len() > MAX_INSTANCE_ID_LENGTH {
        return Err(RpcError::invalid_argument(
            "instance_id",
            "must contain between 1 and 128 characters",
        ));
    }

    if !instance_id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(RpcError::invalid_argument(
            "instance_id",
            "may contain only ASCII letters, digits, hyphens, and underscores",
        ));
    }

    Ok(())
}

fn validate_command(command: &str) -> RpcResult<()> {
    if command.trim().is_empty() {
        return Err(RpcError::invalid_argument("command", "must contain a non-whitespace command"));
    }

    if command.chars().count() > MAX_COMMAND_CHAR_COUNT {
        return Err(RpcError::invalid_argument("command", "exceeds the 32767-character limit"));
    }

    if command.chars().any(|character| character.is_control()) {
        return Err(RpcError::invalid_argument(
            "command",
            "must be a single line without control characters",
        ));
    }

    Ok(())
}

fn map_service_error(error: ConsoleCommandServiceError) -> RpcError {
    match error {
        ConsoleCommandServiceError::InstanceNotFound => RpcError::not_found("server instance"),
        ConsoleCommandServiceError::InputUnavailable => {
            RpcError::conflict("sending console commands to this server")
        }
        ConsoleCommandServiceError::DeliveryFailed => RpcError::unavailable("server console"),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::rpc::{dispatch, RpcContext, RpcErrorCode, RpcRequest, RpcRequestId, RpcTransport};

    struct RecordingConsoleService {
        requests: Mutex<Vec<(String, String)>>,
        failure: Option<ConsoleCommandServiceError>,
    }

    impl ConsoleCommandService for RecordingConsoleService {
        fn send_console_command(
            &self,
            instance_id: &str,
            command: &str,
        ) -> Result<(), ConsoleCommandServiceError> {
            self.requests
                .lock()
                .expect("test request log lock should not be poisoned")
                .push((instance_id.into(), command.into()));
            self.failure.map_or(Ok(()), Err)
        }
    }

    fn rpc_request(params: ConsoleCommandRequest) -> RpcRequest<ConsoleCommandRequest> {
        let request_id = RpcRequestId::new("console-rpc-42").expect("request id should be valid");
        let context = RpcContext::new(request_id, RpcTransport::Internal);
        RpcRequest::new(context, params)
    }

    #[test]
    fn rejects_multiline_commands_before_a_service_can_receive_them() {
        let error = ConsoleCommandRequest::new("server-42", "say first\nstop")
            .expect_err("multiline command must be rejected");

        assert_eq!(error.code(), RpcErrorCode::InvalidArgument);
    }

    #[test]
    fn debug_output_does_not_include_command_contents() {
        let request = ConsoleCommandRequest::new("server-42", "login secret-token")
            .expect("request should be valid");

        assert!(!format!("{request:?}").contains("secret-token"));
    }

    #[tokio::test]
    async fn dispatches_the_validated_command_once() {
        let method = SendConsoleCommand::new(RecordingConsoleService {
            requests: Mutex::new(Vec::new()),
            failure: None,
        });
        let params =
            ConsoleCommandRequest::new("server-42", "say hello").expect("request should be valid");

        dispatch(&method, rpc_request(params))
            .await
            .expect("command should be delivered");

        assert_eq!(
            method
                .service
                .requests
                .into_inner()
                .expect("test request log lock should not be poisoned"),
            vec![("server-42".into(), "say hello".into())]
        );
    }

    #[tokio::test]
    async fn maps_unverified_console_input_to_a_conflict() {
        let method = SendConsoleCommand::new(RecordingConsoleService {
            requests: Mutex::new(Vec::new()),
            failure: Some(ConsoleCommandServiceError::InputUnavailable),
        });
        let params =
            ConsoleCommandRequest::new("server-42", "stop").expect("request should be valid");

        let error = dispatch(&method, rpc_request(params))
            .await
            .expect_err("unverified console input must be rejected");

        assert_eq!(error.code(), RpcErrorCode::Conflict);
        assert_eq!(error.request_id().map(RpcRequestId::as_str), Some("console-rpc-42"));
    }
}
