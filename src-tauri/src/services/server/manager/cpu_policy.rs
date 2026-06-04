use crate::models::server::{CpuPolicyConfig, CpuPolicyMode, ServerInstance};
use crate::services::server::manager::common::StartupMode;
use crate::services::server::manager::startup_support::resolve_effective_startup_config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedCpuPolicy {
    pub cpu_indices: Vec<usize>,
    pub cpuset_display: String,
    pub active_processor_count: u16,
}

pub(crate) fn mode_supports_local_cpu_policy(startup_mode: StartupMode) -> bool {
    matches!(startup_mode, StartupMode::Jar | StartupMode::Starter)
}

pub(crate) fn local_cpu_policy(
    server: &ServerInstance,
    settings: &crate::models::settings::AppSettings,
) -> CpuPolicyConfig {
    resolve_effective_startup_config(server, settings).cpu_policy
}

pub(crate) fn compute_active_processor_count_arg(
    server: &ServerInstance,
    settings: &crate::models::settings::AppSettings,
    startup_mode: StartupMode,
) -> Result<Option<String>, String> {
    let policy = local_cpu_policy(server, settings);

    if !mode_supports_local_cpu_policy(startup_mode) {
        if policy.mode != CpuPolicyMode::Off {
            return Err(format!(
                "当前本地启动模式 {} 暂不支持 CPU policy，请改用 jar/starter 或关闭 CPU 限制",
                startup_mode.as_str()
            ));
        }
        return Ok(None);
    }

    if policy.mode == CpuPolicyMode::Off || !policy.sync_active_processor_count {
        return Ok(None);
    }

    let resolved = resolve_local_cpu_policy(server, settings)?;
    Ok(resolved.map(|value| format!("-XX:ActiveProcessorCount={}", value.active_processor_count)))
}

pub(crate) fn resolve_local_cpu_policy(
    server: &ServerInstance,
    settings: &crate::models::settings::AppSettings,
) -> Result<Option<ResolvedCpuPolicy>, String> {
    let policy = local_cpu_policy(server, settings);
    match policy.mode {
        CpuPolicyMode::Off => Ok(None),
        CpuPolicyMode::Count => {
            let count = policy
                .count
                .ok_or_else(|| "CPU 限制 count 模式缺少 count".to_string())?;
            if count == 0 {
                return Err("CPU 限制 count 模式必须大于 0".to_string());
            }

            let logical_count = logical_cpu_count();
            if usize::from(count) > logical_count {
                return Err(format!(
                    "CPU 限制 count={} 超过当前主机逻辑 CPU 数 {}",
                    count, logical_count
                ));
            }

            let cpu_indices = (0..usize::from(count)).collect::<Vec<_>>();
            Ok(Some(ResolvedCpuPolicy {
                cpuset_display: format_range(&cpu_indices),
                cpu_indices,
                active_processor_count: count,
            }))
        }
        CpuPolicyMode::Explicit => {
            let raw = policy
                .explicit_set
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "CPU 限制 explicit 模式缺少 explicit_set".to_string())?;

            let mut cpu_indices = parse_cpu_set(raw)?;
            let logical_count = logical_cpu_count();
            if cpu_indices.iter().any(|index| *index >= logical_count) {
                return Err(format!(
                    "CPU 核心集合超出当前主机逻辑 CPU 数 {}: {}",
                    logical_count, raw
                ));
            }

            cpu_indices.sort_unstable();
            cpu_indices.dedup();
            if cpu_indices.is_empty() {
                return Err("CPU 核心集合解析后为空".to_string());
            }

            let active_processor_count =
                u16::try_from(cpu_indices.len()).map_err(|_| "CPU 核心集合数量过大".to_string())?;
            Ok(Some(ResolvedCpuPolicy {
                cpuset_display: format_range(&cpu_indices),
                cpu_indices,
                active_processor_count,
            }))
        }
    }
}

