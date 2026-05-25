//! host 侧连接接受循环与玩家会话管理。

use super::*;

use super::auth::auth_verify;
use super::monitor::spawn_path_monitor;
use super::session::HostSessions;
use super::transport::bridge;

/// Host 接受循环的运行时上下文。
pub(super) struct HostContext {
    pub(super) conns: Arc<Mutex<Vec<TrackedConnection>>>,
    pub(super) sessions: Arc<Mutex<HostSessions>>,
    pub(super) event_delay: Duration,
    pub(super) password: Option<String>,
    pub(super) max_players: Option<u32>,
}

/// Host 侧连接循环。
pub(super) async fn host_accept_loop(
    endpoint: Endpoint,
    mc_port: u16,
    tx: mpsc::Sender<TunnelEvent>,
    ctx: HostContext,
) -> crate::Result<()> {
    loop {
        let conn = endpoint
            .accept()
            .await
            .ok_or_else(|| {
                crate::error::TunnelError::AcceptHostConnection("endpoint closed".into())
            })?
            .await
            .map_err(|e| crate::error::TunnelError::AcceptHostConnection(e.into()))?;

        let remote_endpoint_id = conn.remote_id();
        let remote_id = PeerId::new(remote_endpoint_id.fmt_short().to_string());
        tracing::info!(remote = %remote_id, "player connected");

        if !capacity_check_with_grace(ctx.sessions.clone(), remote_endpoint_id, ctx.max_players)
            .await?
        {
            tracing::info!(remote = %remote_id, "server full, rejecting");
            let _ = tx
                .send(TunnelEvent::PlayerRejected {
                    id: remote_id.clone(),
                    reason: "server full".into(),
                })
                .await;
            spawn_rejected_conn_cleanup(conn, CLOSE_SERVER_FULL, b"server full", remote_id);
            continue;
        }

        if let Some(ref pwd) = ctx.password {
            match auth_verify(&conn, pwd).await {
                Ok(true) => {}
                Ok(false) => {
                    tracing::info!(remote = %remote_id, "auth failed");
                    let _ = tx
                        .send(TunnelEvent::AuthFailed {
                            id: remote_id.clone(),
                        })
                        .await;
                    spawn_rejected_conn_cleanup(conn, CLOSE_AUTH_FAILED, b"auth failed", remote_id);
                    continue;
                }
                Err(e) => {
                    tracing::warn!(remote = %remote_id, "auth error: {e}");
                    let _ = tx
                        .send(TunnelEvent::AuthFailed {
                            id: remote_id.clone(),
                        })
                        .await;
                    spawn_rejected_conn_cleanup(conn, CLOSE_AUTH_FAILED, b"auth failed", remote_id);
                    continue;
                }
            }
        }

        let (generation, is_reconnect, old_conn) = {
            let mut guard = super::lock_mutex(&ctx.sessions, "host sessions")?;
            guard.upsert(remote_endpoint_id, conn.clone())
        };
        if let Some(old_conn) = old_conn {
            old_conn.close(CLOSE_REPLACED_BY_RECONNECT, b"replaced by reconnect");
        }

        let conn_handle = conn.weak_handle();
        super::lock_mutex(&ctx.conns, "host connections")?.push(TrackedConnection::new(&conn));

        if is_reconnect {
            tracing::info!(remote = %remote_id, "player reconnected");
        } else {
            let _ = tx
                .send(TunnelEvent::PlayerJoined {
                    id: remote_id.clone(),
                })
                .await;
        }

        spawn_path_monitor(conn.clone(), remote_id.clone(), tx.clone(), ctx.event_delay);

        let tx_left = tx.clone();
        let left_id = remote_id.clone();
        let sessions_on_close = ctx.sessions.clone();
        tokio::spawn(async move {
            let reason = match conn_handle.closed().await {
                Some(closed) => closed.reason.to_string(),
                None => "connection closed".to_string(),
            };
            let mut lock_error = None;
            let should_emit_left = match super::lock_mutex(&sessions_on_close, "host sessions") {
                Ok(mut guard) => guard.remove_if_current(&remote_endpoint_id, generation),
                Err(e) => {
                    lock_error = Some(e);
                    false
                }
            };
            if let Some(e) = lock_error {
                let _ = tx_left
                    .send(TunnelEvent::Error {
                        message: e.to_string(),
                    })
                    .await;
            }
            if should_emit_left {
                let _ = tx_left
                    .send(TunnelEvent::PlayerLeft {
                        id: left_id,
                        reason,
                    })
                    .await;
            } else {
                tracing::debug!(remote = %left_id, "stale connection closed, ignored");
            }
        });

        tokio::spawn(async move {
            if let Err(e) = host_handle_conn(conn, mc_port).await {
                tracing::debug!("connection ended: {e}");
            }
        });
    }
}

