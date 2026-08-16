//! 服务器日志批量写入器。
//!
//! 将提交的日志行按批次写入日志库，降低高频输出场景下的事务与 I/O
//! 开销：固定成本（打开连接、配置）只承担一次，批次采用短事务提交，
//! 兼顾吞吐与实时性。写入器持有独立的日志库连接，与读取侧（按需打开）
//! 读写分离。
//!
//! 每条日志行在落库后通过可选回调上报其行号（AUTOINCREMENT id），
//! 供上层构造带游标的实时事件；回调在写入任务的同步上下文中执行，
//! 应只做轻量转发（如发送到无界通道 / 广播）。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use sealantern_infra::persistence::SqliteDatabase;

use super::LogSource;

/// 每批最多写入的日志行数。
const LOG_BATCH_SIZE: usize = 128;
/// 批量提交的等待窗口；批次未满时最多等待此时间后强制提交。
const LOG_FLUSH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

/// 落库回调：接收写入后的行号（AUTOINCREMENT id）。
pub type LogWrittenCallback = Box<dyn FnOnce(i64) + Send>;

/// 写入器接收的命令。
enum WriteCommand {
    /// 追加一条日志行；落库后调用可选回调并携带行号。
    Line {
        source: LogSource,
        line: String,
        on_written: Option<LogWrittenCallback>,
    },
    /// 收敛写入器：flush 剩余批次并退出。
    Shutdown,
}

/// 服务器日志批量写入器句柄。
///
/// 提交为同步调用（无界通道，不会阻塞调用方）；收敛为异步等待，
/// 保证退出前剩余批次全部落库。句柄可克隆共享给多个读取任务，
/// 收敛操作消耗句柄所有权，且只发出一次停止指令（多克隆防御）。
#[derive(Clone)]
pub struct LogWriter {
    sender: tokio::sync::mpsc::UnboundedSender<WriteCommand>,
    handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    shutdown_requested: Arc<AtomicBool>,
}

impl LogWriter {
    /// 启动写入器：在异步任务中持有日志库连接并批量写库。
    pub fn start(database: SqliteDatabase) -> Self {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let handle = tokio::spawn(async move {
            let mut batch = Vec::new();
            loop {
                tokio::select! {
                    command = receiver.recv() => {
                        match command {
                            Some(WriteCommand::Line { source, line, on_written }) => {
                                batch.push((source, line, on_written));
                                if batch.len() >= LOG_BATCH_SIZE {
                                    flush_batch(&database, &mut batch).await;
                                }
                            }
                            Some(WriteCommand::Shutdown) | None => break,
                        }
                    }
                    _ = tokio::time::sleep(LOG_FLUSH_INTERVAL), if !batch.is_empty() => {
                        flush_batch(&database, &mut batch).await;
                    }
                }
            }
            // 收尾：flush 剩余批次。
            flush_batch(&database, &mut batch).await;
        });
        Self {
            sender,
            handle: Arc::new(Mutex::new(Some(handle))),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 提交一条日志行（无界通道，立即返回）。
    ///
    /// `on_written` 在行落库后调用并携带行号；仅当需要行号（如构造
    /// 带游标的实时事件）时提供。
    pub fn submit(
        &self,
        source: LogSource,
        line: impl Into<String>,
        on_written: Option<LogWrittenCallback>,
    ) {
        let _ = self
            .sender
            .send(WriteCommand::Line { source, line: line.into(), on_written });
    }

    /// 收敛写入器：发送停止指令并等待剩余批次落库。
    ///
    /// 仅第一个调用者会发出停止指令并等待任务结束；其余克隆的收敛
    /// 调用直接返回，避免向已关闭的通道重复发送。
    pub async fn shutdown(self) {
        if self
            .shutdown_requested
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        let _ = self.sender.send(WriteCommand::Shutdown);
        let handle = self.handle.lock().expect("写入器句柄锁不应污染").take();
        if let Some(handle) = handle {
            let _ = handle.await;
        }
    }
}

/// 以单个短事务批量写入日志行，逐行上报行号。
async fn flush_batch(
    database: &SqliteDatabase,
    batch: &mut Vec<(LogSource, String, Option<LogWrittenCallback>)>,
) {
    if batch.is_empty() {
        return;
    }
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let lines = std::mem::take(batch);
    let _ = database
        .write("flush server log batch", move |transaction| {
            for (source, line, on_written) in lines {
                transaction.execute(
                    "INSERT INTO log_lines (timestamp, source, line) VALUES (?1, ?2, ?3)",
                    rusqlite::params![timestamp, source.as_str(), line],
                )?;
                let id = transaction.last_insert_rowid();
                if let Some(on_written) = on_written {
                    on_written(id);
                }
            }
            Ok(())
        })
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::log::store::open_log_database;

    #[tokio::test]
    async fn writer_batches_lines_and_shutdown_flushes_remaining() {
        let directory = tempfile::tempdir().expect("临时目录应创建成功");
        let database = open_log_database(directory.path())
            .await
            .expect("日志库应初始化成功");
        let writer = LogWriter::start(database.clone());

        // 不足一批的行在 shutdown 时 flush。
        writer.submit(LogSource::Server, "line one", None);
        writer.submit(LogSource::SeaLantern, "line two", None);
        writer.shutdown().await;

        let lines = crate::server::log::store::read_logs(&database, 0, None)
            .await
            .expect("读取应成功");
        let texts: Vec<&str> = lines.iter().map(|line| line.line.as_str()).collect();
        assert_eq!(texts, ["line one", "line two"]);
        assert_eq!(lines[0].source, "server");
        assert_eq!(lines[1].source, "sealantern");
    }

    #[tokio::test]
    async fn writer_reports_inserted_row_ids() {
        let directory = tempfile::tempdir().expect("临时目录应创建成功");
        let database = open_log_database(directory.path())
            .await
            .expect("日志库应初始化成功");
        let writer = LogWriter::start(database.clone());

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        writer.submit(
            LogSource::Server,
            "first",
            Some(Box::new(move |id| {
                let _ = tx.send(id);
            })),
        );
        writer.shutdown().await;

        let reported = rx.recv().await.expect("应上报行号");
        let lines = crate::server::log::store::read_logs(&database, 0, None)
            .await
            .expect("读取应成功");
        assert_eq!(lines[0].id, reported);
        assert!(reported > 0);
    }

    #[tokio::test]
    async fn writer_flushes_full_batches_in_time() {
        let directory = tempfile::tempdir().expect("临时目录应创建成功");
        let database = open_log_database(directory.path())
            .await
            .expect("日志库应初始化成功");
        let writer = LogWriter::start(database.clone());

        // 超过一批的行在等待窗口内提交，无需显式 shutdown。
        for index in 0..(LOG_BATCH_SIZE * 2) {
            writer.submit(LogSource::Server, format!("line {index}"), None);
        }
        tokio::time::sleep(LOG_FLUSH_INTERVAL * 2).await;

        let lines = crate::server::log::store::read_logs(&database, 0, None)
            .await
            .expect("读取应成功");
        assert_eq!(lines.len(), LOG_BATCH_SIZE * 2);
        writer.shutdown().await;
    }
}
