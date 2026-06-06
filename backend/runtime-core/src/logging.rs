use chrono::{DateTime, Local};
use std::sync::{Arc, Mutex};

const KNOWN_LEVEL_PREFIXES: [&str; 6] =
    ["[TRACE]", "[DEBUG]", "[INFO]", "[WARN]", "[ERROR]", "[FATAL]"];

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub module: String,
    pub function: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogLine {
    pub timestamp: String,
    pub level: String,
    pub module: String,
    pub function: String,
    pub message: String,
    pub formatted: String,
}

#[derive(Debug, Clone)]
pub struct LogFields<'a> {
    pub module: &'a str,
    pub function: &'a str,
}

impl<'a> LogFields<'a> {
    pub const fn new(module: &'a str, function: &'a str) -> Self {
        Self { module, function }
    }
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

    pub fn add_entry(&self, entry: LogEntry) {
        let mut logs = self.logs.lock().unwrap_or_else(|e| e.into_inner());
        logs.push(entry);

        if logs.len() > self.max_logs {
            logs.remove(0);
        }
    }

    pub fn add_log(&self, level: LogLevel, fields: LogFields<'_>, message: &str) {
        self.add_entry(build_log_entry(level, fields, message));
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

fn now_timestamp() -> String {
    let timestamp: DateTime<Local> = Local::now();
    timestamp.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

fn build_log_entry(level: LogLevel, fields: LogFields<'_>, message: &str) -> LogEntry {
    LogEntry {
        timestamp: now_timestamp(),
        level: level.as_str().to_string(),
        module: fields.module.to_string(),
        function: fields.function.to_string(),
        message: message.to_string(),
    }
}

fn parse_level(message: &str, default_level: LogLevel) -> LogLevel {
    if message.contains("[TRACE]") {
        return LogLevel::Trace;
    }
    if message.contains("[FATAL]") {
        return LogLevel::Fatal;
    }
    if message.contains("[ERROR]") {
        return LogLevel::Error;
    }
    if message.contains("[WARN]") || message.contains("WARNING") {
        return LogLevel::Warn;
    }
    if message.contains("[DEBUG]") {
        return LogLevel::Debug;
    }
    if message.contains("[INFO]") {
        return LogLevel::Info;
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

fn parse_legacy_message(message: &str) -> (String, String, String) {
    let trimmed = strip_level_prefix(message);

    if let Some(body) = trimmed.strip_prefix('[') {
        if let Some((scope, rest)) = body.split_once(']') {
            let msg = rest.trim_start_matches(':').trim_start();
            if let Some((module, function)) = scope.split_once('.') {
                return (module.to_string(), function.to_string(), msg.to_string());
            }

            return (scope.to_string(), "-".to_string(), msg.to_string());
        }
    }

    if let Some((scope, msg)) = trimmed.split_once('|') {
        let scope = scope.trim();
        let msg = msg.trim();
        if let Some((module, function)) = scope.split_once(':') {
            return (module.trim().to_string(), function.trim().to_string(), msg.to_string());
        }

        return (scope.to_string(), "-".to_string(), msg.to_string());
    }

    ("runtime".to_string(), "-".to_string(), trimmed.to_string())
}

pub fn format_log_entry(entry: &LogEntry) -> String {
    format!(
        "{} {} {}:{} | {}",
        entry.timestamp, entry.level, entry.module, entry.function, entry.message
    )
}

pub fn to_log_line(entry: LogEntry) -> LogLine {
    let formatted = format_log_entry(&entry);
    LogLine {
        timestamp: entry.timestamp,
        level: entry.level,
        module: entry.module,
        function: entry.function,
        message: entry.message,
        formatted,
    }
}

pub(crate) fn capture_stdout(message: &str) {
    let level = parse_level(message, LogLevel::Info);
    let (module, function, parsed_message) = parse_legacy_message(message);
    GLOBAL_LOG_COLLECTOR.add_log(level, LogFields::new(&module, &function), &parsed_message);
}

pub(crate) fn capture_stderr(message: &str) {
    let level = parse_level(message, LogLevel::Error);
    let (module, function, parsed_message) = parse_legacy_message(message);
    GLOBAL_LOG_COLLECTOR.add_log(level, LogFields::new(&module, &function), &parsed_message);
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
    pub static ref GLOBAL_LOG_COLLECTOR: LogCollector = LogCollector::new(2000);
}

pub fn log_with(level: LogLevel, fields: LogFields<'_>, message: &str) {
    if matches!(level, LogLevel::Error | LogLevel::Fatal) {
        std::eprintln!("[{}] [{}:{}] {}", level.as_str(), fields.module, fields.function, message);
    }
    GLOBAL_LOG_COLLECTOR.add_log(level, fields, message);
}

pub fn log_warn(message: &str) {
    log_with(LogLevel::Warn, LogFields::new("runtime", "-"), message);
}

pub fn log_trace(message: &str) {
    log_with(LogLevel::Trace, LogFields::new("runtime", "-"), message);
}

pub fn log_debug(message: &str) {
    log_with(LogLevel::Debug, LogFields::new("runtime", "-"), message);
}

pub fn log_info(message: &str) {
    log_with(LogLevel::Info, LogFields::new("runtime", "-"), message);
}

pub fn log_error(message: &str) {
    log_with(LogLevel::Error, LogFields::new("runtime", "-"), message);
}

pub fn log_fatal(message: &str) {
    log_with(LogLevel::Fatal, LogFields::new("runtime", "-"), message);
}

pub fn log_trace_ctx(module: &str, function: &str, message: &str) {
    log_with(LogLevel::Trace, LogFields::new(module, function), message);
}

pub fn log_debug_ctx(module: &str, function: &str, message: &str) {
    log_with(LogLevel::Debug, LogFields::new(module, function), message);
}

pub fn log_info_ctx(module: &str, function: &str, message: &str) {
    log_with(LogLevel::Info, LogFields::new(module, function), message);
}

pub fn log_warn_ctx(module: &str, function: &str, message: &str) {
    log_with(LogLevel::Warn, LogFields::new(module, function), message);
}

pub fn log_error_ctx(module: &str, function: &str, message: &str) {
    log_with(LogLevel::Error, LogFields::new(module, function), message);
}

pub fn log_fatal_ctx(module: &str, function: &str, message: &str) {
    log_with(LogLevel::Fatal, LogFields::new(module, function), message);
}

pub fn log_user_action(scope: &str, action: &str, detail: &str) {
    let message = if detail.trim().is_empty() {
        format!("action={}", action)
    } else {
        format!("action={} {}", action, detail.trim())
    };

    let (module, function) = split_scope(scope);
    log_trace_ctx(module, function, &message);
}

pub fn log_user_action_error(scope: &str, action: &str, detail: &str, error: &str) {
    let message = if detail.trim().is_empty() {
        format!("action={} error={}", action, error.trim())
    } else {
        format!("action={} {} error={}", action, detail.trim(), error.trim())
    };

    let (module, function) = split_scope(scope);
    log_error_ctx(module, function, &message);
}

fn split_scope(scope: &str) -> (&str, &str) {
    scope.split_once('.').unwrap_or((scope, "-"))
}

#[cfg(test)]
mod tests {
    use super::{
        format_log_entry, parse_legacy_message, split_scope, to_log_line, LogEntry, LogLevel,
    };

    fn partial_eq(left: (&str, &str, &str), right: (&str, &str, &str)) {
        assert_eq!(left.0, right.0);
        assert_eq!(left.1, right.1);
        assert_eq!(left.2, right.2);
    }

    #[test]
    fn format_uses_requested_structure() {
        let entry = LogEntry {
            timestamp: "2026-01-01 00:00:00.000".to_string(),
            level: LogLevel::Info.as_str().to_string(),
            module: "server.manager".to_string(),
            function: "start".to_string(),
            message: "ok".to_string(),
        };

        assert_eq!(
            format_log_entry(&entry),
            "2026-01-01 00:00:00.000 INFO server.manager:start | ok"
        );
    }

    #[test]
    fn to_log_line_keeps_structured_fields() {
        let line = to_log_line(LogEntry {
            timestamp: "t".to_string(),
            level: "WARN".to_string(),
            module: "mod".to_string(),
            function: "fn".to_string(),
            message: "msg".to_string(),
        });

        assert_eq!(line.level, "WARN");
        assert_eq!(line.module, "mod");
        assert_eq!(line.function, "fn");
        assert_eq!(line.formatted, "t WARN mod:fn | msg");
    }

    #[test]
    fn legacy_parser_extracts_bracket_scope() {
        let parsed = parse_legacy_message("[server.manager] action=start server_id=a");
        partial_eq(
            (&parsed.0, &parsed.1, &parsed.2),
            ("server", "manager", "action=start server_id=a"),
        );
    }

    #[test]
    fn legacy_parser_extracts_pipe_scope() {
        let parsed = parse_legacy_message("http.api:handle | request failed");
        partial_eq((&parsed.0, &parsed.1, &parsed.2), ("http.api", "handle", "request failed"));
    }

    #[test]
    fn split_scope_falls_back_when_no_function() {
        assert_eq!(split_scope("runtime"), ("runtime", "-"));
        assert_eq!(split_scope("server.manager"), ("server", "manager"));
    }
}
