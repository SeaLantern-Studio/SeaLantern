use std::path::{Path, PathBuf};

const UPDATE_CACHE_APP_DIR: &str = "com.fpsz.sea-lantern";
const UPDATE_CACHE_DIR_NAME: &str = "updates";
const PENDING_UPDATE_FILE_NAME: &str = "pending_update.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallLaunchPlan {
    ElevatedMsi {
        program: &'static str,
        args: Vec<String>,
    },
    ElevatedExe {
        program: String,
        args: Vec<String>,
    },
    OpenDirect,
}

pub fn get_update_cache_dir() -> PathBuf {
    let cache_dir = dirs_next::cache_dir().unwrap_or_else(std::env::temp_dir);
    cache_dir.join(UPDATE_CACHE_APP_DIR).join(UPDATE_CACHE_DIR_NAME)
}

pub fn get_pending_update_file() -> PathBuf {
    get_update_cache_dir().join(PENDING_UPDATE_FILE_NAME)
}

pub fn build_install_launch_plan(path: &Path, file_path: &str) -> InstallLaunchPlan {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    match extension.as_deref() {
        Some("msi") => InstallLaunchPlan::ElevatedMsi {
            program: "msiexec.exe",
            args: vec![
                "/i".to_string(),
                file_path.to_string(),
                "/passive".to_string(),
                "/norestart".to_string(),
            ],
        },
        Some("exe") => InstallLaunchPlan::ElevatedExe {
            program: file_path.to_string(),
            args: vec!["/S".to_string(), "/norestart".to_string()],
        },
        _ => InstallLaunchPlan::OpenDirect,
    }
}

#[cfg(test)]
#[path = "install_support_tests.rs"]
mod tests;
