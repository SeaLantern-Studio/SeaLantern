use super::state::{subscribe_runtime_events, RuntimeEventEnvelope};
use crate::utils::logger::{log_error_ctx, log_warn_ctx};
use axum::response::{sse::Event, IntoResponse, Sse};
use tokio_stream::StreamExt as _;

fn runtime_event_id(event: &RuntimeEventEnvelope) -> &str {
    match event {
        RuntimeEventEnvelope::Server(event) => &event.event_id,
        RuntimeEventEnvelope::App(event) => &event.event_id,
    }
}

fn runtime_event_name(event: &RuntimeEventEnvelope) -> &'static str {
    match event {
        RuntimeEventEnvelope::Server(_) => "server",
        RuntimeEventEnvelope::App(_) => "app",
    }
}

fn runtime_event_to_sse(event: RuntimeEventEnvelope) -> Result<Event, String> {
    let event_id = runtime_event_id(&event).to_string();
    let event_name = runtime_event_name(&event);
    let json = serde_json::to_string(&event).map_err(|error| {
        let message = format!("[SSE] Failed to serialize runtime event: {}", error);
        log_error_ctx("http.server.event_stream", "runtime_event_to_sse", &message);
        message
    })?;

    Ok(Event::default().id(event_id).event(event_name).data(json))
}

fn map_broadcast_result(
    result: Result<
        RuntimeEventEnvelope,
        tokio_stream::wrappers::errors::BroadcastStreamRecvError,
    >,
) -> Result<Event, String> {
    match result {
        Ok(event) => runtime_event_to_sse(event),
        Err(error) => {
            let message = format!("[SSE] Runtime event broadcast error: {}", error);
            log_warn_ctx(
                "http.server.event_stream",
                "map_broadcast_result",
                &message,
            );
            Err(message)
        }
    }
}

/// 处理统一运行时事件 SSE 订阅。
pub(super) async fn handle_runtime_event_stream() -> impl IntoResponse {
    let receiver = subscribe_runtime_events();
    let stream = tokio_stream::wrappers::BroadcastStream::new(receiver).map(map_broadcast_result);

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}

#[cfg(test)]
mod tests {
    use super::{map_broadcast_result, runtime_event_to_sse};
    use crate::adapters::http::server::state::RuntimeEventEnvelope;
    use crate::services::events::{
        AppEventEnvelope, AppEventKind, AppEventPayload, EventScope, ServerEventEnvelope,
        ServerEventKind, ServerEventPayload,
    };
    use tokio_stream::wrappers::errors::BroadcastStreamRecvError;

    #[test]
    fn lagged_runtime_broadcast_error_is_not_silently_dropped() {
        let error = map_broadcast_result(Err(BroadcastStreamRecvError::Lagged(2)))
            .expect_err("lagged runtime SSE broadcast should surface as a stream error");

        assert!(error.contains("Runtime event broadcast error"));
        assert!(error.contains("2"));
    }

    #[test]
    fn runtime_sse_event_serializes_server_payload() {
        let event = RuntimeEventEnvelope::Server(ServerEventEnvelope {
            event_id: "server-event-7".to_string(),
            occurred_at: 7,
            scope: EventScope::Server,
            server_id: "alpha".to_string(),
            source: "runtime_stdout".to_string(),
            kind: ServerEventKind::OutputRawLine,
            payload: ServerEventPayload::RawLine {
                line: "hello".to_string(),
                stream: "stdout".to_string(),
            },
        });

        assert!(
            runtime_event_to_sse(event).is_ok(),
            "server runtime SSE event should serialize"
        );
    }

    #[test]
    fn app_runtime_event_serializes_to_sse() {
        let result = map_broadcast_result(Ok(RuntimeEventEnvelope::App(AppEventEnvelope {
            event_id: "app-event-3".to_string(),
            occurred_at: 3,
            scope: EventScope::App,
            action: "detect_java".to_string(),
            source: "frontend_user".to_string(),
            kind: AppEventKind::OperationSucceeded,
            payload: AppEventPayload::Operation {
                action: "detect_java".to_string(),
                detail: Some("done".to_string()),
                error: None,
            },
        })));

        assert!(result.is_ok(), "app runtime SSE event should keep streaming");
    }
}
