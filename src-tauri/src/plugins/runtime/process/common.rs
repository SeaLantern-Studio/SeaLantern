use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 单次输出缓冲上限
pub const MAX_STDOUT_BUFFER_BYTES: usize = 1024 * 1024;
/// 单次命令最多参数个数
pub const MAX_ARGS_COUNT: usize = 128;
/// 单个参数最大长度
pub const MAX_ARG_LENGTH: usize = 4096;
/// 最多允许的环境变量个数
pub const MAX_ENV_VARS: usize = 32;
/// 环境变量名最大长度
pub const MAX_ENV_KEY_LENGTH: usize = 128;
/// 环境变量值最大长度
pub const MAX_ENV_VALUE_LENGTH: usize = 4096;
/// 每个插件最多允许的后台进程数
pub const MAX_BACKGROUND_PROCESSES_PER_PLUGIN: usize = 8;
/// 前台执行最长时长
pub const MAX_FOREGROUND_EXEC_DURATION: Duration = Duration::from_secs(30);

/// 进程注册表
pub type ProcessRegistry = Arc<Mutex<HashMap<u32, ProcessEntry>>>;

/// 受管进程条目
pub struct ProcessEntry {
    pub owner_plugin_id: String,
    pub program: String,
    pub child: Child,
    pub stdout_buf: Vec<u8>,
    pub started_at: Instant,
}

/// 创建新的进程注册表
pub fn new_process_registry() -> ProcessRegistry {
    Arc::new(Mutex::new(HashMap::new()))
}

/// 判断进程是否属于指定插件
pub fn is_process_owner(entry: &ProcessEntry, plugin_id: &str) -> bool {
    entry.owner_plugin_id == plugin_id
}

/// 截断输出缓冲，保证不会无限增长
pub fn truncate_output(buf: &mut Vec<u8>) {
    if buf.len() > MAX_STDOUT_BUFFER_BYTES {
        let drain_len = buf.len() - MAX_STDOUT_BUFFER_BYTES;
        buf.drain(0..drain_len);
    }
}

/// 清理已经结束的进程条目
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

/// 统计某个插件当前拥有的进程数
pub fn plugin_process_count(procs: &HashMap<u32, ProcessEntry>, plugin_id: &str) -> usize {
    procs
        .values()
        .filter(|entry| entry.owner_plugin_id == plugin_id)
        .count()
}

/// 杀掉某个插件名下的全部进程
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

/// 杀掉注册表里的全部进程
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
