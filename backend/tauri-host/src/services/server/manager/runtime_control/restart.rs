use std::{
    thread,
    time::{Duration, Instant},
};

use crate::{
    models::server::{ServerStatus, ServerStatusInfo},
    utils::server_status::status_detail_indicates_running,
};

use super::ServerManager;

const RESTART_STOP_POLL_INTERVAL_MS: u64 = 500;
const RESTART_STOP_TIMEOUT_SECS: u64 = 30;
const RESTART_AFTER_STOP_DELAY_SECS: u64 = 2;

pub(super) fn restart_server(manager: &ServerManager, id: &str) -> Result<(), String> {
    let status = manager.get_server_status(id);

    if status_requires_running_wait(&status) {
        manager.stop_server(id)?;
        wait_for_server_stop(manager, id, RESTART_STOP_TIMEOUT_SECS)?;
        thread::sleep(Duration::from_secs(RESTART_AFTER_STOP_DELAY_SECS));
    }

    manager.start_server(id)?;
    Ok(())
}

fn wait_for_server_stop(
    manager: &ServerManager,
    server_id: &str,
    timeout_secs: u64,
) -> Result<(), String> {
    let started = Instant::now();
    loop {
        let status = manager.get_server_status(server_id);
        if !status_requires_running_wait(&status) {
            return Ok(());
        }

        if started.elapsed().as_secs() >= timeout_secs {
            return Err(format!(
                "等待服务器停止超时: server_id={} timeout={}s last_status={} detail={} error={}",
                server_id,
                timeout_secs,
                status.status.as_str(),
                status.detail_message.as_deref().unwrap_or(""),
                status.error_message.as_deref().unwrap_or("")
            ));
        }

        thread::sleep(Duration::from_millis(RESTART_STOP_POLL_INTERVAL_MS));
    }
}

fn status_requires_running_wait(status: &ServerStatusInfo) -> bool {
    if matches!(
        status.status,
        ServerStatus::Running | ServerStatus::Starting | ServerStatus::Stopping
    ) {
        return true;
    }

    matches!(status.status, ServerStatus::Error)
        && status_detail_indicates_running(status.detail_message.as_deref())
}
