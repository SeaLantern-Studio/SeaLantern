use chrono::{DateTime, Local};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

fn parse_level<'a>(message: &'a str, default_level: &'a str) -> &'a str {
    if message.contains("[FATAL]") {
        return "FATAL";
    }
    if message.contains("[ERROR]") {
        return "ERROR";
    }
    if message.contains("[WARN]") || message.contains("WARNING") {
        return "WARN";
    }
    if message.contains("[DEBUG]") {
        return "DEBUG";
    }
    if message.contains("[INFO]") {
        return "INFO";
    }
    default_level
}

pub fn format_log_entry(entry: &LogEntry) -> String {
    let has_level_prefix = ["[DEBUG]", "[INFO]", "[WARN]", "[ERROR]", "[FATAL]"]
        .iter()
        .any(|prefix| entry.message.starts_with(prefix));

    if has_level_prefix {
        format!("[{}] {}", entry.timestamp, entry.message)
    } else {
        format!("[{}] [{}] {}", entry.timestamp, entry.level, entry.message)
    }
}

pub fn capture_stdout(message: &str) {
    GLOBAL_LOG_COLLECTOR.add_log(parse_level(message, "INFO"), message);
}

pub fn capture_stderr(message: &str) {
    GLOBAL_LOG_COLLECTOR.add_log(parse_level(message, "ERROR"), message);
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_LOG_COLLECTOR: LogCollector = LogCollector::new(1000);
}

pub fn log_warn(message: &str) {
    std::println!("[WARN] {}", message);
    GLOBAL_LOG_COLLECTOR.add_log("WARN", &format!("[WARN] {}", message));
}

pub fn log_error(message: &str) {
    std::eprintln!("[ERROR] {}", message);
    GLOBAL_LOG_COLLECTOR.add_log("ERROR", &format!("[ERROR] {}", message));
}
