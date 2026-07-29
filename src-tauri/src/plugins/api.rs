mod component_events;
mod component_mirror;
mod context_menu;
mod element_response;
mod handlers;
mod permission_logs;
mod registry;
mod sidebar;
mod ui_snapshot;

pub use component_events::{
    clear_plugin_component_snapshot, take_component_event_snapshot, BufferedComponentEvent,
};
pub use component_mirror::{
    component_mirror_clear, component_mirror_list, component_mirror_register,
    component_mirror_unregister, ComponentEntry,
};
pub use context_menu::{
    clear_plugin_context_menu_snapshot, take_context_menu_snapshot, BufferedContextMenuEvent,
};
pub use element_response::{element_response_create, element_response_resolve};
pub use handlers::{
    call_api, emit_component_event, emit_context_menu_event, emit_i18n_event, emit_log_event,
    emit_permission_log, emit_server_ready, emit_sidebar_event, emit_ui_event,
    register_server_log_processor, set_api_call_handler, set_component_event_handler,
    set_context_menu_handler, set_i18n_event_handler, set_log_event_handler,
    set_permission_log_handler, set_server_ready_handler, set_sidebar_event_handler,
    set_ui_event_handler, ApiCallHandler, ComponentEventHandler, ContextMenuHandler,
    I18nEventHandler, LogEventHandler, PermissionLogHandler, ServerReadyHandler,
    SidebarEventHandler, UiEventHandler,
};
pub use permission_logs::{
    clear_plugin_permission_logs, get_plugin_permission_logs, take_permission_log_snapshot,
    BufferedPermissionLog,
};
pub use registry::{new_api_registry, ApiRegistry, ApiRegistryOps};
pub use sidebar::{
    clear_plugin_sidebar_snapshot, take_sidebar_event_snapshot, BufferedSidebarEvent,
};
pub use ui_snapshot::{clear_plugin_ui_snapshot, take_ui_event_snapshot, BufferedUiEvent};
