use crate::models::server::{LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig};
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub(super) static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub(super) struct EnvGuard {
    name: &'static str,
    original: Option<String>,
}

impl EnvGuard {
    pub(super) fn set(name: &'static str, value: &str) -> Self {
        let original = std::env::var(name).ok();
        std::env::set_var(name, value);
        Self { name, original }
    }

    pub(super) fn remove(name: &'static str) -> Self {
        let original = std::env::var(name).ok();
        std::env::remove_var(name);
        Self { name, original }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(value) = &self.original {
            std::env::set_var(self.name, value);
        } else {
            std::env::remove_var(self.name);
        }
    }
}

pub(super) fn sample_server() -> ServerInstance {
    ServerInstance {
        id: "server-1".to_string(),
        name: "Transport Test".to_string(),
        aliases: vec![],
        core_type: "paper".to_string(),
        core_version: "".to_string(),
        mc_version: "1.21.1".to_string(),
        path: "E:/servers/transport-test".to_string(),
        port: 25565,
        max_memory: 2048,
        min_memory: 1024,
        created_at: 0,
        last_started_at: None,
        runtime_kind: "local".to_string(),
        runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
            jar_path: "E:/servers/transport-test/server.jar".to_string(),
            startup_mode: "jar".to_string(),
            custom_command: None,
            java_path: "C:/Java/bin/java.exe".to_string(),
            jvm_args: vec![],
        }),
    }
}
