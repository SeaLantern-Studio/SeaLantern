use std::path::PathBuf;

use crate::instance::{Instance, InstanceError, InstanceSpec};

use super::{resolve_run_directory, RunDirectoryError, RunDirectoryState};

/// 整合包导入的无副作用输入。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModpackProvisionRequest {
    pub archive_path: PathBuf,
    pub requested_run_directory: PathBuf,
    pub run_directory_state: RunDirectoryState,
    pub instance: InstanceSpec,
}

/// 整合包导入计划。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModpackProvisionPlan {
    pub archive_path: PathBuf,
    pub run_directory: PathBuf,
    pub instance: Instance,
}

/// 校验整合包来源、运行目录与实例目标的一致性。
pub fn plan_modpack(
    request: ModpackProvisionRequest,
) -> Result<ModpackProvisionPlan, ModpackProvisionError> {
    tracing::debug!(
        target: "sealantern.core.provisioning.modpack",
        archive_path = %request.archive_path.display(),
        requested_run_directory = %request.requested_run_directory.display(),
        "planning modpack provision"
    );
    if request.archive_path.as_os_str().is_empty() {
        return Err(ModpackProvisionError::EmptyArchivePath);
    }

    let run_directory =
        resolve_run_directory(request.requested_run_directory, request.run_directory_state)
            .map_err(ModpackProvisionError::RunDirectory)?;
    if request.instance.directory != run_directory {
        return Err(ModpackProvisionError::InstanceDirectoryMismatch {
            instance_directory: request.instance.directory,
            run_directory,
        });
    }

    let instance = Instance::new(request.instance).map_err(ModpackProvisionError::Instance)?;
    let plan = ModpackProvisionPlan {
        archive_path: request.archive_path,
        run_directory,
        instance,
    };
    tracing::debug!(
        target: "sealantern.core.provisioning.modpack",
        archive_path = %plan.archive_path.display(),
        run_directory = %plan.run_directory.display(),
        instance_id = %plan.instance.id.as_str(),
        "modpack provision plan ready"
    );
    Ok(plan)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModpackProvisionError {
    EmptyArchivePath,
    RunDirectory(RunDirectoryError),
    InstanceDirectoryMismatch {
        instance_directory: PathBuf,
        run_directory: PathBuf,
    },
    Instance(InstanceError),
}

impl std::fmt::Display for ModpackProvisionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyArchivePath => write!(formatter, "modpack archive path cannot be empty"),
            Self::RunDirectory(error) => {
                write!(formatter, "invalid modpack run directory: {error}")
            }
            Self::InstanceDirectoryMismatch { instance_directory, run_directory } => write!(
                formatter,
                "instance directory {} must match modpack run directory {}",
                instance_directory.display(),
                run_directory.display()
            ),
            Self::Instance(error) => write!(formatter, "invalid modpack instance: {error}"),
        }
    }
}

impl std::error::Error for ModpackProvisionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RunDirectory(error) => Some(error),
            Self::Instance(error) => Some(error),
            Self::EmptyArchivePath | Self::InstanceDirectoryMismatch { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::path::PathBuf;

    use super::{plan_modpack, ModpackProvisionError, ModpackProvisionRequest};
    use crate::instance::{InstanceId, InstanceSpec, LocalLaunch, StartupMode};
    use crate::provisioning::{RunDirectoryError, RunDirectoryState};

    fn instance_spec(directory: PathBuf) -> InstanceSpec {
        InstanceSpec {
            id: InstanceId::new("modpack").unwrap(),
            name: "Modpack".into(),
            aliases: Vec::new(),
            core_type: "neoforge".into(),
            core_version: String::new(),
            game_version: "1.21.1".into(),
            directory: directory.clone(),
            port: 25565,
            max_memory_mib: 0,
            min_memory_mib: 0,
            created_at_unix_secs: 0,
            last_started_at_unix_secs: None,
            server_metadata: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(directory.join("server.jar")),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        }
    }

    #[test]
    fn modpack_plan_preserves_the_explicit_run_directory() {
        let run_directory = PathBuf::from("E:/servers/neoforge");
        let plan = plan_modpack(ModpackProvisionRequest {
            archive_path: PathBuf::from("E:/downloads/neoforge.zip"),
            requested_run_directory: run_directory.clone(),
            run_directory_state: RunDirectoryState::Missing,
            instance: instance_spec(run_directory.clone()),
        })
        .unwrap();

        assert_eq!(plan.run_directory, run_directory);
        assert_eq!(plan.instance.directory, plan.run_directory);
    }

    #[test]
    fn modpack_plan_rejects_a_different_instance_directory() {
        let error = plan_modpack(ModpackProvisionRequest {
            archive_path: PathBuf::from("E:/downloads/neoforge.zip"),
            requested_run_directory: PathBuf::from("E:/servers/selected"),
            run_directory_state: RunDirectoryState::Missing,
            instance: instance_spec(PathBuf::from("E:/servers/other")),
        })
        .unwrap_err();

        assert!(matches!(error, ModpackProvisionError::InstanceDirectoryMismatch { .. }));
    }

    #[test]
    fn modpack_error_exposes_the_run_directory_source() {
        let error = ModpackProvisionError::RunDirectory(RunDirectoryError::NonEmpty {
            path: PathBuf::from("E:/servers/existing"),
        });

        assert!(error.source().is_some());
    }
}
