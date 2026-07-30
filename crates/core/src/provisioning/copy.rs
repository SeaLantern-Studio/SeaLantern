use std::path::PathBuf;

use crate::instance::{
    plan_import, InstanceImportError, InstanceImportPlan, InstanceImportRequest,
};

/// 复制已有目录到受管实例目录的无副作用计划。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyInstanceRequest {
    pub source_directory: PathBuf,
    pub import: InstanceImportRequest,
}

/// 复制计划。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyInstancePlan {
    pub source_directory: PathBuf,
    pub destination_directory: PathBuf,
    pub import: InstanceImportPlan,
}

pub fn plan_copy(request: CopyInstanceRequest) -> Result<CopyInstancePlan, CopyInstanceError> {
    if request.source_directory.as_os_str().is_empty() {
        return Err(CopyInstanceError::EmptySourceDirectory);
    }
    if request.source_directory != request.import.source_directory {
        return Err(CopyInstanceError::SourceDirectoryMismatch {
            source_directory: request.source_directory,
            import_source_directory: request.import.source_directory,
        });
    }

    let import = plan_import(request.import).map_err(CopyInstanceError::Import)?;
    Ok(CopyInstancePlan {
        source_directory: request.source_directory,
        destination_directory: import.destination_directory.clone(),
        import,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopyInstanceError {
    EmptySourceDirectory,
    SourceDirectoryMismatch {
        source_directory: PathBuf,
        import_source_directory: PathBuf,
    },
    Import(InstanceImportError),
}

impl std::fmt::Display for CopyInstanceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySourceDirectory => {
                write!(formatter, "copy source directory cannot be empty")
            }
            Self::SourceDirectoryMismatch {
                source_directory,
                import_source_directory,
            } => write!(
                formatter,
                "copy source directory {} must match import source directory {}",
                source_directory.display(),
                import_source_directory.display()
            ),
            Self::Import(error) => write!(formatter, "invalid copy import: {error}"),
        }
    }
}

impl std::error::Error for CopyInstanceError {}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{plan_copy, CopyInstanceError, CopyInstanceRequest};
    use crate::instance::{
        InstanceId, InstanceImportRequest, InstanceSpec, LocalLaunch, StartupMode,
    };

    fn import_request(source_directory: PathBuf) -> InstanceImportRequest {
        InstanceImportRequest {
            source_directory: source_directory.clone(),
            instance: InstanceSpec {
                id: InstanceId::new("copied").unwrap(),
                name: "Copied".into(),
                aliases: Vec::new(),
                core_type: "paper".into(),
                core_version: String::new(),
                game_version: "1.21.1".into(),
                directory: PathBuf::from("managed/copied"),
                port: 25565,
                max_memory_mib: 0,
                min_memory_mib: 0,
                created_at_unix_secs: 0,
                last_started_at_unix_secs: None,
                launch: LocalLaunch {
                    startup_mode: StartupMode::Jar,
                    startup_target: Some(source_directory.join("server.jar")),
                    custom_command: None,
                    custom_executable: None,
                    custom_arguments: Vec::new(),
                    java_executable: None,
                    jvm_arguments: Vec::new(),
                },
            },
        }
    }

    #[test]
    fn copy_plan_reports_both_conflicting_source_directories() {
        let source_directory = PathBuf::from("staging/copied");
        let import_source_directory = PathBuf::from("staging/imported");
        let error = plan_copy(CopyInstanceRequest {
            source_directory: source_directory.clone(),
            import: import_request(import_source_directory.clone()),
        })
        .unwrap_err();

        assert_eq!(
            error,
            CopyInstanceError::SourceDirectoryMismatch {
                source_directory,
                import_source_directory,
            }
        );
    }
}
