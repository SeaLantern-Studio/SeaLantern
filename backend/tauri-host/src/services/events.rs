#[allow(unused_imports)]
pub use sea_lantern_event_core::{
    plugin_server_event_subscriptions_map, AppEventEnvelope, AppEventKind, AppEventPayload,
    AppEventSubscriber, AppEventSubscription, EventConsumer, EventConsumerKind,
    EventConsumerMetadata, EventConsumerRegistration, EventManager, EventScope,
    NamedEventConsumerState, PluginServerEventSubscription, ServerEventEnvelope, ServerEventKind,
    ServerEventPayload, ServerEventSource, ServerEventSubscriber, ServerEventSubscription,
};

pub fn publish_server_output_raw(
    server_id: &str,
    source: ServerEventSource,
    line: &str,
    stream: &str,
) -> ServerEventEnvelope {
    crate::services::global::event_manager().publish_server_event(
        server_id,
        source,
        None,
        ServerEventKind::OutputRawLine,
        ServerEventPayload::RawLine {
            line: line.to_string(),
            stream: stream.to_string(),
        },
    )
}

pub fn publish_app_operation_requested(action: &str, detail: Option<String>) -> AppEventEnvelope {
    crate::services::global::event_manager().publish_app_event(
        action,
        "frontend_user",
        AppEventKind::OperationRequested,
        AppEventPayload::Operation {
            action: action.to_string(),
            detail,
            error: None,
        },
    )
}

pub fn publish_app_operation_result(
    action: &str,
    detail: Option<String>,
    error: Option<String>,
) -> AppEventEnvelope {
    let success = error.is_none();
    crate::services::global::event_manager().publish_app_event(
        action,
        "frontend_user",
        if success {
            AppEventKind::OperationSucceeded
        } else {
            AppEventKind::OperationFailed
        },
        AppEventPayload::Operation {
            action: action.to_string(),
            detail,
            error,
        },
    )
}

pub fn publish_server_output_structured(
    server_id: &str,
    source: ServerEventSource,
    line: &str,
    stream: &str,
    event_kind: Option<String>,
    player: Option<String>,
    message: Option<String>,
) -> ServerEventEnvelope {
    crate::services::global::event_manager().publish_server_event(
        server_id,
        source,
        None,
        ServerEventKind::OutputStructuredLog,
        ServerEventPayload::StructuredLog {
            line: line.to_string(),
            stream: stream.to_string(),
            event_kind,
            player,
            message,
        },
    )
}

pub fn publish_server_command_requested(
    server_id: &str,
    source: ServerEventSource,
    plugin_id: Option<&str>,
    command: &str,
) -> ServerEventEnvelope {
    crate::services::global::event_manager().publish_server_event(
        server_id,
        source.clone(),
        plugin_id,
        ServerEventKind::CommandSendRequested,
        ServerEventPayload::Command {
            command: command.to_string(),
            success: None,
            error: None,
            actor: source.as_str(plugin_id),
        },
    )
}

pub fn publish_server_command_result(
    server_id: &str,
    source: ServerEventSource,
    plugin_id: Option<&str>,
    command: &str,
    success: bool,
    error: Option<String>,
) -> ServerEventEnvelope {
    crate::services::global::event_manager().publish_server_event(
        server_id,
        source.clone(),
        plugin_id,
        if success {
            ServerEventKind::CommandSendSucceeded
        } else {
            ServerEventKind::CommandSendFailed
        },
        ServerEventPayload::Command {
            command: command.to_string(),
            success: Some(success),
            error,
            actor: source.as_str(plugin_id),
        },
    )
}

pub fn publish_server_lifecycle(
    server_id: &str,
    kind: ServerEventKind,
    detail: Option<String>,
    error: Option<String>,
    from_mode: Option<String>,
    to_mode: Option<String>,
) -> ServerEventEnvelope {
    crate::services::global::event_manager().publish_server_event(
        server_id,
        ServerEventSource::RuntimeManager,
        None,
        kind,
        ServerEventPayload::Lifecycle { detail, error, from_mode, to_mode },
    )
}
