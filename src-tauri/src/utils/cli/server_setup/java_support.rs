use std::path::{Path, PathBuf};

use crate::services::java_detector;
use crate::utils::cli::server_args::CliServerCommand;
use crate::utils::cli::server_ports::prompt_yes_no;
use crate::utils::cli::server_shared::{trace_cli_action, trace_cli_error};
use crate::utils::path::find_executable_in_path;

pub(super) fn resolve_java_path(
    command: &CliServerCommand,
    default_java_path: &str,
) -> Result<String, String> {
    if let Some(java_path) = command
        .java_path
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        trace_cli_action("java_resolve_explicit", &format!("selector={}", java_path));
        return resolve_explicit_java_path(java_path);
    }

    if let Some(java_from_env) = find_java_from_env() {
        trace_cli_action("java_resolve_env", &format!("path={}", java_from_env));
        return Ok(java_from_env);
    }

    if command.java_from_env_only {
        return Err(
            "--J 模式下未在 JAVA_HOME 或 PATH 中找到 Java，请改用 --java 显式指定路径".to_string()
        );
    }

    let trimmed_default = default_java_path.trim();
    if !trimmed_default.is_empty() {
        if let Ok(info) = java_detector::validate_java(trimmed_default) {
            trace_cli_action(
                "java_resolve_default",
                &format!("path={} version={} vendor={}", info.path, info.version, info.vendor),
            );
            return Ok(info.path);
        }

        trace_cli_error(
            "java_default_invalid",
            &format!("configured_path={}", trimmed_default),
            "configured default Java path is not executable or not a valid Java runtime",
        );
    }

    if let Ok(path) = find_java_in_path() {
        trace_cli_action("java_resolve_path_fallback", &format!("path={}", path));
        return Ok(path);
    }

    let should_scan = prompt_yes_no("未找到可用 Java，是否尝试全局扫描？ [Y/n] ")?;
    if !should_scan {
        trace_cli_error("java_scan_cancelled", "", "user cancelled java global scan");
        return Err("未找到可用 Java，用户取消全局扫描".to_string());
    }

    let found = java_detector::detect_java_installations();
    select_java_from_scan_results(&found)
}

pub(super) fn resolve_explicit_java_path(java_path: &str) -> Result<String, String> {
    resolve_explicit_java_path_with(java_path, validate_java_candidate)
}

fn resolve_explicit_java_path_with<FValidate>(
    java_path: &str,
    validate_candidate: FValidate,
) -> Result<String, String>
where
    FValidate: Fn(&str, &str) -> Result<String, String>,
{
    match normalize_java_env_selector(java_path) {
        Some(JavaEnvSelector::JavaHome) => {
            let candidate = find_java_from_java_home().ok_or_else(|| {
                "--java %env:JAVA_HOME% 未解析到可用 Java，请确认 JAVA_HOME 指向有效 JDK/JRE"
                    .to_string()
            })?;
            validate_candidate(
                &candidate,
                "--java %env:JAVA_HOME% 指向的目标不是可用 Java，请确认 JAVA_HOME 配置正确",
            )
        }
        Some(JavaEnvSelector::Path) => {
            let candidate = find_java_in_path().map_err(|_| {
                "--java %env:Path% 未在 PATH 中解析到可用 Java，请确认 PATH 已包含 java".to_string()
            })?;
            validate_candidate(
                &candidate,
                "--java %env:Path% 解析到的目标不是可用 Java，请确认 PATH 中的 java 可执行且有效",
            )
        }
        None => {
            let trimmed = java_path.trim();
            if trimmed.is_empty() {
                return Err("--java 不能为空路径".to_string());
            }

            if Path::new(trimmed).exists() {
                return validate_candidate(
                    trimmed,
                    &format!("--java 指定路径不是可用 Java 运行时: {}", trimmed),
                );
            }

            if let Some(resolved) = find_executable_in_path(trimmed) {
                return validate_candidate(
                    resolved.to_string_lossy().as_ref(),
                    &format!("--java 指定的可执行文件不是有效 Java 运行时: {}", trimmed),
                );
            }

            Err(format!("--java 指定路径不存在，且未在 PATH 中解析到可执行文件: {}", java_path))
        }
    }
}

