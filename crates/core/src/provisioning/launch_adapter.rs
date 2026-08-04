use std::path::Path;

use crate::instance::{LocalLaunch, StartupMode};

use super::server_inspection::{LaunchProfile, LaunchTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LaunchAdapterError {
    ProgramArguments,
    ScriptType,
    MainClass,
    ArgumentFiles,
}

impl LaunchAdapterError {
    pub(super) const fn diagnostic(self) -> (&'static str, &'static str) {
        match self {
            Self::ProgramArguments => (
                "launch_profile_program_arguments_unsupported",
                "launch profile has program arguments that LocalLaunch cannot represent; choose it manually",
            ),
            Self::ScriptType => (
                "launch_profile_script_type_unsupported",
                "launch profile script extension is not supported by LocalLaunch; choose it manually",
            ),
            Self::MainClass => (
                "launch_profile_main_class_unrepresentable",
                "LocalLaunch cannot represent a main-class target; choose a concrete launcher manually",
            ),
            Self::ArgumentFiles => (
                "launch_profile_argument_files_unrepresentable",
                "LocalLaunch cannot represent argument-file targets; choose a concrete launcher manually",
            ),
        }
    }
}

pub(super) fn adapt_launch_profile(
    profile: &LaunchProfile,
) -> Result<LocalLaunch, LaunchAdapterError> {
    let (startup_mode, startup_target, jvm_arguments) = match &profile.target {
        LaunchTarget::Jar { path } => {
            if !profile.program_arguments.is_empty() {
                return Err(LaunchAdapterError::ProgramArguments);
            }
            (StartupMode::Jar, path.clone(), profile.jvm_arguments.to_vec())
        }
        LaunchTarget::Script { path } => {
            let Some(mode) = script_startup_mode(path) else {
                return Err(LaunchAdapterError::ScriptType);
            };
            (mode, path.clone(), Vec::new())
        }
        LaunchTarget::MainClass { .. } => return Err(LaunchAdapterError::MainClass),
        LaunchTarget::ArgumentFiles { .. } => return Err(LaunchAdapterError::ArgumentFiles),
    };

    Ok(LocalLaunch {
        startup_mode,
        startup_target: Some(startup_target),
        custom_command: None,
        custom_executable: None,
        custom_arguments: Vec::new(),
        java_executable: None,
        jvm_arguments,
    })
}

fn script_startup_mode(path: &Path) -> Option<StartupMode> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "bat" | "cmd" => Some(StartupMode::Batch),
        "sh" => Some(StartupMode::Shell),
        "ps1" => Some(StartupMode::PowerShell),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{adapt_launch_profile, LaunchAdapterError};
    use crate::provisioning::server_inspection::{LaunchPlatform, LaunchProfile, LaunchTarget};

    fn profile(target: LaunchTarget) -> LaunchProfile {
        LaunchProfile {
            id: "test".to_string(),
            platform: LaunchPlatform::Any,
            working_directory: None,
            target,
            jvm_arguments: Vec::new(),
            program_arguments: Vec::new(),
            required_java_major: None,
        }
    }

    #[test]
    fn rejects_targets_without_a_local_launch_representation() {
        assert_eq!(
            adapt_launch_profile(&profile(LaunchTarget::MainClass {
                class_name: "example.Main".to_string(),
            })),
            Err(LaunchAdapterError::MainClass)
        );
        assert_eq!(
            adapt_launch_profile(&profile(LaunchTarget::ArgumentFiles {
                paths: vec![PathBuf::from("args.txt")],
            })),
            Err(LaunchAdapterError::ArgumentFiles)
        );
    }
}
