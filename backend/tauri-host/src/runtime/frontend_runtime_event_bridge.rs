use crate::services;
use crate::services::events::{
    AppEventKind, AppEventPayload, AppEventSubscription, EventConsumer, EventConsumerKind,
    EventConsumerMetadata, ServerEventEnvelope, ServerEventKind, ServerEventPayload,
};
use crate::services::global::i18n_service;

use std::collections::HashMap;
use tauri::Emitter;

fn frontend_runtime_event_bridge_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

#[derive(serde::Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ServerStartFallbackEvent {
    server_id: String,
    server_name: String,
    from_mode: String,
    to_mode: String,
    reason: String,
}

fn build_server_start_fallback_event<F>(
    event: &ServerEventEnvelope,
    detail: Option<&str>,
    from_mode: Option<&str>,
    to_mode: Option<&str>,
    resolve_server_name: F,
) -> ServerStartFallbackEvent
where
    F: FnOnce(&str) -> Option<String>,
{
    let server_name =
        resolve_server_name(&event.server_id).unwrap_or_else(|| event.server_id.clone());

    ServerStartFallbackEvent {
        server_id: event.server_id.clone(),
        server_name,
        from_mode: from_mode.unwrap_or_default().to_string(),
        to_mode: to_mode.unwrap_or_default().to_string(),
        reason: detail.unwrap_or_default().to_string(),
    }
}

