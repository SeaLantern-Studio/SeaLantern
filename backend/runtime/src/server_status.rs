#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusLevel {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

pub trait StatusSnapshot {
    fn level(&self) -> StatusLevel;
    fn detail_message(&self) -> Option<&str>;
}

pub fn status_detail_indicates_running(detail: Option<&str>) -> bool {
    detail.is_some_and(|value| {
        value.split_whitespace().any(|part| {
            part.eq_ignore_ascii_case("running=true")
                || part.eq_ignore_ascii_case("is_running=true")
        })
    })
}

pub fn status_detail_field<'a>(detail: Option<&'a str>, key: &str) -> Option<&'a str> {
    detail?.split_whitespace().find_map(|part| {
        let (field, value) = part.split_once('=')?;
        if field == key {
            Some(value)
        } else {
            None
        }
    })
}

pub fn status_detail_runtime_kind(detail: Option<&str>) -> Option<&str> {
    status_detail_field(detail, "runtime")
}

pub fn status_detail_health(detail: Option<&str>) -> Option<&str> {
    status_detail_field(detail, "health")
        .filter(|value| !value.trim().is_empty() && !value.eq_ignore_ascii_case("none"))
}

pub fn status_is_docker_command_ready(status: &impl StatusSnapshot) -> bool {
    if status.level() != StatusLevel::Running {
        return false;
    }

    let detail = status.detail_message();
    if !status_detail_runtime_kind(detail)
        .is_some_and(|runtime| runtime.eq_ignore_ascii_case("docker_itzg"))
    {
        return true;
    }

    status_detail_health(detail)
        .map(|health| health.eq_ignore_ascii_case("healthy"))
        .unwrap_or(true)
}

pub fn status_is_terminal_start_ready(status: &impl StatusSnapshot) -> bool {
    match status.level() {
        StatusLevel::Error | StatusLevel::Stopped => true,
        StatusLevel::Running => status_is_docker_command_ready(status),
        StatusLevel::Starting | StatusLevel::Stopping => false,
    }
}

pub fn status_blocks_start(status: &impl StatusSnapshot) -> bool {
    matches!(
        status.level(),
        StatusLevel::Running | StatusLevel::Starting | StatusLevel::Stopping
    ) || matches!(status.level(), StatusLevel::Error)
        && status_detail_indicates_running(status.detail_message())
}

#[cfg(test)]
mod tests {
    use super::{
        status_blocks_start, status_detail_indicates_running, StatusLevel, StatusSnapshot,
    };

    #[derive(Clone, Copy)]
    struct TestStatus {
        level: StatusLevel,
        detail: Option<&'static str>,
    }

    impl StatusSnapshot for TestStatus {
        fn level(&self) -> StatusLevel {
            self.level
        }

        fn detail_message(&self) -> Option<&str> {
            self.detail
        }
    }

    #[test]
    fn status_detail_indicates_running_accepts_local_is_running_token() {
        assert!(status_detail_indicates_running(Some(
            "runtime=local is_running=true exit_code=none"
        )));
        assert!(!status_detail_indicates_running(Some(
            "runtime=local is_running=false exit_code=7"
        )));
    }

    #[test]
    fn status_blocks_start_treats_error_with_local_running_detail_as_active() {
        let status = TestStatus {
            level: StatusLevel::Error,
            detail: Some("runtime=local is_running=true exit_code=none"),
        };

        assert!(status_blocks_start(&status));
    }
}
