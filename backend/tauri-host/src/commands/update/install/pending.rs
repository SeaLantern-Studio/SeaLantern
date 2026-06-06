use sea_lantern_update_core::{pending, types::PendingUpdate};

pub(crate) async fn check_pending_update() -> Result<Option<PendingUpdate>, String> {
    pending::check_pending_update(
        &sea_lantern_update_core::install_support::get_pending_update_file(),
        env!("CARGO_PKG_VERSION"),
    )
}

pub(crate) async fn clear_pending_update() -> Result<(), String> {
    pending::clear_pending_update(
        &sea_lantern_update_core::install_support::get_pending_update_file(),
    )
}