/// Connects the shared frontend runtime event bridge.
pub(crate) fn install_frontend_runtime_event_bridge(app: &tauri::App) {
    use serde::Serialize;

    #[derive(Serialize, Clone)]
    struct ServerLogLineEvent {
        server_id: String,
        line: String,
    }

    #[derive(Serialize, Clone)]
    struct StructuredServerLogEvent {
        server_id: String,
        line: String,
        stream: String,
        event_kind: Option<String>,
        player: Option<String>,
        message: Option<String>,
    }

    let server_app_handle = app.handle().clone();
    let app_app_handle = app.handle().clone();
    services::global::event_manager().register_named_consumer_with_metadata(
        "runtime.frontend_runtime_event_bridge",
        EventConsumer::both(
            std::sync::Arc::new(move |event| {
                server_app_handle
                    .emit("server-runtime-event", event.clone())
                    .map_err(|e| {
                        frontend_runtime_event_bridge_t1(
                            "plugin.bridge.emit_server_log_line_failed",
                            e.to_string(),
                        )
                    })?;

                match (&event.kind, &event.payload) {
                    (ServerEventKind::OutputRawLine, ServerEventPayload::RawLine { line, .. }) => {
                        server_app_handle
                            .emit(
                                "server-log-line",
                                ServerLogLineEvent {
                                    server_id: event.server_id.clone(),
                                    line: line.clone(),
                                },
                            )
                            .map_err(|e| {
                                frontend_runtime_event_bridge_t1(
                                    "plugin.bridge.emit_server_log_line_failed",
                                    e.to_string(),
                                )
                            })?;
                    }
                    (
                        ServerEventKind::OutputStructuredLog,
                        ServerEventPayload::StructuredLog {
                            line,
                            stream,
                            event_kind,
                            player,
                            message,
                        },
                    ) => {
                        server_app_handle
                            .emit(
                                "server-log-structured",
                                StructuredServerLogEvent {
                                    server_id: event.server_id.clone(),
                                    line: line.clone(),
                                    stream: stream.clone(),
                                    event_kind: event_kind.clone(),
                                    player: player.clone(),
                                    message: message.clone(),
                                },
                            )
                            .map_err(|e| {
                                frontend_runtime_event_bridge_t1(
                                    "plugin.bridge.emit_server_log_line_failed",
                                    e.to_string(),
                                )
                            })?;
                    }
                    (
                        ServerEventKind::LifecycleStartFallback,
                        ServerEventPayload::Lifecycle { detail, from_mode, to_mode, .. },
                    ) => {
                        server_app_handle
                            .emit(
                                "server-start-fallback",
                                build_server_start_fallback_event(
                                    event,
                                    detail.as_deref(),
                                    from_mode.as_deref(),
                                    to_mode.as_deref(),
                                    |server_id| {
                                        services::global::server_manager()
                                            .find_server_clone_optional(server_id)
                                            .ok()
                                            .flatten()
                                            .map(|server| server.name)
                                    },
                                ),
                            )
                            .map_err(|e| {
                                frontend_runtime_event_bridge_t1(
                                    "plugin.bridge.emit_server_log_line_failed",
                                    e.to_string(),
                                )
                            })?;
                    }
                    (
                        ServerEventKind::LifecycleRuntimeError,
                        ServerEventPayload::Lifecycle { .. },
                    ) => {
                        server_app_handle.emit("server-error", ()).map_err(|e| {
                            frontend_runtime_event_bridge_t1(
                                "plugin.bridge.emit_server_log_line_failed",
                                e.to_string(),
                            )
                        })?;
                    }
                    _ => {}
                }

                Ok(())
            }),
            std::sync::Arc::new(move |event| {
                app_app_handle
                    .emit("app-runtime-event", event.clone())
                    .map_err(|e| {
                        frontend_runtime_event_bridge_t1(
                            "plugin.bridge.emit_server_log_line_failed",
                            e.to_string(),
                        )
                    })?;

                match (&event.kind, &event.payload) {
                    (
                        AppEventKind::OperationRequested
                        | AppEventKind::OperationSucceeded
                        | AppEventKind::OperationFailed,
                        AppEventPayload::Operation { .. },
                    ) => {
                        app_app_handle
                            .emit("app-operation-event", event.clone())
                            .map_err(|e| {
                                frontend_runtime_event_bridge_t1(
                                    "plugin.bridge.emit_server_log_line_failed",
                                    e.to_string(),
                                )
                            })?;
                    }
                }

                Ok(())
            }),
        )
        .with_app_filter(AppEventSubscription {
            actions: Vec::new(),
            kinds: vec![
                "operation_requested".to_string(),
                "operation_succeeded".to_string(),
                "operation_failed".to_string(),
            ],
            sources: Vec::new(),
        }),
        EventConsumerMetadata::new(
            EventConsumerKind::FrontendBridge,
            "runtime.frontend_runtime_event_bridge",
            "Forward runtime server/app events to the Tauri frontend bridge.",
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::{build_server_start_fallback_event, ServerStartFallbackEvent};
    use crate::services::events::{
        EventScope, ServerEventEnvelope, ServerEventKind, ServerEventPayload,
    };

    fn sample_fallback_event() -> ServerEventEnvelope {
        ServerEventEnvelope {
            event_id: "server-event-1".to_string(),
            occurred_at: 0,
            scope: EventScope::Server,
            server_id: "alpha-id".to_string(),
            source: "runtime_manager".to_string(),
            kind: ServerEventKind::LifecycleStartFallback,
            payload: ServerEventPayload::Lifecycle {
                detail: Some("failed to start with starter".to_string()),
                error: None,
                from_mode: Some("starter".to_string()),
                to_mode: Some("jar".to_string()),
            },
        }
    }

    #[test]
    fn build_server_start_fallback_event_uses_resolved_server_name() {
        let event = sample_fallback_event();

        let payload = build_server_start_fallback_event(
            &event,
            Some("failed to start with starter"),
            Some("starter"),
            Some("jar"),
            |_| Some("Visible Alpha".to_string()),
        );

        assert_eq!(
            payload,
            ServerStartFallbackEvent {
                server_id: "alpha-id".to_string(),
                server_name: "Visible Alpha".to_string(),
                from_mode: "starter".to_string(),
                to_mode: "jar".to_string(),
                reason: "failed to start with starter".to_string(),
            }
        );
    }

    #[test]
    fn build_server_start_fallback_event_falls_back_to_server_id_when_name_missing() {
        let event = sample_fallback_event();

        let payload = build_server_start_fallback_event(
            &event,
            Some("failed to start with starter"),
            Some("starter"),
            Some("jar"),
            |_| None,
        );

        assert_eq!(payload.server_name, "alpha-id");
        assert_eq!(payload.server_id, "alpha-id");
    }
}
