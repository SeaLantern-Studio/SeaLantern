use crate::models::server::{ServerStatus, ServerStatusInfo};

pub fn status_detail_indicates_running(detail: Option<&str>) -> bool {
    detail.is_some_and(|value| {
        value
            .split_whitespace()
            .any(|part| part.eq_ignore_ascii_case("running=true"))
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

pub fn status_is_docker_command_ready(status: &ServerStatusInfo) -> bool {
    if status.status != ServerStatus::Running {
        return false;
    }

    let detail = status.detail_message.as_deref();
    if !status_detail_runtime_kind(detail)
        .is_some_and(|runtime| runtime.eq_ignore_ascii_case("docker_itzg"))
    {
        return true;
    }

    status_detail_health(detail)
        .map(|health| health.eq_ignore_ascii_case("healthy"))
        .unwrap_or(true)
}

pub fn status_is_terminal_start_ready(status: &ServerStatusInfo) -> bool {
    match status.status {
        ServerStatus::Error | ServerStatus::Stopped => true,
        ServerStatus::Running => status_is_docker_command_ready(status),
        ServerStatus::Starting | ServerStatus::Stopping => false,
    }
}

pub fn status_blocks_start(status: &ServerStatusInfo) -> bool {
    matches!(
        status.status,
        ServerStatus::Running | ServerStatus::Starting | ServerStatus::Stopping
    ) || matches!(status.status, ServerStatus::Error)
        && status_detail_indicates_running(status.detail_message.as_deref())
}

#[cfg(test)]
mod tests {
    use super::{
        status_detail_field, status_detail_health, status_is_docker_command_ready,
        status_is_terminal_start_ready,
    };
    use crate::models::server::{ServerStatus, ServerStatusInfo};

    fn status_with_detail(status: ServerStatus, detail: &str) -> ServerStatusInfo {
        ServerStatusInfo {
            id: "server-1".to_string(),
            status,
            pid: Some(1),
            uptime: Some(1),
            detail_message: Some(detail.to_string()),
            error_message: None,
        }
    }

    #[test]
    fn status_detail_field_extracts_health_token() {
        let detail = Some(
            "runtime=docker_itzg container=sea-test state=running running=true health=healthy",
        );

        assert_eq!(status_detail_field(detail, "health"), Some("healthy"));
        assert_eq!(status_detail_health(detail), Some("healthy"));
    }

    #[test]
    fn status_is_docker_command_ready_requires_healthy_when_health_exists() {
        let starting = status_with_detail(
            ServerStatus::Running,
            "runtime=docker_itzg container=sea-test state=running running=true health=starting",
        );
        let healthy = status_with_detail(
            ServerStatus::Running,
            "runtime=docker_itzg container=sea-test state=running running=true health=healthy",
        );

        assert!(!status_is_docker_command_ready(&starting));
        assert!(status_is_docker_command_ready(&healthy));
    }

    #[test]
    fn status_is_docker_command_ready_treats_missing_or_none_health_as_ready() {
        let no_healthcheck = status_with_detail(
            ServerStatus::Running,
            "runtime=docker_itzg container=sea-test state=running running=true health=none",
        );
        let local = status_with_detail(ServerStatus::Running, "runtime=local is_running=true");

        assert_eq!(status_detail_health(no_healthcheck.detail_message.as_deref()), None);
        assert!(status_is_docker_command_ready(&no_healthcheck));
        assert!(status_is_docker_command_ready(&local));
    }

    #[test]
    fn status_is_terminal_start_ready_keeps_docker_health_starting_non_terminal() {
        let docker_starting = status_with_detail(
            ServerStatus::Running,
            "runtime=docker_itzg container=sea-test state=running running=true health=starting",
        );
        let docker_healthy = status_with_detail(
            ServerStatus::Running,
            "runtime=docker_itzg container=sea-test state=running running=true health=healthy",
        );

        assert!(!status_is_terminal_start_ready(&docker_starting));
        assert!(status_is_terminal_start_ready(&docker_healthy));
    }

    #[test]
    fn status_is_terminal_start_ready_accepts_running_without_healthcheck() {
        let docker_without_healthcheck = status_with_detail(
            ServerStatus::Running,
            "runtime=docker_itzg container=sea-test state=running running=true health=none",
        );

        assert!(status_is_terminal_start_ready(&docker_without_healthcheck));
    }
}
