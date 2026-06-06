use crate::models::server::{CpuPolicyConfig, CpuPolicyMode, ServerInstance};
use crate::services::server::manager::common::StartupMode;
use crate::services::server::manager::i18n::manager_t1;
use crate::services::server::manager::startup_support::resolve_effective_startup_config_checked;
use sea_lantern_server_config_core::{
    resolve_active_processor_count as resolve_shared_active_processor_count,
    resolve_local_cpu_policy as resolve_shared_local_cpu_policy, ResolvedCpuPolicy,
};

#[cfg(test)]
use crate::services::server::manager::startup_support::resolve_effective_startup_config;

pub(crate) fn mode_supports_local_cpu_policy(startup_mode: StartupMode) -> bool {
    matches!(startup_mode, StartupMode::Jar | StartupMode::Starter)
}

#[cfg(test)]
pub(crate) fn local_cpu_policy(
    server: &ServerInstance,
    settings: &crate::models::settings::AppSettings,
) -> CpuPolicyConfig {
    resolve_effective_startup_config(server, settings).cpu_policy
}

pub(crate) fn local_cpu_policy_checked(
    server: &ServerInstance,
    settings: &crate::models::settings::AppSettings,
) -> Result<CpuPolicyConfig, String> {
    Ok(resolve_effective_startup_config_checked(server, settings)?.cpu_policy)
}

pub(crate) fn compute_active_processor_count_arg(
    server: &ServerInstance,
    settings: &crate::models::settings::AppSettings,
    startup_mode: StartupMode,
) -> Result<Option<String>, String> {
    let policy = local_cpu_policy_checked(server, settings)?;

    if !mode_supports_local_cpu_policy(startup_mode) {
        if policy.mode != CpuPolicyMode::Off {
            return Err(manager_t1(
                "server.manager.cpu_policy_mode_unsupported",
                startup_mode.as_str().to_string(),
            ));
        }
        return Ok(None);
    }

    if policy.mode == CpuPolicyMode::Off || !policy.sync_active_processor_count {
        return Ok(None);
    }

    let logical_cpu_count = logical_cpu_count();
    let resolved = resolve_shared_active_processor_count(&policy, logical_cpu_count)?;
    Ok(resolved.map(|value| format!("-XX:ActiveProcessorCount={}", value)))
}

pub(crate) fn resolve_local_cpu_policy(
    server: &ServerInstance,
    settings: &crate::models::settings::AppSettings,
) -> Result<Option<ResolvedCpuPolicy>, String> {
    let policy = local_cpu_policy_checked(server, settings)?;
    resolve_shared_local_cpu_policy(&policy, logical_cpu_count())
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
        Err(manager_t("server.manager.cpu_affinity_platform_unsupported"))
    }
}

fn logical_cpu_count() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
}

#[cfg(target_os = "windows")]
fn apply_cpu_affinity_windows(pid: u32, resolved: &ResolvedCpuPolicy) -> Result<(), String> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, SetProcessAffinityMask, PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
    };

    let max_bits = usize::BITS as usize;
    if resolved.cpu_indices.iter().any(|index| *index >= max_bits) {
        return Err(manager_t1(
            "server.manager.cpu_affinity_windows_mask_limit",
            max_bits.to_string(),
        ));
    }

    let mut mask: usize = 0;
    for index in &resolved.cpu_indices {
        mask |= 1usize << index;
    }

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_SET_INFORMATION, false, pid)
            .map_err(|e| {
            manager_t1("server.manager.cpu_affinity_open_process_failed", e.to_string())
        })?;

        let result = SetProcessAffinityMask(handle, mask)
            .map_err(|e| manager_t1("server.manager.cpu_affinity_apply_failed", e.to_string()));
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
            return Err(manager_t1(
                "server.manager.cpu_affinity_apply_failed",
                std::io::Error::last_os_error().to_string(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        compute_active_processor_count_arg, local_cpu_policy, mode_supports_local_cpu_policy,
        resolve_local_cpu_policy, CpuPolicyConfig, CpuPolicyMode,
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

    fn test_explicit_cpu_set() -> String {
        let logical_count = super::logical_cpu_count();
        if logical_count >= 6 {
            "1,3,5".to_string()
        } else if logical_count >= 4 {
            "0,1,3".to_string()
        } else if logical_count >= 3 {
            "0,1,2".to_string()
        } else if logical_count >= 2 {
            "0,1".to_string()
        } else {
            "0".to_string()
        }
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

        let resolved = resolve_local_cpu_policy(&server, &test_settings())
            .unwrap()
            .unwrap();
        assert_eq!(resolved.cpu_indices, vec![0, 1]);
        assert_eq!(resolved.cpuset_display, "0-1");
        assert_eq!(resolved.active_processor_count, 2);
    }

    #[test]
    fn explicit_policy_resolves_count_from_cpu_set() {
        let explicit_set = test_explicit_cpu_set();
        let expected_count = explicit_set
            .split(',')
            .flat_map(|chunk| {
                if let Some((start, end)) = chunk.split_once('-') {
                    let start = start.parse::<usize>().unwrap();
                    let end = end.parse::<usize>().unwrap();
                    (start..=end).collect::<Vec<_>>()
                } else {
                    vec![chunk.parse::<usize>().unwrap()]
                }
            })
            .count();
        let server = test_server(
            "starter",
            CpuPolicyConfig {
                mode: CpuPolicyMode::Explicit,
                count: None,
                explicit_set: Some(explicit_set),
                sync_active_processor_count: true,
            },
        );

        let arg =
            compute_active_processor_count_arg(&server, &test_settings(), StartupMode::Starter)
                .unwrap()
                .unwrap();
        assert_eq!(arg, format!("-XX:ActiveProcessorCount={}", expected_count));
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

        let arg = compute_active_processor_count_arg(&server, &test_settings(), StartupMode::Jar)
            .unwrap();
        assert_eq!(arg, None);
    }

    #[test]
    fn local_cpu_policy_defaults_to_off_for_non_local_server_shape() {
        let server = test_server("jar", CpuPolicyConfig::default());
        assert_eq!(local_cpu_policy(&server, &test_settings()).mode, CpuPolicyMode::Off);
    }
}
