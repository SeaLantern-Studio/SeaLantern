use chrono::{DateTime, Local};
use std::sync::{Arc, Mutex};

const KNOWN_LEVEL_PREFIXES: [&str; 6] =
    ["[TRACE]", "[DEBUG]", "[INFO]", "[WARN]", "[ERROR]", "[FATAL]"];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogLine {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub formatted: String,
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
    if message.contains("[TRACE]") {
        return "TRACE";
    }
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

fn strip_level_prefix(message: &str) -> &str {
    let trimmed = message.trim_start();
    for prefix in KNOWN_LEVEL_PREFIXES {
        if let Some(rest) = trimmed.strip_prefix(prefix) {
            return rest.trim_start();
        }
    }
    trimmed
}

pub fn format_log_entry(entry: &LogEntry) -> String {
    format!("[{}] [{}] {}", entry.timestamp, entry.level, entry.message)
}

pub fn to_log_line(entry: LogEntry) -> LogLine {
    let formatted = format_log_entry(&entry);
    LogLine {
        timestamp: entry.timestamp,
        level: entry.level,
        message: entry.message,
        formatted,
    }
}

pub(crate) fn capture_stdout(message: &str) {
    GLOBAL_LOG_COLLECTOR.add_log(parse_level(message, "INFO"), strip_level_prefix(message));
}

pub(crate) fn capture_stderr(message: &str) {
    GLOBAL_LOG_COLLECTOR.add_log(parse_level(message, "ERROR"), strip_level_prefix(message));
}

pub fn capture_println(message: String) {
    std::println!("{}", message);
    capture_stdout(&message);
}

pub fn capture_eprintln(message: String) {
    std::eprintln!("{}", message);
    capture_stderr(&message);
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_LOG_COLLECTOR: LogCollector = LogCollector::new(1000);
}

pub fn log_warn(message: &str) {
    GLOBAL_LOG_COLLECTOR.add_log("WARN", message);
}

pub fn log_trace(message: &str) {
    GLOBAL_LOG_COLLECTOR.add_log("TRACE", message);
}

pub fn log_debug(message: &str) {
    GLOBAL_LOG_COLLECTOR.add_log("DEBUG", message);
}

pub fn log_error(message: &str) {
    std::eprintln!("[ERROR] {}", message);
    GLOBAL_LOG_COLLECTOR.add_log("ERROR", message);
}

pub fn log_user_action(scope: &str, action: &str, detail: &str) {
    let message = if detail.trim().is_empty() {
        format!("[{}] action={}", scope, action)
    } else {
        format!("[{}] action={} {}", scope, action, detail.trim())
    };

    log_trace(&message);
}

pub fn log_user_action_error(scope: &str, action: &str, detail: &str, error: &str) {
    let message = if detail.trim().is_empty() {
        format!("[{}] action={} error={}", scope, action, error.trim())
    } else {
        format!("[{}] action={} {} error={}", scope, action, detail.trim(), error.trim())
    };

    log_error(&message);
}
