//! 服务器日志记录管线编排。
//!
//! 组合 `core` 的输出读取原语（[`read_output_lines`]）与 `extra` 的日志
//! 写入能力（[`LogWriter`]），把进程 stdout / stderr 的读取、解码、批量
//! 落库与实时事件推送串成一条管线。生命周期（启动 / 收敛）由
//! [`CoreServerService`](crate::service::CoreServerService) 驱动。
//!
//! 事件采用异步广播：日志行落库后（携带行号游标）通过
//! [`subscribe_log_events`] 广播，tauri 与 axum 等宿主各自订阅并转成
//! 自己的传输（前端事件 / SSE）。订阅方消费慢导致的事件丢失可由
//! `ConsoleService::logs(since)` 拉取补漏。

use std::path::Path;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use sealantern_core::process::read_output_lines;
use sealantern_core::process::TerminalOutput;
use sealantern_extra::server::log::{open_log_database, LogSource, LogWriter};
use sealantern_interface::console::ConsoleLogLine;

/// 广播通道容量；消费慢时丢弃旧事件，调用方可拉取补漏。
const LOG_EVENT_CHANNEL_CAPACITY: usize = 1024;

/// 实时日志事件（落库后产生，携带行号游标）。
#[derive(Debug, Clone)]
pub struct LogEvent {
    /// 所属实例 ID。
    pub instance_id: String,
    /// 日志行（含行号游标）。
    pub line: ConsoleLogLine,
}

static LOG_EVENT_BROADCAST: OnceLock<tokio::sync::broadcast::Sender<LogEvent>> = OnceLock::new();

/// 订阅全局服务器日志事件流。
///
/// 每次调用返回一个独立的接收端；多个宿主（tauri / axum）可各自订阅。
pub fn subscribe_log_events() -> tokio::sync::broadcast::Receiver<LogEvent> {
    LOG_EVENT_BROADCAST
        .get_or_init(|| {
            let (sender, _receiver) = tokio::sync::broadcast::channel(LOG_EVENT_CHANNEL_CAPACITY);
            sender
        })
        .subscribe()
}

fn publish_log_event(event: LogEvent) {
    if let Some(sender) = LOG_EVENT_BROADCAST.get() {
        let _ = sender.send(event);
    }
}

/// 服务器日志记录管线句柄。
///
/// 持有写入器与读取任务；`shutdown` 收敛写入器（flush 剩余批次），
/// 读取任务在进程退出（EOF）后自行结束。
pub struct LogRecorder {
    instance_id: String,
    writer: Option<LogWriter>,
    readers: Vec<tokio::task::JoinHandle<()>>,
}

impl LogRecorder {
    /// 启动管线：打开日志库、spawn 读取任务与写入器。
    ///
    /// 日志库打开失败时降级：读取任务仍会消费输出（防止管道阻塞进程），
    /// 但不落库不推送事件，并记录错误日志。
    pub async fn start(
        instance_id: impl Into<String>,
        directory: &Path,
        stdout: Option<TerminalOutput>,
        stderr: Option<TerminalOutput>,
    ) -> Self {
        let instance_id = instance_id.into();
        let database = match open_log_database(directory).await {
            Ok(database) => Some(database),
            Err(error) => {
                tracing::error!(
                    target: "sealantern.application.log_recorder",
                    instance_id,
                    directory = %directory.display(),
                    error = %error,
                    "failed to open server log database; recorder runs in discard mode"
                );
                None
            }
        };
        let writer = database.map(LogWriter::start);

        let mut readers = Vec::new();
        for output in [stdout, stderr].into_iter().flatten() {
            let instance_id = instance_id.clone();
            let writer = writer.clone();
            readers.push(tokio::task::spawn_blocking(move || {
                let _ = read_output_lines(output, |line| {
                    let line = line.to_string();
                    let Some(writer) = &writer else {
                        return;
                    };
                    let instance_id = instance_id.clone();
                    let timestamp = current_timestamp_secs();
                    writer.submit(
                        LogSource::Server,
                        line.clone(),
                        Some(Box::new(move |sequence| {
                            publish_log_event(LogEvent {
                                instance_id,
                                line: ConsoleLogLine {
                                    sequence,
                                    timestamp,
                                    source: "server".to_owned(),
                                    line,
                                },
                            });
                        })),
                    );
                });
            }));
        }

        // 启动说明日志（Sea Lantern 来源）：落库后同样广播实时事件，
        // 与进程输出行的"持久化 + 推送"行为保持一致。
        if let Some(writer) = &writer {
            let instance_id = instance_id.clone();
            let timestamp = current_timestamp_secs();
            let line = "服务器启动中...".to_owned();
            writer.submit(
                LogSource::SeaLantern,
                line.clone(),
                Some(Box::new(move |sequence| {
                    publish_log_event(LogEvent {
                        instance_id,
                        line: ConsoleLogLine {
                            sequence,
                            timestamp,
                            source: "sealantern".to_owned(),
                            line,
                        },
                    });
                })),
            );
        }

        Self { instance_id, writer, readers }
    }