pub(super) fn select_java_from_scan_results(
    found: &[java_detector::JavaInfo],
) -> Result<String, String> {
    select_java_from_scan_results_with(found, prompt_java_scan_selection)
}

pub(super) fn select_java_from_scan_results_with<FPrompt>(
    found: &[java_detector::JavaInfo],
    mut prompt_select: FPrompt,
) -> Result<String, String>
where
    FPrompt: FnMut(usize) -> Result<usize, String>,
{
    if found.is_empty() {
        trace_cli_error("java_scan_empty", "", "no java installation discovered");
        return Err("全局扫描后仍未找到 Java，请使用 --java 或配置默认 Java 路径".to_string());
    }

    trace_cli_action("java_scan_found", &format!("count={}", found.len()));
    println!("全局扫描发现以下 Java 安装:");
    for (index, item) in found.iter().enumerate() {
        println!(
            "  [{}] Java {} | vendor={} | arch={} | path={}",
            index + 1,
            item.version,
            item.vendor,
            if item.is_64bit { "64-bit" } else { "32-bit" },
            item.path
        );
    }

    if found.len() == 1 {
        let selected = &found[0];
        trace_cli_action("java_scan_auto_select", &format!("index=1 path={}", selected.path));
        println!("仅检测到 1 个可用 Java，已自动选择: {}", selected.path);
        return Ok(selected.path.clone());
    }

    let selected_index = prompt_select(found.len())?;
    let selected = &found[selected_index];
    trace_cli_action(
        "java_scan_select",
        &format!("index={} path={}", selected_index + 1, selected.path),
    );
    Ok(selected.path.clone())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum JavaEnvSelector {
    JavaHome,
    Path,
}

pub(super) fn normalize_java_env_selector(value: &str) -> Option<JavaEnvSelector> {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("%env:JAVA_HOME%") {
        return Some(JavaEnvSelector::JavaHome);
    }
    if trimmed.eq_ignore_ascii_case("%env:PATH%") || trimmed.eq_ignore_ascii_case("%env:Path%") {
        return Some(JavaEnvSelector::Path);
    }
    None
}

pub(super) fn find_java_from_env() -> Option<String> {
    find_java_from_java_home()
        .and_then(|candidate| validate_java_candidate(&candidate, "JAVA_HOME invalid").ok())
        .or_else(|| {
            find_java_in_path()
                .ok()
                .and_then(|candidate| validate_java_candidate(&candidate, "PATH java invalid").ok())
        })
}

fn prompt_java_scan_selection(count: usize) -> Result<usize, String> {
    loop {
        print!("请选择要使用的 Java 编号 [1-{}，默认 1]: ", count);
        use std::io::Write;
        std::io::stdout()
            .flush()
            .map_err(|err| format!("输出 Java 选择提示失败: {}", err))?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|err| format!("读取 Java 选择失败: {}", err))?;

        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(0);
        }

        match trimmed.parse::<usize>() {
            Ok(value) if (1..=count).contains(&value) => return Ok(value - 1),
            _ => {
                println!("无效选择: {}。请输入 1 到 {} 之间的编号。", trimmed, count);
            }
        }
    }
}

fn find_java_in_path() -> Result<String, String> {
    let executable = if cfg!(windows) { "java.exe" } else { "java" };
    find_executable_in_path(executable)
        .map(|path| path.to_string_lossy().to_string())
        .ok_or_else(|| "PATH 中未找到 java".to_string())
}

fn validate_java_candidate(candidate: &str, context_message: &str) -> Result<String, String> {
    java_detector::validate_java(candidate)
        .map(|info| info.path)
        .map_err(|_| context_message.to_string())
}

