//! 服务器日志 host glue：保留事件发布、ServerManager 集成和路径解析，
//! 纯日志域逻辑由 `server-log` 承担。

use crate::services::events::{
    publish_server_output_raw, publish_server_output_structured, ServerEventSource,
};
use crate::utils::logger;
use server_log::{
    map_domain_event as map_core_domain_event, read_logs,
    spawn_server_output_reader as spawn_reader, OutputReaderHooks, StructuredLogEventFields,
};
use sl_server_info::log::{parse_log_line, DomainEvent, LogLineInput, LogStream, ParsedLogLine};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

pub type ServerLogEventHandler = dyn Fn(&str, &str, LogStream) -> Result<(), String> + Send + Sync;
pub type ServerLogProcessor = dyn Fn(&str, &str) -> String + Send + Sync;
type ServerLogProcessorList = Arc<Mutex<Vec<Arc<ServerLogProcessor>>>>;

static SERVER_LOG_EVENT_HANDLER: OnceLock<Arc<ServerLogEventHandler>> = OnceLock::new();
static SERVER_LOG_PROCESSORS: OnceLock<ServerLogProcessorList> = OnceLock::new();

pub(crate) fn map_domain_event(event: Option<DomainEvent>) -> StructuredLogEventFields {
    map_core_domain_event(event)
}

#[allow(dead_code)]
pub fn set_server_log_event_handler(handler: Arc<ServerLogEventHandler>) -> Result<(), String> {
    SERVER_LOG_EVENT_HANDLER
        .set(handler)
        .map_err(|_| "server log event handler already set".to_string())
}

#[allow(dead_code)]
pub fn add_server_log_processor(processor: Arc<ServerLogProcessor>) -> Result<(), String> {
    let processors = server_log_processors();
    let mut guard = processors
        .lock()
        .map_err(|_| "server log processors lock poisoned".to_string())?;
    guard.push(processor);
    Ok(())
}

#[allow(dead_code)]
pub fn clear_server_log_processors() -> Result<(), String> {
    let processors = server_log_processors();
    let mut guard = processors
        .lock()
        .map_err(|_| "server log processors lock poisoned".to_string())?;
    guard.clear();
    Ok(())
}

pub fn init_db(server_path: &Path) -> Result<(), String> {
    server_log::init_db(server_path)
}

pub fn shutdown_writer(server_id: &str) {
    server_log::shutdown_writer(server_id)
}

pub fn append_sealantern_log(server_id: &str, message: &str) -> Result<(), String> {
    let server_path = resolve_server_path(server_id)?;
    server_log::append_sealantern_log(server_id, &server_path, message)?;
    emit_server_log_line_with_stream(server_id, message, LogStream::Unknown);
    Ok(())
}

#[allow(dead_code)]
pub fn append_server_log(server_id: &str, message: &str) -> Result<(), String> {
    let server_path = resolve_server_path(server_id)?;
    server_log::append_server_log(server_id, &server_path, message)?;
    emit_server_log_line_with_stream(server_id, message, LogStream::Unknown);
    Ok(())
}

pub fn get_logs_checked(
    server_id: &str,
    since: usize,
    recent_limit: Option<usize>,
) -> Result<Vec<String>, String> {
    let server_path = resolve_server_path(server_id)?;
    read_logs(&server_path, since as u64, recent_limit)
}

pub fn get_all_logs_checked() -> Result<Vec<(String, Vec<String>)>, String> {
    let server_ids = crate::services::global::server_manager()
        .get_server_list_checked()?
        .into_iter()
        .map(|server| server.id)
        .collect::<Vec<String>>();

    collect_logs_for_server_ids(&server_ids, |server_id| get_logs_checked(server_id, 0, None))
}

pub fn spawn_server_output_reader<R>(server_id: String, stream: LogStream, reader: R)
where
    R: std::io::Read + Send + 'static,
{
    let server_path = match resolve_server_path(&server_id) {
        Ok(path) => path,
        Err(error) => {
            logger::log_warn_ctx(
                "server.log_pipeline",
                "spawn_server_output_reader",
                &format!(
                    "server output reader skipped because server path resolution failed: server_id={} error={}",
                    server_id, error
                ),
            );
            return;
        }
    };

    let source_stream = stream;
    spawn_reader(
        server_id,
        server_path,
        stream,
        reader,
        OutputReaderHooks {
            on_line: Arc::new(move |server_id, line, parsed| {
                emit_server_log_line_with_stream_and_parsed(server_id, line, source_stream, parsed);
                Ok(())
            }),
            on_server_ready: Some(Arc::new(move |server_id| {
                crate::services::global::server_manager().clear_starting(server_id);
                let _ = crate::plugins::api::emit_server_ready(server_id);
                Ok(())
            })),
            on_error: Some(Arc::new(move |server_id, phase, error| {
                logger::log_warn_ctx(
                    "server.log_pipeline",
                    "spawn_server_output_reader",
                    &format!(
                        "server output reader issue: server_id={} phase={} error={}",
                        server_id, phase, error
                    ),
                );
            })),
        },
    )
}

#[allow(dead_code)]
pub fn emit_server_log_line(server_id: &str, line: &str) {
    emit_server_log_line_with_stream(server_id, line, LogStream::Unknown)
}

