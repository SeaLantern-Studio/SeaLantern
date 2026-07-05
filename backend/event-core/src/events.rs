use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// High-level scope used to separate app-wide events from per-server events.
pub enum EventScope {
    App,
    Server,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// Lifecycle of app-level operations emitted through the shared event bus.
pub enum AppEventKind {
    OperationRequested,
    OperationSucceeded,
    OperationFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Envelope delivered to app-event subscribers and persisted in the recent-event buffer.
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
/// App-event payload variants carried inside [`AppEventEnvelope`].
pub enum AppEventPayload {
    Operation {
        action: String,
        detail: Option<String>,
        error: Option<String>,
    },
}

/// Callback signature for app-scoped event subscribers.
pub type AppEventSubscriber = dyn Fn(&AppEventEnvelope) -> Result<(), String> + Send + Sync;

#[derive(Clone)]
/// Combined registration payload for consumers that may listen to server events, app events, or both.
pub struct EventConsumer {
    pub server_events: Option<Arc<ServerEventSubscriber>>,
    pub server_filter: Option<ServerEventSubscription>,
    pub app_events: Option<Arc<AppEventSubscriber>>,
    pub app_filter: Option<AppEventSubscription>,
}

impl EventConsumer {
    /// Creates a consumer with optional server and app callbacks.
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

    /// Creates a consumer that only subscribes to server events.
    pub fn server(server_events: Arc<ServerEventSubscriber>) -> Self {
        Self::new(Some(server_events), None)
    }

    /// Creates a consumer that only subscribes to app events.
    pub fn app(app_events: Arc<AppEventSubscriber>) -> Self {
        Self::new(None, Some(app_events))
    }

    /// Creates a consumer that subscribes to both server and app event streams.
    pub fn both(
        server_events: Arc<ServerEventSubscriber>,
        app_events: Arc<AppEventSubscriber>,
    ) -> Self {
        Self::new(Some(server_events), Some(app_events))
    }

    /// Attaches a normalized server-event filter to the consumer.
    pub fn with_server_filter(mut self, filter: ServerEventSubscription) -> Self {
        self.server_filter = Some(filter.normalized());
        self
    }

