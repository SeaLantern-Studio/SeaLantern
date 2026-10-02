//! 事件 WebSocket 传输适配器（axum）。
//!
//! 客户端连接 `GET /api/events/ws` 后，服务端把事件总线上的事件实时推送出去。
//!
//! # 消息形状
//!
//! 服务端 → 客户端：
//!
//! ```json
//! { "event": "server-log-line", "instance_id": "…", "line": { … } }  // 事件
//! { "type": "lagged", "skipped": 12 }                                 // 落后提示
//! { "type": "pong" }                                                  // 心跳响应
//! ```
//!
//! 客户端 → 服务端（可选，仅用于心跳）：
//!
//! ```json
//! { "type": "ping" }
//! ```
//!
//! 区分规则：带 `event` 字段的是事件，带 `type` 字段的是控制消息。
//!
//! # 心跳
//!
//! 服务端每 [`HEARTBEAT_INTERVAL`] 发送一个协议层 Ping 帧，浏览器会自动回
//! Pong，据此探测死连接并清理资源。
//!
//! # 补漏
//!
//! 本适配器**不做补发**。消费落后（收到 `lagged`）或断线重连后，客户端应调用
//! `GET /api/instances/{id}/logs?since=<sequence>` 补齐历史——事件通道只保证
//! 实时性，不保证送达。

use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use tokio::sync::broadcast::error::RecvError;

use super::ServerEventBus;

/// 协议层心跳间隔。
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// 升级为 WebSocket 并开始推送事件。
pub async fn events_ws(State(bus): State<Arc<ServerEventBus>>, ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(move |socket| push_events(socket, bus))
}

/// 单连接的事件推送循环。
///
/// 三路并发：事件总线推送、客户端消息（心跳）、协议层心跳。任一方向出错即结束
/// 本连接——客户端负责重连，并用 REST 接口补齐断开期间的日志。
async fn push_events(mut socket: WebSocket, bus: Arc<ServerEventBus>) {
    let mut events = bus.subscribe();
    let mut heartbeat = tokio::time::interval(HEARTBEAT_INTERVAL);
    heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            // ── 事件总线 → 客户端 ──
            received = events.recv() => match received {
                Ok(event) => {
                    if send_json(&mut socket, &event).await.is_err() {
                        break;
                    }
                }
                // 本连接消费落后：提示客户端补漏，不在此回放。
                Err(RecvError::Lagged(skipped)) => {
                    let notice = LaggedNotice { kind: "lagged", skipped };
                    if send_json(&mut socket, &notice).await.is_err() {
                        break;
                    }
                }
                // 总线关闭：仅进程退出时发生，防御性退出。
                Err(RecvError::Closed) => break,
            },

            // ── 客户端 → 服务端（当前仅应用层心跳）──
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Text(text))) => {
                    if handle_client_text(&mut socket, text.as_str()).await.is_err() {
                        break;
                    }
                }
                // 对端正常关闭或连接结束。
                Some(Ok(Message::Close(_))) | None => break,
                Some(Err(_)) => break,
                // Ping / Pong 由协议层处理，Binary 暂不使用。
                _ => {}
            },

            // ── 协议层心跳 ──
            _ = heartbeat.tick() => {
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break;
                }
            }
        }
    }
}

/// 处理客户端应用层消息；当前仅识别心跳 `{"type":"ping"}`。
///
/// 非法 JSON 静默忽略：单个消息格式问题不应导致连接断开。
async fn handle_client_text(socket: &mut WebSocket, text: &str) -> Result<(), axum::Error> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text) else {
        return Ok(());
    };
    if value.get("type").and_then(serde_json::Value::as_str) == Some("ping") {
        return send_json(socket, &PongNotice { kind: "pong" }).await;
    }
    Ok(())
}

/// 序列化并发送一条 JSON 文本消息。
///
/// 序列化失败时跳过该条（不应因单条消息断开连接）；返回 `Err` 表示连接已不可用。
async fn send_json<T: serde::Serialize>(
    socket: &mut WebSocket,
    value: &T,
) -> Result<(), axum::Error> {
    let Ok(text) = serde_json::to_string(value) else {
        return Ok(());
    };
    socket.send(Message::Text(text.into())).await
}

/// 控制消息：本连接已落后 `skipped` 条事件，客户端应用 REST 接口补漏。
#[derive(serde::Serialize)]
struct LaggedNotice {
    #[serde(rename = "type")]
    kind: &'static str,
    skipped: u64,
}

/// 控制消息：心跳响应。
#[derive(serde::Serialize)]
struct PongNotice {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_messages_are_distinguished_by_the_type_field() {
        let lagged = serde_json::to_value(LaggedNotice { kind: "lagged", skipped: 12 })
            .expect("控制消息应可序列化");
        assert_eq!(lagged["type"], "lagged");
        assert_eq!(lagged["skipped"], 12);
        // 控制消息不应带 `event` 字段，否则无法与事件区分。
        assert!(lagged.get("event").is_none());

        let pong = serde_json::to_value(PongNotice { kind: "pong" }).expect("控制消息应可序列化");
        assert_eq!(pong["type"], "pong");
        assert!(pong.get("event").is_none());
    }

    #[test]
    fn heartbeat_interval_is_long_enough_to_avoid_chatter() {
        assert!(HEARTBEAT_INTERVAL >= Duration::from_secs(10));
    }
}