pub(crate) fn apply_cpu_affinity_to_pid(
    pid: u32,
    resolved: &ResolvedCpuPolicy,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        apply_cpu_affinity_windows(pid, resolved)
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        apply_cpu_affinity_linux(pid, resolved)
    }

    #[cfg(any(target_os = "macos", not(any(unix, target_os = "windows"))))]
    {
        let _ = pid;
        let _ = resolved;
        Err("当前平台暂不支持本地 CPU affinity".to_string())
    }
}

fn logical_cpu_count() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
}

fn parse_cpu_set(raw: &str) -> Result<Vec<usize>, String> {
    let mut values = Vec::new();
    for chunk in raw.split(',') {
        let token = chunk.trim();
        if token.is_empty() {
            return Err(format!("CPU 核心集合格式无效: {}", raw));
        }

        if let Some((start_raw, end_raw)) = token.split_once('-') {
            let start = parse_cpu_index(start_raw, raw)?;
            let end = parse_cpu_index(end_raw, raw)?;
            if end < start {
                return Err(format!("CPU 核心区间无效: {}", raw));
            }
            values.extend(start..=end);
        } else {
            values.push(parse_cpu_index(token, raw)?);
        }
    }

    values.sort_unstable();
    values.dedup();
    Ok(values)
}

fn parse_cpu_index(raw: &str, whole: &str) -> Result<usize, String> {
    raw.trim()
        .parse::<usize>()
        .map_err(|_| format!("CPU 核心集合格式无效: {}", whole))
}

fn format_range(indices: &[usize]) -> String {
    if indices.is_empty() {
        return String::new();
    }

    let mut ranges = Vec::new();
    let mut start = indices[0];
    let mut prev = indices[0];

    for &value in &indices[1..] {
        if value == prev + 1 {
            prev = value;
            continue;
        }

        ranges.push(format_segment(start, prev));
        start = value;
        prev = value;
    }

    ranges.push(format_segment(start, prev));
    ranges.join(",")
}

fn format_segment(start: usize, end: usize) -> String {
    if start == end {
        start.to_string()
    } else {
        format!("{}-{}", start, end)
    }
}

