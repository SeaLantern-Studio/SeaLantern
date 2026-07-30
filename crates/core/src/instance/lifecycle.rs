use std::fmt;
use std::time::Duration;

use super::Instance;
use crate::observability;

/// 受管实例的稳定生命周期状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceLifecycleState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

impl InstanceLifecycleState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Error => "error",
        }
    }

    pub const fn is_active(self) -> bool {
        matches!(self, Self::Starting | Self::Running | Self::Stopping)
    }

    const fn requires_stop_before_restart(self) -> bool {
        !matches!(self, Self::Stopped)
    }
}

/// 生命周期状态迁移的原因。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceLifecycleAction {
    StartRequested,
    Started,
    StopRequested,
    Stopped,
    Failed,
}

/// 验证并应用一次生命周期迁移。
pub fn transition(
    state: InstanceLifecycleState,
    action: InstanceLifecycleAction,
) -> Result<InstanceLifecycleState, LifecycleTransitionError> {
    let next = match (state, action) {
        (
            InstanceLifecycleState::Stopped | InstanceLifecycleState::Error,
            InstanceLifecycleAction::StartRequested,
        ) => InstanceLifecycleState::Starting,
        (InstanceLifecycleState::Starting, InstanceLifecycleAction::Started) => {
            InstanceLifecycleState::Running
        }
        (
            InstanceLifecycleState::Starting
            | InstanceLifecycleState::Running
            | InstanceLifecycleState::Error,
            InstanceLifecycleAction::StopRequested,
        ) => InstanceLifecycleState::Stopping,
        (InstanceLifecycleState::Stopping, InstanceLifecycleAction::Stopped) => {
            InstanceLifecycleState::Stopped
        }
        (_, InstanceLifecycleAction::Failed) => InstanceLifecycleState::Error,
        _ => {
            return Err(LifecycleTransitionError { state, action });
        }
    };
    Ok(next)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleTransitionError {
    pub state: InstanceLifecycleState,
    pub action: InstanceLifecycleAction,
}

impl fmt::Display for LifecycleTransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cannot apply lifecycle action {:?} while instance is {}",
            self.action,
            self.state.as_str()
        )
    }
}

impl std::error::Error for LifecycleTransitionError {}

/// 重启时传递给运行时驱动的等待策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestartPolicy {
    pub stop_timeout: Duration,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self { stop_timeout: Duration::from_secs(30) }
    }
}

/// 运行时驱动完成重启所需的最小能力。
///
/// 具体轮询、进程控制、Docker 调用和异步实现均由上层运行时提供。
pub trait InstanceRestartDriver {
    type Error: std::error::Error + Send + Sync + 'static;

    fn state(&self, instance: &Instance) -> Result<InstanceLifecycleState, Self::Error>;

    fn request_stop(&self, instance: &Instance) -> Result<(), Self::Error>;

    fn await_terminal(
        &self,
        instance: &Instance,
        timeout: Duration,
    ) -> Result<InstanceLifecycleState, Self::Error>;

    fn start(&self, instance: &Instance) -> Result<(), Self::Error>;
}

/// 重启完成后的领域结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RestartOutcome {
    pub previous_state: InstanceLifecycleState,
    pub stop_requested: bool,
}

#[derive(Debug)]
pub enum RestartError<E> {
    State(E),
    Stop(E),
    AwaitTerminal(E),
    TerminalState { state: InstanceLifecycleState },
    Start(E),
}

impl<E: fmt::Display> fmt::Display for RestartError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State(error) => write!(formatter, "failed to read instance state: {error}"),
            Self::Stop(error) => write!(formatter, "failed to request instance stop: {error}"),
            Self::AwaitTerminal(error) => {
                write!(formatter, "failed while waiting for instance stop: {error}")
            }
            Self::TerminalState { state } => {
                write!(
                    formatter,
                    "instance did not reach a restartable terminal state: {}",
                    state.as_str()
                )
            }
            Self::Start(error) => write!(formatter, "failed to start instance after stop: {error}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for RestartError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::State(error)
            | Self::Stop(error)
            | Self::AwaitTerminal(error)
            | Self::Start(error) => Some(error),
            Self::TerminalState { .. } => None,
        }
    }
}

