use super::state::{
    log_writers, open_or_create_log_db, resolve_server_path, LogSource, LogWriteEntry,
    ServerLogEventHandler, ServerLogWriter, WriterCommand, SERVER_LOG_EVENT_HANDLER,
};
use rusqlite::{params, Connection, TransactionBehavior};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::utils::constants::{LOG_BATCH_SIZE, LOG_FLUSH_INTERVAL_MS};

pub fn set_server_log_event_handler(handler: ServerLogEventHandler) -> Result<(), String> {
    SERVER_LOG_EVENT_HANDLER
        .set(handler)
        .map_err(|_| "server log event handler already set".to_string())
}

pub fn init_db(server_path: &Path) -> Result<(), String> {
    open_or_create_log_db(server_path).map(|_| ())
}

pub fn shutdown_writer(server_id: &str) {
    let writer = log_writers()
        .lock()
        .ok()
        .and_then(|mut writers| writers.remove(server_id));

    if let Some(writer) = writer {
        let _ = writer.sender.send(WriterCommand::Shutdown);
        let _ = writer.worker.join();
    }
}

pub fn append_sealantern_log(server_id: &str, message: &str) -> Result<(), String> {
    append_log_by_id(server_id, message, LogSource::SeaLantern)
}

pub fn append_server_log(server_id: &str, message: &str) -> Result<(), String> {
    append_log_by_id(server_id, message, LogSource::Server)
}

fn get_or_create_writer(
    server_id: &str,
    server_path: &Path,
) -> Result<mpsc::Sender<WriterCommand>, String> {
    {
        let writers = log_writers()
            .lock()
            .map_err(|_| "log writers lock poisoned".to_string())?;
        if let Some(writer) = writers.get(server_id) {
            return Ok(writer.sender.clone());
        }
    }

    open_or_create_log_db(server_path)?;

    let (tx, rx) = mpsc::channel::<WriterCommand>();
    let path = server_path.to_path_buf();
    let sid = server_id.to_string();
    let worker = thread::spawn(move || run_log_writer(sid, path, rx));

    let mut writers = log_writers()
        .lock()
        .map_err(|_| "log writers lock poisoned".to_string())?;
    if let Some(existing) = writers.get(server_id) {
        let _ = tx.send(WriterCommand::Shutdown);
        let _ = worker.join();
        return Ok(existing.sender.clone());
    }

    writers.insert(server_id.to_string(), ServerLogWriter { sender: tx.clone(), worker });

    Ok(tx)
}

fn run_log_writer(server_id: String, server_path: PathBuf, rx: mpsc::Receiver<WriterCommand>) {
    let mut conn = match open_or_create_log_db(&server_path) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!(
                "[server_log_pipeline] failed to open writer db id={} path={} err={}",
                server_id,
                server_path.display(),
                err
            );
            return;
        }
    };

    let flush_interval = Duration::from_millis(LOG_FLUSH_INTERVAL_MS);
    let mut batch = Vec::<LogWriteEntry>::with_capacity(LOG_BATCH_SIZE);

    loop {
        let first = match rx.recv() {
            Ok(cmd) => cmd,
            Err(_) => {
                if !batch.is_empty() {
                    let _ = flush_batch(&mut conn, &batch);
                }
                break;
            }
        };

        match first {
            WriterCommand::Append(entry) => {
                batch.push(entry);
                let deadline = Instant::now() + flush_interval;
                while batch.len() < LOG_BATCH_SIZE {
                    let remain = deadline.saturating_duration_since(Instant::now());
                    if remain.is_zero() {
                        break;
                    }
                    match rx.recv_timeout(remain) {
                        Ok(WriterCommand::Append(entry)) => batch.push(entry),
                        Ok(WriterCommand::Shutdown) => {
                            let _ = flush_batch(&mut conn, &batch);
                            return;
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => break,
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            let _ = flush_batch(&mut conn, &batch);
                            return;
                        }
                    }
                }

                if let Err(err) = flush_batch(&mut conn, &batch) {
                    eprintln!(
                        "[server_log_pipeline] flush batch failed id={} path={} err={}",
                        server_id,
                        server_path.display(),
                        err
                    );
                }
                batch.clear();
            }
            WriterCommand::Shutdown => {
                if !batch.is_empty() {
                    let _ = flush_batch(&mut conn, &batch);
                }
                break;
            }
        }
    }
}

fn flush_batch(conn: &mut Connection, batch: &[LogWriteEntry]) -> Result<(), String> {
    if batch.is_empty() {
        return Ok(());
    }

    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| format!("打开日志写事务失败: {}", e))?;

    {
        let mut stmt = tx
            .prepare("INSERT INTO log_lines (timestamp, source, line) VALUES (?1, ?2, ?3)")
            .map_err(|e| format!("准备日志写入失败: {}", e))?;
        for entry in batch {
            stmt.execute(params![entry.timestamp, entry.source.as_str(), entry.message])
                .map_err(|e| format!("写入日志失败: {}", e))?;
        }
    }

    tx.commit()
        .map_err(|e| format!("提交日志写事务失败: {}", e))
}

pub fn append_log(
    server_id: &str,
    server_path: &Path,
    message: &str,
    source: LogSource,
) -> Result<(), String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("获取日志时间戳失败: {}", e))?
        .as_millis() as i64;
    let entry = LogWriteEntry {
        timestamp,
        source,
        message: message.to_string(),
    };

    let sender = get_or_create_writer(server_id, server_path)?;
    if sender.send(WriterCommand::Append(entry.clone())).is_err() {
        shutdown_writer(server_id);
        let retry_sender = get_or_create_writer(server_id, server_path)?;
        retry_sender
            .send(WriterCommand::Append(entry))
            .map_err(|e| format!("提交日志写入队列失败: {}", e))?;
    }
    super::output::emit_server_log_line(server_id, message);
    Ok(())
}

fn append_log_by_id(server_id: &str, message: &str, source: LogSource) -> Result<(), String> {
    let server_path = resolve_server_path(server_id)?;
    append_log(server_id, &server_path, message, source)
}