/// 满员时短暂复核，避免重连误拒绝。
async fn capacity_check_with_grace(
    sessions: Arc<Mutex<HostSessions>>,
    incoming_id: EndpointId,
    max_players: Option<u32>,
) -> crate::Result<bool> {
    capacity_check_with_grace_delay(sessions, incoming_id, max_players, FULL_RECHECK_DELAY).await
}

async fn capacity_check_with_grace_delay(
    sessions: Arc<Mutex<HostSessions>>,
    incoming_id: EndpointId,
    max_players: Option<u32>,
    recheck_delay: Duration,
) -> crate::Result<bool> {
    let Some(max) = max_players else {
        return Ok(true);
    };

    let has_capacity_or_reconnect = |guard: &HostSessions| {
        guard.contains(&incoming_id)
            || u32::try_from(guard.active_players()).unwrap_or(u32::MAX) < max
    };

    {
        let guard = super::lock_mutex(&sessions, "host sessions")?;
        if has_capacity_or_reconnect(&guard) {
            return Ok(true);
        }
    }

    tokio::time::sleep(recheck_delay).await;

    let guard = super::lock_mutex(&sessions, "host sessions")?;
    Ok(has_capacity_or_reconnect(&guard))
}

/// 拒绝连接后异步 close 并等待收敛。
fn spawn_rejected_conn_cleanup(
    conn: Connection,
    code: VarInt,
    reason: &'static [u8],
    remote_id: PeerId,
) {
    tokio::spawn(async move {
        let handle = conn.weak_handle();
        conn.close(code, reason);
        let _ = tokio::time::timeout(REJECT_DRAIN_TIMEOUT, handle.closed()).await;
        tracing::debug!(remote = %remote_id, "rejected connection cleanup finished");
    });
}

/// 处理单个连接内的双向流转发。
async fn host_handle_conn(conn: Connection, mc_port: u16) -> crate::Result<()> {
    loop {
        let (send, recv) = conn
            .accept_bi()
            .await
            .map_err(|e| crate::error::TunnelError::AcceptQuicBiStream(e.into()))?;

        tokio::spawn(async move {
            let tcp = match TcpStream::connect(("127.0.0.1", mc_port)).await {
                Ok(tcp) => tcp,
                Err(e) => {
                    tracing::error!(mc_port, "failed to connect MC server: {e}");
                    return;
                }
            };

            if let Err(e) = bridge(send, recv, tcp).await {
                tracing::debug!("stream closed: {e}");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_endpoint_id() -> EndpointId {
        let bytes: [u8; 32] = rand::random();
        iroh::SecretKey::from_bytes(&bytes).public()
    }

    #[tokio::test]
    async fn capacity_allows_reconnect_when_full() {
        let endpoint_id = test_endpoint_id();
        let sessions = Arc::new(Mutex::new(HostSessions::default()));
        {
            let lock_res = sessions.lock();
            assert!(lock_res.is_ok(), "host sessions lock poisoned");
            if let Ok(mut guard) = lock_res {
                guard.insert_for_test(endpoint_id, 1);
            } else {
                return;
            }
        }

        let allowed_res =
            capacity_check_with_grace_delay(sessions, endpoint_id, Some(1), Duration::ZERO).await;
        assert!(allowed_res.is_ok(), "capacity check failed");
        let allowed = if let Ok(v) = allowed_res { v } else { return };
        assert!(allowed);
    }

    #[tokio::test]
    async fn capacity_rejects_new_player_when_full() {
        let existing = test_endpoint_id();
        let incoming = test_endpoint_id();
        let sessions = Arc::new(Mutex::new(HostSessions::default()));
        {
            let lock_res = sessions.lock();
            assert!(lock_res.is_ok(), "host sessions lock poisoned");
            if let Ok(mut guard) = lock_res {
                guard.insert_for_test(existing, 1);
            } else {
                return;
            }
        }

        let allowed_res =
            capacity_check_with_grace_delay(sessions, incoming, Some(1), Duration::ZERO).await;
        assert!(allowed_res.is_ok(), "capacity check failed");
        let allowed = if let Ok(v) = allowed_res { v } else { return };
        assert!(!allowed);
    }
}
