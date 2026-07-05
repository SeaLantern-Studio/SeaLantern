use crate::services::global::i18n_service;
use std::collections::{HashMap, VecDeque};
use std::io::Read;
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn process_t(key: &str) -> String {
    i18n_service().t(key)
}

fn process_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

fn process_t2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    m.insert("1".to_string(), b.into());
    i18n_service().t_with_options(key, &m)
}

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

/// 后台进程输出缓冲
#[derive(Default)]
pub struct ProcessOutputBuffers {
    pub stdout_buf: VecDeque<u8>,
    pub stderr_buf: VecDeque<u8>,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub stdout_closed: bool,
    pub stderr_closed: bool,
    pub next_chunk_seq: u64,
    pub last_update_unix_ms: Option<u64>,
}

#[derive(Clone, Copy)]
pub enum ProcessStream {
    Stdout,
    Stderr,
}

/// 受管进程条目
pub struct ProcessEntry {
    pub owner_plugin_id: String,
    pub program: String,
    pub child: Child,
    pub output: Arc<Mutex<ProcessOutputBuffers>>,
    pub started_at: Instant,
}

/// 创建新的进程注册表
pub fn new_process_registry() -> Arc<Mutex<HashMap<u32, ProcessEntry>>> {
    Arc::new(Mutex::new(HashMap::new()))
}

/// 创建后台进程输出缓冲
pub fn new_process_output() -> Arc<Mutex<ProcessOutputBuffers>> {
    Arc::new(Mutex::new(ProcessOutputBuffers::default()))
}

pub fn unix_timestamp_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}

/// 记录后台输出最近一次更新时间，并保证同一进程内不会倒退。
pub(crate) fn update_output_timestamp(output: &mut ProcessOutputBuffers, observed_at_ms: u64) {
    output.last_update_unix_ms = Some(
        output
            .last_update_unix_ms
            .map_or(observed_at_ms, |last| last.max(observed_at_ms)),
    );
}

/// 后台输出是否已经全部被读取并消费完成。
pub fn is_output_drained(output: &ProcessOutputBuffers) -> bool {
    output.stdout_closed
        && output.stderr_closed
        && output.stdout_buf.is_empty()
        && output.stderr_buf.is_empty()
}

/// 判断进程是否属于指定插件
pub fn is_process_owner(entry: &ProcessEntry, plugin_id: &str) -> bool {
    entry.owner_plugin_id == plugin_id
}

/// 截断输出缓冲，保证不会无限增长。
///
/// 返回值表示本次调用是否真的发生了截断。
pub fn truncate_output(buf: &mut Vec<u8>) -> bool {
    if buf.len() > MAX_STDOUT_BUFFER_BYTES {
        let drain_len = buf.len() - MAX_STDOUT_BUFFER_BYTES;
        buf.drain(0..drain_len);
        true
    } else {
        false
    }
}

pub fn append_bounded_output(buf: &mut VecDeque<u8>, chunk: &[u8]) -> bool {
    if chunk.is_empty() {
        return false;
    }

    buf.extend(chunk.iter().copied());

    if buf.len() > MAX_STDOUT_BUFFER_BYTES {
        let overflow = buf.len() - MAX_STDOUT_BUFFER_BYTES;
        buf.drain(..overflow);
        true
    } else {
        false
    }
}

pub fn take_output_bytes(buf: &mut VecDeque<u8>) -> Vec<u8> {
    buf.drain(..).collect()
}

/// 为后台进程的单个输出流启动持续读取线程。
pub fn spawn_background_pipe_reader<R>(
    mut reader: R,
    output: Arc<Mutex<ProcessOutputBuffers>>,
    stream: ProcessStream,
) where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buf = [0u8; 8192];

        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    let mut output = output.lock().unwrap_or_else(|e| {
                        eprintln!("[WARN] Process output lock poisoned: {}", e);
                        e.into_inner()
                    });
                    match stream {
                        ProcessStream::Stdout => output.stdout_closed = true,
                        ProcessStream::Stderr => output.stderr_closed = true,
                    }
                    break;
                }
                Ok(n) => {
                    let mut output = output.lock().unwrap_or_else(|e| {
                        eprintln!("[WARN] Process output lock poisoned: {}", e);
                        e.into_inner()
                    });

                    let truncated = {
                        let target = match stream {
                            ProcessStream::Stdout => &mut output.stdout_buf,
                            ProcessStream::Stderr => &mut output.stderr_buf,
                        };
                        append_bounded_output(target, &buf[..n])
                    };

                    update_output_timestamp(&mut output, unix_timestamp_ms_now());

                    if truncated {
                        match stream {
                            ProcessStream::Stdout => output.stdout_truncated = true,
                            ProcessStream::Stderr => output.stderr_truncated = true,
                        }
                    }
                }
                Err(error) => {
                    let mut output = output.lock().unwrap_or_else(|e| {
                        eprintln!("[WARN] Process output lock poisoned: {}", e);
                        e.into_inner()
                    });
                    match stream {
                        ProcessStream::Stdout => output.stdout_closed = true,
                        ProcessStream::Stderr => output.stderr_closed = true,
                    }
                    eprintln!(
                        "[WARN] Failed to read background process {}: {}",
                        match stream {
                            ProcessStream::Stdout => "stdout",
                            ProcessStream::Stderr => "stderr",
                        },
                        error
                    );
                    break;
                }
            }
        }
    });
}

