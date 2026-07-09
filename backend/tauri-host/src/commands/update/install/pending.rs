use update::{pending, types::PendingUpdate};

pub(crate) async fn check_pending_update() -> Result<Option<PendingUpdate>, String> {
    pending::check_pending_update(
        &update::install_support::get_pending_update_file(),
        env!("CARGO_PKG_VERSION"),
    )
}

pub(crate) async fn clear_pending_update() -> Result<(), String> {
    pending::clear_pending_update(&update::install_support::get_pending_update_file())
}
