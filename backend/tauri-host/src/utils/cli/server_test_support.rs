use crate::models::server::{
    CpuPolicyConfig, JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
};
pub(super) use crate::test_support::{lock_env, EnvGuard};

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
            jvm_args: Vec::new(),
            cpu_policy: CpuPolicyConfig::default(),
            jvm_preset: JvmPresetConfig::default(),
        }),
    }
}
