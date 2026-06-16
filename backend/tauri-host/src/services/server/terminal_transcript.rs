use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::models::server::ServerInstance;
use crate::utils::logger;

pub const TERMINAL_TRANSCRIPT_FILE: &str = "sea_lantern_terminal.transcript";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TerminalTranscriptChunk {
    pub cursor: u64,
    pub next_cursor: u64,
    pub data: String,
}

fn transcript_path_from_server_path(server_path: &Path) -> PathBuf {
    server_path.join(TERMINAL_TRANSCRIPT_FILE)
}

pub fn transcript_path(server: &ServerInstance) -> PathBuf {
    transcript_path_from_server_path(Path::new(&server.path))
}

pub fn reset_transcript(server: &ServerInstance) -> Result<(), String> {
    std::fs::write(transcript_path(server), b"").map_err(|e| e.to_string())
}

pub fn append_transcript_text(_server_id: &str, server_path: &Path, text: &str) -> Result<(), String> {
    let path = transcript_path_from_server_path(server_path);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    file.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn read_transcript(server: &ServerInstance, cursor: u64, max_bytes: Option<usize>) -> Result<TerminalTranscriptChunk, String> {
    let path = transcript_path(server);
    let max_bytes = max_bytes.unwrap_or(64 * 1024).clamp(1, 512 * 1024);

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .truncate(false)
        .open(&path)
        .map_err(|e| e.to_string())?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    let file_len = metadata.len();
    let start = cursor.min(file_len);
    file.seek(SeekFrom::Start(start)).map_err(|e| e.to_string())?;

    let mut buffer = vec![0_u8; max_bytes];
    let bytes_read = file.read(&mut buffer).map_err(|e| e.to_string())?;
    buffer.truncate(bytes_read);
    let data = String::from_utf8_lossy(&buffer).to_string();
    let next_cursor = start.saturating_add(bytes_read as u64);

    Ok(TerminalTranscriptChunk {
        cursor: start,
        next_cursor,
        data,
    })
}

pub fn read_transcript_checked(server_id: &str, cursor: u64, max_bytes: Option<usize>) -> Result<TerminalTranscriptChunk, String> {
    let manager = crate::services::global::server_manager();
    let server = manager.find_server_clone(server_id)?;
    read_transcript(&server, cursor, max_bytes)
}

pub fn append_transcript_text_logged(server_id: &str, server_path: &Path, text: &str) {
    if text.is_empty() {
        return;
    }

    if let Err(error) = append_transcript_text(server_id, server_path, text) {
        logger::log_warn_ctx(
            "server.terminal_transcript",
            "append_transcript_text",
            &format!("server_id={} error={}", server_id, error),
        );
    }
}