/// 清理已经结束的进程条目
pub fn collect_finished_processes(procs: &mut HashMap<u32, ProcessEntry>) {
    let finished: Vec<u32> = procs
        .iter_mut()
        .filter_map(|(pid, entry)| match entry.child.try_wait() {
            Ok(Some(_)) => {
                let output = entry.output.lock().unwrap_or_else(|e| {
                    eprintln!("[WARN] Process output lock poisoned: {}", e);
                    e.into_inner()
                });
                is_output_drained(&output).then_some(*pid)
            }
            Ok(None) => None,
            Err(_) => Some(*pid),
        })
        .collect();

    for pid in finished {
        procs.remove(&pid);
    }
}

/// 统计某个插件当前拥有的进程数
pub fn plugin_process_count(procs: &mut HashMap<u32, ProcessEntry>, plugin_id: &str) -> usize {
    let mut count = 0;

    for entry in procs.values_mut() {
        if entry.owner_plugin_id != plugin_id {
            continue;
        }

        if matches!(entry.child.try_wait(), Ok(None)) {
            count += 1;
        }
    }

    count
}

/// 杀掉某个插件名下的全部进程
pub fn kill_plugin_processes(registry: &Arc<Mutex<HashMap<u32, ProcessEntry>>>, plugin_id: &str) {
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
pub fn kill_all_processes(registry: &Arc<Mutex<HashMap<u32, ProcessEntry>>>) {
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

pub(super) fn process_err(key: &str) -> mlua::Error {
    mlua::Error::runtime(process_t(key))
}

pub(super) fn process_err1(key: &str, a: impl Into<String>) -> mlua::Error {
    mlua::Error::runtime(process_t1(key, a))
}

pub(super) fn process_err2(key: &str, a: impl Into<String>, b: impl Into<String>) -> mlua::Error {
    mlua::Error::runtime(process_t2(key, a, b))
}

pub(super) fn process_msg1(key: &str, a: impl Into<String>) -> String {
    process_t1(key, a)
}

pub(super) fn process_msg2(key: &str, a: impl Into<String>, b: impl Into<String>) -> String {
    process_t2(key, a, b)
}

#[cfg(test)]
mod tests {
    use super::{
        append_bounded_output, take_output_bytes, truncate_output, MAX_STDOUT_BUFFER_BYTES,
    };
    use std::collections::VecDeque;

    #[test]
    fn append_bounded_output_keeps_latest_bytes_without_front_copy_semantics_change() {
        let mut buf = VecDeque::from(vec![b'a'; MAX_STDOUT_BUFFER_BYTES - 2]);

        assert!(append_bounded_output(&mut buf, b"bcde"));

        let bytes = take_output_bytes(&mut buf);
        assert_eq!(bytes.len(), MAX_STDOUT_BUFFER_BYTES);
        assert_eq!(&bytes[..4], b"aaaa");
        assert_eq!(&bytes[bytes.len() - 4..], b"bcde");
    }

    #[test]
    fn truncate_output_keeps_latest_bytes_for_foreground_exec_buffers() {
        let mut buf = vec![b'x'; MAX_STDOUT_BUFFER_BYTES + 3];
        let tail_start = buf.len() - 3;
        buf[tail_start..].copy_from_slice(b"end");

        assert!(truncate_output(&mut buf));
        assert_eq!(buf.len(), MAX_STDOUT_BUFFER_BYTES);
        assert_eq!(&buf[buf.len() - 3..], b"end");
    }
}
