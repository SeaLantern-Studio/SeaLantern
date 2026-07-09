pub(super) use server_local_setup::inspect_local_folder;

#[cfg(test)]
mod tests {
    use super::inspect_local_folder;
    use tempfile::tempdir;

    #[test]
    fn inspect_local_folder_detects_attachable_script_and_metadata() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("paper-prod-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::write(folder.join("start.sh"), b"#!/bin/sh\n").expect("script should write");

        let inspection = inspect_local_folder(&folder);
        assert!(inspection.is_attachable());
        assert_eq!(inspection.startup_mode.as_deref(), Some("sh"));
        assert_eq!(inspection.inferred_core_type.as_deref(), Some("paper"));
        assert_eq!(inspection.inferred_mc_version.as_deref(), Some("1.21.1"));
    }

    #[test]
    fn inspect_local_folder_detects_attachable_jar_without_script() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("fabric-1.20.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        std::fs::write(folder.join("fabric-server.jar"), b"placeholder").expect("jar should write");

        let inspection = inspect_local_folder(&folder);
        assert!(inspection.is_attachable());
        assert_eq!(inspection.startup_mode.as_deref(), Some("jar"));
        assert!(inspection.startup_entry_path.is_none());
        assert!(inspection.detected_jar_path.is_some());
        assert_eq!(inspection.inferred_core_type.as_deref(), Some("fabric"));
        assert_eq!(inspection.inferred_mc_version.as_deref(), Some("1.20.1"));
    }

    #[test]
    fn inspect_local_folder_rejects_empty_folder_shape() {
        let temp_dir = tempdir().expect("temp dir should exist");

        let inspection = inspect_local_folder(temp_dir.path());
        assert!(!inspection.is_attachable());
        assert!(inspection.startup_mode.is_none());
    }
}
