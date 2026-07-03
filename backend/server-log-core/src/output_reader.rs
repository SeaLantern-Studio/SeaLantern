use crate::append_server_log;
use crate::state::decode_console_bytes;
use sl_server_info::log::{parse_log_line, DomainEvent, LogLineInput, LogStream};
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
use std::sync::Arc;

pub type OutputLineHandler = dyn Fn(&str, &str) -> Result<(), String> + Send + Sync;
pub type ServerReadyHandler = dyn Fn(&str) -> Result<(), String> + Send + Sync;
pub type OutputErrorHandler = dyn Fn(&str, &str, &str) + Send + Sync;

#[derive(Clone)]
pub struct OutputReaderHooks {
    pub on_line: Arc<OutputLineHandler>,
    pub on_server_ready: Option<Arc<ServerReadyHandler>>,
    pub on_error: Option<Arc<OutputErrorHandler>>,
}

pub fn spawn_server_output_reader<R>(
    server_id: String,
    server_path: PathBuf,
    stream: LogStream,
    reader: R,
    hooks: OutputReaderHooks,
) where
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

                    if let Err(error) = append_server_log(&server_id, &server_path, &line) {
                        report_error(&hooks, &server_id, "persist", &error);
                    }

                    if let Err(error) = (hooks.on_line)(&server_id, &line) {
                        report_error(&hooks, &server_id, "emit", &error);
                    }

                    let parsed = parse_log_line(None, LogLineInput { raw: line, stream });
                    if matches!(parsed.event, Some(DomainEvent::ServerReady)) {
                        if let Some(on_server_ready) = &hooks.on_server_ready {
                            if let Err(error) = on_server_ready(&server_id) {
                                report_error(&hooks, &server_id, "server_ready", &error);
                            }
                        }
                    }
                }
                Err(error) => {
                    report_error(&hooks, &server_id, "read", &error.to_string());
                    break;
                }
            }
        }
    });
}

fn report_error(hooks: &OutputReaderHooks, server_id: &str, phase: &str, error: &str) {
    if let Some(on_error) = &hooks.on_error {
        on_error(server_id, phase, error);
    }
}
