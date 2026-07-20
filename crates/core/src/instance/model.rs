use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

/// A stable identifier allocated by the host for a managed instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstanceId(String);

impl InstanceId {
    pub fn new(value: impl Into<String>) -> Result<Self, InstanceError> {
        let value = value.into().trim().to_string();
        if value.is_empty() {
            return Err(InstanceError::EmptyId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The configured mechanism used to start a local instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupMode {
    Jar,
    Batch,
    Shell,
    PowerShell,
    Starter,
    Custom,
}

impl StartupMode {
    pub fn parse(value: &str) -> Result<Self, InstanceError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "jar" => Ok(Self::Jar),
            "bat" | "batch" => Ok(Self::Batch),
            "sh" | "shell" => Ok(Self::Shell),
            "ps1" | "powershell" => Ok(Self::PowerShell),
            "starter" => Ok(Self::Starter),
            "custom" => Ok(Self::Custom),
            _ => Err(InstanceError::UnsupportedStartupMode { value: value.to_string() }),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Jar => "jar",
            Self::Batch => "bat",
            Self::Shell => "sh",
            Self::PowerShell => "ps1",
            Self::Starter => "starter",
            Self::Custom => "custom",
        }
    }
}

/// Local runtime data owned by an instance, independent of process construction.
///
/// `Custom` mode accepts either legacy shell-backed `custom_command` text or a direct
/// `custom_executable` with `custom_arguments`. The two forms are mutually exclusive, and
/// arguments are valid only for a direct executable. Empty executable paths normalize to `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalLaunch {
    pub startup_mode: StartupMode,
    pub startup_target: Option<PathBuf>,
    pub custom_command: Option<String>,
    pub custom_executable: Option<PathBuf>,
    pub custom_arguments: Vec<OsString>,
    pub java_executable: Option<PathBuf>,
    pub jvm_arguments: Vec<OsString>,
}

impl LocalLaunch {
    pub(crate) fn normalize_and_validate(&mut self) -> Result<Option<PathBuf>, InstanceError> {
        self.custom_command = self
            .custom_command
            .as_deref()
            .map(str::trim)
            .filter(|command| !command.is_empty())
            .map(str::to_string);
        self.custom_executable = self
            .custom_executable
            .take()
            .filter(|path| !path.as_os_str().is_empty());

        match self.startup_mode {
            StartupMode::Custom => {
                let has_command = self.custom_command.is_some();
                let has_executable = self.custom_executable.is_some();

                if !has_command && !has_executable {
                    return Err(InstanceError::MissingCustomLaunch);
                }
                if has_command && (has_executable || !self.custom_arguments.is_empty()) {
                    return Err(InstanceError::ConflictingCustomLaunch);
                }
                if self.startup_target.is_some() {
                    return Err(InstanceError::UnexpectedStartupTarget { mode: self.startup_mode });
                }
                Ok(None)
            }
            mode => {
                if self.custom_command.is_some()
                    || self.custom_executable.is_some()
                    || !self.custom_arguments.is_empty()
                {
                    return Err(InstanceError::UnexpectedCustomLaunch { mode });
                }
                let startup_target = self
                    .startup_target
                    .clone()
                    .ok_or(InstanceError::MissingStartupTarget { mode })?;
                Ok(Some(startup_target))
            }
        }
    }
}

/// Input used to validate and construct an [`Instance`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceSpec {
    pub id: InstanceId,
    pub name: String,
    pub aliases: Vec<String>,
    pub core_type: String,
    pub core_version: String,
    pub game_version: String,
    pub directory: PathBuf,
    pub port: u16,
    pub max_memory_mib: u32,
    pub min_memory_mib: u32,
    pub created_at_unix_secs: u64,
    pub last_started_at_unix_secs: Option<u64>,
    pub launch: LocalLaunch,
}

/// A validated managed instance and its local runtime configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    pub id: InstanceId,
    pub name: String,
    pub aliases: Vec<String>,
    pub core_type: String,
    pub core_version: String,
    pub game_version: String,
    pub directory: PathBuf,
    pub port: u16,
    pub max_memory_mib: u32,
    pub min_memory_mib: u32,
    pub created_at_unix_secs: u64,
    pub last_started_at_unix_secs: Option<u64>,
    pub launch: LocalLaunch,
}

