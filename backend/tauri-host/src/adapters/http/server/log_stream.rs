use super::state::subscribe_log_events;
use crate::utils::logger::capture_eprintln;
use axum::{response::{IntoResponse, Sse, sse::Event}};
use tokio_stream::StreamExt as _;

/// 处理日志 SSE 订阅
pub(super) async fn handle_log_stream() -> impl IntoResponse {
    let receiver = subscribe_log_events();
    let stream =
        tokio_stream::wrappers::BroadcastStream::new(receiver).filter_map(|result| match result {
            Ok(event) => {
                let json = serde_json::to_string(&event).ok()?;
                Some(Ok::<_, String>(Event::default().data(json)))
            }
            Err(error) => {
                capture_eprintln(format!("[SSE] Broadcast error: {}", error));
                None
            }
        });
    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}
