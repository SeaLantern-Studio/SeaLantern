//! 服务器实例持久化模型。

use sealantern_core::instance::{Instance, InstanceId, InstanceSpec, LocalLaunch};
use serde::{Deserialize, Serialize};

/// 当前实例列表持久化格式版本。
///
/// 对 [`InstanceList`] 或 [`Instance`] 的持久化结构做**不向后兼容**的改动时递增此值，
/// 并同步在实例存储的加载路径上补一段对应版本的升级逻辑，否则旧数据会因字段不匹配
/// 而读取失败，表现为"所有服务器消失"。
///
/// - 版本 `0`：早于版本化改造的数据（对象形式但缺 `version` 字段）。
/// - 版本 `1`：`{ "version": 1, "instances": [...] }` 对象形式；1.2.0 的裸数组在
///   加载时被迁移为该形式。
pub const CURRENT_INSTANCE_SCHEMA_VERSION: u32 = 1;

/// `version` 字段缺失时采用的历史版本号。
///
/// 反序列化时缺字段说明数据早于版本化改造，必须按最旧的版本处理，才能在升级链里
/// 逐级迁移；若直接按当前版本处理，将来的新版会误判旧数据从而跳过迁移步骤。
///
/// 实例存储的版本预读复用了这一常量，保证「缺字段按版本 0 处理」只有一个事实来源。
pub(crate) fn legacy_schema_version() -> u32 {
    0
}

/// 实例列表的持久化包装。
///
/// 实例本体由 `sealantern-core` 维护，`feature` 只拥有存储格式版本和集合边界。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceList {
    /// 持久化格式版本。
    ///
    /// 从磁盘读出且缺失该字段时按 [`legacy_schema_version`] 处理；新建列表时使用
    /// [`CURRENT_INSTANCE_SCHEMA_VERSION`]。
    #[serde(default = "legacy_schema_version")]
    pub version: u32,
    #[serde(default)]
    pub instances: Vec<Instance>,
}

impl Default for InstanceList {
    fn default() -> Self {
        Self {
            version: CURRENT_INSTANCE_SCHEMA_VERSION,
            instances: Vec::new(),
        }
    }
}

/// 1.2.0 旧版实例记录（扁平结构，直接持久化为裸数组）。
///
/// 仅用于升级迁移的反序列化，不参与新格式写入。
#[derive(Debug, Deserialize)]
pub(crate) struct LegacyServerInstance {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) core_type: String,
    #[serde(default)]
    pub(crate) core_version: String,
    pub(crate) mc_version: String,
    pub(crate) path: String,
    pub(crate) jar_path: String,
    #[serde(default = "default_startup_mode")]
    pub(crate) startup_mode: String,
    #[serde(default)]
    pub(crate) custom_command: Option<String>,
    #[serde(default)]
    pub(crate) java_path: String,
    pub(crate) max_memory: u32,
    pub(crate) min_memory: u32,
    #[serde(default)]
    pub(crate) jvm_args: Vec<String>,
    pub(crate) port: u16,
    pub(crate) created_at: u64,
    #[serde(default)]
    pub(crate) last_started_at: Option<u64>,
}

fn default_startup_mode() -> String {
    "jar".to_string()
}

