use crate::models::server::{ServerStatus, ServerStatusInfo};
use crate::services::server::manager::i18n::manager_t1;

use super::super::common::current_timestamp_secs;
use super::ServerManager;
use crate::services::server::runtime;

/// 读取服务器当前运行状态
pub(super) fn get_server_status(manager: &ServerManager, id: &str) -> ServerStatusInfo {
    let server = match manager.find_server_clone_optional(id) {
        Ok(server) => server,
        Err(err) => {
            return ServerStatusInfo {
                id: id.to_string(),
                status: ServerStatus::Error,
                pid: None,
                uptime: None,
                detail_message: None,
                error_message: Some(err),
                terminal: None,
            };
        }
    };

    let Some(server) = server else {
        return ServerStatusInfo {
            id: id.to_string(),
            status: ServerStatus::Stopped,
            pid: None,
            uptime: None,
            detail_message: None,
            error_message: Some(manager_t1("server.manager.server_not_found", id.to_string())),
            terminal: None,
        };
    };

    let resolved_runtime = match runtime::resolve_runtime(&server) {
        Ok(runtime) => runtime,
        Err(err) => {
            return ServerStatusInfo {
                id: id.to_string(),
                status: ServerStatus::Error,
                pid: None,
                uptime: None,
                detail_message: None,
                error_message: Some(err),
                terminal: None,
            };
        }
    };

    let snapshot = match resolved_runtime.status_with_manager(manager, &server) {
        Ok(snapshot) => snapshot,
        Err(err) => {
            return ServerStatusInfo {
                id: id.to_string(),
                status: ServerStatus::Error,
                pid: None,
                uptime: None,
                detail_message: None,
                error_message: Some(err),
                terminal: None,
            };
        }
    };

    let uptime = server
        .last_started_at
        .and_then(|started_at| current_timestamp_secs().checked_sub(started_at));

    ServerStatusInfo {
        id: id.to_string(),
        status: snapshot.status,
        pid: snapshot.pid,
        uptime,
        detail_message: snapshot.detail_message,
        error_message: snapshot.error_message,
        terminal: snapshot.terminal,
    }
}

#[cfg(test)]
mod tests {
    use super::get_server_status;
    use crate::models::server::ServerStatus;
    use crate::services::server::manager::ServerManager;
    use std::sync::Arc;

    #[test]
    fn get_server_status_surfaces_server_lock_failures_instead_of_missing_server() {
        let manager = Arc::new(ServerManager::new());
        let cloned = Arc::clone(&manager);
        let poison_thread = std::thread::spawn(move || {
            let _guard = cloned
                .servers
                .lock()
                .expect("servers lock should be acquired");
            panic!("poison server list lock");
        });
        assert!(poison_thread.join().is_err(), "poison thread should panic");

        let status = get_server_status(&manager, "server-1");

        assert_eq!(status.status, ServerStatus::Error);
        assert_eq!(status.pid, None);
        assert_eq!(status.uptime, None);
        assert_eq!(status.detail_message, None);
        assert_eq!(status.error_message.as_deref(), Some("servers lock poisoned"));
    }
}
