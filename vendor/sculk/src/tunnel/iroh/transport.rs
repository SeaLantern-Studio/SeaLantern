//! 双向字节流桥接：iroh QUIC 双向流与 TCP 连接互转。

use std::time::Duration;

use super::*;

/// 半关闭后等待另一方向排空的超时。
const DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

/// 在 QUIC 双向流与 TCP 连接之间桥接数据。
///
/// 一方向结束后，等待另一方向剩余数据排空（带超时），避免截断。
pub(super) async fn bridge(
    mut send: SendStream,
    mut recv: RecvStream,
    tcp: TcpStream,
) -> crate::Result<()> {
    let (mut tcp_read, mut tcp_write) = tcp.into_split();

    tokio::select! {
        r = tokio::io::copy(&mut tcp_read, &mut send) => {
            let _ = send.finish();
            r.map_err(|e| crate::error::TunnelError::BridgeTcpToQuic(e.into()))?;
            // TCP->QUIC 方向结束，等待 QUIC->TCP 方向排空
            let _ = tokio::time::timeout(
                DRAIN_TIMEOUT,
                tokio::io::copy(&mut recv, &mut tcp_write),
            ).await;
        }
        r = tokio::io::copy(&mut recv, &mut tcp_write) => {
            r.map_err(|e| crate::error::TunnelError::BridgeQuicToTcp(e.into()))?;
            // QUIC->TCP 方向结束，等待 TCP->QUIC 方向排空
            let drain = tokio::time::timeout(
                DRAIN_TIMEOUT,
                tokio::io::copy(&mut tcp_read, &mut send),
            ).await;
            let _ = send.finish();
            let _ = drain;
        }
    }

    Ok(())
}
