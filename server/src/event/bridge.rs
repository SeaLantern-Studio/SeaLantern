//! 从 application 事件源桥接到 server 事件总线。
//!
//! application 层的日志记录管线在每行落库后广播 [`LogEvent`]；本模块订阅该
//! 广播并转成 [`ServerEvent`] 发布到总线，使 server 的传输层（WebSocket）
//! 无需了解日志管线的存在。
//!
//! [`LogEvent`]: sealantern_application::service::LogEvent

use std::sync::Arc;

use sealantern_application::service::subscribe_log_events;
use tokio::sync::broadcast::error::RecvError;
use tokio::task::JoinHandle;

use super::{ServerEvent, ServerEventBus};

/// 启动日志事件桥接：订阅 application 的日志广播并转发到 server 事件总线。
///
/// 返回任务句柄；广播通道关闭时任务自行结束（进程退出即结束，无需显式停止）。
///
/// 应在进程启动时**调用一次**：重复调用会创建多条桥接，同一日志被重复发布。
pub fn spawn_log_bridge(bus: Arc<ServerEventBus>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut receiver = subscribe_log_events();
        loop {
            match receiver.recv().await {
                Ok(log) => {
                    bus.publish(ServerEvent::log(log.instance_id, log.line));
                }
                // 静态广播不会关闭；防御性退出。
                Err(RecvError::Closed) => break,
                // 桥接自身落后：下游订阅者各自用 REST 补漏，此处仅留痕。
                Err(RecvError::Lagged(skipped)) => {
                    tracing::warn!(
                        target: "sealantern.server.event",
                        skipped,
                        "log bridge lagged; clients should backfill via the logs API"
                    );
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};
    use std::time::Duration;

    use sealantern_application::service::LogRecorder;
    use sealantern_core::process::{Daemon, Terminal, TerminalStream};

    use super::*;
    use crate::event::ServerEventPayload;

    /// 构造一个输出两行文本后自行退出的子进程命令。
    #[cfg(windows)]
    fn output_command() -> Command {
        let mut command = Command::new("cmd");
        command.args(["/C", "echo bridge line one & echo bridge line two"]);
        command
    }

    #[cfg(unix)]
    fn output_command() -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", "echo bridge line one; echo bridge line two"]);
        command
    }

    #[tokio::test]
    async fn bridge_subscribes_without_publishing_spurious_events() {
        let bus = Arc::new(ServerEventBus::new(8));
        let handle = spawn_log_bridge(Arc::clone(&bus));

        // 桥接会持有 bus 的一个订阅端（自身），但不产生事件。
        let mut receiver = bus.subscribe();
        assert!(receiver.try_recv().is_err(), "无日志产生时不应有事件");

        handle.abort();
    }

    #[tokio::test]
    async fn forwards_application_log_events_to_the_bus() {
        let directory = tempfile::tempdir().expect("临时目录应创建成功");
        let bus = Arc::new(ServerEventBus::new(64));
        let mut receiver = bus.subscribe();
        let handle = spawn_log_bridge(Arc::clone(&bus));

        // 用真实子进程驱动日志管线：进程输出应经桥接出现在事件总线上。
        let mut command = output_command();
        command.stdout(Stdio::piped()).stderr(Stdio::null());
        let mut daemon = Daemon::spawn(&mut command).expect("子进程应启动成功");
        let mut terminal = Terminal::from_daemon_with_input(&mut daemon, false);
        let stdout = terminal.take_output(TerminalStream::Stdout);
        let recorder =
            LogRecorder::start("instance-bridge", directory.path(), stdout, None, true).await;

        let mut saw_line = false;
        for _ in 0..12 {
            match tokio::time::timeout(Duration::from_secs(2), receiver.recv()).await {
                Ok(Ok(event)) => {
                    // 同一广播上可能有其他测试的事件，只认本实例的进程输出行。
                    if event.instance_id == "instance-bridge"
                        && let ServerEventPayload::Log { line } = &event.payload
                        && line.line.contains("bridge line one")
                    {
                        saw_line = true;
                        break;
                    }
                }
                Ok(Err(_)) | Err(_) => break,
            }
        }

        let _ = daemon.wait().expect("子进程应正常退出");
        recorder.shutdown().await;
        handle.abort();

        assert!(saw_line, "进程输出应经桥接出现在事件总线上");
    }
}
