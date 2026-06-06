use rusqlite::Connection;
use sea_lantern_server_local_setup_core::decode_console_bytes as decode_shared_console_bytes;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::thread;

pub type ServerLogEventHandler = dyn Fn(&str, &str) -> Result<(), String> + Send + Sync;
pub type ServerLogProcessor = dyn Fn(&str, &str) -> String + Send + Sync;
pub type ServerLogProcessorList = Arc<Mutex<Vec<Arc<ServerLogProcessor>>>>;

pub static SERVER_LOG_EVENT_HANDLER: OnceLock<Arc<ServerLogEventHandler>> = OnceLock::new();
pub static SERVER_LOG_PROCESSORS: OnceLock<ServerLogProcessorList> = OnceLock::new();
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

pub fn server_log_processors() -> &'static ServerLogProcessorList {
    SERVER_LOG_PROCESSORS.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

pub fn log_writers() -> &'static Mutex<HashMap<String, ServerLogWriter>> {
    LOG_WRITERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn open_or_create_log_db(server_path: &Path) -> Result<Connection, String> {
    let db_path = server_path.join(crate::utils::constants::LATEST_LOG_DB_FILE);
    match super::db::init_sqlite_log_db(&db_path) {
        Ok(conn) => Ok(conn),
        Err(err) if err.contains("file is not a database") => {
            let _ = std::fs::remove_file(&db_path);
            super::db::init_sqlite_log_db(&db_path)
                .map_err(|e| format!("重建日志数据库失败 ({}): {}", db_path.display(), e))
        }
        Err(err) => Err(format!("打开日志数据库失败 ({}): {}", db_path.display(), err)),
    }
}

pub fn resolve_server_path(server_id: &str) -> Result<PathBuf, String> {
    let manager = crate::services::global::server_manager();
    let servers = manager.get_server_list_checked()?;
    servers
        .iter()
        .find(|server| server.id == server_id)
        .map(|server| PathBuf::from(server.path.clone()))
        .ok_or_else(|| format!("未找到服务器: {}", server_id))
}

pub fn decode_console_bytes(bytes: &[u8]) -> String {
    decode_shared_console_bytes(bytes)
}
