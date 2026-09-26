//! 服务端事件推送（宿主端能力）。
//!
//! # 职责
//!
//! 把 application 层产生的事件（当前为服务器控制台日志）通过 WebSocket 实时
//! 推送给客户端，使前端不必轮询日志接口。
//!
//! # 分层
//!
//! - 协议无关：[`model`]（事件模型）、[`bus`]（广播总线）、[`bridge`]（事件源桥接）；
//! - 传输适配器：[`ws`]（axum WebSocket）。
//!
//! # 送达语义（重要）
//!
//! **事件是瞬时通知，不保证送达。** 广播通道有容量上限，消费慢的订阅者会被
//! 跳过若干条，此时由 [`ws`] 向客户端发送 `lagged` 提示，客户端自行补漏。
//!
//! 可补漏的前提是该事件有对应的持久化查询接口：控制台日志落在实例日志库中，
//! 客户端用 `GET /api/instances/{id}/logs?since=<sequence>` 补齐即可。未来若
//! 增加无持久化的事件（如进程状态变化），客户端应重新调用对应的状态接口
//! 重建视图，而不是等待补发。
//!
//! # 事件名与负载
//!
//! 事件名与负载形状**与 Tauri 宿主保持一致**（如 `server-log-line` 对应
//! `{ instance_id, line }`），使前端可以用同一套处理逻辑对接两种宿主。
//!
//! # 鉴权
//!
//! 当前**未接线鉴权**（后续接入 `HttpRpcAccessResolver` 与订阅权限）。

mod bridge;
mod bus;
mod model;
pub mod ws;

pub use bridge::spawn_log_bridge;
pub use bus::{DEFAULT_EVENT_CAPACITY, ServerEventBus};
pub use model::{EVENT_SERVER_LOG_LINE, ServerEvent, ServerEventPayload};