    /// 追加一条 Sea Lantern 来源日志（如停止 / 错误说明），
    /// 落库后同步广播实时事件。
    pub fn append_system_log(&self, line: impl Into<String>) {
        let Some(writer) = &self.writer else {
            return;
        };
        let instance_id = self.instance_id.clone();
        let timestamp = current_timestamp_secs();
        let line = line.into();
        writer.submit(
            LogSource::SeaLantern,
            line.clone(),
            Some(Box::new(move |sequence| {
                publish_log_event(LogEvent {
                    instance_id,
                    line: ConsoleLogLine {
                        sequence,
                        timestamp,
                        source: "sealantern".to_owned(),
                        line,
                    },
                });
            })),
        );
    }

    /// 收敛管线：flush 剩余日志批次并结束写入任务。
    ///
    /// 读取任务依赖进程退出后的 EOF 自行结束，无需在此等待。
    pub async fn shutdown(mut self) {
        if let Some(writer) = self.writer.take() {
            writer.shutdown().await;
        }
        self.readers.clear();
    }
}

/// 当前 Unix 时间戳（秒）。
fn current_timestamp_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};

    use sealantern_core::process::{Daemon, Terminal, TerminalStream};
    use sealantern_extra::server::log::{open_log_database, read_logs};

    use super::*;

    /// 用真实子进程构造日志管线（输出几行后退出）。
    #[cfg(windows)]
    fn output_command() -> Command {
        let mut command = Command::new("cmd");
        command.args(["/C", "echo line one & echo line two & echo line three"]);
        command
    }

    #[cfg(unix)]
    fn output_command() -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", "echo line one; echo line two; echo line three"]);
        command
    }

    #[tokio::test]
    async fn recorder_persists_process_output_and_publishes_events() {
        let directory = tempfile::tempdir().expect("临时目录应创建成功");
        let mut command = output_command();
        command.stdout(Stdio::piped()).stderr(Stdio::null());
        let mut daemon = Daemon::spawn(&mut command).expect("子进程应启动成功");
        let mut terminal = Terminal::from_daemon_with_input(&mut daemon, false);
        let stdout = terminal.take_output(TerminalStream::Stdout);

        let mut receiver = subscribe_log_events();
        let recorder = LogRecorder::start("server-a", directory.path(), stdout, None).await;
        // 给读取任务调度窗口，确保进程输出被读取后再收敛。
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let _ = daemon.wait().expect("子进程应正常退出");
        recorder.shutdown().await;

        // 日志库应持久化进程输出（含启动说明日志）。
        let database = open_log_database(directory.path())
            .await
            .expect("日志库应存在");
        let lines = read_logs(&database, 0, None).await.expect("读取应成功");
        let texts: Vec<&str> = lines.iter().map(|line| line.line.as_str()).collect();
        assert!(texts.iter().any(|text| text.contains("服务器启动中")), "db: {texts:?}");
        assert!(texts.iter().any(|text| text.contains("line one")), "db: {texts:?}");

        // 事件流应包含启动说明日志与三行进程输出（阻塞等待，容忍写入窗口）。
        let mut event_texts = Vec::new();
        let mut saw_startup = false;
        for _ in 0..6 {
            match tokio::time::timeout(std::time::Duration::from_secs(2), receiver.recv()).await {
                Ok(Ok(event)) if event.instance_id == "server-a" => {
                    if event.line.source == "server" {
                        event_texts.push(event.line.line.clone());
                    } else if event.line.source == "sealantern" {
                        saw_startup = true;
                    }
                }
                Ok(Ok(_)) => continue,
                Ok(Err(_)) | Err(_) => break,
            }
        }
        assert!(saw_startup, "启动说明日志应广播为实时事件");
        assert!(
            event_texts.iter().any(|text| text.contains("line one")),
            "events: {event_texts:?}"
        );
        assert!(
            event_texts.iter().any(|text| text.contains("line two")),
            "events: {event_texts:?}"
        );
        assert!(
            event_texts.iter().any(|text| text.contains("line three")),
            "events: {event_texts:?}"
        );
    }

    #[test]
    fn broadcast_sender_is_available_without_recorder() {
        // 未启动任何 recorder 时订阅也不应 panic。
        let mut receiver = subscribe_log_events();
        let _ = receiver.try_recv();
    }
}
