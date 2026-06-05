use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::thread;

pub type ServerLogEventHandler = Arc<dyn Fn(&str, &str) -> Result<(), String> + Send + Sync>;
pub type ServerLogProcessor = Arc<dyn Fn(&str, &str) -> String + Send + Sync>;

pub static SERVER_LOG_EVENT_HANDLER: OnceLock<ServerLogEventHandler> = OnceLock::new();
pub static SERVER_LOG_PROCESSORS: OnceLock<Arc<Mutex<Vec<ServerLogProcessor>>>> = OnceLock::new();
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

pub fn server_log_processors() -> &'static Arc<Mutex<Vec<ServerLogProcessor>>> {
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
    let servers = manager.get_server_list();
    servers
        .iter()
        .find(|server| server.id == server_id)
        .map(|server| PathBuf::from(server.path.clone()))
        .ok_or_else(|| format!("未找到服务器: {}", server_id))
}

pub fn decode_console_bytes(bytes: &[u8]) -> String {
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
