use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, Mutex};

pub type ProcessRegistry = Arc<Mutex<HashMap<u32, ProcessEntry>>>;

pub struct ProcessEntry {
    pub program: String,
    pub child: Child,
    pub stdout_buf: Vec<u8>,
}

pub fn new_process_registry() -> ProcessRegistry {
    Arc::new(Mutex::new(HashMap::new()))
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
