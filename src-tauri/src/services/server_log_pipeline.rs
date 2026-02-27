//! 服务器日志管线模块：统一负责日志读流、来源标注、SQLite 持久化、事件推送和历史读取。
//! ServerManager 只负责流程编排，日志实现细节都收敛在本文件。
//!
//! ------------------------------- 设计说明（请先读） -------------------------------
//! 这个文件现在有两条明确分离的链路：
//!
//! 1) 写入链路（高频、强调吞吐）
//!    append_sealantern_log / append_server_log
//!    - append_log_by_id
//!    - append_log
//!    - 每个 server_id 对应一个常驻 Writer 线程（run_log_writer）
//!    - 按批次短事务写入 SQLite（flush_batch）
//!
//!    这样做的原因：
//!    - 旧实现是“每行日志都 open + pragma + 事务 + commit”，高并发输出下会放大 I/O 和锁竞争。
//!    - 新实现把固定成本挪到“创建 Writer 时只做一次”，随后用队列缓冲并小批量提交。
//!    - 事务仍然是短事务（每批提交），避免长时间持有写锁，兼顾吞吐和并发读取。
//!
//! 2) 读取链路（按需、强调稳定）
//!    get_logs / get_all_logs
//!    - read_logs
//!    - 独立连接读取 SQLite
//!
//!    写读解耦的意义：
//!    - 写入是否突发，不会直接阻塞“读取函数的调用结构”。
//!    - 插件和前端的读取入口语义保持不变，迁移风险低。
//!
//! 生命周期约束：
//! - Writer 在 stop/delete/异常回收等场景通过 shutdown_writer 收敛：
//!   发送 Shutdown 指令 -> flush 队列 -> join 线程 -> 释放数据库句柄。
//! - 这一步很关键，否则 Windows 上可能出现文件句柄占用，影响外部工具访问 DB。
//!
//! 可调参数：
//! - LOG_BATCH_SIZE: 每批最多写入条数（越大吞吐更高，单批延迟也会增大）
//! - LOG_FLUSH_INTERVAL_MS: 批处理等待窗口（越小实时性越好，事务次数更多）
//!
//! 维护者注意：
//! - 不要把 read_logs 改为复用 Writer 连接（读写耦合会放大故障面）。
//! - 不要在 Writer 线程里长时间持有事务做复杂逻辑（会伤并发）。
//! - 新增调用点时，优先走 append_* 封装，避免绕过队列直接写库。
//! -------------------------------------------------------------------------------

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, TransactionBehavior};

const LATEST_LOG_DB_FILE: &str = "latest_log.db";

pub type ServerLogEventHandler = Arc<dyn Fn(&str, &str) -> Result<(), String> + Send + Sync>;

static SERVER_LOG_EVENT_HANDLER: OnceLock<ServerLogEventHandler> = OnceLock::new();
static LOG_WRITERS: OnceLock<Mutex<HashMap<String, ServerLogWriter>>> = OnceLock::new();

const LOG_BATCH_SIZE: usize = 128;
const LOG_FLUSH_INTERVAL_MS: u64 = 50;

#[derive(Clone)]
struct LogWriteEntry {
    timestamp: i64,
    source: LogSource,
    message: String,
}

enum WriterCommand {
    Append(LogWriteEntry),
    Shutdown,
}

struct ServerLogWriter {
    sender: mpsc::Sender<WriterCommand>,
    worker: thread::JoinHandle<()>,
}

#[derive(Clone, Copy)]
pub enum LogSource {
    SeaLantern,
    Server,
}

impl LogSource {
    fn as_str(self) -> &'static str {
        match self {
            LogSource::SeaLantern => "sealantern",
            LogSource::Server => "server",
        }
    }
}

pub fn set_server_log_event_handler(handler: ServerLogEventHandler) -> Result<(), String> {
    SERVER_LOG_EVENT_HANDLER
        .set(handler)
        .map_err(|_| "server log event handler already set".to_string())
}

pub fn init_db(server_path: &Path) -> Result<(), String> {
    open_or_create_log_db(server_path).map(|_| ())
}

