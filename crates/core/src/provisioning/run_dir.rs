use std::path::PathBuf;

/// 主机检查运行目录后传入 core 的目录状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunDirectoryState {
    Missing,
    EmptyDirectory,
    NonEmptyDirectory,
    File,
}

/// 验证用户显式选择的整合包运行目录。
///
/// core 从不追加服务端名称，避免选择路径与实际创建路径不一致。
pub fn resolve_run_directory(
    requested: impl Into<PathBuf>,
    state: RunDirectoryState,
) -> Result<PathBuf, RunDirectoryError> {
    let requested = requested.into();
    tracing::debug!(
        target: "sealantern.core.provisioning.run_directory",
        path = %requested.display(),
        state = ?state,
        "resolving modpack run directory"
    );
    if requested.as_os_str().is_empty() {
        return Err(RunDirectoryError::Empty);
    }

    match state {
        RunDirectoryState::Missing | RunDirectoryState::EmptyDirectory => Ok(requested),
        RunDirectoryState::NonEmptyDirectory => {
            Err(RunDirectoryError::NonEmpty { path: requested })
        }
        RunDirectoryState::File => Err(RunDirectoryError::NotDirectory { path: requested }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunDirectoryError {
    Empty,
    NonEmpty { path: PathBuf },
    NotDirectory { path: PathBuf },
}

impl std::fmt::Display for RunDirectoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(formatter, "run directory cannot be empty"),
            Self::NonEmpty { path } => {
                write!(formatter, "run directory is not empty: {}", path.display())
            }
            Self::NotDirectory { path } => {
                write!(formatter, "run directory points to a file: {}", path.display())
            }
        }
    }
}

impl std::error::Error for RunDirectoryError {}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{resolve_run_directory, RunDirectoryError, RunDirectoryState};

    #[test]
    fn explicit_missing_directory_is_preserved_without_appending_a_name() {
        let path = PathBuf::from("E:/servers/neoforge");
        let resolved = resolve_run_directory(path.clone(), RunDirectoryState::Missing).unwrap();

        assert_eq!(resolved, path);
    }

    #[test]
    fn non_empty_directory_is_rejected() {
        let error =
            resolve_run_directory("E:/servers/existing", RunDirectoryState::NonEmptyDirectory)
                .unwrap_err();

        assert!(matches!(error, RunDirectoryError::NonEmpty { .. }));
    }
}
