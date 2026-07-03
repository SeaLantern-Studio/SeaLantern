mod db;
mod output_reader;
mod reader;
mod state;
mod writer;

use sl_server_info::log::DomainEvent;

pub use output_reader::{
    spawn_server_output_reader, OutputErrorHandler, OutputLineHandler, OutputReaderHooks,
    ServerReadyHandler,
};
pub use reader::read_logs;
pub use state::{LogSource, LATEST_LOG_DB_FILE};
pub use writer::{append_log, append_sealantern_log, append_server_log, init_db, shutdown_writer};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructuredLogEventFields {
    pub event_kind: Option<String>,
    pub player: Option<String>,
    pub message: Option<String>,
}

pub fn map_domain_event(event: Option<DomainEvent>) -> StructuredLogEventFields {
    match event {
        Some(DomainEvent::ServerReady) => StructuredLogEventFields {
            event_kind: Some("server_ready".to_string()),
            ..StructuredLogEventFields::default()
        },
        Some(DomainEvent::PlayerJoin { player }) => StructuredLogEventFields {
            event_kind: Some("player_join".to_string()),
            player: Some(player),
            ..StructuredLogEventFields::default()
        },
        Some(DomainEvent::PlayerLeave { player }) => StructuredLogEventFields {
            event_kind: Some("player_leave".to_string()),
            player: Some(player),
            ..StructuredLogEventFields::default()
        },
        Some(DomainEvent::Chat { player, message }) => StructuredLogEventFields {
            event_kind: Some("chat".to_string()),
            player: Some(player),
            message: Some(message),
        },
        Some(DomainEvent::ErrorLike { message }) => StructuredLogEventFields {
            event_kind: Some("error".to_string()),
            message: Some(message),
            ..StructuredLogEventFields::default()
        },
        None => StructuredLogEventFields::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        append_server_log, map_domain_event, read_logs, shutdown_writer,
        spawn_server_output_reader, OutputReaderHooks, StructuredLogEventFields,
    };
    use sl_server_info::log::{DomainEvent, LogStream};
    use std::io::Cursor;
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

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
    }

    #[test]
    fn read_logs_surfaces_database_open_failures() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let blocked_root = temp_dir.path().join("blocked-root");
        std::fs::write(&blocked_root, b"not a directory")
            .expect("file-backed log root should exist");
        let blocked_server_path = blocked_root.join("server-a");

        let error = read_logs(&blocked_server_path, 0, None)
            .expect_err("log DB open failure should not be downgraded to empty logs");

        assert!(
            error.contains("打开日志数据库失败") || error.contains("重建日志数据库失败"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn append_server_log_persists_lines_for_history_reads() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let server_path = temp_dir.path().join("server-a");
        std::fs::create_dir_all(&server_path).expect("server dir should exist");

        append_server_log("core-history-server", &server_path, "line-1")
            .expect("first append should succeed");
        append_server_log("core-history-server", &server_path, "line-2")
            .expect("second append should succeed");
        shutdown_writer("core-history-server");

        let logs = read_logs(&server_path, 0, None).expect("history read should succeed");
        assert_eq!(logs, vec!["line-1".to_string(), "line-2".to_string()]);
    }

    #[test]
    fn spawn_server_output_reader_persists_lines_and_reports_server_ready() {
        let temp_dir = tempfile::tempdir().expect("temp dir should exist");
        let server_path = temp_dir.path().join("server-ready");
        std::fs::create_dir_all(&server_path).expect("server dir should exist");

        let (line_tx, line_rx) = mpsc::channel::<String>();
        let ready_hits = Arc::new(Mutex::new(0usize));
        let ready_hits_cloned = Arc::clone(&ready_hits);

        spawn_server_output_reader(
            "core-output-reader".to_string(),
            server_path.clone(),
            LogStream::Stdout,
            Cursor::new(
                b"[12:00:00] [Server thread/INFO]: Done (5.123s)! For help, type \"help\"\n"
                    .to_vec(),
            ),
            OutputReaderHooks {
                on_line: Arc::new(move |_server_id, line| {
                    line_tx.send(line.to_string()).map_err(|e| e.to_string())
                }),
                on_server_ready: Some(Arc::new(move |_server_id| {
                    *ready_hits_cloned
                        .lock()
                        .expect("ready hits lock should succeed") += 1;
                    Ok(())
                })),
                on_error: None,
            },
        );

        let line = line_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("output reader should emit line");
        assert!(line.contains("Done (5.123s)!"));

        shutdown_writer("core-output-reader");
        let persisted = read_logs(&server_path, 0, None).expect("history read should succeed");
        assert_eq!(persisted.len(), 1);
        assert_eq!(*ready_hits.lock().expect("ready hits lock should succeed"), 1);
    }
}
