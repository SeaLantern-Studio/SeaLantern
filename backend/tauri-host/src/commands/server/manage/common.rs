pub(super) fn manager() -> &'static crate::services::server::manager::ServerManager {
    crate::services::global::server_manager()
}
