use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use crate::models::server::{ServerInstance, ServerStatus};

use super::fs::remove_run_path_mapping;
use super::ServerManager;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateServerRecordEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub runtime_kind: String,
    pub created_at: u64,
    pub last_started_at: Option<u64>,
    pub active: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DuplicateServerRecordGroup {
    pub canonical_id: String,
    pub canonical_name: String,
    pub reasons: Vec<String>,
    pub entries: Vec<DuplicateServerRecordEntry>,
    pub removable_ids: Vec<String>,
    pub blocked_ids: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerRegistryDedupeReport {
    pub total_servers: usize,
    pub duplicate_groups: Vec<DuplicateServerRecordGroup>,
    pub removed_ids: Vec<String>,
}

pub(super) fn audit_duplicate_server_records(
    manager: &ServerManager,
) -> Result<ServerRegistryDedupeReport, String> {
    let servers = manager.get_server_list();
    Ok(build_registry_dedupe_report(manager, &servers, Vec::new()))
}

pub(super) fn dedupe_duplicate_server_records(
    manager: &ServerManager,
) -> Result<ServerRegistryDedupeReport, String> {
    let servers = manager.get_server_list();
    let report = build_registry_dedupe_report(manager, &servers, Vec::new());
    let removable_ids = report
        .duplicate_groups
        .iter()
        .flat_map(|group| group.removable_ids.iter().cloned())
        .collect::<Vec<_>>();

    if removable_ids.is_empty() {
        return Ok(report);
    }

    {
        let removable_id_set = removable_ids.iter().cloned().collect::<HashSet<_>>();
        let mut locked = manager.lock_servers()?;
        locked.retain(|server| !removable_id_set.contains(&server.id));
    }

    let data_dir = manager.data_dir_value()?;
    for id in &removable_ids {
        remove_run_path_mapping(&data_dir, id);
    }
    manager.save()?;

    Ok(build_registry_dedupe_report(manager, &manager.get_server_list(), removable_ids))
}

fn build_registry_dedupe_report(
    manager: &ServerManager,
    servers: &[ServerInstance],
    removed_ids: Vec<String>,
) -> ServerRegistryDedupeReport {
    let statuses = collect_server_activity(manager, servers);
    let groups = build_connected_duplicate_groups(servers)
        .into_iter()
        .filter(|members| members.len() > 1)
        .map(|members| summarize_duplicate_group(members, &statuses))
        .collect::<Vec<_>>();

    ServerRegistryDedupeReport {
        total_servers: servers.len(),
        duplicate_groups: groups,
        removed_ids,
    }
}

fn collect_server_activity(
    manager: &ServerManager,
    servers: &[ServerInstance],
) -> HashMap<String, bool> {
    servers
        .iter()
        .map(|server| {
            let status = manager.get_server_status(&server.id).status;
            let active = matches!(
                status,
                ServerStatus::Running | ServerStatus::Starting | ServerStatus::Stopping
            );
            (server.id.clone(), active)
        })
        .collect()
}

fn build_connected_duplicate_groups(servers: &[ServerInstance]) -> Vec<Vec<&ServerInstance>> {
    let mut adjacency = HashMap::<String, HashSet<String>>::new();
    let mut by_id = HashMap::<String, &ServerInstance>::new();
    for server in servers {
        adjacency.entry(server.id.clone()).or_default();
        by_id.insert(server.id.clone(), server);
    }

    let duplicate_keys = build_duplicate_key_maps(servers);
    for map in duplicate_keys {
        for ids in map.into_values() {
            if ids.len() < 2 {
                continue;
            }
            for left in &ids {
                for right in &ids {
                    if left != right {
                        adjacency
                            .entry(left.clone())
                            .or_default()
                            .insert(right.clone());
                    }
                }
            }
        }
    }

    let mut visited = HashSet::new();
    let mut groups = Vec::new();
    for server in servers {
        if visited.contains(&server.id) {
            continue;
        }
        let mut stack = vec![server.id.clone()];
        let mut ids = Vec::new();
        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            ids.push(current.clone());
            if let Some(neighbors) = adjacency.get(&current) {
                stack.extend(neighbors.iter().cloned());
            }
        }

        if ids.len() > 1 {
            let members = ids
                .iter()
                .filter_map(|id| by_id.get(id).copied())
                .collect::<Vec<_>>();
            if members.len() > 1 {
                groups.push(members);
            }
        }
    }

    groups
}

