use crate::{
    AppEventSubscription, EventConsumerKind, EventConsumerMetadata, EventManager,
    NamedEventConsumerState, ServerEventSubscription,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventConsumerRegistryMetadataDto {
    pub kind: EventConsumerKind,
    pub owner: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventConsumerRegistryServerFilterDto {
    pub classes: Vec<String>,
    pub event_kinds: Vec<String>,
    pub server_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventConsumerRegistryAppFilterDto {
    pub actions: Vec<String>,
    pub kinds: Vec<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventConsumerRegistryEntryDto {
    pub name: String,
    pub enabled: bool,
    pub metadata: EventConsumerRegistryMetadataDto,
    pub server_subscription_id: Option<u64>,
    pub app_subscription_id: Option<u64>,
    pub server_filter: Option<EventConsumerRegistryServerFilterDto>,
    pub app_filter: Option<EventConsumerRegistryAppFilterDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventConsumerRegistryFilterUpdateRequest {
    pub server_filter: Option<EventConsumerRegistryServerFilterDto>,
    pub app_filter: Option<EventConsumerRegistryAppFilterDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventConsumerRegistryMetadataUpdateRequest {
    pub kind: EventConsumerKind,
    pub owner: String,
    pub description: String,
}

impl From<EventConsumerMetadata> for EventConsumerRegistryMetadataDto {
    fn from(value: EventConsumerMetadata) -> Self {
        Self {
            kind: value.kind,
            owner: value.owner,
            description: value.description,
        }
    }
}

impl From<EventConsumerRegistryMetadataUpdateRequest> for EventConsumerMetadata {
    fn from(value: EventConsumerRegistryMetadataUpdateRequest) -> Self {
        Self {
            kind: value.kind,
            owner: value.owner,
            description: value.description,
        }
    }
}

impl From<ServerEventSubscription> for EventConsumerRegistryServerFilterDto {
    fn from(value: ServerEventSubscription) -> Self {
        Self {
            classes: value.classes,
            event_kinds: value.event_kinds,
            server_ids: value.server_ids,
        }
    }
}

impl From<EventConsumerRegistryServerFilterDto> for ServerEventSubscription {
    fn from(value: EventConsumerRegistryServerFilterDto) -> Self {
        Self {
            classes: value.classes,
            event_kinds: value.event_kinds,
            server_ids: value.server_ids,
        }
    }
}

impl From<AppEventSubscription> for EventConsumerRegistryAppFilterDto {
    fn from(value: AppEventSubscription) -> Self {
        Self {
            actions: value.actions,
            kinds: value.kinds,
            sources: value.sources,
        }
    }
}

impl From<EventConsumerRegistryAppFilterDto> for AppEventSubscription {
    fn from(value: EventConsumerRegistryAppFilterDto) -> Self {
        Self {
            actions: value.actions,
            kinds: value.kinds,
            sources: value.sources,
        }
    }
}

impl From<NamedEventConsumerState> for EventConsumerRegistryEntryDto {
    fn from(value: NamedEventConsumerState) -> Self {
        Self {
            name: value.name,
            enabled: value.enabled,
            metadata: value.metadata.into(),
            server_subscription_id: value.server_subscription_id,
            app_subscription_id: value.app_subscription_id,
            server_filter: value.server_filter.map(Into::into),
            app_filter: value.app_filter.map(Into::into),
        }
    }
}

pub struct EventConsumerRegistry<'a> {
    manager: &'a EventManager,
}

impl<'a> EventConsumerRegistry<'a> {
    pub fn new(manager: &'a EventManager) -> Self {
        Self { manager }
    }

    pub fn list(&self) -> Vec<EventConsumerRegistryEntryDto> {
        self.manager
            .registered_consumers()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn get(&self, name: &str) -> Option<EventConsumerRegistryEntryDto> {
        self.manager.registered_consumer(name).map(Into::into)
    }

    pub fn set_enabled(
        &self,
        name: &str,
        enabled: bool,
    ) -> Result<EventConsumerRegistryEntryDto, String> {
        self.manager.set_named_consumer_enabled(name, enabled)?;
        self.get(name)
            .ok_or_else(|| format!("event consumer '{}' not found after enable update", name))
    }

    pub fn update_filters(
        &self,
        name: &str,
        request: EventConsumerRegistryFilterUpdateRequest,
    ) -> Result<EventConsumerRegistryEntryDto, String> {
        self.manager.update_named_consumer_filters(
            name,
            request.server_filter.map(Into::into),
            request.app_filter.map(Into::into),
        )?;
        self.get(name)
            .ok_or_else(|| format!("event consumer '{}' not found after filter update", name))
    }

    pub fn update_metadata(
        &self,
        name: &str,
        request: EventConsumerRegistryMetadataUpdateRequest,
    ) -> Result<EventConsumerRegistryEntryDto, String> {
        self.manager
            .update_named_consumer_metadata(name, request.into())?;
        self.get(name)
            .ok_or_else(|| format!("event consumer '{}' not found after metadata update", name))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EventConsumerRegistry, EventConsumerRegistryAppFilterDto,
        EventConsumerRegistryFilterUpdateRequest, EventConsumerRegistryMetadataUpdateRequest,
        EventConsumerRegistryServerFilterDto,
    };
    use crate::{
        EventConsumer, EventConsumerKind, EventConsumerMetadata, EventManager,
        ServerEventSubscription,
    };

    #[test]
    fn registry_can_read_and_update_named_consumer_state() {
        let manager = EventManager::new();
        manager.register_named_consumer_with_metadata(
            "test.registry.service",
            EventConsumer::server(std::sync::Arc::new(move |_event| Ok(()))).with_server_filter(
                ServerEventSubscription {
                    classes: vec!["command".to_string()],
                    event_kinds: vec!["command_send_requested".to_string()],
                    server_ids: vec!["alpha".to_string()],
                },
            ),
            EventConsumerMetadata::new(
                EventConsumerKind::Internal,
                "tests",
                "registry service test",
            ),
        );

        let service = EventConsumerRegistry::new(&manager);
        assert_eq!(
            service
                .get("test.registry.service")
                .expect("entry")
                .metadata
                .owner,
            "tests"
        );

        service
            .set_enabled("test.registry.service", false)
            .expect("disable");
        service
            .update_filters(
                "test.registry.service",
                EventConsumerRegistryFilterUpdateRequest {
                    server_filter: Some(EventConsumerRegistryServerFilterDto {
                        classes: vec!["lifecycle".to_string()],
                        event_kinds: vec!["lifecycle_started".to_string()],
                        server_ids: vec!["beta".to_string()],
                    }),
                    app_filter: Some(EventConsumerRegistryAppFilterDto {
                        actions: vec!["create_server".to_string()],
                        kinds: vec!["operation_requested".to_string()],
                        sources: vec!["frontend_user".to_string()],
                    }),
                },
            )
            .expect("filters");
        let entry = service
            .update_metadata(
                "test.registry.service",
                EventConsumerRegistryMetadataUpdateRequest {
                    kind: EventConsumerKind::TransportAdapter,
                    owner: "updated-owner".to_string(),
                    description: "updated description".to_string(),
                },
            )
            .expect("metadata");
        assert_eq!(entry.metadata.owner, "updated-owner");
    }
}
