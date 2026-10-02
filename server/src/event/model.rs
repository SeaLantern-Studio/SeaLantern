//! 服务端事件模型（协议无关）。
//!
//! 事件名与负载形状与 Tauri 宿主保持一致，前端据此用同一套逻辑对接两种宿主。

use sealantern_contract::console::ConsoleLogLine;
use serde::Serialize;

/// 控制台日志行的事件名。
///
/// 与 Tauri 宿主 `emit("server-log-line", …)` 使用同一名称，前端凭事件名分发。
pub const EVENT_SERVER_LOG_LINE: &str = "server-log-line";

/// 推送给客户端的事件。
///
/// 序列化为**扁平 JSON**：事件名在顶层，负载字段与 Tauri 宿主同名事件一致，
/// 便于前端写一份解析：
///
/// ```json
/// { "event": "server-log-line", "instance_id": "…", "line": { "sequence": 1201 } }
/// ```
///
/// 控制消息（`lagged` / `pong` 等）没有 `event` 字段、带 `type` 字段，
/// 客户端据此区分事件与控制消息。
#[derive(Debug, Clone, Serialize)]
pub struct ServerEvent {
    /// 事件名（与 Tauri 宿主一致，供前端按名分发）。
    pub event: &'static str,
    /// 所属实例 ID。
    pub instance_id: String,
    /// 负载字段（平铺进顶层，字段随事件类型而异）。
    #[serde(flatten)]
    pub payload: ServerEventPayload,
}

/// 事件负载：一个变体对应一类事件。
///
/// 新增事件类型时在此添加变体并补 [`ServerEvent`] 的构造方法，传输层无需改动。
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ServerEventPayload {
    /// 控制台日志行。
    Log {
        /// 日志行（含 `sequence` 游标，用于与 REST 补漏衔接）。
        line: ConsoleLogLine,
    },
}

impl ServerEvent {
    /// 构造一条控制台日志事件。
    pub fn log(instance_id: impl Into<String>, line: ConsoleLogLine) -> Self {
        Self {
            event: EVENT_SERVER_LOG_LINE,
            instance_id: instance_id.into(),
            payload: ServerEventPayload::Log { line },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_line(sequence: i64) -> ConsoleLogLine {
        ConsoleLogLine {
            sequence,
            timestamp: 1_789_788_161,
            source: "server".to_owned(),
            line: "Done (1.557s)!".to_owned(),
        }
    }

    #[test]
    fn log_event_is_flat_and_carries_the_tauri_shape() {
        let event = ServerEvent::log("instance-a", sample_line(7));
        let value = serde_json::to_value(&event).expect("事件应可序列化");

        // 事件名在顶层，负载字段与其平铺（与 Tauri 的 `{ instance_id, line }` 对齐）。
        assert_eq!(value["event"], EVENT_SERVER_LOG_LINE);
        assert_eq!(value["instance_id"], "instance-a");
        assert_eq!(value["line"]["sequence"], 7);
        assert_eq!(value["line"]["source"], "server");
        assert_eq!(value["line"]["line"], "Done (1.557s)!");

        // 负载不应被包在中间层里。
        assert!(value.get("payload").is_none());
    }
}
