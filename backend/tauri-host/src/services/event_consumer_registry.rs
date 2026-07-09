#[allow(unused_imports)]
pub use event::{
    EventConsumerRegistryAppFilterDto, EventConsumerRegistryEntryDto,
    EventConsumerRegistryFilterUpdateRequest, EventConsumerRegistryMetadataDto,
    EventConsumerRegistryMetadataUpdateRequest, EventConsumerRegistryServerFilterDto,
};

pub struct EventConsumerRegistryService;

impl EventConsumerRegistryService {
    pub fn new() -> Self {
        Self
    }

    fn registry(&self) -> event::EventConsumerRegistry<'static> {
        event::EventConsumerRegistry::new(crate::services::global::event_manager())
    }

    pub fn list(&self) -> Vec<EventConsumerRegistryEntryDto> {
        self.registry().list()
    }

    pub fn get(&self, name: &str) -> Option<EventConsumerRegistryEntryDto> {
        self.registry().get(name)
    }

    pub fn set_enabled(
        &self,
        name: &str,
        enabled: bool,
    ) -> Result<EventConsumerRegistryEntryDto, String> {
        self.registry().set_enabled(name, enabled)
    }

    pub fn update_filters(
        &self,
        name: &str,
        request: EventConsumerRegistryFilterUpdateRequest,
    ) -> Result<EventConsumerRegistryEntryDto, String> {
        self.registry().update_filters(name, request)
    }

    pub fn update_metadata(
        &self,
        name: &str,
        request: EventConsumerRegistryMetadataUpdateRequest,
    ) -> Result<EventConsumerRegistryEntryDto, String> {
        self.registry().update_metadata(name, request)
    }
}

impl Default for EventConsumerRegistryService {
    fn default() -> Self {
        Self::new()
    }
}