    /// Attaches a normalized app-event filter to the consumer.
    pub fn with_app_filter(mut self, filter: AppEventSubscription) -> Self {
        self.app_filter = Some(filter.normalized());
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Subscription identifiers returned after registering a consumer.
pub struct EventConsumerRegistration {
    pub server_subscription_id: Option<u64>,
    pub app_subscription_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// Broad classification used for diagnostics and registry visibility.
pub enum EventConsumerKind {
    Internal,
    PluginRuntime,
    FrontendBridge,
    TransportAdapter,
    ProtocolAdapter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Human-readable metadata attached to a named consumer registration.
pub struct EventConsumerMetadata {
    pub kind: EventConsumerKind,
    pub owner: String,
    pub description: String,
}

impl EventConsumerMetadata {
    /// Creates metadata for a named consumer entry.
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
/// Snapshot of a named consumer as exposed by registry and inspection APIs.
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
/// Server runtime event families emitted by the backend event bus.
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
/// Source classification for server events.
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
    /// Formats the event source into the string representation used in envelopes.
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
/// Envelope delivered to server-event subscribers and recent-event buffers.
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
/// Server-event payload variants carried inside [`ServerEventEnvelope`].
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

/// Callback signature for server-scoped event subscribers.
pub type ServerEventSubscriber = dyn Fn(&ServerEventEnvelope) -> Result<(), String> + Send + Sync;

#[derive(Clone)]
struct SubscriberEntry {
    id: u64,
    callback: Arc<ServerEventSubscriber>,
}

#[derive(Clone)]
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

/// In-memory event bus that buffers recent events and manages named consumer registrations.
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
    /// Creates an event manager with bounded recent-event buffers.
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

    /// Publishes an app-scoped event, records it, and notifies subscribers.
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

    /// Publishes a server-scoped event, records it, and notifies subscribers.
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

    /// Registers a raw app-event subscriber and returns its subscription id.
    pub fn subscribe_app_events(&self, callback: Arc<AppEventSubscriber>) -> u64 {
        let id = next_subscriber_id();
        self.app_subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(AppSubscriberEntry { id, callback });
        id
    }

    /// Registers a raw server-event subscriber and returns its subscription id.
    pub fn subscribe_server_events(&self, callback: Arc<ServerEventSubscriber>) -> u64 {
        let id = next_subscriber_id();
        self.server_subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(SubscriberEntry { id, callback });
        id
    }

    /// Removes a previously registered server-event subscriber.
    pub fn unsubscribe_server_events(&self, subscriber_id: u64) {
        self.server_subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retain(|entry| entry.id != subscriber_id);
    }

    /// Removes a previously registered app-event subscriber.
    pub fn unsubscribe_app_events(&self, subscriber_id: u64) {
        self.app_subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retain(|entry| entry.id != subscriber_id);
    }

    /// Registers an anonymous consumer and returns the active subscription ids.
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

    /// Registers or replaces a named consumer using default metadata.
    pub fn register_named_consumer(
        &self,
        name: &str,
        consumer: EventConsumer,
    ) -> EventConsumerRegistration {
        self.register_named_consumer_with_metadata(name, consumer, EventConsumerMetadata::default())
    }

    /// Registers or replaces a named consumer with explicit metadata.
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

    /// Unregisters a named consumer and tears down its active subscriptions.
    pub fn unregister_named_consumer(&self, name: &str) -> Option<EventConsumerRegistration> {
        let managed = self
            .named_consumers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(name);
        if let Some(managed) = managed {
            if let Some(id) = managed.registration.server_subscription_id {
                self.unsubscribe_server_events(id);
            }
            if let Some(id) = managed.registration.app_subscription_id {
                self.unsubscribe_app_events(id);
            }
            Some(managed.registration)
        } else {
            None
        }
    }

    /// Enables or disables a named consumer by rebuilding its subscriptions.
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
        if let Some(id) = managed.registration.server_subscription_id {
            self.unsubscribe_server_events(id);
        }
        if let Some(id) = managed.registration.app_subscription_id {
            self.unsubscribe_app_events(id);
        }
        managed.registration = if enabled {
            self.register_consumer(managed.consumer.clone())
        } else {
            EventConsumerRegistration::default()
        };
        managed.enabled = enabled;
        Ok(())
    }

    /// Replaces the filters of a named consumer and reapplies subscriptions when enabled.
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
        if let Some(id) = managed.registration.server_subscription_id {
            self.unsubscribe_server_events(id);
        }
        if let Some(id) = managed.registration.app_subscription_id {
            self.unsubscribe_app_events(id);
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

    /// Updates the metadata attached to a named consumer.
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

    /// Returns all named consumer snapshots sorted by name.
    pub fn registered_consumers(&self) -> Vec<NamedEventConsumerState> {
        let mut registrations = self
            .named_consumers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
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

    /// Returns the snapshot for a single named consumer.
    pub fn registered_consumer(&self, name: &str) -> Option<NamedEventConsumerState> {
        self.registered_consumers()
            .into_iter()
            .find(|consumer| consumer.name == name)
    }

    /// Returns the newest server events up to `limit` entries.
    pub fn recent_server_events(&self, limit: Option<usize>) -> Vec<ServerEventEnvelope> {
        let events = self
            .recent_server_events
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let limit = limit.unwrap_or(events.len()).min(events.len());
        events[events.len().saturating_sub(limit)..].to_vec()
    }

    /// Returns the newest app events up to `limit` entries.
    pub fn recent_app_events(&self, limit: Option<usize>) -> Vec<AppEventEnvelope> {
        let events = self
            .recent_app_events
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
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        for entry in subscribers {
            let _ = (entry.callback)(event);
        }
    }

    fn notify_server_subscribers(&self, event: &ServerEventEnvelope) {
        let subscribers = self
            .server_subscribers
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        for entry in subscribers {
            let _ = (entry.callback)(event);
        }
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
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
/// Filter applied to app-event subscriptions.
pub struct AppEventSubscription {
    pub actions: Vec<String>,
    pub kinds: Vec<String>,
    pub sources: Vec<String>,
}

impl AppEventSubscription {
    /// Lowercases comparison fields so matching stays case-insensitive.
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

    /// Returns whether the filter accepts `event`.
    pub fn matches(&self, event: &AppEventEnvelope) -> bool {
        (self.actions.is_empty()
            || self
                .actions
                .iter()
                .any(|action| action == &event.action.to_ascii_lowercase()))
            && (self.kinds.is_empty()
                || self
                    .kinds
                    .iter()
                    .any(|kind| kind == app_event_kind_key(&event.kind)))
            && (self.sources.is_empty()
                || self
                    .sources
                    .iter()
                    .any(|source| source == &event.source.to_ascii_lowercase()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
/// Plugin-manifest representation of server-event subscriptions.
pub struct PluginServerEventSubscription {
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub event_kinds: Vec<String>,
    #[serde(default)]
    pub server_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Runtime representation of server-event filters used by the event bus.
pub struct ServerEventSubscription {
    pub classes: Vec<String>,
    pub event_kinds: Vec<String>,
    pub server_ids: Vec<String>,
}

impl ServerEventSubscription {
    /// Lowercases class and event-kind selectors for stable matching.
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

    /// Returns whether the filter accepts `event`.
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

/// Converts plugin manifest subscriptions into normalized runtime filters.
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
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

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
        assert_eq!(registered[0].metadata.owner, "test-owner");
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

    #[test]
    fn server_subscribers_can_reenter_publish_without_deadlocking() {
        let manager = Arc::new(EventManager::new());
        let reentered = Arc::new(AtomicBool::new(false));
        let hits = Arc::new(AtomicUsize::new(0));

        manager.subscribe_server_events({
            let manager = Arc::clone(&manager);
            let reentered = Arc::clone(&reentered);
            let hits = Arc::clone(&hits);
            Arc::new(move |event| {
                hits.fetch_add(1, Ordering::SeqCst);
                if matches!(event.kind, ServerEventKind::LifecycleStarted)
                    && !reentered.swap(true, Ordering::SeqCst)
                {
                    manager.publish_server_event(
                        &event.server_id,
                        ServerEventSource::System,
                        None,
                        ServerEventKind::LifecycleStopped,
                        ServerEventPayload::Lifecycle {
                            detail: Some("reentered".to_string()),
                            error: None,
                            from_mode: None,
                            to_mode: None,
                        },
                    );
                }
                Ok(())
            })
        });

        manager.publish_server_event(
            "alpha",
            ServerEventSource::System,
            None,
            ServerEventKind::LifecycleStarted,
            ServerEventPayload::Lifecycle {
                detail: Some("initial".to_string()),
                error: None,
                from_mode: None,
                to_mode: None,
            },
        );

        assert!(reentered.load(Ordering::SeqCst));
        assert_eq!(hits.load(Ordering::SeqCst), 2);
        assert_eq!(manager.recent_server_events(None).len(), 2);
    }

    #[test]
    fn app_subscribers_can_reenter_publish_without_deadlocking() {
        let manager = Arc::new(EventManager::new());
        let reentered = Arc::new(AtomicBool::new(false));
        let hits = Arc::new(AtomicUsize::new(0));

        manager.subscribe_app_events({
            let manager = Arc::clone(&manager);
            let reentered = Arc::clone(&reentered);
            let hits = Arc::clone(&hits);
            Arc::new(move |event| {
                hits.fetch_add(1, Ordering::SeqCst);
                if matches!(event.kind, AppEventKind::OperationRequested)
                    && !reentered.swap(true, Ordering::SeqCst)
                {
                    manager.publish_app_event(
                        "reentered_action",
                        "test",
                        AppEventKind::OperationSucceeded,
                        AppEventPayload::Operation {
                            action: "reentered_action".to_string(),
                            detail: Some("reentered".to_string()),
                            error: None,
                        },
                    );
                }
                Ok(())
            })
        });

        manager.publish_app_event(
            "initial_action",
            "test",
            AppEventKind::OperationRequested,
            AppEventPayload::Operation {
                action: "initial_action".to_string(),
                detail: Some("initial".to_string()),
                error: None,
            },
        );

        assert!(reentered.load(Ordering::SeqCst));
        assert_eq!(hits.load(Ordering::SeqCst), 2);
    }
}
