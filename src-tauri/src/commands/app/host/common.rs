use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use sysinfo::System;

pub(super) static SYSTEM: Lazy<Mutex<System>> = Lazy::new(|| Mutex::new(System::new_all()));

pub(super) static SERVER_DISK_USAGE_CACHE: Lazy<Mutex<HashMap<String, CachedDirectorySize>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub(super) const PROCESS_CPU_SAMPLE_INTERVAL: Duration = Duration::from_millis(200);
pub(super) const SERVER_DISK_USAGE_CACHE_TTL: Duration = Duration::from_secs(30);

pub(super) struct CachedDirectorySize {
    pub used: u64,
    pub computed_at: Instant,
}
