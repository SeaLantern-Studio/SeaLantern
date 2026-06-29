use crate::models::settings::{OneBot11Settings, OneBotTargetType};
use crate::services::events::{ServerEventEnvelope, ServerEventKind, ServerEventPayload};
use crate::utils::logger::{log_info_ctx, log_warn_ctx};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Serialize;

fn class_matches(settings: &OneBot11Settings, event: &ServerEventEnvelope) -> bool {
    if settings.event_classes.is_empty() {
        return true;
    }

    settings
        .event_classes
        .iter()
        .any(|class| match class.as_str() {
            "output" => matches!(
                event.kind,
                ServerEventKind::OutputRawLine | ServerEventKind::OutputStructuredLog
            ),
            "command" => matches!(
                event.kind,
                ServerEventKind::CommandSendRequested
                    | ServerEventKind::CommandSendSucceeded
                    | ServerEventKind::CommandSendFailed
            ),
            "lifecycle" => matches!(
                event.kind,
                ServerEventKind::LifecycleStartRequested
                    | ServerEventKind::LifecycleStartSkippedExistingState
                    | ServerEventKind::LifecycleStartFallback
                    | ServerEventKind::LifecycleStarted
                    | ServerEventKind::LifecycleStopRequested
                    | ServerEventKind::LifecycleStopRequestedAsync
                    | ServerEventKind::LifecycleStopped
                    | ServerEventKind::LifecycleRuntimeError
            ),
            _ => false,
        })
}

fn structured_kind_matches(settings: &OneBot11Settings, event: &ServerEventEnvelope) -> bool {
    if settings.structured_event_kinds.is_empty() {
        return true;
    }

    match &event.payload {
        ServerEventPayload::StructuredLog { event_kind, .. } => {
            event_kind.as_ref().is_some_and(|kind| {
                settings
                    .structured_event_kinds
                    .iter()
                    .any(|item| item == kind)
            })
        }
        _ => true,
    }
}

fn server_matches(settings: &OneBot11Settings, event: &ServerEventEnvelope) -> bool {
    settings.server_ids.is_empty() || settings.server_ids.iter().any(|id| id == &event.server_id)
}

fn should_send(settings: &OneBot11Settings, event: &ServerEventEnvelope) -> bool {
    settings.enabled
        && !settings.api_base_url.trim().is_empty()
        && !settings.targets.is_empty()
        && class_matches(settings, event)
        && structured_kind_matches(settings, event)
        && server_matches(settings, event)
}

fn event_summary(event: &ServerEventEnvelope) -> String {
    match &event.payload {
        ServerEventPayload::RawLine { line, .. } => line.clone(),
        ServerEventPayload::StructuredLog { event_kind, player, message, line, .. } => {
            match event_kind.as_deref() {
                Some("player_join") => format!("{} joined", player.as_deref().unwrap_or("player")),
                Some("player_leave") => format!("{} left", player.as_deref().unwrap_or("player")),
                Some("chat") => format!(
                    "{}: {}",
                    player.as_deref().unwrap_or("player"),
                    message.as_deref().unwrap_or("")
                ),
                Some("server_ready") => "server ready".to_string(),
                Some("error") => message.clone().unwrap_or_else(|| line.clone()),
                _ => line.clone(),
            }
        }
        ServerEventPayload::Command { command, success, error, actor } => match success {
            Some(true) => format!("{} sent command: {}", actor, command),
            Some(false) => format!(
                "{} command failed: {} ({})",
                actor,
                command,
                error.clone().unwrap_or_default()
            ),
            None => format!("{} requested command: {}", actor, command),
        },
        ServerEventPayload::Lifecycle { detail, error, from_mode, to_mode } => {
            if let (Some(from_mode), Some(to_mode)) = (from_mode, to_mode) {
                return format!(
                    "{} ({})",
                    detail.clone().unwrap_or_else(|| "lifecycle".to_string()),
                    [from_mode.as_str(), to_mode.as_str()].join(" -> ")
                );
            }
            error
                .clone()
                .or_else(|| detail.clone())
                .unwrap_or_else(|| "lifecycle event".to_string())
        }
    }
}

fn render_message(settings: &OneBot11Settings, event: &ServerEventEnvelope) -> String {
    let summary = event_summary(event);
    settings
        .message_template
        .replace("{server_id}", &event.server_id)
        .replace("{kind}", &format!("{:?}", event.kind).to_ascii_lowercase())
        .replace("{source}", &event.source)
        .replace("{summary}", &summary)
}

#[derive(Serialize)]
struct SendMsgRequest<'a> {
    message_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_id: Option<i64>,
    message: String,
    auto_escape: bool,
}

fn parse_target_id(value: &str) -> Option<i64> {
    value.trim().parse::<i64>().ok()
}

fn normalized_api_url(base: &str) -> String {
    format!("{}/send_msg", base.trim_end_matches('/'))
}

fn send_via_http(settings: OneBot11Settings, event: ServerEventEnvelope) {
    let client = reqwest::blocking::Client::new();
    let url = normalized_api_url(&settings.api_base_url);
    let message = render_message(&settings, &event);

    for target in &settings.targets {
        let parsed_id = match parse_target_id(&target.id) {
            Some(id) => id,
            None => {
                log_warn_ctx(
                    "services.online.onebot",
                    "send_via_http",
                    &format!("invalid target id: {}", target.id),
                );
                continue;
            }
        };

        let request = match target.target_type {
            OneBotTargetType::Group => SendMsgRequest {
                message_type: "group",
                user_id: None,
                group_id: Some(parsed_id),
                message: message.clone(),
                auto_escape: false,
            },
            OneBotTargetType::Private => SendMsgRequest {
                message_type: "private",
                user_id: Some(parsed_id),
                group_id: None,
                message: message.clone(),
                auto_escape: false,
            },
        };

        let mut builder = client.post(&url).header(CONTENT_TYPE, "application/json");
        if !settings.access_token.trim().is_empty() {
            builder =
                builder.header(AUTHORIZATION, format!("Bearer {}", settings.access_token.trim()));
        }

        match builder.json(&request).send() {
            Ok(response) if response.status().is_success() => {
                log_info_ctx(
                    "services.online.onebot",
                    "send_via_http",
                    &format!(
                        "sent kind={:?} server_id={} target={}",
                        event.kind, event.server_id, target.id
                    ),
                );
            }
            Ok(response) => {
                log_warn_ctx(
                    "services.online.onebot",
                    "send_via_http",
                    &format!(
                        "send failed kind={:?} server_id={} target={} status={}",
                        event.kind,
                        event.server_id,
                        target.id,
                        response.status()
                    ),
                );
            }
            Err(error) => {
                log_warn_ctx(
                    "services.online.onebot",
                    "send_via_http",
                    &format!(
                        "send errored kind={:?} server_id={} target={} error={}",
                        event.kind, event.server_id, target.id, error
                    ),
                );
            }
        }
    }
}

pub fn handle_server_event(event: &ServerEventEnvelope) {
    let settings = crate::services::global::settings_manager().get().onebot_11;
    if !should_send(&settings, event) {
        return;
    }

    let event = event.clone();
    std::thread::spawn(move || send_via_http(settings, event));
}