pub fn emit_server_log_line_with_stream(server_id: &str, line: &str, stream: LogStream) {
    let parsed = parse_log_line(None, LogLineInput { raw: line.to_string(), stream });
    emit_server_log_line_with_stream_and_parsed(server_id, line, stream, &parsed)
}

fn emit_server_log_line_with_stream_and_parsed(
    server_id: &str,
    line: &str,
    stream: LogStream,
    parsed_before_processors: &ParsedLogLine,
) {
    let processed_line = process_log_line(server_id, line);
    let source = match stream {
        LogStream::Stdout => ServerEventSource::RuntimeStdout,
        LogStream::Stderr => ServerEventSource::RuntimeStderr,
        LogStream::Unknown => ServerEventSource::RuntimeUnknown,
    };

    let parsed = if processed_line == line {
        parsed_before_processors.clone()
    } else {
        parse_log_line(None, LogLineInput { raw: processed_line.clone(), stream })
    };
    let mapped = map_domain_event(parsed.event);
    let stream_name = match stream {
        LogStream::Stdout => "stdout",
        LogStream::Stderr => "stderr",
        LogStream::Unknown => "unknown",
    };

    let _ = publish_server_output_raw(server_id, source.clone(), &processed_line, stream_name);
    let _ = publish_server_output_structured(
        server_id,
        source,
        &processed_line,
        stream_name,
        mapped.event_kind,
        mapped.player,
        mapped.message,
    );

    if let Some(handler) = SERVER_LOG_EVENT_HANDLER.get() {
        let _ = handler(server_id, &processed_line, stream);
    }

    #[cfg(feature = "docker")]
    {
        let event = crate::adapters::http::server::LogEvent {
            server_id: server_id.to_string(),
            line: processed_line.clone(),
        };
        let _ = crate::adapters::http::server::get_log_sender().send(event);
    }
}

fn server_log_processors() -> &'static ServerLogProcessorList {
    SERVER_LOG_PROCESSORS.get_or_init(|| Arc::new(Mutex::new(Vec::new())))
}

fn process_log_line(server_id: &str, line: &str) -> String {
    let processors = server_log_processors();
    let guard = match processors.lock() {
        Ok(guard) => guard,
        Err(_) => return line.to_string(),
    };

    let mut processed_line = line.to_string();
    for processor in &*guard {
        processed_line = processor(server_id, &processed_line);
    }
    processed_line
}

fn resolve_server_path(server_id: &str) -> Result<PathBuf, String> {
    crate::services::global::server_manager()
        .get_server_list_checked()?
        .into_iter()
        .find(|server| server.id == server_id)
        .map(|server| PathBuf::from(server.path))
        .ok_or_else(|| format!("未找到服务器: {}", server_id))
}

fn collect_logs_for_server_ids<F>(
    server_ids: &[String],
    mut read_logs_for_server: F,
) -> Result<Vec<(String, Vec<String>)>, String>
where
    F: FnMut(&str) -> Result<Vec<String>, String>,
{
    let mut result = Vec::with_capacity(server_ids.len());
    for server_id in server_ids {
        let logs = read_logs_for_server(server_id)?;
        result.push((server_id.clone(), logs));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{collect_logs_for_server_ids, map_domain_event, StructuredLogEventFields};
    use sl_server_info::log::DomainEvent;

    #[test]
    fn map_domain_event_returns_consistent_structured_fields() {
        assert_eq!(
            map_domain_event(Some(DomainEvent::ServerReady)),
            StructuredLogEventFields {
                event_kind: Some("server_ready".to_string()),
                ..StructuredLogEventFields::default()
            }
        );

        assert_eq!(
            map_domain_event(Some(DomainEvent::PlayerJoin { player: "Alex".to_string() })),
            StructuredLogEventFields {
                event_kind: Some("player_join".to_string()),
                player: Some("Alex".to_string()),
                ..StructuredLogEventFields::default()
            }
        );

        assert_eq!(
            map_domain_event(Some(DomainEvent::Chat {
                player: "Alex".to_string(),
                message: "Hello".to_string(),
            })),
            StructuredLogEventFields {
                event_kind: Some("chat".to_string()),
                player: Some("Alex".to_string()),
                message: Some("Hello".to_string()),
            }
        );

        assert_eq!(
            map_domain_event(Some(DomainEvent::ErrorLike { message: "boom".to_string() })),
            StructuredLogEventFields {
                event_kind: Some("error".to_string()),
                message: Some("boom".to_string()),
                ..StructuredLogEventFields::default()
            }
        );
    }

    #[test]
    fn collect_logs_for_server_ids_surfaces_first_read_failure() {
        let server_ids = vec!["server-a".to_string(), "server-b".to_string()];

        let error = collect_logs_for_server_ids(&server_ids, |server_id| match server_id {
            "server-a" => Ok(vec!["line-1".to_string()]),
            "server-b" => Err("db broken".to_string()),
            other => Err(format!("unexpected server id: {other}")),
        })
        .expect_err("aggregate log loading should not silently skip failed servers");

        assert_eq!(error, "db broken");
    }
}