impl InstanceList {
    /// 将旧版扁平实例记录迁移为新版实例列表。
    ///
    /// 字段映射：`mc_version → game_version`、`path → directory`、
    /// `jar_path → launch.startup_target`、`java_path → launch.java_executable`、
    /// `max_memory → max_memory_mib` 等。无法构造的旧记录被跳过并记录日志。
    pub(crate) fn migrate_legacy(records: Vec<LegacyServerInstance>) -> Self {
        let mut instances = Vec::new();
        for record in records {
            let launch = LocalLaunch {
                startup_mode: sealantern_core::instance::StartupMode::parse(&record.startup_mode)
                    .unwrap_or(sealantern_core::instance::StartupMode::Jar),
                startup_target: if record.jar_path.is_empty() {
                    None
                } else {
                    Some(record.jar_path.into())
                },
                custom_command: record.custom_command,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: if record.java_path.is_empty() {
                    None
                } else {
                    Some(record.java_path.into())
                },
                jvm_arguments: record.jvm_args,
            };
            let spec = InstanceSpec {
                id: match InstanceId::new(record.id.clone()) {
                    Ok(id) => id,
                    Err(error) => {
                        tracing::warn!(
                            target: "sealantern.feature.config",
                            id = %record.id,
                            error = %error,
                            "跳过旧版实例迁移：无效的实例 ID"
                        );
                        continue;
                    }
                },
                name: record.name,
                aliases: Vec::new(),
                core_type: record.core_type,
                core_version: record.core_version,
                game_version: record.mc_version,
                directory: record.path.into(),
                port: record.port,
                max_memory_mib: record.max_memory,
                min_memory_mib: record.min_memory,
                created_at_unix_secs: record.created_at,
                last_started_at_unix_secs: record.last_started_at,
                server_metadata: None,
                launch,
            };
            match Instance::new(spec) {
                Ok(instance) => instances.push(instance),
                Err(error) => {
                    tracing::warn!(
                            target: "sealantern.feature.config",
                        id = %record.id,
                        error = %error,
                        "跳过旧版实例迁移：实例校验失败"
                    );
                }
            }
        }
        tracing::info!(
            target: "sealantern.feature.config",
            migrated = instances.len(),
            "旧版服务器记录迁移完成"
        );
        Self {
            version: CURRENT_INSTANCE_SCHEMA_VERSION,
            instances,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_plain_array_migrates_to_instance_list() {
        let json = r#"[
          {
            "id": "srv-1",
            "name": "My Server",
            "core_type": "paper",
            "core_version": "1.20.4",
            "mc_version": "1.20.4",
            "path": "D:\\MCServers\\A",
            "jar_path": "D:\\MCServers\\A\\server.jar",
            "startup_mode": "jar",
            "custom_command": null,
            "java_path": "D:\\Java\\jdk-21\\bin\\java.exe",
            "max_memory": 2048,
            "min_memory": 512,
            "jvm_args": [],
            "port": 25565,
            "created_at": 1786865648,
            "last_started_at": 1786865895
          }
        ]"#;
        let records: Vec<LegacyServerInstance> =
            serde_json::from_str(json).expect("parse legacy records");
        let list = InstanceList::migrate_legacy(records);
        assert_eq!(list.version, CURRENT_INSTANCE_SCHEMA_VERSION);
        assert_eq!(list.instances.len(), 1);
        let instance = &list.instances[0];
        assert_eq!(instance.name, "My Server");
        assert_eq!(instance.game_version, "1.20.4");
        assert_eq!(instance.directory.to_string_lossy(), "D:\\MCServers\\A");
        assert_eq!(
            instance
                .launch
                .startup_target
                .as_deref()
                .map(|p| p.to_string_lossy().to_string()),
            Some("D:\\MCServers\\A\\server.jar".to_string())
        );
        assert_eq!(
            instance
                .launch
                .java_executable
                .as_deref()
                .map(|p| p.to_string_lossy().to_string()),
            Some("D:\\Java\\jdk-21\\bin\\java.exe".to_string())
        );
        assert_eq!(instance.max_memory_mib, 2048);
        assert_eq!(instance.min_memory_mib, 512);
    }

    #[test]
    fn current_object_format_still_deserializes() {
        let json = r#"{
          "version": 1,
          "instances": []
        }"#;
        let list: InstanceList = serde_json::from_str(json).expect("new format");
        assert_eq!(list.version, 1);
        assert!(list.instances.is_empty());
    }

    #[test]
    fn invalid_legacy_entries_are_skipped_not_fatal() {
        let json = r#"[
          {
            "id": "",
            "name": "",
            "core_type": "paper",
            "mc_version": "1.20.4",
            "path": "",
            "jar_path": "",
            "port": 0,
            "max_memory": 0,
            "min_memory": 0,
            "created_at": 0
          }
        ]"#;
        let records: Vec<LegacyServerInstance> =
            serde_json::from_str(json).expect("parse legacy records");
        let list = InstanceList::migrate_legacy(records);
        assert!(list.instances.is_empty(), "invalid entries should be skipped");
    }
}
