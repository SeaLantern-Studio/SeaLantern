use super::{Instance, InstanceId};

/// 实例可被人类输入解析的稳定身份信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceIdentity {
    pub id: InstanceId,
    pub name: String,
    pub aliases: Vec<String>,
}

impl InstanceIdentity {
    /// 从已验证实例生成身份快照。
    pub fn from_instance(instance: &Instance) -> Self {
        Self {
            id: instance.id.clone(),
            name: instance.name.clone(),
            aliases: instance.aliases.clone(),
        }
    }

    /// 判断 ID、名称或别名是否匹配给定查询。
    pub fn matches(&self, query: &str) -> bool {
        let query = query.trim();
        !query.is_empty()
            && (self.id.as_str().eq_ignore_ascii_case(query)
                || self.name.eq_ignore_ascii_case(query)
                || self
                    .aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(query)))
    }
}

#[cfg(test)]
mod tests {
    use super::InstanceIdentity;
    use crate::instance::{Instance, InstanceId, InstanceSpec, LocalLaunch, StartupMode};
    use std::path::PathBuf;

    fn instance() -> Instance {
        Instance::new(InstanceSpec {
            id: InstanceId::new("paper-main").unwrap(),
            name: "Paper Main".into(),
            aliases: vec!["production".into()],
            core_type: "paper".into(),
            core_version: String::new(),
            game_version: "1.21.1".into(),
            directory: PathBuf::from("servers/paper-main"),
            port: 25565,
            max_memory_mib: 0,
            min_memory_mib: 0,
            created_at_unix_secs: 0,
            last_started_at_unix_secs: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(PathBuf::from("servers/paper-main/server.jar")),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        })
        .unwrap()
    }

    #[test]
    fn identity_matches_id_name_and_alias_case_insensitively() {
        let identity = InstanceIdentity::from_instance(&instance());

        assert!(identity.matches("PAPER-MAIN"));
        assert!(identity.matches("paper main"));
        assert!(identity.matches("Production"));
        assert!(!identity.matches("unknown"));
    }
}
