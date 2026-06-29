use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::utils::logger::log_warn_ctx;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventScope {
    App,
    Server,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppEventKind {
    OperationRequested,
    OperationSucceeded,
    OperationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppEventEnvelope {
    pub event_id: String,
    pub occurred_at: u64,
    pub scope: EventScope,
    pub action: String,
    pub source: String,
    pub kind: AppEventKind,
    pub payload: AppEventPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppEventPayload {
    Operation {
        action: String,
        detail: Option<String>,
        error: Option<String>,
    },
}

pub type AppEventSubscriber = dyn Fn(&AppEventEnvelope) -> Result<(), String> + Send + Sync;

#[derive(Clone)]
pub struct EventConsumer {
    pub server_events: Option<Arc<ServerEventSubscriber>>,
    pub server_filter: Option<ServerEventSubscription>,
    pub app_events: Option<Arc<AppEventSubscriber>>,
    pub app_filter: Option<AppEventSubscription>,
}

impl EventConsumer {
    pub fn new(
        server_events: Option<Arc<ServerEventSubscriber>>,
        app_events: Option<Arc<AppEventSubscriber>>,
    ) -> Self {
        Self {
            server_events,
            server_filter: None,
            app_events,
            app_filter: None,
        }
    }

    pub fn server(server_events: Arc<ServerEventSubscriber>) -> Self {
        Self::new(Some(server_events), None)
    }

    #[allow(dead_code)]
    pub fn app(app_events: Arc<AppEventSubscriber>) -> Self {
        Self::new(None, Some(app_events))
    }

    pub fn both(
        server_events: Arc<ServerEventSubscriber>,
        app_events: Arc<AppEventSubscriber>,
    ) -> Self {
        Self::new(Some(server_events), Some(app_events))
    }

    pub fn with_server_filter(mut self, filter: ServerEventSubscription) -> Self {
        self.server_filter = Some(filter.normalized());
        self
    }

    pub fn with_app_filter(mut self, filter: AppEventSubscription) -> Self {
        self.app_filter = Some(filter.normalized());
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EventConsumerRegistration {
    pub server_subscription_id: Option<u64>,
    pub app_subscription_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventConsumerKind {
    Internal,
    PluginRuntime,
    FrontendBridge,
    TransportAdapter,
    ProtocolAdapter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventConsumerMetadata {
    pub kind: EventConsumerKind,
    pub owner: String,
    pub description: String,
}

impl EventConsumerMetadata {
    pub fn new(
        kind: EventConsumerKind,
        owner: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            owner: owner.into(),
            description: description.into(),
        }
    }
}

impl Default for EventConsumerMetadata {
    fn default() -> Self {
        Self {
            kind: EventConsumerKind::Internal,
            owner: "unknown".to_string(),
            description: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct NamedEventConsumerState {
    pub name: String,
    pub enabled: bool,
    pub metadata: EventConsumerMetadata,
    pub server_subscription_id: Option<u64>,
    pub app_subscription_id: Option<u64>,
    pub server_filter: Option<ServerEventSubscription>,
    pub app_filter: Option<AppEventSubscription>,
}

#[derive(Clone)]
struct ManagedEventConsumer {
    consumer: EventConsumer,
    enabled: bool,
    metadata: EventConsumerMetadata,
    registration: EventConsumerRegistration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerEventKind {
    OutputRawLine,
    OutputStructuredLog,
    CommandSendRequested,
    CommandSendSucceeded,
    CommandSendFailed,
    LifecycleStartRequested,
    LifecycleStartSkippedExistingState,
    LifecycleStartFallback,
    LifecycleStarted,
    LifecycleStopRequested,
    LifecycleStopRequestedAsync,
    LifecycleStopped,
    LifecycleRuntimeError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerEventSource {
    RuntimeStdout,
    RuntimeStderr,
    RuntimeUnknown,
    FrontendUser,
    Plugin,
    System,
    RuntimeManager,
}

impl ServerEventSource {
    pub fn as_str(&self, plugin_id: Option<&str>) -> String {
        match self {
            Self::RuntimeStdout => "runtime_stdout".to_string(),
            Self::RuntimeStderr => "runtime_stderr".to_string(),
            Self::RuntimeUnknown => "runtime_unknown".to_string(),
            Self::FrontendUser => "frontend_user".to_string(),
            Self::Plugin => plugin_id
                .map(|id| format!("plugin:{}", id))
                .unwrap_or_else(|| "plugin".to_string()),
            Self::System => "system".to_string(),
            Self::RuntimeManager => "runtime_manager".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerEventEnvelope {
    pub event_id: String,
    pub occurred_at: u64,
    pub scope: EventScope,
    pub server_id: String,
    pub source: String,
    pub kind: ServerEventKind,
    pub payload: ServerEventPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEventPayload {
    RawLine {
        line: String,
        stream: String,
    },
    StructuredLog {
        line: String,
        stream: String,
        event_kind: Option<String>,
        player: Option<String>,
        message: Option<String>,
    },
    Command {
        command: String,
        success: Option<bool>,
        error: Option<String>,
        actor: String,
    },
    Lifecycle {
        detail: Option<String>,
        error: Option<String>,
        from_mode: Option<String>,
        to_mode: Option<String>,
    },
}

pub type ServerEventSubscriber = dyn Fn(&ServerEventEnvelope) -> Result<(), String> + Send + Sync;

struct SubscriberEntry {
    id: u64,
    callback: Arc<ServerEventSubscriber>,
}

struct AppSubscriberEntry {
    id: u64,
    callback: Arc<AppEventSubscriber>,
}

fn next_event_counter() -> &'static AtomicU64 {
    static COUNTER: OnceLock<AtomicU64> = OnceLock::new();
    COUNTER.get_or_init(|| AtomicU64::new(1))
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn next_event_id() -> String {
    format!("server-event-{}", next_event_counter().fetch_add(1, Ordering::Relaxed))
}

fn next_subscriber_id() -> u64 {
    static COUNTER: OnceLock<AtomicU64> = OnceLock::new();
    COUNTER
        .get_or_init(|| AtomicU64::new(1))
        .fetch_add(1, Ordering::Relaxed)
}

pub struct EventManager {
    named_consumers: Mutex<HashMap<String, ManagedEventConsumer>>,
    recent_app_events: Mutex<Vec<AppEventEnvelope>>,
    app_subscribers: Mutex<Vec<AppSubscriberEntry>>,
    recent_server_events: Mutex<Vec<ServerEventEnvelope>>,
    server_subscribers: Mutex<Vec<SubscriberEntry>>,
    max_recent_app_events: usize,
    max_recent_server_events: usize,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            named_consumers: Mutex::new(HashMap::new()),
            recent_app_events: Mutex::new(Vec::new()),
            app_subscribers: Mutex::new(Vec::new()),
            recent_server_events: Mutex::new(Vec::new()),
            server_subscribers: Mutex::new(Vec::new()),
            max_recent_app_events: 512,
            max_recent_server_events: 512,
        }
    }

    pub fn publish_app_event(
        &self,
        action: &str,
        source: &str,
        kind: AppEventKind,
        payload: AppEventPayload,
    ) -> AppEventEnvelope {
        let event = AppEventEnvelope {
            event_id: next_event_id(),
            occurred_at: now_millis(),
            scope: EventScope::App,
            action: action.to_string(),
            source: source.to_string(),
            kind,
            payload,
        };

        self.buffer_app_event(event.clone());
        self.notify_app_subscribers(&event);
        event
    }

    pub fn subscribe_app_events(&self, callback: Arc<AppEventSubscriber>) -> u64 {
        let id = next_subscriber_id();
        let mut subscribers = self
            .app_subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        subscribers.push(AppSubscriberEntry { id, callback });
        id
    }

    pub fn register_consumer(&self, consumer: EventConsumer) -> EventConsumerRegistration {
        let server_filter = consumer.server_filter.clone();
        let app_filter = consumer.app_filter.clone();

        EventConsumerRegistration {
            server_subscription_id: consumer.server_events.map(|callback| {
                if let Some(filter) = server_filter {
                    self.subscribe_server_events(Arc::new(move |event| {
                        if filter.matches(event) {
                            callback(event)
                        } else {
                            Ok(())
                        }
                    }))
                } else {
                    self.subscribe_server_events(callback)
                }
            }),
            app_subscription_id: consumer.app_events.map(|callback| {
                if let Some(filter) = app_filter {
                    self.subscribe_app_events(Arc::new(move |event| {
                        if filter.matches(event) {
                            callback(event)
                        } else {
                            Ok(())
                        }
                    }))
                } else {
                    self.subscribe_app_events(callback)
                }
            }),
        }
    }

    #[allow(dead_code)]
    pub fn register_named_consumer(
        &self,
        name: &str,
        consumer: EventConsumer,
    ) -> EventConsumerRegistration {
        self.register_named_consumer_with_metadata(name, consumer, EventConsumerMetadata::default())
    }

    pub fn register_named_consumer_with_metadata(
        &self,
        name: &str,
        consumer: EventConsumer,
        metadata: EventConsumerMetadata,
    ) -> EventConsumerRegistration {
        let _ = self.unregister_named_consumer(name);
        let registration = self.register_consumer(consumer.clone());

        self.named_consumers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(
                name.to_string(),
                ManagedEventConsumer {
                    consumer,
                    enabled: true,
                    metadata,
                    registration,
                },
            );

        registration
    }

    pub fn publish_server_event(
        &self,
        server_id: &str,
        source: ServerEventSource,
        plugin_id: Option<&str>,
        kind: ServerEventKind,
        payload: ServerEventPayload,
    ) -> ServerEventEnvelope {
        let event = ServerEventEnvelope {
            event_id: next_event_id(),
            occurred_at: now_millis(),
            scope: EventScope::Server,
            server_id: server_id.to_string(),
            source: source.as_str(plugin_id),
            kind,
            payload,
        };

        self.buffer_server_event(event.clone());
        self.notify_server_subscribers(&event);
        event
    }

    pub fn subscribe_server_events(&self, callback: Arc<ServerEventSubscriber>) -> u64 {
        let id = next_subscriber_id();
        let mut subscribers = self
            .server_subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        subscribers.push(SubscriberEntry { id, callback });
        id
    }

    #[allow(dead_code)]
    pub fn unsubscribe_server_events(&self, subscriber_id: u64) {
        let mut subscribers = self
            .server_subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        subscribers.retain(|entry| entry.id != subscriber_id);
    }

    #[allow(dead_code)]
    pub fn unsubscribe_app_events(&self, subscriber_id: u64) {
        let mut subscribers = self
            .app_subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        subscribers.retain(|entry| entry.id != subscriber_id);
    }

    pub fn unregister_named_consumer(&self, name: &str) -> Option<EventConsumerRegistration> {
        let managed = self
            .named_consumers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(name);

        if let Some(managed) = managed {
            let registration = managed.registration;
            if let Some(subscriber_id) = registration.server_subscription_id {
                self.unsubscribe_server_events(subscriber_id);
            }
            if let Some(subscriber_id) = registration.app_subscription_id {
                self.unsubscribe_app_events(subscriber_id);
            }
            Some(registration)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn set_named_consumer_enabled(&self, name: &str, enabled: bool) -> Result<(), String> {
        let mut consumers = self
            .named_consumers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let managed = consumers
            .get_mut(name)
            .ok_or_else(|| format!("event consumer '{}' not found", name))?;

        if managed.enabled == enabled {
            return Ok(());
        }

        if let Some(subscriber_id) = managed.registration.server_subscription_id {
            self.unsubscribe_server_events(subscriber_id);
        }
        if let Some(subscriber_id) = managed.registration.app_subscription_id {
            self.unsubscribe_app_events(subscriber_id);
        }

        managed.registration = if enabled {
            self.register_consumer(managed.consumer.clone())
        } else {
            EventConsumerRegistration::default()
        };
        managed.enabled = enabled;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn replace_named_consumer(
        &self,
        name: &str,
        consumer: EventConsumer,
    ) -> Result<EventConsumerRegistration, String> {
        self.replace_named_consumer_with_metadata(name, consumer, None)
    }

    #[allow(dead_code)]
    pub fn replace_named_consumer_with_metadata(
        &self,
        name: &str,
        consumer: EventConsumer,
        metadata: Option<EventConsumerMetadata>,
    ) -> Result<EventConsumerRegistration, String> {
        let mut consumers = self
            .named_consumers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let managed = consumers
            .get_mut(name)
            .ok_or_else(|| format!("event consumer '{}' not found", name))?;

        if let Some(subscriber_id) = managed.registration.server_subscription_id {
            self.unsubscribe_server_events(subscriber_id);
        }
        if let Some(subscriber_id) = managed.registration.app_subscription_id {
            self.unsubscribe_app_events(subscriber_id);
        }

        let registration = if managed.enabled {
            self.register_consumer(consumer.clone())
        } else {
            EventConsumerRegistration::default()
        };

        managed.consumer = consumer;
        if let Some(metadata) = metadata {
            managed.metadata = metadata;
        }
        managed.registration = registration;

        Ok(registration)
    }

    #[allow(dead_code)]
    pub fn update_named_consumer_filters(
        &self,
        name: &str,
        server_filter: Option<ServerEventSubscription>,
        app_filter: Option<AppEventSubscription>,
    ) -> Result<EventConsumerRegistration, String> {
        let mut consumers = self
            .named_consumers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let managed = consumers
            .get_mut(name)
            .ok_or_else(|| format!("event consumer '{}' not found", name))?;

        if let Some(subscriber_id) = managed.registration.server_subscription_id {
            self.unsubscribe_server_events(subscriber_id);
        }
        if let Some(subscriber_id) = managed.registration.app_subscription_id {
            self.unsubscribe_app_events(subscriber_id);
        }

        managed.consumer.server_filter = server_filter.map(|filter| filter.normalized());
        managed.consumer.app_filter = app_filter.map(|filter| filter.normalized());
        managed.registration = if managed.enabled {
            self.register_consumer(managed.consumer.clone())
        } else {
            EventConsumerRegistration::default()
        };

        Ok(managed.registration)
    }

    #[allow(dead_code)]
    pub fn registered_consumers(&self) -> Vec<NamedEventConsumerState> {
        let consumers = self
            .named_consumers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut registrations = consumers
            .iter()
            .map(|(name, managed)| NamedEventConsumerState {
                name: name.clone(),
                enabled: managed.enabled,
                metadata: managed.metadata.clone(),
                server_subscription_id: managed.registration.server_subscription_id,
                app_subscription_id: managed.registration.app_subscription_id,
                server_filter: managed.consumer.server_filter.clone(),
                app_filter: managed.consumer.app_filter.clone(),
            })
            .collect::<Vec<_>>();
        registrations.sort_by(|left, right| left.name.cmp(&right.name));
        registrations
    }

    #[allow(dead_code)]
    pub fn registered_consumer(&self, name: &str) -> Option<NamedEventConsumerState> {
        self.registered_consumers()
            .into_iter()
            .find(|consumer| consumer.name == name)
    }

    #[allow(dead_code)]
    pub fn update_named_consumer_metadata(
        &self,
        name: &str,
        metadata: EventConsumerMetadata,
    ) -> Result<(), String> {
        let mut consumers = self
            .named_consumers
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let managed = consumers
            .get_mut(name)
            .ok_or_else(|| format!("event consumer '{}' not found", name))?;

        managed.metadata = metadata;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn recent_server_events(&self, limit: Option<usize>) -> Vec<ServerEventEnvelope> {
        let events = self
            .recent_server_events
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let limit = limit.unwrap_or(events.len()).min(events.len());
        events[events.len().saturating_sub(limit)..].to_vec()
    }

    fn buffer_server_event(&self, event: ServerEventEnvelope) {
        let mut events = self
            .recent_server_events
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        events.push(event);
        if events.len() > self.max_recent_server_events {
            let overflow = events.len() - self.max_recent_server_events;
            events.drain(0..overflow);
        }
    }

    fn buffer_app_event(&self, event: AppEventEnvelope) {
        let mut events = self
            .recent_app_events
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        events.push(event);
        if events.len() > self.max_recent_app_events {
            let overflow = events.len() - self.max_recent_app_events;
            events.drain(0..overflow);
        }
    }

    fn notify_app_subscribers(&self, event: &AppEventEnvelope) {
        let subscribers = self
            .app_subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        for entry in &*subscribers {
            if let Err(error) = (entry.callback)(event) {
                log_warn_ctx(
                    "services.events",
                    "notify_app_subscribers",
                    &format!(
                        "subscriber_id={} event_id={} kind={:?} error={}",
                        entry.id, event.event_id, event.kind, error
                    ),
                );
            }
        }
    }

    fn notify_server_subscribers(&self, event: &ServerEventEnvelope) {
        let subscribers = self
            .server_subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        for entry in &*subscribers {
            if let Err(error) = (entry.callback)(event) {
                log_warn_ctx(
                    "services.events",
                    "notify_server_subscribers",
                    &format!(
                        "subscriber_id={} event_id={} kind={:?} error={}",
                        entry.id, event.event_id, event.kind, error
                    ),
                );
            }
        }
    }
}

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

fn app_event_kind_key(kind: &AppEventKind) -> &'static str {
    match kind {
        AppEventKind::OperationRequested => "operation_requested",
        AppEventKind::OperationSucceeded => "operation_succeeded",
        AppEventKind::OperationFailed => "operation_failed",
    }
}

fn server_event_kind_key(kind: &ServerEventKind) -> &'static str {
    match kind {
        ServerEventKind::OutputRawLine => "output_raw_line",
        ServerEventKind::OutputStructuredLog => "output_structured_log",
        ServerEventKind::CommandSendRequested => "command_send_requested",
        ServerEventKind::CommandSendSucceeded => "command_send_succeeded",
        ServerEventKind::CommandSendFailed => "command_send_failed",
        ServerEventKind::LifecycleStartRequested => "lifecycle_start_requested",
        ServerEventKind::LifecycleStartSkippedExistingState => {
            "lifecycle_start_skipped_existing_state"
        }
        ServerEventKind::LifecycleStartFallback => "lifecycle_start_fallback",
        ServerEventKind::LifecycleStarted => "lifecycle_started",
        ServerEventKind::LifecycleStopRequested => "lifecycle_stop_requested",
        ServerEventKind::LifecycleStopRequestedAsync => "lifecycle_stop_requested_async",
        ServerEventKind::LifecycleStopped => "lifecycle_stopped",
        ServerEventKind::LifecycleRuntimeError => "lifecycle_runtime_error",
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppEventSubscription {
    pub actions: Vec<String>,
    pub kinds: Vec<String>,
    pub sources: Vec<String>,
}

impl AppEventSubscription {
    pub fn normalized(&self) -> Self {
        Self {
            actions: self
                .actions
                .iter()
                .map(|item| item.to_ascii_lowercase())
                .collect(),
            kinds: self
                .kinds
                .iter()
                .map(|item| item.to_ascii_lowercase())
                .collect(),
            sources: self
                .sources
                .iter()
                .map(|item| item.to_ascii_lowercase())
                .collect(),
        }
    }

    pub fn matches(&self, event: &AppEventEnvelope) -> bool {
        if !self.actions.is_empty()
            && !self
                .actions
                .iter()
                .any(|action| action == &event.action.to_ascii_lowercase())
        {
            return false;
        }

        if !self.kinds.is_empty()
            && !self
                .kinds
                .iter()
                .any(|kind| kind == app_event_kind_key(&event.kind))
        {
            return false;
        }

        if !self.sources.is_empty()
            && !self
                .sources
                .iter()
                .any(|source| source == &event.source.to_ascii_lowercase())
        {
            return false;
        }

        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PluginServerEventSubscription {
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub event_kinds: Vec<String>,
    #[serde(default)]
    pub server_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerEventSubscription {
    pub classes: Vec<String>,
    pub event_kinds: Vec<String>,
    pub server_ids: Vec<String>,
}

impl ServerEventSubscription {
    pub fn normalized(&self) -> Self {
        Self {
            classes: self
                .classes
                .iter()
                .map(|item| item.to_ascii_lowercase())
                .collect(),
            event_kinds: self
                .event_kinds
                .iter()
                .map(|item| item.to_ascii_lowercase())
                .collect(),
            server_ids: self.server_ids.clone(),
        }
    }

    pub fn matches(&self, event: &ServerEventEnvelope) -> bool {
        let class_matches = if self.classes.is_empty() {
            true
        } else {
            self.classes.iter().any(|class| match class.as_str() {
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
        };

        if !class_matches {
            return false;
        }

        if !self.server_ids.is_empty() && !self.server_ids.iter().any(|id| id == &event.server_id) {
            return false;
        }

        if self.event_kinds.is_empty() {
            return true;
        }

        match &event.payload {
            ServerEventPayload::StructuredLog { event_kind, .. } => event_kind
                .as_ref()
                .is_some_and(|kind| self.event_kinds.iter().any(|item| item == kind)),
            _ => self
                .event_kinds
                .iter()
                .any(|item| item == server_event_kind_key(&event.kind)),
        }
    }
}

impl From<&PluginServerEventSubscription> for ServerEventSubscription {
    fn from(value: &PluginServerEventSubscription) -> Self {
        Self {
            classes: value.classes.clone(),
            event_kinds: value.event_kinds.clone(),
            server_ids: value.server_ids.clone(),
        }
        .normalized()
    }
}

pub fn plugin_server_event_subscriptions_map(
    manifest_subscriptions: &HashMap<String, PluginServerEventSubscription>,
) -> HashMap<String, ServerEventSubscription> {
    manifest_subscriptions
        .iter()
        .map(|(key, value)| (key.clone(), ServerEventSubscription::from(value)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        AppEventKind, AppEventPayload, AppEventSubscription, EventConsumer, EventConsumerKind,
        EventConsumerMetadata, EventManager, ServerEventKind, ServerEventPayload,
        ServerEventSource, ServerEventSubscription,
    };
    use std::sync::{Arc, Mutex};

    #[test]
    fn recent_server_events_keeps_tail_only() {
        let manager = EventManager {
            named_consumers: std::sync::Mutex::new(std::collections::HashMap::new()),
            recent_app_events: std::sync::Mutex::new(Vec::new()),
            app_subscribers: std::sync::Mutex::new(Vec::new()),
            recent_server_events: std::sync::Mutex::new(Vec::new()),
            server_subscribers: std::sync::Mutex::new(Vec::new()),
            max_recent_app_events: 2,
            max_recent_server_events: 2,
        };

        manager.publish_server_event(
            "alpha",
            ServerEventSource::System,
            None,
            ServerEventKind::LifecycleStarted,
            ServerEventPayload::Lifecycle {
                detail: Some("1".to_string()),
                error: None,
                from_mode: None,
                to_mode: None,
            },
        );
        manager.publish_server_event(
            "alpha",
            ServerEventSource::System,
            None,
            ServerEventKind::LifecycleStarted,
            ServerEventPayload::Lifecycle {
                detail: Some("2".to_string()),
                error: None,
                from_mode: None,
                to_mode: None,
            },
        );
        manager.publish_server_event(
            "alpha",
            ServerEventSource::System,
            None,
            ServerEventKind::LifecycleStarted,
            ServerEventPayload::Lifecycle {
                detail: Some("3".to_string()),
                error: None,
                from_mode: None,
                to_mode: None,
            },
        );

        let events = manager.recent_server_events(None);
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0].payload,
            ServerEventPayload::Lifecycle {
                detail: Some("2".to_string()),
                error: None,
                from_mode: None,
                to_mode: None,
            }
        );
    }

    #[test]
    fn register_consumer_subscribes_server_and_app_handlers_together() {
        let manager = EventManager::new();
        let server_hits = Arc::new(Mutex::new(Vec::new()));
        let app_hits = Arc::new(Mutex::new(Vec::new()));

        let registration = manager.register_consumer(EventConsumer::both(
            {
                let server_hits = Arc::clone(&server_hits);
                Arc::new(move |event| {
                    server_hits
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .push(event.event_id.clone());
                    Ok(())
                })
            },
            {
                let app_hits = Arc::clone(&app_hits);
                Arc::new(move |event| {
                    app_hits
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .push(event.event_id.clone());
                    Ok(())
                })
            },
        ));

        assert!(registration.server_subscription_id.is_some());
        assert!(registration.app_subscription_id.is_some());

        manager.publish_server_event(
            "alpha",
            ServerEventSource::System,
            None,
            ServerEventKind::LifecycleStarted,
            ServerEventPayload::Lifecycle {
                detail: Some("ready".to_string()),
                error: None,
                from_mode: None,
                to_mode: None,
            },
        );

        manager.publish_app_event(
            "create_server",
            "frontend_user",
            AppEventKind::OperationRequested,
            AppEventPayload::Operation {
                action: "create_server".to_string(),
                detail: Some("requested".to_string()),
                error: None,
            },
        );

        assert_eq!(server_hits.lock().unwrap_or_else(|e| e.into_inner()).len(), 1);
        assert_eq!(app_hits.lock().unwrap_or_else(|e| e.into_inner()).len(), 1);
    }

    #[test]
    fn consumer_filters_skip_non_matching_events() {
        let manager = EventManager::new();
        let server_hits = Arc::new(Mutex::new(0usize));
        let app_hits = Arc::new(Mutex::new(0usize));

        manager.register_consumer(
            EventConsumer::both(
                {
                    let server_hits = Arc::clone(&server_hits);
                    Arc::new(move |_event| {
                        *server_hits.lock().unwrap_or_else(|e| e.into_inner()) += 1;
                        Ok(())
                    })
                },
                {
                    let app_hits = Arc::clone(&app_hits);
                    Arc::new(move |_event| {
                        *app_hits.lock().unwrap_or_else(|e| e.into_inner()) += 1;
                        Ok(())
                    })
                },
            )
            .with_server_filter(ServerEventSubscription {
                classes: vec!["command".to_string()],
                event_kinds: vec!["command_send_requested".to_string()],
                server_ids: vec!["alpha".to_string()],
            })
            .with_app_filter(AppEventSubscription {
                actions: vec!["create_server".to_string()],
                kinds: vec!["operation_requested".to_string()],
                sources: vec!["frontend_user".to_string()],
            }),
        );

        manager.publish_server_event(
            "beta",
            ServerEventSource::System,
            None,
            ServerEventKind::LifecycleStarted,
            ServerEventPayload::Lifecycle {
                detail: None,
                error: None,
                from_mode: None,
                to_mode: None,
            },
        );
        manager.publish_server_event(
            "alpha",
            ServerEventSource::FrontendUser,
            None,
            ServerEventKind::CommandSendRequested,
            ServerEventPayload::Command {
                command: "say hi".to_string(),
                success: None,
                error: None,
                actor: "frontend_user".to_string(),
            },
        );

        manager.publish_app_event(
            "import_server",
            "frontend_user",
            AppEventKind::OperationRequested,
            AppEventPayload::Operation {
                action: "import_server".to_string(),
                detail: None,
                error: None,
            },
        );
        manager.publish_app_event(
            "create_server",
            "frontend_user",
            AppEventKind::OperationRequested,
            AppEventPayload::Operation {
                action: "create_server".to_string(),
                detail: None,
                error: None,
            },
        );

        assert_eq!(*server_hits.lock().unwrap_or_else(|e| e.into_inner()), 1);
        assert_eq!(*app_hits.lock().unwrap_or_else(|e| e.into_inner()), 1);
    }

    #[test]
    fn named_consumer_replaces_previous_registration_and_updates_registry() {
        let manager = EventManager::new();
        let first_hits = Arc::new(Mutex::new(0usize));
        let second_hits = Arc::new(Mutex::new(0usize));

        manager.register_named_consumer(
            "test.consumer",
            EventConsumer::server({
                let first_hits = Arc::clone(&first_hits);
                Arc::new(move |_event| {
                    *first_hits.lock().unwrap_or_else(|e| e.into_inner()) += 1;
                    Ok(())
                })
            }),
        );

        manager.register_named_consumer(
            "test.consumer",
            EventConsumer::server({
                let second_hits = Arc::clone(&second_hits);
                Arc::new(move |_event| {
                    *second_hits.lock().unwrap_or_else(|e| e.into_inner()) += 1;
                    Ok(())
                })
            }),
        );

        manager.publish_server_event(
            "alpha",
            ServerEventSource::System,
            None,
            ServerEventKind::LifecycleStarted,
            ServerEventPayload::Lifecycle {
                detail: Some("ready".to_string()),
                error: None,
                from_mode: None,
                to_mode: None,
            },
        );

        assert_eq!(*first_hits.lock().unwrap_or_else(|e| e.into_inner()), 0);
        assert_eq!(*second_hits.lock().unwrap_or_else(|e| e.into_inner()), 1);

        let registered = manager.registered_consumers();
        assert_eq!(registered.len(), 1);
        assert_eq!(registered[0].name, "test.consumer");
        assert!(registered[0].enabled);
        assert!(registered[0].server_subscription_id.is_some());

        let removed = manager.unregister_named_consumer("test.consumer");
        assert!(removed.is_some());
        assert!(manager.registered_consumers().is_empty());
    }

    #[test]
    fn named_consumer_can_be_disabled_and_reenabled() {
        let manager = EventManager::new();
        let hits = Arc::new(Mutex::new(0usize));

        manager.register_named_consumer(
            "toggle.consumer",
            EventConsumer::server({
                let hits = Arc::clone(&hits);
                Arc::new(move |_event| {
                    *hits.lock().unwrap_or_else(|e| e.into_inner()) += 1;
                    Ok(())
                })
            }),
        );

        manager
            .set_named_consumer_enabled("toggle.consumer", false)
            .expect("disable named consumer");
        manager.publish_server_event(
            "alpha",
            ServerEventSource::System,
            None,
            ServerEventKind::LifecycleStarted,
            ServerEventPayload::Lifecycle {
                detail: None,
                error: None,
                from_mode: None,
                to_mode: None,
            },
        );
        assert_eq!(*hits.lock().unwrap_or_else(|e| e.into_inner()), 0);

        let registered = manager.registered_consumers();
        assert_eq!(registered.len(), 1);
        assert!(!registered[0].enabled);
        assert!(registered[0].server_subscription_id.is_none());

        manager
            .set_named_consumer_enabled("toggle.consumer", true)
            .expect("reenable named consumer");
        manager.publish_server_event(
            "alpha",
            ServerEventSource::System,
            None,
            ServerEventKind::LifecycleStarted,
            ServerEventPayload::Lifecycle {
                detail: Some("up".to_string()),
                error: None,
                from_mode: None,
                to_mode: None,
            },
        );
        assert_eq!(*hits.lock().unwrap_or_else(|e| e.into_inner()), 1);
    }

    #[test]
    fn named_consumer_filter_can_be_replaced_dynamically() {
        let manager = EventManager::new();
        let hits = Arc::new(Mutex::new(Vec::new()));

        manager.register_named_consumer(
            "replace.consumer",
            EventConsumer::server({
                let hits = Arc::clone(&hits);
                Arc::new(move |event| {
                    hits.lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .push(event.server_id.clone());
                    Ok(())
                })
            })
            .with_server_filter(ServerEventSubscription {
                classes: vec!["command".to_string()],
                event_kinds: vec!["command_send_requested".to_string()],
                server_ids: vec!["alpha".to_string()],
            }),
        );

        manager.publish_server_event(
            "beta",
            ServerEventSource::FrontendUser,
            None,
            ServerEventKind::CommandSendRequested,
            ServerEventPayload::Command {
                command: "say beta".to_string(),
                success: None,
                error: None,
                actor: "frontend_user".to_string(),
            },
        );
        assert!(hits.lock().unwrap_or_else(|e| e.into_inner()).is_empty());

        manager
            .replace_named_consumer(
                "replace.consumer",
                EventConsumer::server({
                    let hits = Arc::clone(&hits);
                    Arc::new(move |event| {
                        hits.lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .push(format!("replaced:{}", event.server_id));
                        Ok(())
                    })
                })
                .with_server_filter(ServerEventSubscription {
                    classes: vec!["command".to_string()],
                    event_kinds: vec!["command_send_requested".to_string()],
                    server_ids: vec!["beta".to_string()],
                }),
            )
            .expect("replace named consumer");

        manager.publish_server_event(
            "beta",
            ServerEventSource::FrontendUser,
            None,
            ServerEventKind::CommandSendRequested,
            ServerEventPayload::Command {
                command: "say beta".to_string(),
                success: None,
                error: None,
                actor: "frontend_user".to_string(),
            },
        );

        let hits = hits.lock().unwrap_or_else(|e| e.into_inner()).clone();
        assert_eq!(hits, vec!["replaced:beta".to_string()]);

        let registered = manager.registered_consumers();
        assert_eq!(registered.len(), 1);
        assert_eq!(registered[0].name, "replace.consumer");
        assert_eq!(
            registered[0]
                .server_filter
                .as_ref()
                .expect("server filter")
                .server_ids,
            vec!["beta".to_string()]
        );
    }

    #[test]
    fn named_consumer_metadata_and_filter_updates_are_visible_in_registry() {
        let manager = EventManager::new();

        manager.register_named_consumer_with_metadata(
            "meta.consumer",
            EventConsumer::app(Arc::new(move |_event| Ok(()))).with_app_filter(
                AppEventSubscription {
                    actions: vec!["create_server".to_string()],
                    kinds: vec!["operation_requested".to_string()],
                    sources: vec!["frontend_user".to_string()],
                },
            ),
            EventConsumerMetadata::new(
                EventConsumerKind::TransportAdapter,
                "test-owner",
                "test description",
            ),
        );

        manager
            .update_named_consumer_filters(
                "meta.consumer",
                Some(ServerEventSubscription {
                    classes: vec!["command".to_string()],
                    event_kinds: vec!["command_send_requested".to_string()],
                    server_ids: vec!["alpha".to_string()],
                }),
                Some(AppEventSubscription {
                    actions: vec!["import_server".to_string()],
                    kinds: vec!["operation_succeeded".to_string()],
                    sources: vec!["frontend_user".to_string()],
                }),
            )
            .expect("update named consumer filters");

        let registered = manager.registered_consumers();
        assert_eq!(registered.len(), 1);
        assert_eq!(registered[0].name, "meta.consumer");
        assert_eq!(registered[0].metadata.kind, EventConsumerKind::TransportAdapter);
        assert_eq!(registered[0].metadata.owner, "test-owner");
        assert_eq!(registered[0].metadata.description, "test description");
        assert_eq!(
            registered[0]
                .server_filter
                .as_ref()
                .expect("server filter")
                .server_ids,
            vec!["alpha".to_string()]
        );
        assert_eq!(
            registered[0]
                .app_filter
                .as_ref()
                .expect("app filter")
                .actions,
            vec!["import_server".to_string()]
        );
    }
}