fn build_duplicate_key_maps(servers: &[ServerInstance]) -> Vec<HashMap<String, Vec<String>>> {
    let mut name_map = HashMap::<String, Vec<String>>::new();
    let mut path_map = HashMap::<String, Vec<String>>::new();
    let mut container_map = HashMap::<String, Vec<String>>::new();

    for server in servers {
        let name_key = normalize_name(&server.name);
        if !name_key.is_empty() {
            name_map
                .entry(name_key)
                .or_default()
                .push(server.id.clone());
        }

        let path_key = normalize_path(&server.path);
        if !path_key.is_empty() {
            path_map
                .entry(path_key)
                .or_default()
                .push(server.id.clone());
        }

        if let Some(container_name) = docker_container_name(server) {
            container_map
                .entry(container_name)
                .or_default()
                .push(server.id.clone());
        }
    }

    vec![name_map, path_map, container_map]
}

fn summarize_duplicate_group(
    mut members: Vec<&ServerInstance>,
    statuses: &HashMap<String, bool>,
) -> DuplicateServerRecordGroup {
    members.sort_by_key(|server| duplicate_sort_key(server, statuses));
    let canonical = members[0];
    let mut reasons = collect_group_reasons(&members);
    reasons.sort();
    reasons.dedup();

    let entries = members
        .iter()
        .map(|server| DuplicateServerRecordEntry {
            id: server.id.clone(),
            name: server.name.clone(),
            path: server.path.clone(),
            runtime_kind: server.runtime_kind.clone(),
            created_at: server.created_at,
            last_started_at: server.last_started_at,
            active: statuses.get(&server.id).copied().unwrap_or(false),
        })
        .collect::<Vec<_>>();

    let mut removable_ids = Vec::new();
    let mut blocked_ids = Vec::new();
    for entry in entries.iter().skip(1) {
        if entry.active {
            blocked_ids.push(entry.id.clone());
        } else {
            removable_ids.push(entry.id.clone());
        }
    }

    DuplicateServerRecordGroup {
        canonical_id: canonical.id.clone(),
        canonical_name: canonical.name.clone(),
        reasons,
        entries,
        removable_ids,
        blocked_ids,
    }
}

fn duplicate_sort_key<'a>(
    server: &'a ServerInstance,
    statuses: &HashMap<String, bool>,
) -> (Reverse<bool>, Reverse<u64>, Reverse<u64>, &'a str) {
    (
        Reverse(statuses.get(&server.id).copied().unwrap_or(false)),
        Reverse(server.last_started_at.unwrap_or(0)),
        Reverse(server.created_at),
        server.id.as_str(),
    )
}

fn collect_group_reasons(members: &[&ServerInstance]) -> Vec<String> {
    let mut reasons = Vec::new();
    push_reason_if_duplicate(&mut reasons, members, |server| normalize_name(&server.name), "name");
    push_reason_if_duplicate(&mut reasons, members, |server| normalize_path(&server.path), "path");
    push_reason_if_duplicate(
        &mut reasons,
        members,
        |server| docker_container_name(server).unwrap_or_default(),
        "container_name",
    );
    reasons
}

fn push_reason_if_duplicate<F>(
    reasons: &mut Vec<String>,
    members: &[&ServerInstance],
    key_fn: F,
    reason: &str,
) where
    F: Fn(&ServerInstance) -> String,
{
    let mut seen = HashSet::new();
    for member in members {
        let key = key_fn(member);
        if key.is_empty() {
            continue;
        }
        if !seen.insert(key) {
            reasons.push(reason.to_string());
            return;
        }
    }
}

fn normalize_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn normalize_path(path: &str) -> String {
    path.trim().replace('\\', "/").to_ascii_lowercase()
}

