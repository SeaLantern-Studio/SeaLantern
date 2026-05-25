//! join 侧连接、自动重连与本地 TCP 代理。

use super::*;

use super::auth::auth_send;
use super::monitor::spawn_path_monitor;
use super::transport::bridge;

/// Join 重连 supervisor 的运行时上下文。
pub(super) struct JoinContext {
    pub(super) listener: Arc<TcpListener>,
    pub(super) conns: Arc<Mutex<Vec<TrackedConnection>>>,
    pub(super) config: JoinConfig,
    /// 关闭信号：为 true 或 sender 被丢弃时，supervisor 应立即退出。
    pub(super) shutdown: tokio::sync::watch::Receiver<bool>,
}

/// Join 侧重连 supervisor。
pub(super) async fn reconnect_supervisor(
    endpoint: Endpoint,
    endpoint_id: iroh::EndpointId,
    mut conn: Connection,
    tx: mpsc::Sender<TunnelEvent>,
    mut ctx: JoinContext,
) {
    loop {
        let remote_id = PeerId::new(conn.remote_id().fmt_short().to_string());
        spawn_path_monitor(conn.clone(), remote_id, tx.clone(), ctx.config.event_delay);
        let accept_handle = spawn_join_accept_loop(conn.clone(), ctx.listener.clone(), tx.clone());
        let conn_handle = conn.weak_handle();

        // 等待连接关闭，或提前收到关闭信号
        let permanent_reject = tokio::select! {
            result = conn_handle.closed() => {
                accept_handle.abort();
                if let Some(closed) = result {
                    let rejected = is_permanent_rejection(&closed.reason);
                    let _ = tx
                        .send(TunnelEvent::Disconnected {
                            reason: closed.reason.to_string(),
                        })
                        .await;
                    rejected
                } else {
                    false
                }
            }
            _ = wait_for_shutdown(&mut ctx.shutdown) => {
                accept_handle.abort();
                return;
            }
        };

        if permanent_reject {
            return;
        }

        if ctx.config.max_retries == Some(0) {
            return;
        }

        let mut attempt: u32 = 0;
        let reconnected = loop {
            attempt = attempt.saturating_add(1);

            if let Some(max) = ctx.config.max_retries
                && attempt > max
            {
                let _ = tx
                    .send(TunnelEvent::Error {
                        message: format!("max retries ({max}) exceeded, giving up"),
                    })
                    .await;
                return;
            }

            let backoff = std::cmp::min(
                ctx.config
                    .base_backoff
                    .saturating_mul(2u32.saturating_pow(attempt - 1)),
                ctx.config.max_backoff,
            );

            let _ = tx.send(TunnelEvent::Reconnecting { attempt }).await;

            tracing::info!(attempt, ?backoff, "reconnecting...");

            // backoff sleep 期间响应关闭信号
            tokio::select! {
                _ = tokio::time::sleep(backoff) => {}
                _ = wait_for_shutdown(&mut ctx.shutdown) => return,
            }

            if *ctx.shutdown.borrow() {
                return;
            }

            match endpoint.connect(endpoint_id, ALPN).await {
                Ok(new_conn) => {
                    if let Some(ref password) = ctx.config.password
                        && let Err(e) = auth_send(&new_conn, password).await
                    {
                        tracing::warn!(attempt, "reconnect auth failed: {e}");
                        continue;
                    }
                    break new_conn;
                }
                Err(e) => {
                    tracing::warn!(attempt, "reconnect failed: {e}");
                    continue;
                }
            }
        };

        conn = reconnected;

        let lock_error = {
            match super::lock_mutex(&ctx.conns, "join connections") {
                Ok(mut guard) => {
                    guard.retain(|c| c.is_alive());
                    guard.push(TrackedConnection::new(&conn));
                    None
                }
                Err(e) => Some(e),
            }
        };
        if let Some(e) = lock_error {
            let _ = tx
                .send(TunnelEvent::Error {
                    message: e.to_string(),
                })
                .await;
            return;
        }

        let _ = tx.send(TunnelEvent::Reconnected).await;
        tracing::info!("reconnected successfully");
    }
}

/// value 变为 true 或 sender 被丢弃时返回（均视为关闭信号）。
async fn wait_for_shutdown(rx: &mut tokio::sync::watch::Receiver<bool>) {
    if *rx.borrow() {
        return;
    }
    let _ = rx.changed().await;
}

/// 启动 join accept loop。
fn spawn_join_accept_loop(
    conn: Connection,
    listener: Arc<TcpListener>,
    tx: mpsc::Sender<TunnelEvent>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(e) = join_accept_loop(conn, listener).await {
            let _ = tx
                .send(TunnelEvent::Error {
                    message: format!("join loop ended: {e}"),
                })
                .await;
        }
    })
}

/// 判断是否为不应重连的拒绝类型。
fn is_permanent_rejection(err: &ConnectionError) -> bool {
    if let ConnectionError::ApplicationClosed(ApplicationClose { error_code, .. }) = err {
        *error_code == CLOSE_AUTH_FAILED || *error_code == CLOSE_SERVER_FULL
    } else {
        false
    }
}

/// 含 auth的重试连接流程。
pub(super) async fn connect_with_retry(
    endpoint: &Endpoint,
    endpoint_id: iroh::EndpointId,
    config: &JoinConfig,
    tx: &mpsc::Sender<TunnelEvent>,
) -> crate::Result<Connection> {
    let max = config.initial_retries;
    let mut last_err = None;

    for attempt in 0..=max {
        if attempt > 0 {
            let backoff = std::cmp::min(
                config
                    .base_backoff
                    .saturating_mul(2u32.saturating_pow(attempt - 1)),
                config.max_backoff,
            );
            tracing::info!(attempt, ?backoff, "retrying initial connection...");
            let _ = tx.send(TunnelEvent::Reconnecting { attempt }).await;
            tokio::time::sleep(backoff).await;
        } else {
            tracing::info!("connecting to host...");
        }

        match endpoint.connect(endpoint_id, ALPN).await {
            Ok(conn) => {
                if let Some(ref password) = config.password {
                    auth_send(&conn, password).await?;
                }
                tracing::info!("connected to host");
                return Ok(conn);
            }
            Err(e) => {
                tracing::warn!(attempt, "connection failed: {e}");
                last_err = Some(e);
            }
        }
    }

    if let Some(err) = last_err {
        Err(crate::error::TunnelError::ConnectHostEndpoint(err.into()).into())
    } else {
        Err(crate::error::TunnelError::InitialConnectionExhausted {
            attempts: max.saturating_add(1),
        }
        .into())
    }
}

/// Join 侧本地监听并转发到 QUIC 双向流。
async fn join_accept_loop(conn: Connection, listener: Arc<TcpListener>) -> crate::Result<()> {
    loop {
        let (tcp, peer) = listener
            .accept()
            .await
            .map_err(|e| crate::error::TunnelError::AcceptLocalTcpClient(e.into()))?;
        tracing::info!(%peer, "MC client connected");

        let conn = conn.clone();
        tokio::spawn(async move {
            let (send, recv) = match conn.open_bi().await {
                Ok(pair) => pair,
                Err(e) => {
                    tracing::error!("failed to open QUIC stream: {e}");
                    return;
                }
            };

            if let Err(e) = bridge(send, recv, tcp).await {
                tracing::debug!(%peer, "stream closed: {e}");
            }
        });
    }
}
