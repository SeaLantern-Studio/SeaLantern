//! 服务器控制台输入的宿主能力端口。

use super::ConsoleCommandExecutor;
use crate::observability;

/// 由受管服务器运行时返回的控制台输入失败类别。
///
/// 宿主实现必须在向任何子进程写入前验证该实例的 stdin 确实属于受管服务端进程。脚本、
/// shell 或自定义启动包装进程的 stdin 必须返回 [`Self::InputUnavailable`]，而不能被视为
/// 可写控制台。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsoleCommandServiceError {
    /// 指定的服务器实例不存在。
    InstanceNotFound,
    /// 实例未运行，或其 stdin 不属于已验证的服务端进程。
    InputUnavailable,
    /// 已验证的服务端控制台暂时无法接收输入。
    DeliveryFailed,
}

/// 向已验证服务器控制台发送单行命令的宿主能力。
///
/// 该端口不接受 Tauri、HTTP 或插件运行时类型。实现必须保证错误不会携带命令正文、凭据
/// 或主机路径；底层失败详情应仅写入受控的宿主日志。
pub trait ConsoleCommandService: Send + Sync {
    /// 向指定实例的受管服务端进程发送一条已验证命令。
    fn send_console_command(
        &self,
        instance_id: &str,
        command: &str,
    ) -> Result<(), ConsoleCommandServiceError>;
}

/// 调度一条服务器控制台命令并记录一次脱敏追踪事件。
///
/// 命令正文可能包含密码、令牌或玩家输入，事件只记录文本长度（按字符计）、实例标识和结果。
/// 验证和授权由调用方及具体执行器承担，避免在迁移期间改变已有命令语义。
pub fn dispatch_console_command<E>(
    executor: &E,
    instance_id: &str,
    command: &str,
) -> Result<(), E::Error>
where
    E: ConsoleCommandExecutor,
{
    let result = executor.send_console_command(instance_id, command);
    observability::console_command_dispatched(
        instance_id,
        command_char_count(command),
        result.is_ok(),
    );
    result
}

fn command_char_count(command: &str) -> usize {
    command.chars().count()
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fmt;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TestError;

    impl fmt::Display for TestError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "test executor failed")
        }
    }

    struct RecordingExecutor {
        requests: RefCell<Vec<(String, String)>>,
        failure: Option<TestError>,
    }

    impl ConsoleCommandExecutor for RecordingExecutor {
        type Error = TestError;

        fn send_console_command(
            &self,
            instance_id: &str,
            command: &str,
        ) -> Result<(), Self::Error> {
            self.requests
                .borrow_mut()
                .push((instance_id.into(), command.into()));
            self.failure.map_or(Ok(()), Err)
        }
    }

    #[test]
    fn dispatches_the_original_instance_and_command() {
        let executor = RecordingExecutor {
            requests: RefCell::new(Vec::new()),
            failure: None,
        };

        dispatch_console_command(&executor, "instance-a", "say hello").unwrap();

        assert_eq!(executor.requests.into_inner(), vec![("instance-a".into(), "say hello".into())]);
    }

    #[test]
    fn preserves_executor_errors() {
        let executor = RecordingExecutor {
            requests: RefCell::new(Vec::new()),
            failure: Some(TestError),
        };

        assert_eq!(dispatch_console_command(&executor, "instance-a", "stop"), Err(TestError));
    }

    #[test]
    fn counts_unicode_scalars_instead_of_utf8_bytes() {
        assert_eq!(command_char_count("say 你好"), 6);
    }
}
