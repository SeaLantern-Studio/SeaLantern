use std::path::PathBuf;

pub(super) fn get_update_cache_dir() -> PathBuf {
    let cache_dir = dirs_next::cache_dir().unwrap_or_else(std::env::temp_dir);
    cache_dir.join("com.fpsz.sea-lantern").join("updates")
}

pub(super) fn get_pending_update_file() -> PathBuf {
    get_update_cache_dir().join("pending_update.json")
}
