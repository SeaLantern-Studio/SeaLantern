//! 服务器日志管线模块：统一负责日志读流、来源标注、SQLite 持久化、事件推送和历史读取。
//! ServerManager 只负责流程编排，日志实现细节都收敛在本文件。

use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, TransactionBehavior};

const LATEST_LOG_DB_FILE: &str = "latest_log.db";

pub type ServerLogEventHandler = Arc<dyn Fn(&str, &str) -> Result<(), String> + Send + Sync>;

static SERVER_LOG_EVENT_HANDLER: OnceLock<ServerLogEventHandler> = OnceLock::new();

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

pub fn append_sealantern_log(server_id: &str, message: &str) -> Result<(), String> {
    append_log_by_id(server_id, message, LogSource::SeaLantern)
}

pub fn append_server_log(server_id: &str, message: &str) -> Result<(), String> {
    append_log_by_id(server_id, message, LogSource::Server)
}

pub fn get_logs(server_id: &str, since: usize) -> Vec<String> {
    resolve_server_path(server_id)
        .ok()
        .and_then(|server_path| read_logs(&server_path, since as u64).ok())
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
        result.push((server_id.clone(), get_logs(&server_id, 0)));
    }
    result
}

pub fn append_log(
    server_id: &str,
    server_path: &Path,
    message: &str,
    source: LogSource,
    max_lines: i64,
) -> Result<(), String> {
    let mut conn = open_or_create_log_db(server_path)?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let tx = conn
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|e| format!("打开日志写事务失败: {}", e))?;

    tx.execute(
        "INSERT INTO log_lines (timestamp, source, line) VALUES (?1, ?2, ?3)",
        params![timestamp, source.as_str(), message],
    )
    .map_err(|e| format!("写入日志失败: {}", e))?;

    tx.execute(
        "DELETE FROM log_lines \
         WHERE rowid NOT IN (SELECT rowid FROM log_lines ORDER BY rowid DESC LIMIT ?1)",
        params![max_lines.max(1)],
    )
    .map_err(|e| format!("裁剪日志失败: {}", e))?;

    tx.commit()
        .map_err(|e| format!("提交日志写事务失败: {}", e))?;
    emit_server_log_line(server_id, message);
    Ok(())
}

fn append_log_by_id(server_id: &str, message: &str, source: LogSource) -> Result<(), String> {
    let server_path = resolve_server_path(server_id)?;
    let max_lines = super::global::settings_manager().get().max_log_lines.max(1) as i64;
    append_log(server_id, &server_path, message, source, max_lines)
}

pub fn read_logs(server_path: &Path, since: u64) -> Result<Vec<String>, String> {
    let conn = open_or_create_log_db(server_path)?;
    let mut stmt = conn
        .prepare("SELECT line FROM log_lines ORDER BY rowid ASC LIMIT -1 OFFSET ?1")
        .map_err(|e| format!("准备日志读取失败: {}", e))?;

    let rows = stmt
        .query_map(params![since as i64], |row| row.get::<_, String>(0))
        .map_err(|e| format!("读取日志失败: {}", e))?;

    let mut logs = Vec::new();
    for line in rows {
        logs.push(line.map_err(|e| format!("解析日志失败: {}", e))?);
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
        "CREATE TABLE IF NOT EXISTS log_lines (\
             id INTEGER PRIMARY KEY AUTOINCREMENT,\
             timestamp INTEGER NOT NULL,\
             source TEXT NOT NULL CHECK(source IN ('sealantern','server')),\
             line TEXT NOT NULL\
         );",
    )
    .map_err(|e| e.to_string())?;

    let has_timestamp = table_has_column(&conn, "log_lines", "timestamp")?;
    let has_source = table_has_column(&conn, "log_lines", "source")?;
    if !has_timestamp || !has_source {
        conn.execute_batch(
            "DROP TABLE IF EXISTS log_lines;\
             CREATE TABLE log_lines (\
               id INTEGER PRIMARY KEY AUTOINCREMENT,\
               timestamp INTEGER NOT NULL,\
               source TEXT NOT NULL CHECK(source IN ('sealantern','server')),\
               line TEXT NOT NULL\
             );",
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