#[cfg(target_os = "windows")]
fn apply_cpu_affinity_windows(pid: u32, resolved: &ResolvedCpuPolicy) -> Result<(), String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, SetProcessAffinityMask, PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
    };

    let max_bits = usize::BITS as usize;
    if resolved.cpu_indices.iter().any(|index| *index >= max_bits) {
        return Err(format!("当前 Windows affinity 实现仅支持 {} 个逻辑 CPU 以内的掩码", max_bits));
    }

    let mut mask: usize = 0;
    for index in &resolved.cpu_indices {
        mask |= 1usize << index;
    }

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_SET_INFORMATION, false, pid)
            .map_err(|e| format!("打开进程句柄失败: {}", e))?;

        let result = SetProcessAffinityMask(handle, mask)
            .map_err(|e| format!("设置 CPU affinity 失败: {}", e));
        let _ = CloseHandle(handle);
        result
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn apply_cpu_affinity_linux(pid: u32, resolved: &ResolvedCpuPolicy) -> Result<(), String> {
    unsafe {
        let mut cpu_set: libc::cpu_set_t = std::mem::zeroed();
        libc::CPU_ZERO(&mut cpu_set);
        for index in &resolved.cpu_indices {
            libc::CPU_SET(*index, &mut cpu_set);
        }

        let result = libc::sched_setaffinity(
            pid as libc::pid_t,
            std::mem::size_of::<libc::cpu_set_t>(),
            &cpu_set,
        );
        if result != 0 {
            return Err(format!("设置 CPU affinity 失败: {}", std::io::Error::last_os_error()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        compute_active_processor_count_arg, format_range, local_cpu_policy,
        mode_supports_local_cpu_policy, parse_cpu_set, resolve_local_cpu_policy, CpuPolicyConfig,
        CpuPolicyMode,
    };
    use crate::models::server::{
        JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    };
    use crate::models::settings::AppSettings;
    use crate::services::server::manager::common::StartupMode;

    fn test_server(startup_mode: &str, cpu_policy: CpuPolicyConfig) -> ServerInstance {
        ServerInstance {
            id: "cpu-policy".to_string(),
            name: "CPU Policy".to_string(),
            aliases: Vec::new(),
            core_type: "paper".to_string(),
            core_version: "paper".to_string(),
            mc_version: "1.21.1".to_string(),
            path: "E:/servers/cpu-policy".to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "server.jar".to_string(),
                startup_mode: startup_mode.to_string(),
                custom_command: None,
                java_path: "java".to_string(),
                jvm_args: Vec::new(),
                cpu_policy,
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    fn test_settings() -> AppSettings {
        AppSettings::default()
    }

    #[test]
    fn parse_cpu_set_supports_ranges_and_lists() {
        assert_eq!(parse_cpu_set("0-3,6,7").unwrap(), vec![0, 1, 2, 3, 6, 7]);
    }

    #[test]
    fn format_range_compacts_consecutive_values() {
        assert_eq!(format_range(&[0, 1, 2, 4, 6, 7]), "0-2,4,6-7");
    }

    #[test]
    fn only_jar_and_starter_support_local_cpu_policy() {
        assert!(mode_supports_local_cpu_policy(StartupMode::Jar));
        assert!(mode_supports_local_cpu_policy(StartupMode::Starter));
        assert!(!mode_supports_local_cpu_policy(StartupMode::Bat));
        assert!(!mode_supports_local_cpu_policy(StartupMode::Custom));
    }

    #[test]
    fn count_policy_resolves_first_n_cpus() {
        let server = test_server(
            "jar",
            CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(2),
                explicit_set: None,
                sync_active_processor_count: true,
            },
        );

        let resolved = resolve_local_cpu_policy(&server, &test_settings()).unwrap().unwrap();
        assert_eq!(resolved.cpu_indices, vec![0, 1]);
        assert_eq!(resolved.cpuset_display, "0-1");
        assert_eq!(resolved.active_processor_count, 2);
    }

    #[test]
    fn explicit_policy_resolves_count_from_cpu_set() {
        let server = test_server(
            "starter",
            CpuPolicyConfig {
                mode: CpuPolicyMode::Explicit,
                count: None,
                explicit_set: Some("1,3,5".to_string()),
                sync_active_processor_count: true,
            },
        );

        let arg = compute_active_processor_count_arg(&server, &test_settings(), StartupMode::Starter)
            .unwrap()
            .unwrap();
        assert_eq!(arg, "-XX:ActiveProcessorCount=3");
    }

    #[test]
    fn unsupported_mode_rejects_enabled_cpu_policy() {
        let server = test_server(
            "bat",
            CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(2),
                explicit_set: None,
                sync_active_processor_count: true,
            },
        );

        let err = compute_active_processor_count_arg(&server, &test_settings(), StartupMode::Bat)
            .expect_err("unsupported mode should fail");
        assert!(err.contains("暂不支持 CPU policy"));
    }

    #[test]
    fn disabled_sync_skips_active_processor_count() {
        let server = test_server(
            "jar",
            CpuPolicyConfig {
                mode: CpuPolicyMode::Count,
                count: Some(2),
                explicit_set: None,
                sync_active_processor_count: false,
            },
        );

        let arg = compute_active_processor_count_arg(&server, &test_settings(), StartupMode::Jar).unwrap();
        assert_eq!(arg, None);
    }

    #[test]
    fn local_cpu_policy_defaults_to_off_for_non_local_server_shape() {
        let server = test_server("jar", CpuPolicyConfig::default());
        assert_eq!(local_cpu_policy(&server, &test_settings()).mode, CpuPolicyMode::Off);
    }
}