impl Instance {
    pub fn new(mut spec: InstanceSpec) -> Result<Self, InstanceError> {
        spec.name = spec.name.trim().to_string();
        if spec.name.is_empty() {
            return Err(InstanceError::EmptyName);
        }
        if spec.directory.as_os_str().is_empty() {
            return Err(InstanceError::EmptyDirectory);
        }
        if spec.port == 0 {
            return Err(InstanceError::UnsupportedPortZero);
        }
        if spec.max_memory_mib != 0
            && spec.min_memory_mib != 0
            && spec.min_memory_mib > spec.max_memory_mib
        {
            return Err(InstanceError::InvalidMemoryRange {
                min_memory_mib: spec.min_memory_mib,
                max_memory_mib: spec.max_memory_mib,
            });
        }

        spec.aliases = normalize_aliases(&spec.name, spec.aliases);
        spec.launch.normalize_and_validate()?;

        Ok(Self {
            id: spec.id,
            name: spec.name,
            aliases: spec.aliases,
            core_type: spec.core_type,
            core_version: spec.core_version,
            game_version: spec.game_version,
            directory: spec.directory,
            port: spec.port,
            max_memory_mib: spec.max_memory_mib,
            min_memory_mib: spec.min_memory_mib,
            created_at_unix_secs: spec.created_at_unix_secs,
            last_started_at_unix_secs: spec.last_started_at_unix_secs,
            launch: spec.launch,
        })
    }
}

/// Identifies invalid or internally inconsistent instance data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstanceError {
    EmptyId,
    EmptyName,
    EmptyDirectory,
    UnsupportedPortZero,
    InvalidMemoryRange {
        min_memory_mib: u32,
        max_memory_mib: u32,
    },
    UnsupportedStartupMode {
        value: String,
    },
    MissingStartupTarget {
        mode: StartupMode,
    },
    UnexpectedStartupTarget {
        mode: StartupMode,
    },
    MissingCustomLaunch,
    ConflictingCustomLaunch,
    UnexpectedCustomLaunch {
        mode: StartupMode,
    },
}

impl fmt::Display for InstanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => write!(formatter, "instance ID cannot be empty"),
            Self::EmptyName => write!(formatter, "instance name cannot be empty"),
            Self::EmptyDirectory => write!(formatter, "instance directory cannot be empty"),
            Self::UnsupportedPortZero => write!(
                formatter,
                "port 0 is not supported; managed instances require an explicit port"
            ),
            Self::InvalidMemoryRange { min_memory_mib, max_memory_mib } => write!(
                formatter,
                "minimum memory {min_memory_mib} MiB exceeds maximum memory {max_memory_mib} MiB"
            ),
            Self::UnsupportedStartupMode { value } => {
                write!(formatter, "unsupported startup mode: {value}")
            }
            Self::MissingStartupTarget { mode } => {
                write!(formatter, "{} mode requires a startup target", mode.as_str())
            }
            Self::UnexpectedStartupTarget { mode } => {
                write!(formatter, "{} mode must not define a startup target", mode.as_str())
            }
            Self::MissingCustomLaunch => {
                write!(formatter, "custom mode requires a command or executable")
            }
            Self::ConflictingCustomLaunch => {
                write!(formatter, "custom command text and executable arguments cannot be combined")
            }
            Self::UnexpectedCustomLaunch { mode } => {
                write!(formatter, "{} mode must not define custom launch data", mode.as_str())
            }
        }
    }
}

impl std::error::Error for InstanceError {}