fn find_java_from_java_home() -> Option<String> {
    let java_home = std::env::var_os("JAVA_HOME")?;
    let mut path = PathBuf::from(java_home);
    path.push("bin");
    path.push(if cfg!(windows) { "java.exe" } else { "java" });
    if path.exists() {
        Some(path.to_string_lossy().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        find_java_from_env, normalize_java_env_selector, resolve_explicit_java_path,
        resolve_explicit_java_path_with, resolve_java_path, select_java_from_scan_results,
        select_java_from_scan_results_with, JavaEnvSelector,
    };
    use crate::services::java_detector;
    use crate::utils::cli::server_args::CliServerCommand;
    use once_cell::sync::Lazy;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn lock_env() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    struct EnvVarGuard {
        key: String,
        original: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &str, value: &str) -> Self {
            let original = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key: key.to_string(), original }
        }

        fn remove(key: &str) -> Self {
            let original = std::env::var(key).ok();
            std::env::remove_var(key);
            Self { key: key.to_string(), original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => std::env::set_var(&self.key, value),
                None => std::env::remove_var(&self.key),
            }
        }
    }

    #[test]
    fn find_java_from_env_is_safe_to_call_without_assumptions() {
        let _ = find_java_from_env();
    }

    #[test]
    fn normalize_java_env_selector_recognizes_supported_tokens() {
        assert_eq!(normalize_java_env_selector("%env:JAVA_HOME%"), Some(JavaEnvSelector::JavaHome));
        assert_eq!(normalize_java_env_selector("%env:Path%"), Some(JavaEnvSelector::Path));
        assert_eq!(normalize_java_env_selector("%env:PATH%"), Some(JavaEnvSelector::Path));
        assert_eq!(normalize_java_env_selector("C:/Java/bin/java.exe"), None);
    }

    #[test]
    fn resolve_explicit_java_path_supports_java_home_selector() {
        let _env_lock = lock_env();
        let temp_dir = tempdir().expect("temp dir should exist");
        let java_bin_dir = temp_dir.path().join("bin");
        std::fs::create_dir_all(&java_bin_dir).expect("java bin dir should create");
        let java_path = java_bin_dir.join(if cfg!(windows) { "java.exe" } else { "java" });
        std::fs::write(&java_path, b"placeholder").expect("java placeholder should write");
        let _java_home = EnvVarGuard::set("JAVA_HOME", temp_dir.path().to_string_lossy().as_ref());

        let resolved = resolve_explicit_java_path_with("%env:JAVA_HOME%", |candidate, _| {
            Ok(candidate.to_string())
        })
        .expect("JAVA_HOME selector should resolve");
        assert_eq!(resolved, java_path.to_string_lossy().to_string());
    }

    #[test]
    fn resolve_explicit_java_path_supports_path_selector() {
        let _env_lock = lock_env();
        let temp_dir = tempdir().expect("temp dir should exist");
        let java_path = temp_dir
            .path()
            .join(if cfg!(windows) { "java.exe" } else { "java" });
        std::fs::write(&java_path, b"placeholder").expect("java placeholder should write");

        let path_value = if cfg!(windows) {
            temp_dir.path().to_string_lossy().to_string()
        } else {
            format!(
                "{}:{}",
                temp_dir.path().to_string_lossy(),
                std::env::var("PATH").unwrap_or_default()
            )
        };
        let _path = EnvVarGuard::set("PATH", &path_value);

        let resolved =
            resolve_explicit_java_path_with("%env:Path%", |candidate, _| Ok(candidate.to_string()))
                .expect("PATH selector should resolve");
        assert_eq!(resolved, java_path.to_string_lossy().to_string());
    }

    #[test]
    fn resolve_explicit_java_path_keeps_plain_path_literal_after_validation() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let java_path = temp_dir
            .path()
            .join(if cfg!(windows) { "java.exe" } else { "java" });
        std::fs::write(&java_path, b"placeholder").expect("java placeholder should write");

        let resolved = resolve_explicit_java_path_with(
            java_path.to_string_lossy().as_ref(),
            |candidate, _| Ok(candidate.to_string()),
        )
        .expect("plain explicit path should validate and resolve");
        assert_eq!(resolved, java_path.to_string_lossy().to_string());
    }

    #[test]
    fn resolve_explicit_java_path_rejects_invalid_literal_path() {
        let err = resolve_explicit_java_path("C:/__sealantern_missing_java__/java.exe")
            .expect_err("invalid explicit path should fail early");
        assert!(err.contains("不存在"));
    }

    #[test]
    fn resolve_explicit_java_path_rejects_existing_but_invalid_placeholder_file() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let java_path = temp_dir
            .path()
            .join(if cfg!(windows) { "java.exe" } else { "java" });
        std::fs::write(&java_path, b"not a java runtime").expect("placeholder should write");

        let err =
            resolve_explicit_java_path_with(java_path.to_string_lossy().as_ref(), |_, message| {
                Err(message.to_string())
            })
            .expect_err("invalid java placeholder should be rejected");
        assert!(err.contains("不是可用 Java 运行时"));
    }

    #[test]
    fn resolve_java_path_in_env_only_mode_fails_without_falling_back_to_defaults() {
        let _env_lock = lock_env();
        let _java_home = EnvVarGuard::remove("JAVA_HOME");
        let _path = if cfg!(windows) {
            EnvVarGuard::set("PATH", "C:/__sealantern_missing_java_path__")
        } else {
            EnvVarGuard::set("PATH", "/__sealantern_missing_java_path__")
        };

        let command = CliServerCommand {
            java_from_env_only: true,
            ..Default::default()
        };

        let err = resolve_java_path(&command, "C:/Default/Java/bin/java.exe")
            .expect_err("env-only mode should not fall back to configured default");

        assert!(err.contains("--J"));
        assert!(err.contains("JAVA_HOME") || err.contains("PATH"));
    }

    #[test]
    fn find_java_from_env_skips_invalid_java_home_candidate() {
        let _env_lock = lock_env();
        let _java_home = EnvVarGuard::remove("JAVA_HOME");
        let _path = if cfg!(windows) {
            EnvVarGuard::set("PATH", "C:/__sealantern_missing_java_path__")
        } else {
            EnvVarGuard::set("PATH", "/__sealantern_missing_java_path__")
        };

        let scan_root = tempdir().expect("temp dir should exist");
        let java_home = scan_root.path().join("jdk-21");
        let java_bin_dir = java_home.join("bin");
        std::fs::create_dir_all(&java_bin_dir).expect("java bin dir should create");
        let java_path = java_bin_dir.join(if cfg!(windows) { "java.exe" } else { "java" });
        std::fs::write(&java_path, b"placeholder").expect("java placeholder should write");
        let _scan_java_home = EnvVarGuard::set("JAVA_HOME", java_home.to_string_lossy().as_ref());

        assert!(find_java_from_env().is_none());
    }

    #[test]
    fn select_java_from_scan_results_auto_selects_single_result() {
        let selected = select_java_from_scan_results(&[java_detector::JavaInfo {
            path: "C:/Java/bin/java.exe".to_string(),
            version: "21.0.1".to_string(),
            vendor: "OpenJDK".to_string(),
            is_64bit: true,
            major_version: 21,
        }])
        .expect("single result should auto select");

        assert_eq!(selected, "C:/Java/bin/java.exe");
    }

    #[test]
    fn select_java_from_scan_results_uses_prompt_for_multiple_results() {
        let found = vec![
            java_detector::JavaInfo {
                path: "C:/Java/jdk-17/bin/java.exe".to_string(),
                version: "17.0.12".to_string(),
                vendor: "OpenJDK".to_string(),
                is_64bit: true,
                major_version: 17,
            },
            java_detector::JavaInfo {
                path: "C:/Java/jdk-21/bin/java.exe".to_string(),
                version: "21.0.1".to_string(),
                vendor: "Oracle".to_string(),
                is_64bit: true,
                major_version: 21,
            },
        ];

        let selected = select_java_from_scan_results_with(&found, |_| Ok(1))
            .expect("prompt-selected java should resolve");

        assert_eq!(selected, "C:/Java/jdk-21/bin/java.exe");
    }
}
