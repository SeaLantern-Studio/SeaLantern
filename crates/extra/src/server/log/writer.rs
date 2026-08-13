//! 服务器日志批量写入器。
//!
//! 将提交的日志行按批次写入日志库，降低高频输出场景下的事务与 I/O
//! 开销：固定成本（打开连接、配置）只承担一次，批次采用短事务提交，
//! 兼顾吞吐与实时性。写入器持有独立的日志库连接，与读取侧（按需打开）
//! 读写分离。

use std::time::{SystemTime, UNIX_EPOCH};

use sealantern_infra::persistence::{SqliteDatabase, SqlValue};

use super::LogSource;

/// 每批最多写入的日志行数。
const LOG_BATCH_SIZE: usize = 128;
/// 批量提交的等待窗口；批次未满时最多等待此时间后强制提交。
const LOG_FLUSH_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

/// 写入器接收的命令。
enum WriteCommand {
    /// 追加一条日志行。
    Line { source: LogSource, line: String },
    /// 收敛写入器：flush 剩余批次并退出。
    Shutdown,
}

/// 服务器日志批量写入器句柄。
///
/// 提交为同步调用（无界通道，不会阻塞调用方）；收敛为异步等待，
/// 保证退出前剩余批次全部落库。
pub struct LogWriter {
    sender: tokio::sync::mpsc::UnboundedSender<WriteCommand>,
    handle: Option<tokio::task::JoinHandle<()>>,
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
                            Some(WriteCommand::Line { source, line }) => {
                                batch.push((source, line));
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
        Self { sender, handle: Some(handle) }
    }

    /// 提交一条日志行（无界通道，立即返回）。
    pub fn submit(&self, source: LogSource, line: impl Into<String>) {
        let _ = self
            .sender
            .send(WriteCommand::Line { source, line: line.into() });
    }

    /// 收敛写入器：发送停止指令并等待剩余批次落库。
    pub async fn shutdown(mut self) {
        let _ = self.sender.send(WriteCommand::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
    }
}

/// 以单个短事务批量写入日志行。
async fn flush_batch(database: &SqliteDatabase, batch: &mut Vec<(LogSource, String)>) {
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
            for (source, line) in &lines {
                transaction.execute(
                    "INSERT INTO log_lines (timestamp, source, line) VALUES (?1, ?2, ?3)",
                    rusqlite::params![timestamp, source.as_str(), line],
                )?;
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
        writer.submit(LogSource::Server, "line one");
        writer.submit(LogSource::SeaLantern, "line two");
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
    async fn writer_flushes_full_batches_in_time() {
        let directory = tempfile::tempdir().expect("临时目录应创建成功");
        let database = open_log_database(directory.path())
            .await
            .expect("日志库应初始化成功");
        let writer = LogWriter::start(database.clone());

        // 超过一批的行在等待窗口内提交，无需显式 shutdown。
        for index in 0..(LOG_BATCH_SIZE * 2) {
            writer.submit(LogSource::Server, format!("line {index}"));
        }
        tokio::time::sleep(LOG_FLUSH_INTERVAL * 2).await;

        let lines = crate::server::log::store::read_logs(&database, 0, None)
            .await
            .expect("读取应成功");
        assert_eq!(lines.len(), LOG_BATCH_SIZE * 2);
        writer.shutdown().await;
    }
}
