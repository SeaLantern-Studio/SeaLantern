use super::{
    build_install_launch_plan, get_pending_update_file, get_update_cache_dir, InstallLaunchPlan,
};
use std::path::Path;

#[test]
fn update_paths_end_with_expected_segments() {
    let cache_dir = get_update_cache_dir();
    assert!(cache_dir.ends_with(Path::new("com.fpsz.sea-lantern").join("updates")));

    let pending_file = get_pending_update_file();
    assert!(pending_file.ends_with(
        Path::new("com.fpsz.sea-lantern")
            .join("updates")
            .join("pending_update.json")
    ));
}

#[test]
fn build_install_launch_plan_for_msi() {
    let path = Path::new("E:/repo/SeaLantern/SeaLantern.msi");
    assert_eq!(
        build_install_launch_plan(path, "E:/repo/SeaLantern/SeaLantern.msi"),
        InstallLaunchPlan::ElevatedMsi {
            program: "msiexec.exe",
            args: vec![
                "/i".to_string(),
                "E:/repo/SeaLantern/SeaLantern.msi".to_string(),
                "/passive".to_string(),
                "/norestart".to_string()
            ],
        }
    );
}

#[test]
fn build_install_launch_plan_for_exe() {
    let path = Path::new("E:/repo/SeaLantern/SeaLantern.exe");
    assert_eq!(
        build_install_launch_plan(path, "E:/repo/SeaLantern/SeaLantern.exe"),
        InstallLaunchPlan::ElevatedExe {
            program: "E:/repo/SeaLantern/SeaLantern.exe".to_string(),
            args: vec!["/S".to_string(), "/norestart".to_string()],
        }
    );
}

#[test]
fn build_install_launch_plan_for_other_files() {
    let path = Path::new("E:/repo/SeaLantern/SeaLantern.zip");
    assert_eq!(
        build_install_launch_plan(path, "E:/repo/SeaLantern/SeaLantern.zip"),
        InstallLaunchPlan::OpenDirect
    );
}
