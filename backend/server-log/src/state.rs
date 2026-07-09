use rusqlite::Connection;
use server_local_setup::decode_console_bytes as decode_shared_console_bytes;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread;

pub const LATEST_LOG_DB_FILE: &str = "latest_log.db";
pub const LOG_BATCH_SIZE: usize = 128;
pub const LOG_FLUSH_INTERVAL_MS: u64 = 50;

pub static LOG_WRITERS: OnceLock<Mutex<HashMap<String, ServerLogWriter>>> = OnceLock::new();

#[derive(Clone)]
pub struct LogWriteEntry {
    pub timestamp: i64,
    pub source: LogSource,
    pub message: String,
}

pub enum WriterCommand {
    Append(LogWriteEntry),
    Shutdown,
}

pub struct ServerLogWriter {
    pub sender: mpsc::Sender<WriterCommand>,
    pub worker: thread::JoinHandle<()>,
}

#[derive(Clone, Copy)]
pub enum LogSource {
    SeaLantern,
    Server,
}

impl LogSource {
    pub fn as_str(self) -> &'static str {
        match self {
            LogSource::SeaLantern => "sealantern",
            LogSource::Server => "server",
        }
    }
}

pub fn log_writers() -> &'static Mutex<HashMap<String, ServerLogWriter>> {
    LOG_WRITERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn open_or_create_log_db(server_path: &Path) -> Result<Connection, String> {
    let db_path = server_path.join(LATEST_LOG_DB_FILE);
    match crate::db::init_sqlite_log_db(&db_path) {
        Ok(conn) => Ok(conn),
        Err(err) if err.contains("file is not a database") => {
            let _ = std::fs::remove_file(&db_path);
            crate::db::init_sqlite_log_db(&db_path)
                .map_err(|e| format!("重建日志数据库失败 ({}): {}", db_path.display(), e))
        }
        Err(err) => Err(format!("打开日志数据库失败 ({}): {}", db_path.display(), err)),
    }
}

pub fn decode_console_bytes(bytes: &[u8]) -> String {
    decode_shared_console_bytes(bytes)
}