fn docker_container_name(server: &ServerInstance) -> Option<String> {
    server
        .docker_itzg_runtime()
        .map(|runtime| runtime.container_name.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use crate::models::server::{
        CpuPolicyConfig, DockerBackendKind, DockerCommandMode, DockerItzgRuntimeConfig,
        JvmPresetConfig, LocalRuntimeConfig, ServerInstance, ServerRuntimeConfig,
    };

    use super::{
        build_connected_duplicate_groups, collect_group_reasons, summarize_duplicate_group,
    };

    fn sample_local(id: &str, name: &str, path: &str, created_at: u64) -> ServerInstance {
        ServerInstance {
            id: id.to_string(),
            name: name.to_string(),
            aliases: vec![],
            core_type: "paper".to_string(),
            core_version: "".to_string(),
            mc_version: "1.21.1".to_string(),
            path: path.to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at,
            last_started_at: Some(created_at),
            runtime_kind: "local".to_string(),
            runtime: ServerRuntimeConfig::Local(LocalRuntimeConfig {
                jar_path: format!("{}/server.jar", path),
                startup_mode: "jar".to_string(),
                custom_command: None,
                java_path: "C:/Java/bin/java.exe".to_string(),
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    fn sample_docker(id: &str, name: &str, path: &str, container: &str) -> ServerInstance {
        ServerInstance {
            id: id.to_string(),
            name: name.to_string(),
            aliases: vec![],
            core_type: "paper".to_string(),
            core_version: "".to_string(),
            mc_version: "1.21.1".to_string(),
            path: path.to_string(),
            port: 25565,
            max_memory: 4096,
            min_memory: 2048,
            created_at: 1,
            last_started_at: None,
            runtime_kind: "docker_itzg".to_string(),
            runtime: ServerRuntimeConfig::DockerItzg(DockerItzgRuntimeConfig {
                image: "itzg/minecraft-server".to_string(),
                image_tag: "latest".to_string(),
                container_name: container.to_string(),
                type_value: "PAPER".to_string(),
                version: "1.21.1".to_string(),
                data_dir_mount: path.to_string(),
                published_game_port: 25565,
                env: BTreeMap::new(),
                extra_ports: vec![],
                volume_mounts: vec![],
                docker_backend_kind: DockerBackendKind::Cli,
                command_mode: DockerCommandMode::Rcon,
                rcon: None,
                jvm_args: Vec::new(),
                cpu_policy: CpuPolicyConfig::default(),
                jvm_preset: JvmPresetConfig::default(),
            }),
        }
    }

    #[test]
    fn connected_duplicate_groups_join_name_and_path_matches() {
        let servers = vec![
            sample_local("a", "paper", "E:/servers/a", 1),
            sample_local("b", "paper", "E:/servers/b", 2),
            sample_local("c", "paper-2", "E:/servers/b", 3),
        ];

        let groups = build_connected_duplicate_groups(&servers);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 3);
    }

    #[test]
    fn collect_group_reasons_reports_name_path_and_container_signals() {
        let a = sample_docker("a", "paper", "E:/servers/paper", "sealantern-paper");
        let b = sample_docker("b", "paper", "E:/servers/other", "sealantern-paper");
        let reasons = collect_group_reasons(&[&a, &b]);

        assert!(reasons.contains(&"name".to_string()));
        assert!(reasons.contains(&"container_name".to_string()));
        assert!(!reasons.contains(&"path".to_string()));
    }

    #[test]
    fn summarize_duplicate_group_keeps_most_recent_entry() {
        let older = sample_local("old", "paper", "E:/servers/paper", 1);
        let newer = sample_local("new", "paper", "E:/servers/paper-copy", 10);
        let statuses = HashMap::from([(older.id.clone(), false), (newer.id.clone(), false)]);

        let group = summarize_duplicate_group(vec![&older, &newer], &statuses);
        assert_eq!(group.canonical_id, "new");
        assert_eq!(group.removable_ids, vec!["old".to_string()]);
    }
}
