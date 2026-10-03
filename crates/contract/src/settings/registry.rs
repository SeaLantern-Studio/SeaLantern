//! 实例注册表设置分区。
//!
//! 记录主资源目录、附加服务器目录与信任状态。实例本身的权威配置位于各服务器
//! 目录内的 `sl.json`；本分区只保存「哪些目录被本机管理」这类机器级信息。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::app::SettingsValidationError;

/// 实例注册表：主资源目录、附加服务器目录与信任状态。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct InstanceRegistrySection {
    /// 主资源目录；`None` 表示与主配置目录同址。
    pub main_resource_dir: Option<PathBuf>,

    /// 附加服务器目录；每个目录本身就是一个服务器。
    pub extra_server_dirs: Vec<PathBuf>,

    /// 已信任的实例标识。
    ///
    /// 信任绑定的是 `sl.json` 里的实例 id 而非目录：实例目录移动后信任关系
    /// 跟随文档走，新目录凭同一 id 继承管理状态（信任的是"这个服务器"，
    /// 不是"这个位置"）。
    pub trusted_instances: Vec<String>,

    /// 用户明确忽略、不再提示的目录。
    ///
    /// 忽略绑定的是目录路径而非实例 id：同一份目录拷贝到别处会被当作新的
    /// 待处理实例重新提示（忽略的是"这个位置的目录"）。
    pub ignored_dirs: Vec<PathBuf>,
}

impl InstanceRegistrySection {
    /// 校验注册表分区的业务约束。
    ///
    /// 空路径条目与空实例标识会静默破坏目录匹配与信任判定，重复条目则
    /// 暗示名单维护逻辑出错——两者都在写入前拒绝，与 `AppSettings::validate`
    /// 保持同一校验风格。
    pub fn validate(&self) -> Result<(), SettingsValidationError> {
        if let Some(dir) = &self.main_resource_dir {
            reject_empty_path("main_resource_dir", dir)?;
        }
        for dir in &self.extra_server_dirs {
            reject_empty_path("extra_server_dirs", dir)?;
        }
        reject_duplicates(&self.extra_server_dirs, "extra_server_dirs")?;
        for id in &self.trusted_instances {
            if id.trim().is_empty() {
                return Err(SettingsValidationError::new(
                    "trusted_instances",
                    "must not contain empty entries",
                ));
            }
        }
        reject_duplicates(&self.trusted_instances, "trusted_instances")?;
        for dir in &self.ignored_dirs {
            reject_empty_path("ignored_dirs", dir)?;
        }
        reject_duplicates(&self.ignored_dirs, "ignored_dirs")?;
        Ok(())
    }
}

/// 路径条目为空时拒绝（`Some("")` 或列表中的空项）。
fn reject_empty_path(field: &'static str, path: &Path) -> Result<(), SettingsValidationError> {
    if path.as_os_str().is_empty() {
        return Err(SettingsValidationError::new(field, "must not contain empty entries"));
    }
    Ok(())
}

/// 列表中出现重复条目时拒绝。
fn reject_duplicates<T: Eq + std::hash::Hash>(
    entries: &[T],
    field: &'static str,
) -> Result<(), SettingsValidationError> {
    let mut seen = std::collections::HashSet::with_capacity(entries.len());
    if entries.iter().any(|entry| !seen.insert(entry)) {
        return Err(SettingsValidationError::new(field, "must not contain duplicates"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let registry: InstanceRegistrySection =
            serde_json::from_str("{}").expect("empty registry should deserialize");

        assert_eq!(registry, InstanceRegistrySection::default());
    }

    #[test]
    fn registry_round_trips_through_json() {
        let registry = InstanceRegistrySection {
            main_resource_dir: Some(PathBuf::from("D:\\SeaLanternData")),
            extra_server_dirs: vec![PathBuf::from("E:\\MyServers\\paper-a")],
            trusted_instances: vec!["a1b2c3".to_string()],
            ignored_dirs: vec![PathBuf::from("F:\\skip-me")],
        };

        let json = serde_json::to_string(&registry).expect("registry should serialize");
        let restored: InstanceRegistrySection =
            serde_json::from_str(&json).expect("registry should deserialize");

        assert_eq!(registry, restored);
    }

    #[test]
    fn settings_without_a_registry_section_still_load() {
        let settings: crate::settings::AppSettings =
            serde_json::from_str("{}").expect("settings without a registry should load");

        assert_eq!(settings.registry, InstanceRegistrySection::default());
    }
}
