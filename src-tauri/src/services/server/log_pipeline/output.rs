use super::state::{decode_console_bytes, server_log_processors, ServerLogProcessor, SERVER_LOG_EVENT_HANDLER};
use std::io::{BufRead, BufReader, Read};

pub fn add_server_log_processor(processor: ServerLogProcessor) -> Result<(), String> {
    let processors = server_log_processors();
    let mut guard = processors
        .lock()
        .map_err(|_| "server log processors lock poisoned".to_string())?;
    guard.push(processor);
    Ok(())
}

pub fn clear_server_log_processors() -> Result<(), String> {
    let processors = server_log_processors();
    let mut guard = processors
        .lock()
        .map_err(|_| "server log processors lock poisoned".to_string())?;
    guard.clear();
    Ok(())
}

pub fn spawn_server_output_reader<R>(server_id: String, reader: R)
where
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

                    let _ = super::writer::append_server_log(&server_id, &line);

                    if line.contains("Done (") && line.contains(")! For help") {
                        crate::services::global::server_manager().clear_starting(&server_id);
                        let _ = crate::plugins::api::emit_server_ready(&server_id);
                    }
                }
                Err(_) => break,
            }
        }
    });
}

pub fn emit_server_log_line(server_id: &str, line: &str) {
    let processed_line = process_log_line(server_id, line);
    if let Some(handler) = SERVER_LOG_EVENT_HANDLER.get() {
        let _ = handler(server_id, &processed_line);
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
