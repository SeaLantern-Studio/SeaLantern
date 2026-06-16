use crate::models::server::{ServerStatus, ServerStatusInfo};
pub(crate) use sea_lantern_runtime::{
    status_blocks_start, status_detail_health, status_detail_indicates_running,
    status_detail_runtime_kind, status_is_docker_command_ready, status_is_terminal_start_ready,
    status_detail_field,
    StatusLevel, StatusSnapshot,
};

impl StatusSnapshot for ServerStatusInfo {
    fn level(&self) -> StatusLevel {
        match self.status {
            ServerStatus::Stopped => StatusLevel::Stopped,
            ServerStatus::Starting => StatusLevel::Starting,
            ServerStatus::Running => StatusLevel::Running,
            ServerStatus::Stopping => StatusLevel::Stopping,
            ServerStatus::Error => StatusLevel::Error,
        }
    }

    fn detail_message(&self) -> Option<&str> {
        self.detail_message.as_deref()
    }
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
            terminal: None,
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
