use std::fmt;
use std::path::PathBuf;

use super::model::{Instance, InstanceError, InstanceSpec};

/// 用于规划主机管理的本地实例导入的输入。
///
/// 此处不读取源目录和启动目标。在此计划确定了相对于目标目录的启动目标后，主机将验证并复制它们。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceImportRequest {
    pub source_directory: PathBuf,
    pub instance: InstanceSpec,
}

/// 一个已验证的导入计划，包含供主机适配器持久化的已复制实例模型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceImportPlan {
    pub source_directory: PathBuf,
    pub destination_directory: PathBuf,
    pub startup_target_relative: Option<PathBuf>,
    pub instance: Instance,
}

/// 构建导入计划，不执行文件系统操作。
pub fn plan_import(
    request: InstanceImportRequest,
) -> Result<InstanceImportPlan, InstanceImportError> {
    let InstanceImportRequest { source_directory, mut instance } = request;

    if source_directory.as_os_str().is_empty() {
        return Err(InstanceImportError::EmptySourceDirectory);
    }
    if instance.directory.as_os_str().is_empty() {
        return Err(InstanceImportError::EmptyDestinationDirectory);
    }

    let source_startup_target = instance
        .launch
        .normalize_and_validate()
        .map_err(InstanceImportError::Instance)?;
    let startup_target_relative = match source_startup_target {
        Some(source_target) => {
            let relative = source_target.strip_prefix(&source_directory).map_err(|_| {
                InstanceImportError::StartupTargetOutsideSource {
                    source_directory: source_directory.clone(),
                    startup_target: source_target.clone(),
                }
            })?;
            if relative.as_os_str().is_empty() {
                return Err(InstanceImportError::StartupTargetMustBeBelowSource {
                    source_directory: source_directory.clone(),
                });
            }

            let relative = relative.to_path_buf();
            instance.launch.startup_target = Some(instance.directory.join(&relative));
            Some(relative)
        }
        None => None,
    };

    let instance = Instance::new(instance).map_err(InstanceImportError::Instance)?;

    Ok(InstanceImportPlan {
        source_directory,
        destination_directory: instance.directory.clone(),
        startup_target_relative,
        instance,
    })
}

/// 描述无法创建实例导入计划的原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceImportError {
    EmptySourceDirectory,
    EmptyDestinationDirectory,
    StartupTargetMustBeBelowSource {
        source_directory: PathBuf,
    },
    StartupTargetOutsideSource {
        source_directory: PathBuf,
        startup_target: PathBuf,
    },
    Instance(InstanceError),
}

impl fmt::Display for InstanceImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySourceDirectory => {
                write!(formatter, "import source directory cannot be empty")
            }
            Self::EmptyDestinationDirectory => {
                write!(formatter, "import destination directory cannot be empty")
            }
            Self::StartupTargetMustBeBelowSource { source_directory } => write!(
                formatter,
                "import startup target must be a path below source directory {}; the host verifies file type",
                source_directory.display()
            ),
            Self::StartupTargetOutsideSource { source_directory, startup_target } => write!(
                formatter,
                "startup target {} is outside import source directory {}",
                startup_target.display(),
                source_directory.display()
            ),
            Self::Instance(error) => write!(formatter, "invalid imported instance: {error}"),
        }
    }
}

impl std::error::Error for InstanceImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Instance(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{plan_import, InstanceImportError, InstanceImportRequest};
    use crate::instance::{InstanceError, InstanceId, InstanceSpec, LocalLaunch, StartupMode};

    fn import_spec(startup_target: Option<PathBuf>, startup_mode: StartupMode) -> InstanceSpec {
        InstanceSpec {
            id: InstanceId::new("imported-a").expect("instance ID should be valid"),
            name: "Imported".to_string(),
            aliases: Vec::new(),
            core_type: "paper".to_string(),
            core_version: String::new(),
            game_version: "unknown".to_string(),
            directory: PathBuf::from("managed/imported-a"),
            port: 25565,
            max_memory_mib: 4096,
            min_memory_mib: 1024,
            created_at_unix_secs: 100,
            last_started_at_unix_secs: None,
            launch: LocalLaunch {
                startup_mode,
                startup_target,
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: Some(PathBuf::from("java")),
                jvm_arguments: Vec::new(),
            },
        }
    }

    #[test]
    fn import_plan_rewrites_a_nested_startup_target_under_the_destination() {
        let source_directory = PathBuf::from("imports/paper");
        let startup_target = source_directory.join("libraries/server.jar");
        let request = InstanceImportRequest {
            source_directory: source_directory.clone(),
            instance: import_spec(Some(startup_target), StartupMode::Jar),
        };

        let plan = plan_import(request).expect("import should plan");

        assert_eq!(plan.startup_target_relative, Some(PathBuf::from("libraries/server.jar")));
        assert_eq!(plan.instance.directory, Path::new("managed/imported-a"));
        assert_eq!(
            plan.instance.launch.startup_target,
            Some(PathBuf::from("managed/imported-a/libraries/server.jar"))
        );
    }

    #[test]
    fn import_plan_rejects_a_startup_target_outside_the_source() {
        let request = InstanceImportRequest {
            source_directory: PathBuf::from("imports/paper"),
            instance: import_spec(
                Some(PathBuf::from("imports/shared/server.jar")),
                StartupMode::Jar,
            ),
        };

        let error = plan_import(request).expect_err("outside target must fail");

        assert!(matches!(error, InstanceImportError::StartupTargetOutsideSource { .. }));
    }

    #[test]
    fn custom_import_needs_no_startup_target() {
        let mut spec = import_spec(None, StartupMode::Custom);
        spec.launch.custom_command = Some("launch-custom --nogui".to_string());
        let request = InstanceImportRequest {
            source_directory: PathBuf::from("imports/custom"),
            instance: spec,
        };

        let plan = plan_import(request).expect("custom import should plan");

        assert!(plan.startup_target_relative.is_none());
        assert!(plan.instance.launch.startup_target.is_none());
        assert_eq!(plan.instance.launch.custom_command.as_deref(), Some("launch-custom --nogui"));
    }

    #[test]
    fn import_uses_the_shared_launch_validation_for_missing_targets() {
        let request = InstanceImportRequest {
            source_directory: PathBuf::from("imports/paper"),
            instance: import_spec(None, StartupMode::Jar),
        };

        let error = plan_import(request).expect_err("non-custom imports require a startup target");

        assert!(matches!(
            error,
            InstanceImportError::Instance(InstanceError::MissingStartupTarget {
                mode: StartupMode::Jar
            })
        ));
    }
}
