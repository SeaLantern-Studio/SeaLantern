use std::io::Read;
use std::path::PathBuf;

use crate::services::server::log_pipeline as server_log_pipeline;
use crate::services::server::terminal_transcript;
use crate::utils::logger;
use sea_lantern_server_local_setup_core::decode_console_bytes;
use sl_server_info::log::{parse_log_line, DomainEvent, LogLineInput, LogStream};

const READ_CHUNK_SIZE: usize = 8192;

pub fn spawn_local_terminal_reader<R>(server_id: String, server_path: PathBuf, stream: LogStream, reader: R)
where
    R: Read + Send + 'static,
{
    std::thread::spawn(move || {
        let mut reader = reader;
        let mut read_buffer = vec![0_u8; READ_CHUNK_SIZE];
        let mut pending_line = String::new();

        loop {
            match reader.read(&mut read_buffer) {
                Ok(0) => {
                    flush_pending_line(&server_id, stream, &mut pending_line);
                    break;
                }
                Ok(bytes_read) => {
                    let chunk_bytes = &read_buffer[..bytes_read];
                    let decoded_chunk = decode_console_bytes(chunk_bytes);
                    terminal_transcript::append_transcript_text_logged(
                        &server_id,
                        &server_path,
                        &decoded_chunk,
                    );
                    drain_normalized_log_lines(&server_id, stream, &decoded_chunk, &mut pending_line);
                }
                Err(error) => {
                    logger::log_warn_ctx(
                        "server.runtime.local_terminal_reader",
                        "spawn_local_terminal_reader",
                        &format!("server_id={} error={}", server_id, error),
                    );
                    flush_pending_line(&server_id, stream, &mut pending_line);
                    break;
                }
            }
        }
    });
}

fn drain_normalized_log_lines(
    server_id: &str,
    stream: LogStream,
    chunk: &str,
    pending_line: &mut String,
) {
    if chunk.is_empty() {
        return;
    }

    pending_line.push_str(chunk);

    while let Some(newline_index) = pending_line.find('\n') {
        let drained: String = pending_line.drain(..=newline_index).collect();
        let line = drained.trim_end_matches(['\r', '\n']);
        emit_normalized_log_line(server_id, stream, line);
    }
}

fn flush_pending_line(server_id: &str, stream: LogStream, pending_line: &mut String) {
    if pending_line.is_empty() {
        return;
    }

    let line = pending_line.trim_end_matches(['\r', '\n']).to_string();
    pending_line.clear();
    emit_normalized_log_line(server_id, stream, &line);
}

fn emit_normalized_log_line(server_id: &str, stream: LogStream, line: &str) {
    if line.trim().is_empty() {
        return;
    }

    if let Err(error) = server_log_pipeline::append_server_log(server_id, line) {
        logger::log_warn_ctx(
            "server.runtime.local_terminal_reader",
            "append_server_log",
            &format!("server_id={} error={}", server_id, error),
        );
    }

    let parsed = parse_log_line(None, LogLineInput { raw: line.to_string(), stream });
    if matches!(parsed.event, Some(DomainEvent::ServerReady)) {
        crate::services::global::server_manager().clear_starting(server_id);
        let _ = crate::plugins::api::emit_server_ready(server_id);
    }
}

#[cfg(test)]
mod tests {
    use super::{drain_normalized_log_lines, flush_pending_line};
    use sl_server_info::log::LogStream;

    #[test]
    fn drain_normalized_log_lines_keeps_prompt_like_partial_text_pending() {
        let server_id = "reader-test-pending";
        let mut pending = String::new();

        drain_normalized_log_lines(server_id, LogStream::Stdout, "> prompt", &mut pending);

        assert_eq!(pending, "> prompt");
    }

    #[test]
    fn flush_pending_line_clears_partial_line_buffer() {
        let server_id = "reader-test-flush";
        let mut pending = String::from("partial");

        flush_pending_line(server_id, LogStream::Stdout, &mut pending);

        assert!(pending.is_empty());
    }
}
