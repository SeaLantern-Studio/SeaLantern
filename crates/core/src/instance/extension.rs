use std::path::PathBuf;

/// 实例目录中可管理的扩展类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceExtensionKind {
    Plugin,
    Mod,
    Datapack,
}

/// 单个实例扩展的只读描述。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceExtension {
    pub kind: InstanceExtensionKind,
    pub file_name: String,
    pub path: PathBuf,
    pub enabled: bool,
}

impl InstanceExtension {
    pub fn new(
        kind: InstanceExtensionKind,
        file_name: impl Into<String>,
        path: PathBuf,
        enabled: bool,
    ) -> Result<Self, InstanceExtensionError> {
        let file_name = file_name.into().trim().to_string();
        if file_name.is_empty() {
            return Err(InstanceExtensionError::EmptyFileName);
        }
        if path.as_os_str().is_empty() {
            return Err(InstanceExtensionError::EmptyPath);
        }
        Ok(Self { kind, file_name, path, enabled })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceExtensionError {
    EmptyFileName,
    EmptyPath,
}

impl std::fmt::Display for InstanceExtensionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyFileName => write!(formatter, "extension file name cannot be empty"),
            Self::EmptyPath => write!(formatter, "extension path cannot be empty"),
        }
    }
}

impl std::error::Error for InstanceExtensionError {}
