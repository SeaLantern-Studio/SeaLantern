use std::path::Path;

use crate::commands::server::config::SLStartupConfig;
use crate::models::server::ServerInstance;
use crate::models::settings::AppSettings;

use super::common::ManagedConsoleEncoding;

pub(super) fn build_managed_jvm_args(
    server: &ServerInstance,
    settings: &AppSettings,
    console_encoding: ManagedConsoleEncoding,
) -> Vec<String> {
    let java_encoding = console_encoding.java_name();
    let default_memory = (settings.default_max_memory, settings.default_min_memory);
    let (max_mem, min_mem) = read_sl_startup_config(server, settings).unwrap_or(default_memory);
    let mut args = vec![
        format!("-Xmx{}M", max_mem),
        format!("-Xms{}M", min_mem),
        format!("-Dfile.encoding={}", java_encoding),
        format!("-Dsun.stdout.encoding={}", java_encoding),
        format!("-Dsun.stderr.encoding={}", java_encoding),
    ];

    let jvm = settings.default_jvm_args.trim();
    if !jvm.is_empty() {
        args.extend(jvm.split_whitespace().map(|arg| arg.to_string()));
    }

    args.extend(server.jvm_args().iter().cloned());
    args
}

pub(super) fn write_user_jvm_args(
    server: &ServerInstance,
    settings: &AppSettings,
    console_encoding: ManagedConsoleEncoding,
) -> Result<(), String> {
    let args = build_managed_jvm_args(server, settings, console_encoding);
    let user_jvm_args_path = Path::new(&server.path).join("user_jvm_args.txt");
    let content = if args.is_empty() {
        String::new()
    } else {
        format!("{}\n", args.join("\n"))
    };

    std::fs::write(&user_jvm_args_path, content)
        .map_err(|e| format!("写入 user_jvm_args.txt 失败: {}", e))
}

fn read_sl_startup_config(server: &ServerInstance, settings: &AppSettings) -> Option<(u32, u32)> {
    let sl_path = Path::new(&server.path).join("SL.json");
    if !sl_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&sl_path).ok()?;
    let config: SLStartupConfig = serde_json::from_str(&content).ok()?;
    match (config.max_memory, config.min_memory) {
        (Some(max), Some(min)) => Some((max, min)),
        (Some(max), None) => Some((max, settings.default_min_memory)),
        (None, Some(min)) => Some((settings.default_max_memory, min)),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{build_managed_jvm_args, write_user_jvm_args};
    use crate::models::server::{LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig};
    use crate::models::settings::AppSettings;
    use crate::services::server::manager::common::ManagedConsoleEncoding;
    use tempfile::tempdir;

    fn test_server(path: String) -> ServerInstance {
        ServerInstance {
            id: "startup-support".to_string(),
            name: "Startup Support".to_string(),
            aliases: Vec::new(),
            core_type: "fabric".to_string(),
            core_version: "fabric".to_string(),
            mc_version: "1.20.1".to_string(),
            path,
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 0,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "java".to_string(),
                jvm_args: vec!["-Dserver.flag=true".to_string()],
            }),
        }
    }

    fn test_settings() -> AppSettings {
        AppSettings {
            default_max_memory: 8192,
            default_min_memory: 1024,
            default_jvm_args: "-Dglobal.flag=true -XX:+UseG1GC".to_string(),
            ..AppSettings::default()
        }
    }

    #[test]
    fn build_managed_jvm_args_prefers_sl_json_memory_and_preserves_arg_order() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        std::fs::write(temp_dir.path().join("SL.json"), r#"{"max_memory":3072,"min_memory":1536}"#)
            .expect("SL.json should be written");

        let args = build_managed_jvm_args(&server, &test_settings(), ManagedConsoleEncoding::Utf8);

        assert_eq!(
            args,
            vec![
                "-Xmx3072M".to_string(),
                "-Xms1536M".to_string(),
                "-Dfile.encoding=UTF-8".to_string(),
                "-Dsun.stdout.encoding=UTF-8".to_string(),
                "-Dsun.stderr.encoding=UTF-8".to_string(),
                "-Dglobal.flag=true".to_string(),
                "-XX:+UseG1GC".to_string(),
                "-Dserver.flag=true".to_string(),
            ]
        );
    }

    #[test]
    fn write_user_jvm_args_uses_default_min_memory_when_sl_json_omits_it() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let server = test_server(temp_dir.path().to_string_lossy().to_string());
        std::fs::write(temp_dir.path().join("SL.json"), r#"{"max_memory":6144}"#)
            .expect("SL.json should be written");

        write_user_jvm_args(&server, &test_settings(), ManagedConsoleEncoding::Utf8)
            .expect("user_jvm_args.txt should be written");

        let content = std::fs::read_to_string(temp_dir.path().join("user_jvm_args.txt"))
            .expect("user_jvm_args.txt should exist");

        assert_eq!(
            content,
            concat!(
                "-Xmx6144M\n",
                "-Xms1024M\n",
                "-Dfile.encoding=UTF-8\n",
                "-Dsun.stdout.encoding=UTF-8\n",
                "-Dsun.stderr.encoding=UTF-8\n",
                "-Dglobal.flag=true\n",
                "-XX:+UseG1GC\n",
                "-Dserver.flag=true\n"
            )
        );
    }
}
