use crate::observability;
use crate::rpc::service::ConsoleCommandExecutor;

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
