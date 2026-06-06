pub(super) use sea_lantern_server_local_setup_core::{
    infer_core_type_from_local_inputs, infer_local_create_mc_version, infer_mc_version_from_folder,
    infer_mc_version_hint,
};

#[cfg(test)]
mod tests {
    use super::{
        infer_core_type_from_local_inputs, infer_local_create_mc_version,
        infer_mc_version_from_folder, infer_mc_version_hint,
    };
    use tempfile::tempdir;

    #[test]
    fn infer_mc_version_hint_extracts_version_from_path_like_names() {
        let version = infer_mc_version_hint(&["E:/servers/fabric-1.20.1", "cache-server"]);
        assert_eq!(version.as_deref(), Some("1.20.1"));
    }

    #[test]
    fn infer_local_folder_metadata_from_detectable_jar_name() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let jar_path = temp_dir
            .path()
            .join("fabric-server-mc.1.20.1-loader.0.15.11.jar");
        std::fs::write(&jar_path, b"placeholder").expect("test jar placeholder should write");

        let core_type = infer_core_type_from_local_inputs(temp_dir.path(), None)
            .expect("core type should infer from jar filename");
        let mc_version = infer_mc_version_from_folder(temp_dir.path(), None)
            .expect("mc version should infer from jar filename");

        assert_eq!(core_type, "Fabric");
        assert_eq!(mc_version, "1.20.1");
    }

    #[test]
    fn infer_core_type_from_folder_name_when_no_detectable_jar_exists() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let named_dir = temp_dir.path().join("1.20.1fabric");
        std::fs::create_dir_all(&named_dir).expect("named dir should create");

        let core_type = infer_core_type_from_local_inputs(&named_dir, None)
            .expect("core type should infer from folder name");
        let mc_version = infer_mc_version_from_folder(&named_dir, None)
            .expect("mc version should infer from folder name");

        assert_eq!(core_type, "Fabric");
        assert_eq!(mc_version, "1.20.1");
    }

    #[test]
    fn infer_local_create_mc_version_uses_entry_parent_and_folder_hints() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let folder = temp_dir.path().join("paper-1.21.1");
        std::fs::create_dir_all(&folder).expect("folder should create");
        let entry_path = folder.join("start.sh");
        std::fs::write(&entry_path, b"#!/bin/sh\n").expect("entry should write");

        let version = infer_local_create_mc_version(
            "server.jar",
            "paper-cache",
            Some(entry_path.to_string_lossy().as_ref()),
            Some(&folder),
            None,
        )
        .expect("version should infer from entry folder");

        assert_eq!(version, "1.21.1");
    }
}
