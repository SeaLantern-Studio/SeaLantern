use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const MAX_STDOUT_BUFFER_BYTES: usize = 1024 * 1024;
pub const MAX_ARGS_COUNT: usize = 128;
pub const MAX_ARG_LENGTH: usize = 4096;
pub const MAX_ENV_VARS: usize = 32;
pub const MAX_ENV_KEY_LENGTH: usize = 128;
pub const MAX_ENV_VALUE_LENGTH: usize = 4096;
pub const MAX_BACKGROUND_PROCESSES_PER_PLUGIN: usize = 8;
pub const MAX_FOREGROUND_EXEC_DURATION: Duration = Duration::from_secs(30);

pub type ProcessRegistry = Arc<Mutex<HashMap<u32, ProcessEntry>>>;

pub struct ProcessEntry {
    pub owner_plugin_id: String,
    pub program: String,
    pub child: Child,
    pub stdout_buf: Vec<u8>,
    pub started_at: Instant,
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

pub fn collect_finished_processes(procs: &mut HashMap<u32, ProcessEntry>) {
    let finished: Vec<u32> = procs
        .iter_mut()
        .filter_map(|(pid, entry)| match entry.child.try_wait() {
            Ok(Some(_)) => Some(*pid),
            Ok(None) => None,
            Err(_) => Some(*pid),
        })
        .collect();

    for pid in finished {
        procs.remove(&pid);
    }
}

pub fn plugin_process_count(procs: &HashMap<u32, ProcessEntry>, plugin_id: &str) -> usize {
    procs
        .values()
        .filter(|entry| entry.owner_plugin_id == plugin_id)
        .count()
}

pub fn kill_plugin_processes(registry: &ProcessRegistry, plugin_id: &str) {
    let mut procs = registry.lock().unwrap_or_else(|e| {
        eprintln!("[WARN] Process registry lock poisoned, recovering: {}", e);
        e.into_inner()
    });

    let owned_pids: Vec<u32> = procs
        .iter()
        .filter_map(|(pid, entry)| (entry.owner_plugin_id == plugin_id).then_some(*pid))
        .collect();

    for pid in owned_pids {
        if let Some(mut entry) = procs.remove(&pid) {
            if let Err(e) = entry.child.kill() {
                eprintln!(
                    "[WARN] Failed to kill plugin-owned process {} (pid {}): {}",
                    entry.program, pid, e
                );
            }
            let _ = entry.child.wait();
        }
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
        let _ = entry.child.wait();
    }
    procs.clear();
}
