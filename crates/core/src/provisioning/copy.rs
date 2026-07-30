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
        return Err(CopyInstanceError::SourceDirectoryMismatch);
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
    SourceDirectoryMismatch,
    Import(InstanceImportError),
}

impl std::fmt::Display for CopyInstanceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySourceDirectory => {
                write!(formatter, "copy source directory cannot be empty")
            }
            Self::SourceDirectoryMismatch => {
                write!(formatter, "copy source directory must match the import source directory")
            }
            Self::Import(error) => write!(formatter, "invalid copy import: {error}"),
        }
    }
}

impl std::error::Error for CopyInstanceError {}