fn normalize_aliases(instance_name: &str, aliases: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::with_capacity(aliases.len());
    for alias in aliases {
        let alias = alias.trim();
        if alias.is_empty()
            || alias.eq_ignore_ascii_case(instance_name)
            || normalized
                .iter()
                .any(|existing: &String| existing.eq_ignore_ascii_case(alias))
        {
            continue;
        }
        normalized.push(alias.to_string());
    }
    normalized
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use super::{Instance, InstanceError, InstanceId, InstanceSpec, LocalLaunch, StartupMode};

    fn base_spec() -> InstanceSpec {
        InstanceSpec {
            id: InstanceId::new("instance-a").expect("instance ID should be valid"),
            name: "Primary".to_string(),
            aliases: Vec::new(),
            core_type: "paper".to_string(),
            core_version: String::new(),
            game_version: "1.21.1".to_string(),
            directory: PathBuf::from("servers/instance-a"),
            port: 25565,
            max_memory_mib: 4096,
            min_memory_mib: 1024,
            created_at_unix_secs: 100,
            last_started_at_unix_secs: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(Path::new("servers/instance-a/server.jar").to_path_buf()),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: Some(Path::new("java").to_path_buf()),
                jvm_arguments: vec![OsString::from("-Xmx4G")],
            },
        }
    }

    #[test]
    fn instance_normalizes_aliases_and_trims_the_name() {
        let mut spec = base_spec();
        spec.name = " Primary ".to_string();
        spec.aliases = vec![
            " Alpha ".to_string(),
            "alpha".to_string(),
            "PRIMARY".to_string(),
            "".to_string(),
            "Beta".to_string(),
        ];

        let instance = Instance::new(spec).expect("instance should be valid");

        assert_eq!(instance.name, "Primary");
        assert_eq!(instance.aliases, vec!["Alpha", "Beta"]);
    }

    #[test]
    fn instance_rejects_an_invalid_memory_range() {
        let mut spec = base_spec();
        spec.min_memory_mib = 4096;
        spec.max_memory_mib = 2048;

        let error = Instance::new(spec).expect_err("memory range must be ordered");

        assert_eq!(
            error,
            InstanceError::InvalidMemoryRange {
                min_memory_mib: 4096,
                max_memory_mib: 2048,
            }
        );
    }

    #[test]
    fn instance_rejects_port_zero_instead_of_requesting_an_ephemeral_port() {
        let mut spec = base_spec();
        spec.port = 0;

        let error = Instance::new(spec).expect_err("managed instances require a fixed port");

        assert_eq!(error, InstanceError::UnsupportedPortZero);
    }

    #[test]
    fn custom_launch_requires_an_executable_and_no_startup_target() {
        let mut spec = base_spec();
        spec.launch.startup_mode = StartupMode::Custom;
        spec.launch.startup_target = None;
        spec.launch.custom_executable = Some(PathBuf::from("launch-server.exe"));
        spec.launch.custom_arguments = vec![OsString::from("--nogui")];

        let instance = Instance::new(spec).expect("custom launch should be valid");

        assert_eq!(instance.launch.custom_executable, Some(PathBuf::from("launch-server.exe")));
        assert_eq!(instance.launch.custom_arguments, vec![OsString::from("--nogui")]);
    }

    #[test]
    fn custom_launch_preserves_legacy_shell_command_text() {
        let mut spec = base_spec();
        spec.launch.startup_mode = StartupMode::Custom;
        spec.launch.startup_target = None;
        spec.launch.custom_command = Some("  java -jar server.jar  ".to_string());

        let instance = Instance::new(spec).expect("legacy custom command should be valid");

        assert_eq!(instance.launch.custom_command.as_deref(), Some("java -jar server.jar"));
    }

    #[test]
    fn empty_custom_executable_is_normalized_before_mode_validation() {
        let mut direct_jar = base_spec();
        direct_jar.launch.custom_executable = Some(PathBuf::new());

        let instance =
            Instance::new(direct_jar).expect("empty custom executable should be ignored");
        assert!(instance.launch.custom_executable.is_none());

        let mut custom = base_spec();
        custom.launch.startup_mode = StartupMode::Custom;
        custom.launch.startup_target = None;
        custom.launch.custom_executable = Some(PathBuf::new());

        let error = Instance::new(custom).expect_err("custom mode still needs a launch target");
        assert_eq!(error, InstanceError::MissingCustomLaunch);
    }

    #[test]
    fn startup_mode_parsing_rejects_unknown_modes() {
        let error = StartupMode::parse("docker").expect_err("unknown mode should fail");

        assert_eq!(error, InstanceError::UnsupportedStartupMode { value: "docker".to_string() });
    }
}
