use crate::models::server::ServerInstance;
use crate::services::global;

pub(super) fn resolve_server_reference(target: &str) -> Result<ServerInstance, String> {
    let servers = global::server_manager().get_server_list();
    resolve_server_reference_from_servers(&servers, target)
}

pub(super) fn resolve_server_reference_from_servers(
    servers: &[ServerInstance],
    target: &str,
) -> Result<ServerInstance, String> {
    if let Some(server) = servers.iter().find(|server| server.id == target) {
        return Ok(server.clone());
    }

    let target_lower = target.to_ascii_lowercase();
    let mut matches = Vec::new();
    for server in servers {
        let name_match = server.name.to_ascii_lowercase() == target_lower;
        let alias_match = server
            .aliases
            .iter()
            .any(|alias| alias.to_ascii_lowercase() == target_lower);
        if name_match || alias_match {
            matches.push(server.clone());
        }
    }

    match matches.len() {
        0 => Err(format!("未找到服务器: {}", target)),
        1 => Ok(matches.remove(0)),
        _ => Err(format!(
            "服务器引用不唯一: {} -> {}",
            target,
            matches
                .iter()
                .map(|server| format!("{}({})", server.name, server.id))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_server_reference_from_servers;
    use crate::models::server::{
        CpuPolicyConfig, JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    };

    fn sample_local_server(name: &str, alias: &str) -> ServerInstance {
        ServerInstance {
            id: format!("{}-id", name),
            name: name.to_string(),
            aliases: vec![alias.to_string()],
            core_type: "fabric".to_string(),
            core_version: "".to_string(),
            mc_version: "1.20.1".to_string(),
            path: format!("E:/servers/{}", name),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 1,
            last_started_at: None,
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: "E:/servers/test/server.jar".to_string(),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn resolve_server_reference_matches_alias() {
        let servers = vec![sample_local_server("fabric-main", "fabric")];
        let server = resolve_server_reference_from_servers(&servers, "fabric").unwrap();
        assert_eq!(server.name, "fabric-main");
    }

    #[test]
    fn resolve_server_reference_rejects_ambiguous_alias() {
        let servers = vec![
            sample_local_server("fabric-a", "shared"),
            sample_local_server("fabric-b", "shared"),
        ];
        let err = resolve_server_reference_from_servers(&servers, "shared").unwrap_err();
        assert!(err.contains("不唯一"));
    }
}
