use chrono::{DateTime, Local};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

pub struct LogCollector {
    logs: Arc<Mutex<Vec<LogEntry>>>,
    max_logs: usize,
}

impl LogCollector {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::with_capacity(max_logs))),
            max_logs,
        }
    }

    pub fn add_log(&self, level: &str, message: &str) {
        let mut logs = self.logs.lock().unwrap_or_else(|e| e.into_inner());
        let timestamp: DateTime<Local> = Local::now();
        let entry = LogEntry {
            timestamp: timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            level: level.to_string(),
            message: message.to_string(),
        };
        logs.push(entry);

        if logs.len() > self.max_logs {
            logs.remove(0);
        }
    }

    pub fn get_logs(&self, limit: Option<usize>) -> Vec<LogEntry> {
        let logs = self.logs.lock().unwrap_or_else(|e| e.into_inner());
        let len = logs.len();
        let start = limit.map_or(0, |limit| len.saturating_sub(limit));
        logs[start..].to_vec()
    }

    pub fn clear(&self) {
        let mut logs = self.logs.lock().unwrap_or_else(|e| e.into_inner());
        logs.clear();
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_LOG_COLLECTOR: LogCollector = LogCollector::new(1000);
}

pub fn log_warn(message: &str) {
    println!("[WARN] {}", message);
    GLOBAL_LOG_COLLECTOR.add_log("WARN", message);
}

pub fn log_error(message: &str) {
    eprintln!("[ERROR] {}", message);
    GLOBAL_LOG_COLLECTOR.add_log("ERROR", message);
}