pub fn shutdown_writer(server_id: &str) {
    let writer = {
        let mut writers = log_writers().lock().expect("log writers lock poisoned");
        writers.remove(server_id)
    };

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

pub fn get_logs(server_id: &str, since: usize, recent_limit: Option<usize>) -> Vec<String> {
    resolve_server_path(server_id)
        .ok()
        .and_then(|server_path| read_logs(&server_path, since as u64, recent_limit).ok())
        .unwrap_or_default()
}

pub fn get_all_logs() -> Vec<(String, Vec<String>)> {
    let server_ids = super::global::server_manager()
        .servers
        .lock()
        .map(|servers| {
            servers
                .iter()
                .map(|server| server.id.clone())
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let mut result = Vec::with_capacity(server_ids.len());
    for server_id in server_ids {
        result.push((server_id.clone(), get_logs(&server_id, 0, None)));
    }
    result
}

fn log_writers() -> &'static Mutex<HashMap<String, ServerLogWriter>> {
    LOG_WRITERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_or_create_writer(
    server_id: &str,
    server_path: &Path,
) -> Result<mpsc::Sender<WriterCommand>, String> {
    {
        let writers = log_writers().lock().expect("log writers lock poisoned");
        if let Some(writer) = writers.get(server_id) {
            return Ok(writer.sender.clone());
        }
    }

    open_or_create_log_db(server_path)?;

    let (tx, rx) = mpsc::channel::<WriterCommand>();
    let path = server_path.to_path_buf();
    let sid = server_id.to_string();
    let worker = thread::spawn(move || run_log_writer(sid, path, rx));

    let mut writers = log_writers().lock().expect("log writers lock poisoned");
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

    // Writer 主循环：
    // - 至少取到一条日志后再进入“时间窗口聚合”，减少空转
    // - 到达批大小上限，或时间窗口耗尽，就立刻 flush
    // - 收到 Shutdown/断连时确保尽力刷盘后退出
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

    // 保持短事务：只覆盖本批次写入，避免长事务造成锁竞争放大。
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
        .unwrap_or_default()
        .as_millis() as i64;
    let entry = LogWriteEntry {
        timestamp,
        source,
        message: message.to_string(),
    };

    // append_log 是高频入口：只负责“入队 + 事件推送”，
    // 不在调用线程执行 SQLite 写事务，避免阻塞 stdout/stderr 消费链路。
    let sender = get_or_create_writer(server_id, server_path)?;
    if sender.send(WriterCommand::Append(entry.clone())).is_err() {
        shutdown_writer(server_id);
        let retry_sender = get_or_create_writer(server_id, server_path)?;
        retry_sender
            .send(WriterCommand::Append(entry))
            .map_err(|e| format!("提交日志写入队列失败: {}", e))?;
    }
    emit_server_log_line(server_id, message);
    Ok(())
}

fn append_log_by_id(server_id: &str, message: &str, source: LogSource) -> Result<(), String> {
    let server_path = resolve_server_path(server_id)?;
    append_log(server_id, &server_path, message, source)
}

pub fn read_logs(
    server_path: &Path,
    since: u64,
    recent_limit: Option<usize>,
) -> Result<Vec<String>, String> {
    let conn = open_or_create_log_db(server_path)?;
    let mut logs = Vec::new();

    if let Some(limit) = recent_limit.filter(|v| *v > 0) {
        let mut stmt = conn
            .prepare(
                r#"SELECT line FROM (
                       SELECT rowid, line FROM log_lines ORDER BY rowid DESC LIMIT ?1
                   ) recent
                   ORDER BY rowid ASC LIMIT -1 OFFSET ?2"#,
            )
            .map_err(|e| format!("准备日志读取失败: {}", e))?;
        let rows = stmt
            .query_map(params![limit as i64, since as i64], |row| row.get::<_, String>(0))
            .map_err(|e| format!("读取日志失败: {}", e))?;
        for line in rows {
            logs.push(line.map_err(|e| format!("解析日志失败: {}", e))?);
        }
    } else {
        let mut stmt = conn
            .prepare("SELECT line FROM log_lines ORDER BY rowid ASC LIMIT -1 OFFSET ?1")
            .map_err(|e| format!("准备日志读取失败: {}", e))?;
        let rows = stmt
            .query_map(params![since as i64], |row| row.get::<_, String>(0))
            .map_err(|e| format!("读取日志失败: {}", e))?;
        for line in rows {
            logs.push(line.map_err(|e| format!("解析日志失败: {}", e))?);
        }
    }
    Ok(logs)
}

pub fn spawn_server_output_reader<R>(server_id: String, reader: R)
where
    R: Read + Send + 'static,
{
    std::thread::spawn(move || {
        let mut buf_reader = BufReader::new(reader);
        let mut buffer = Vec::new();

        loop {
            buffer.clear();
            match buf_reader.read_until(b'\n', &mut buffer) {
                Ok(0) => break,
                Ok(_) => {
                    let mut line = decode_console_bytes(&buffer);
                    line = line.trim_end_matches(['\r', '\n']).to_string();
                    if line.trim().is_empty() {
                        continue;
                    }

                    let _ = append_server_log(&server_id, &line);

                    if line.contains("Done (") && line.contains(")! For help") {
                        super::global::server_manager().clear_starting(&server_id);
                        let _ = crate::plugins::api::emit_server_ready(&server_id);
                    }
                }
                Err(_) => break,
            }
        }
    });
}

fn emit_server_log_line(server_id: &str, line: &str) {
    if let Some(handler) = SERVER_LOG_EVENT_HANDLER.get() {
        let _ = handler(server_id, line);
    }
}

fn open_or_create_log_db(server_path: &Path) -> Result<Connection, String> {
    let db_path = server_path.join(LATEST_LOG_DB_FILE);
    match init_sqlite_log_db(&db_path) {
        Ok(conn) => Ok(conn),
        Err(err) if err.contains("file is not a database") => {
            let _ = std::fs::remove_file(&db_path);
            init_sqlite_log_db(&db_path)
                .map_err(|e| format!("重建日志数据库失败 ({}): {}", db_path.display(), e))
        }
        Err(err) => Err(format!("打开日志数据库失败 ({}): {}", db_path.display(), err)),
    }
}

fn init_sqlite_log_db(db_path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.busy_timeout(Duration::from_millis(2000))
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "synchronous", "NORMAL")
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "locking_mode", "NORMAL")
        .map_err(|e| e.to_string())?;
    conn.pragma_update(None, "wal_autocheckpoint", 1000)
        .map_err(|e| e.to_string())?;

    conn.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS log_lines (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             timestamp INTEGER NOT NULL,
             source TEXT NOT NULL CHECK(source IN ('sealantern','server')),
             line TEXT NOT NULL
         );"#,
    )
    .map_err(|e| e.to_string())?;

    let has_timestamp = table_has_column(&conn, "log_lines", "timestamp")?;
    let has_source = table_has_column(&conn, "log_lines", "source")?;
    if !has_timestamp || !has_source {
        conn.execute_batch(
            r#"DROP TABLE IF EXISTS log_lines;
             CREATE TABLE log_lines (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               timestamp INTEGER NOT NULL,
               source TEXT NOT NULL CHECK(source IN ('sealantern','server')),
               line TEXT NOT NULL
             );"#,
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(conn)
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool, String> {
    let sql = format!("PRAGMA table_info({})", table);
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let name: String = row.get(1).map_err(|e| e.to_string())?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

fn resolve_server_path(server_id: &str) -> Result<PathBuf, String> {
    let manager = super::global::server_manager();
    let servers = manager
        .servers
        .lock()
        .map_err(|_| "servers lock poisoned".to_string())?;
    servers
        .iter()
        .find(|server| server.id == server_id)
        .map(|server| PathBuf::from(server.path.clone()))
        .ok_or_else(|| format!("未找到服务器: {}", server_id))
}

fn decode_console_bytes(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }

    #[cfg(target_os = "windows")]
    {
        let (decoded, _, _) = encoding_rs::GBK.decode(bytes);
        decoded.into_owned()
    }
    #[cfg(not(target_os = "windows"))]
    {
        String::from_utf8_lossy(bytes).into_owned()
    }
}
