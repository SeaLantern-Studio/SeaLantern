//! 应用层密码认证协议，用于 host/join 握手阶段。
//!
//! 协议格式：join 侧发送 `[AUTH_VERSION, ...password_bytes]`，
//! host 侧验证后回写单字节 `AUTH_OK` 或 `AUTH_REJECTED`。

use subtle::ConstantTimeEq;

use super::*;

const AUTH_VERSION: u8 = 0x01;
/// 认证数据读取的固定上限。
const AUTH_READ_LIMIT: usize = 512;
const AUTH_OK: u8 = 0x00;
const AUTH_REJECTED: u8 = 0x01;

/// Join 侧发送密码并等待验证结果。
pub(super) async fn auth_send(conn: &Connection, password: &str) -> crate::Result<()> {
    let (mut send, mut recv) = conn
        .open_bi()
        .await
        .map_err(|e| crate::error::TunnelError::OpenAuthStream(e.into()))?;

    let mut buf = Vec::with_capacity(1 + password.len());
    buf.push(AUTH_VERSION);
    buf.extend_from_slice(password.as_bytes());
    send.write_all(&buf)
        .await
        .map_err(|e| crate::error::TunnelError::WriteAuthPayload(e.into()))?;
    send.finish()
        .map_err(|e| crate::error::TunnelError::FinishAuthStream(e.into()))?;

    let result = recv
        .read_to_end(1)
        .await
        .map_err(|e| crate::error::TunnelError::ReadAuthResult(e.into()))?;

    if result.first() == Some(&AUTH_OK) {
        Ok(())
    } else {
        Err(crate::error::TunnelError::AuthRejectedByHost.into())
    }
}

/// Host 侧验证密码并回写结果。
pub(super) async fn auth_verify(conn: &Connection, expected: &str) -> crate::Result<bool> {
    let (mut send, mut recv) = conn
        .accept_bi()
        .await
        .map_err(|e| crate::error::TunnelError::AcceptAuthStream(e.into()))?;

    let data = recv
        .read_to_end(AUTH_READ_LIMIT)
        .await
        .map_err(|e| crate::error::TunnelError::ReadAuthPayload(e.into()))?;

    if data.is_empty() {
        send.write_all(&[AUTH_REJECTED])
            .await
            .map_err(|e| crate::error::TunnelError::WriteAuthRejected(e.into()))?;
        send.finish()
            .map_err(|e| crate::error::TunnelError::FinishAuthStream(e.into()))?;
        return Ok(false);
    }

    let version = data[0];
    if version != AUTH_VERSION {
        send.write_all(&[AUTH_REJECTED])
            .await
            .map_err(|e| crate::error::TunnelError::WriteAuthRejected(e.into()))?;
        send.finish()
            .map_err(|e| crate::error::TunnelError::FinishAuthStream(e.into()))?;
        return Ok(false);
    }

    let password = &data[1..];
    let ok: bool = password.ct_eq(expected.as_bytes()).into();

    send.write_all(&[if ok { AUTH_OK } else { AUTH_REJECTED }])
        .await
        .map_err(|e| crate::error::TunnelError::WriteAuthDecision(e.into()))?;
    send.finish()
        .map_err(|e| crate::error::TunnelError::FinishAuthStream(e.into()))?;

    Ok(ok)
}