/// 执行一次由运行时驱动提供具体能力的重启。
pub fn restart_instance<D: InstanceRestartDriver>(
    driver: &D,
    instance: &Instance,
    policy: RestartPolicy,
) -> Result<RestartOutcome, RestartError<D::Error>> {
    let previous_state = driver.state(instance).map_err(RestartError::State)?;
    observability::instance_restart_requested(instance.id.as_str(), previous_state.as_str());

    let stop_requested = previous_state.requires_stop_before_restart();
    if stop_requested {
        if let Err(error) = driver.request_stop(instance) {
            let error = RestartError::Stop(error);
            observability::instance_restart_failed(
                instance.id.as_str(),
                previous_state.as_str(),
                &error,
            );
            return Err(error);
        }

        let terminal_state = match driver.await_terminal(instance, policy.stop_timeout) {
            Ok(state) => state,
            Err(error) => {
                let error = RestartError::AwaitTerminal(error);
                observability::instance_restart_failed(
                    instance.id.as_str(),
                    previous_state.as_str(),
                    &error,
                );
                return Err(error);
            }
        };
        if !matches!(
            terminal_state,
            InstanceLifecycleState::Stopped | InstanceLifecycleState::Error
        ) {
            let error = RestartError::TerminalState { state: terminal_state };
            observability::instance_restart_failed(
                instance.id.as_str(),
                previous_state.as_str(),
                &error,
            );
            return Err(error);
        }
    }

    if let Err(error) = driver.start(instance) {
        let error = RestartError::Start(error);
        observability::instance_restart_failed(
            instance.id.as_str(),
            previous_state.as_str(),
            &error,
        );
        return Err(error);
    }

    observability::instance_restart_completed(instance.id.as_str(), previous_state.as_str());
    Ok(RestartOutcome { previous_state, stop_requested })
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::io;
    use std::path::PathBuf;

    use super::{
        restart_instance, transition, InstanceLifecycleAction, InstanceLifecycleState,
        InstanceRestartDriver, RestartPolicy,
    };
    use crate::instance::{Instance, InstanceId, InstanceSpec, LocalLaunch, StartupMode};

    struct Driver {
        state: InstanceLifecycleState,
        terminal: InstanceLifecycleState,
        calls: RefCell<Vec<&'static str>>,
    }

    impl InstanceRestartDriver for Driver {
        type Error = io::Error;

        fn state(&self, _: &Instance) -> Result<InstanceLifecycleState, Self::Error> {
            self.calls.borrow_mut().push("state");
            Ok(self.state)
        }

        fn request_stop(&self, _: &Instance) -> Result<(), Self::Error> {
            self.calls.borrow_mut().push("stop");
            Ok(())
        }

        fn await_terminal(
            &self,
            _: &Instance,
            _: std::time::Duration,
        ) -> Result<InstanceLifecycleState, Self::Error> {
            self.calls.borrow_mut().push("wait");
            Ok(self.terminal)
        }

        fn start(&self, _: &Instance) -> Result<(), Self::Error> {
            self.calls.borrow_mut().push("start");
            Ok(())
        }
    }

    fn instance() -> Instance {
        Instance::new(InstanceSpec {
            id: InstanceId::new("restartable").unwrap(),
            name: "Restartable".into(),
            aliases: Vec::new(),
            core_type: "paper".into(),
            core_version: String::new(),
            game_version: "1.21.1".into(),
            directory: PathBuf::from("servers/restartable"),
            port: 25565,
            max_memory_mib: 0,
            min_memory_mib: 0,
            created_at_unix_secs: 0,
            last_started_at_unix_secs: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(PathBuf::from("servers/restartable/server.jar")),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        })
        .unwrap()
    }

    #[test]
    fn restart_stops_waits_and_starts_an_active_instance() {
        let driver = Driver {
            state: InstanceLifecycleState::Running,
            terminal: InstanceLifecycleState::Stopped,
            calls: RefCell::new(Vec::new()),
        };

        let result = restart_instance(&driver, &instance(), RestartPolicy::default()).unwrap();

        assert!(result.stop_requested);
        assert_eq!(*driver.calls.borrow(), ["state", "stop", "wait", "start"]);
    }

    #[test]
    fn restart_starts_a_stopped_instance_without_a_stop_request() {
        let driver = Driver {
            state: InstanceLifecycleState::Stopped,
            terminal: InstanceLifecycleState::Stopped,
            calls: RefCell::new(Vec::new()),
        };

        let result = restart_instance(&driver, &instance(), RestartPolicy::default()).unwrap();

        assert!(!result.stop_requested);
        assert_eq!(*driver.calls.borrow(), ["state", "start"]);
    }

    #[test]
    fn lifecycle_transition_rejects_start_completion_before_a_start_request() {
        assert!(
            transition(InstanceLifecycleState::Stopped, InstanceLifecycleAction::Started).is_err()
        );
    }
}
