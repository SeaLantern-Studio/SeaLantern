use std::path::PathBuf;

pub fn get_update_cache_dir() -> PathBuf {
    let cache_dir = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
    cache_dir.join("com.fpsz.sea-lantern").join("updates")
}

pub fn get_pending_update_file() -> PathBuf {
    get_update_cache_dir().join("pending_update.json")
}