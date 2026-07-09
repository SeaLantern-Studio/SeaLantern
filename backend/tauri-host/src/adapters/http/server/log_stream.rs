use super::state::subscribe_log_events;
use crate::utils::logger::{log_error_ctx, log_warn_ctx};
use axum::response::{sse::Event, IntoResponse, Sse};
use tokio_stream::StreamExt as _;

fn log_event_to_sse(event: super::state::LogEvent) -> Result<Event, String> {
    let json = serde_json::to_string(&event).map_err(|error| {
        let message = format!("[SSE] Failed to serialize log event: {}", error);
        log_error_ctx("http.server.log_stream", "log_event_to_sse", &message);
        message
    })?;

    Ok(Event::default().data(json))
}

fn map_broadcast_result(
    result: Result<
        super::state::LogEvent,
        tokio_stream::wrappers::errors::BroadcastStreamRecvError,
    >,
) -> Result<Event, String> {
    match result {
        Ok(event) => log_event_to_sse(event),
        Err(error) => {
            let message = format!("[SSE] Broadcast error: {}", error);
            log_warn_ctx("http.server.log_stream", "map_broadcast_result", &message);
            Err(message)
        }
    }
}

/// 处理日志 SSE 订阅
pub(super) async fn handle_log_stream() -> impl IntoResponse {
    let receiver = subscribe_log_events();
    let stream = tokio_stream::wrappers::BroadcastStream::new(receiver).map(map_broadcast_result);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}

#[cfg(test)]
mod tests {
    use super::map_broadcast_result;
    use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

    #[test]
    fn lagged_broadcast_error_is_not_silently_dropped() {
        let error = map_broadcast_result(Err(BroadcastStreamRecvError::Lagged(3)))
            .expect_err("lagged SSE broadcast should surface as a stream error");

        assert!(error.contains("Broadcast error"), "unexpected error: {}", error);
        assert!(error.contains("3"), "unexpected error: {}", error);
    }

    #[test]
    fn normal_broadcast_event_still_maps_to_sse_event() {
        let result = map_broadcast_result(Ok(super::super::state::LogEvent {
            server_id: "alpha".to_string(),
            line: "[Sea Lantern] hello".to_string(),
        }));

        assert!(result.is_ok(), "normal SSE event should keep streaming");
    }
}
