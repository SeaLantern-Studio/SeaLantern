//! 服务器控制台命令响应捕获。
//!
//! 有些管理能力（在线玩家、白名单、封禁、OP 列表）更准确的做法不是去
//! 扒服务器本地文件，而是像主流启动器那样：直接给运行中的服务器发一条
//! 控制台命令（如 `list` / `whitelist list`），再捕获它在控制台上的回显。
//!
//! 本模块就是做"发命令 → 等响应行出现"这件事：先订阅全局日志广播事件，
//! 再写入命令，然后从事件流里挑出属于当前实例、且来源为服务器自身的日志行，
//! 直到一段时间没有新行（响应稳定）或超过超时上限。

use std::time::{Duration, Instant};

use sealantern_core::instance::InstanceId;
use sealantern_interface::ServerService;
use tracing::warn;

use crate::service::subscribe_log_events;
use crate::services::AppServices;

/// 捕获过程中的错误。
#[derive(Debug)]
pub enum CaptureError {
    /// 实例 ID 非法。
    InvalidInput,
    /// 服务器未在运行（无法写入 stdin）。
    ServerNotRunning,
    /// 服务装配层不可用（控制台 / 服务器服务拿不到）。
    Unavailable,
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::InvalidInput => write!(f, "invalid server id"),
            CaptureError::ServerNotRunning => write!(f, "server is not running"),
            CaptureError::Unavailable => write!(f, "services unavailable"),
        }
    }
}

/// 向运行中的服务器发送一条命令，返回其控制台回显的行（已去重）。
///
/// 实现要点：
/// 1. 先订阅日志广播，确保不会错过命令发出后的响应；
/// 2. 写入命令到子进程 stdin（需服务器在运行）；
/// 3. 从广播事件里筛选当前实例、来源为 `server` 的行；
/// 4. 当连续 `stable_wait` 没有新行（且至少收到过一行响应），或整体超过
///    `timeout` 时结束。
///
/// 注：超时是从发命令那一刻开始计的，包含 Java 端处理、回显落库、
/// writer 批量 flush（默认 100ms 窗口）再到 broadcast 的端到端延迟。
pub async fn capture_command_output(
    server_id: &str,
    command: &str,
    timeout: Duration,
) -> Result<Vec<String>, CaptureError> {
    let id = InstanceId::new(server_id).map_err(|_| CaptureError::InvalidInput)?;
    let server_svc = AppServices::server_service()
        .await
        .map_err(|_| CaptureError::Unavailable)?;

    // 先订阅，再发命令，保证响应行不被漏掉。
    let mut rx = subscribe_log_events();

    server_svc
        .send_command(&id, command)
        .await
        .map_err(|_| CaptureError::ServerNotRunning)?;

    let deadline = Instant::now() + timeout;
    // 响应通常很快回来；用 750ms 的"静默窗口"判定响应结束 —— 比 500ms
    // 稍宽裕，避开日志批 flush 抖动导致提前退出。
    let stable_wait = Duration::from_millis(750);
    let mut captured: Vec<String> = Vec::new();
    // 是否已收到首个服务器回显行。未收到前即使静默也不结束（持续等待至超时），
    // 避免命令回显链路（写 stdin -> Java 处理 -> 日志落库广播）的延迟导致
    // 在回显到达前就退出而漏掉整段响应。
    let mut got_first = false;

    loop {
        if Instant::now() >= deadline {
            break;
        }
        match tokio::time::timeout(stable_wait, rx.recv()).await {
            // 当前实例、且来源为服务器的日志行：剥掉 `[Server thread/INFO]:` 这类
            // 日志前缀后收集（解析端只看到真实回显内容，避免 `[Server thread/INFO]:`
            // 误被当成玩家名 / 封禁原因）。
            Ok(Ok(event)) if event.instance_id == server_id && event.line.source == "server" => {
                let line = strip_log_prefix(&event.line.line);
                if !line.is_empty() && !captured.iter().any(|x| x == line) {
                    captured.push(line.to_string());
                    got_first = true;
                }
            }
            // 其他实例的事件，或来源非服务器的事件，忽略但继续等待。
            Ok(Ok(_)) => {}
            // 广播通道关闭，收不到更多了。
            Ok(Err(_)) => break,
            // 静默窗口内没有新事件：若已收到过响应行则判定响应结束；
            // 若尚未收到任何响应行，则继续等待（直至 deadline）。
            Err(_) => {
                if got_first {
                    break;
                }
            }
        }
    }

    if !got_first {
        // 给开发 / 运维留一条排查线索：哪些命令在哪种环境下完全收不到
        // 回显（可能因为启用了 RCON、stdout 被重定向等情况）。
        warn!(
            target: "sealantern.application.command_capture",
            server_id,
            command,
            "命令已发出，但未在 {timeout:?} 内捕获到任何服务器回显行"
        );
    }

    Ok(captured)
}

/// 剥掉 Minecraft 服务器日志的前缀，例如 `[Server thread/INFO]: ` /
/// `[AsyncChatThread/INFO] [minecraft/MinecraftServer]: `，只保留真实内容。
///
/// 如果一行不以 `[` 开头则原样返回（不做假设）。
fn strip_log_prefix(line: &str) -> &str {
    let Some(rest) = line.strip_prefix('[') else {
        return line;
    };
    // 找到第一个 `]:` 作为前缀终结符。
    let Some(end) = rest.find("]:") else {
        return line;
    };
    rest[end + 2..].trim_start()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_log_prefix_basic_server_thread() {
        assert_eq!(
            strip_log_prefix("[Server thread/INFO]: There are 2 bans: alice, bob"),
            "There are 2 bans: alice, bob"
        );
    }

    #[test]
    fn strip_log_prefix_async_chat_thread_double_brackets() {
        assert_eq!(
            strip_log_prefix("[AsyncChatThread/INFO] [minecraft/MinecraftServer]: hjcboar joined"),
            "hjcboar joined"
        );
    }

    #[test]
    fn strip_log_prefix_no_prefix_unchanged() {
        assert_eq!(strip_log_prefix("There are no bans"), "There are no bans");
    }

    #[test]
    fn strip_log_prefix_unclosed_bracket_unchanged() {
        // 没有 `]:` 终结符的（例如不完整日志），保留原文。
        assert_eq!(
            strip_log_prefix("[malformed line without close bracket"),
            "[malformed line without close bracket"
        );
    }
}
