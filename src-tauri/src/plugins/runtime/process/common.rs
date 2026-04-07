use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, Mutex};

pub const MAX_STDOUT_BUFFER_BYTES: usize = 1024 * 1024;
pub const MAX_ARGS_COUNT: usize = 128;
pub const MAX_ARG_LENGTH: usize = 4096;
pub const MAX_ENV_VARS: usize = 32;
pub const MAX_ENV_KEY_LENGTH: usize = 128;
pub const MAX_ENV_VALUE_LENGTH: usize = 4096;

pub type ProcessRegistry = Arc<Mutex<HashMap<u32, ProcessEntry>>>;

pub struct ProcessEntry {
    pub owner_plugin_id: String,
    pub program: String,
    pub child: Child,
    pub stdout_buf: Vec<u8>,
}

pub fn new_process_registry() -> ProcessRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn is_process_owner(entry: &ProcessEntry, plugin_id: &str) -> bool {
    entry.owner_plugin_id == plugin_id
}

pub fn truncate_output(buf: &mut Vec<u8>) {
    if buf.len() > MAX_STDOUT_BUFFER_BYTES {
        let drain_len = buf.len() - MAX_STDOUT_BUFFER_BYTES;
        buf.drain(0..drain_len);
    }
}

pub fn kill_all_processes(registry: &ProcessRegistry) {
    let mut procs = registry.lock().unwrap_or_else(|e| {
        eprintln!("[WARN] Process registry lock poisoned, recovering: {}", e);
        e.into_inner()
    });
    for (pid, entry) in procs.iter_mut() {
        if let Err(e) = entry.child.kill() {
            eprintln!("[WARN] Failed to kill process {} (pid {}): {}", entry.program, pid, e);
        }
    }
    procs.clear();
}
