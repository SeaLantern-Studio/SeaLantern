#[cfg(test)]
mod backup_tests {
    use std::collections::HashSet;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::{Path, PathBuf};

    use flate2::Compression;
    use flate2::write::GzEncoder;
    use proptest::prelude::*;
    use proptest::test_runner::RngSeed;
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::super::archive;
    use super::super::error::BackupError;
    use super::super::manager::BackupManager;
    use super::super::models::*;

    fn proptest_config() -> ProptestConfig {
        ProptestConfig {
            cases: 16,
            max_shrink_iters: 128,
            failure_persistence: None,
            rng_seed: RngSeed::Fixed(0x5EA1_7A11_u64),
            ..ProptestConfig::default()
        }
    }

    fn content_type_strategy() -> impl Strategy<Value = BackupContentType> {
        prop_oneof![
            Just(BackupContentType::Core),
            Just(BackupContentType::Config),
            Just(BackupContentType::Plugins),
            Just(BackupContentType::World),
            Just(BackupContentType::Logs),
        ]
    }

    fn create_manager(temp_dir: &Path) -> BackupManager {
        BackupManager::new_at(temp_dir.join("backups")).unwrap()
    }

    fn create_test_server_dir(temp_dir: &Path, name: &str) -> PathBuf {
        let server_dir = temp_dir.join(name);
        fs::create_dir_all(&server_dir).unwrap();

        fs::write(server_dir.join("server.properties"), "motd=Test Server").unwrap();
        fs::write(server_dir.join("server.jar"), "fake jar content").unwrap();

        let config_dir = server_dir.join("config");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(config_dir.join("config.yml"), "test: value").unwrap();

        let world_dir = server_dir.join("world");
        fs::create_dir_all(&world_dir).unwrap();
        fs::write(world_dir.join("level.dat"), "level data").unwrap();

        let plugins_dir = server_dir.join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();
        fs::write(plugins_dir.join("test.jar"), "plugin content").unwrap();

        let logs_dir = server_dir.join("logs");
        fs::create_dir_all(&logs_dir).unwrap();
        fs::write(logs_dir.join("latest.log"), "log content").unwrap();

        server_dir
    }

    fn create_request(
        server_id: &str,
        contents: Vec<BackupContentType>,
        format: BackupFormat,
    ) -> CreateBackupRequest {
        CreateBackupRequest {
            server_id: server_id.to_string(),
            contents,
            format,
            compression_level: CompressionLevel::Medium,
            name: None,
        }
    }

    #[test]
    fn test_backup_manager_creation() {
        let temp_dir = tempdir().unwrap();
        assert!(BackupManager::new_at(temp_dir.path().join("backups")).is_ok());
    }

    #[test]
    fn test_create_backup_missing_server_dir() {
        let temp_dir = tempdir().unwrap();
        let manager = create_manager(temp_dir.path());
        let server_id = format!("non-existent-server-{}", Uuid::new_v4());
        let nonexistent_dir = temp_dir.path().join("missing-server");

        let result = manager.create_backup(
            create_request(
                &server_id,
                vec![BackupContentType::Core, BackupContentType::World],
                BackupFormat::Zip,
            ),
            &nonexistent_dir,
            |_| true,
        );

        assert!(matches!(
            result,
            Err(BackupError::ServerNotFound(id)) if id == server_id
        ));
    }

    #[test]
    fn test_delete_backup_nonexistent_id() {
        let temp_dir = tempdir().unwrap();
        let manager = create_manager(temp_dir.path());
        let backup_id = Uuid::new_v4().to_string();

        assert!(matches!(
            manager.delete_backup(&backup_id),
            Err(BackupError::NotFound(id)) if id == backup_id
        ));
    }

    #[test]
    fn test_restore_backup_nonexistent_id() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let backup_id = Uuid::new_v4().to_string();

        assert!(matches!(
            manager.restore_backup(&backup_id, "test-server", &server_dir, |_| true),
            Err(BackupError::NotFound(id)) if id == backup_id
        ));
    }

    #[test]
    fn test_restore_backup_with_check_server_stopped() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-restore-check";

        let backup = manager
            .create_backup(
                create_request(
                    server_id,
                    vec![BackupContentType::Core, BackupContentType::World],
                    BackupFormat::Zip,
                ),
                &server_dir,
                |_| true,
            )
            .unwrap();

        assert!(
            manager
                .restore_backup(&backup.id, server_id, &server_dir, |_| true,)
                .is_ok()
        );
    }

    #[test]
    fn test_restore_backup_corrupted_archive() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-corrupted";

        let backup = manager
            .create_backup(
                create_request(
                    server_id,
                    vec![BackupContentType::Core, BackupContentType::World],
                    BackupFormat::Zip,
                ),
                &server_dir,
                |_| true,
            )
            .unwrap();
        let archive_path = temp_dir
            .path()
            .join("backups")
            .join(server_id)
            .join(format!("{}.zip", backup.id));
        fs::remove_file(&archive_path).unwrap();

        assert!(matches!(
            manager.restore_backup(&backup.id, server_id, &server_dir, |_| true),
            Err(BackupError::CorruptedBackup(path)) if path == archive_path
        ));
    }

    #[test]
    fn test_create_and_list_backup() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-002";

        let backup = manager
            .create_backup(
                CreateBackupRequest {
                    server_id: server_id.to_string(),
                    contents: vec![
                        BackupContentType::Core,
                        BackupContentType::Core,
                        BackupContentType::World,
                    ],
                    format: BackupFormat::Zip,
                    compression_level: CompressionLevel::Medium,
                    name: Some("test-backup".to_string()),
                },
                &server_dir,
                |_| true,
            )
            .unwrap();

        assert_eq!(backup.server_id, server_id);
        assert_eq!(backup.format, BackupFormat::Zip);
        assert_eq!(backup.contents, vec![BackupContentType::Core, BackupContentType::World]);
        assert!(backup.size > 0);

        let backups = manager.get_backup_list(server_id).unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].id, backup.id);
        assert_eq!(backups[0].contents, backup.contents);
    }

    #[test]
    fn test_delete_backup() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-delete";

        let backup = manager
            .create_backup(
                create_request(server_id, vec![BackupContentType::Core], BackupFormat::Zip),
                &server_dir,
                |_| true,
            )
            .unwrap();

        manager.delete_backup(&backup.id).unwrap();
        assert!(manager.get_backup_list(server_id).unwrap().is_empty());
    }

    #[test]
    fn test_restore_backup() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-restore";

        let backup = manager
            .create_backup(
                create_request(
                    server_id,
                    vec![BackupContentType::Core, BackupContentType::World],
                    BackupFormat::Zip,
                ),
                &server_dir,
                |_| true,
            )
            .unwrap();

        fs::write(server_dir.join("server.properties"), "motd=Modified").unwrap();
        fs::write(server_dir.join("world").join("level.dat"), "modified world").unwrap();
        manager
            .restore_backup(&backup.id, server_id, &server_dir, |_| true)
            .unwrap();

        assert_eq!(
            fs::read_to_string(server_dir.join("server.properties")).unwrap(),
            "motd=Test Server"
        );
        assert_eq!(
            fs::read_to_string(server_dir.join("world").join("level.dat")).unwrap(),
            "level data"
        );
    }

    #[test]
    fn test_core_does_not_include_selectable_directories() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-core";

        let backup = manager
            .create_backup(
                create_request(server_id, vec![BackupContentType::Core], BackupFormat::Zip),
                &server_dir,
                |_| true,
            )
            .unwrap();

        fs::write(server_dir.join("server.properties"), "motd=Modified").unwrap();
        fs::write(server_dir.join("stale-core.txt"), "should be removed").unwrap();
        fs::write(server_dir.join("config").join("config.yml"), "modified config").unwrap();
        fs::write(server_dir.join("plugins").join("test.jar"), "modified plugin").unwrap();
        fs::write(server_dir.join("world").join("level.dat"), "modified world").unwrap();
        fs::write(server_dir.join("logs").join("latest.log"), "modified log").unwrap();

        manager
            .restore_backup(&backup.id, server_id, &server_dir, |_| true)
            .unwrap();

        assert_eq!(
            fs::read_to_string(server_dir.join("server.properties")).unwrap(),
            "motd=Test Server"
        );
        assert!(!server_dir.join("stale-core.txt").exists());
        assert_eq!(
            fs::read_to_string(server_dir.join("config").join("config.yml")).unwrap(),
            "modified config"
        );
        assert_eq!(
            fs::read_to_string(server_dir.join("plugins").join("test.jar")).unwrap(),
            "modified plugin"
        );
        assert_eq!(
            fs::read_to_string(server_dir.join("world").join("level.dat")).unwrap(),
            "modified world"
        );
        assert_eq!(
            fs::read_to_string(server_dir.join("logs").join("latest.log")).unwrap(),
            "modified log"
        );
    }

    #[test]
    fn test_restore_failure_keeps_original_server_directory() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-transaction";

        let backup = manager
            .create_backup(
                create_request(server_id, vec![BackupContentType::Core], BackupFormat::Zip),
                &server_dir,
                |_| true,
            )
            .unwrap();
        let metadata_path = temp_dir
            .path()
            .join("backups")
            .join(server_id)
            .join(format!("{}.json", backup.id));
        let mut metadata: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&metadata_path).unwrap()).unwrap();
        metadata["contents"] = serde_json::json!(["core", "world"]);
        fs::write(&metadata_path, serde_json::to_vec_pretty(&metadata).unwrap()).unwrap();

        fs::write(server_dir.join("server.properties"), "must survive failure").unwrap();
        let result = manager.restore_backup(&backup.id, server_id, &server_dir, |_| true);

        assert!(matches!(result, Err(BackupError::CorruptedBackup(_))));
        assert_eq!(
            fs::read_to_string(server_dir.join("server.properties")).unwrap(),
            "must survive failure"
        );
    }

    #[test]
    fn test_tar_gz_round_trip() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-tar";

        let backup = manager
            .create_backup(
                create_request(
                    server_id,
                    vec![BackupContentType::Core, BackupContentType::Config],
                    BackupFormat::TarGz,
                ),
                &server_dir,
                |_| true,
            )
            .unwrap();
        let archive_path = temp_dir
            .path()
            .join("backups")
            .join(server_id)
            .join(format!("{}.tar.gz", backup.id));

        assert_eq!(backup.format, BackupFormat::TarGz);
        assert_eq!(&fs::read(&archive_path).unwrap()[..2], [0x1f, 0x8b]);

        fs::write(server_dir.join("server.properties"), "motd=Modified").unwrap();
        fs::write(server_dir.join("config").join("config.yml"), "modified config").unwrap();
        manager
            .restore_backup(&backup.id, server_id, &server_dir, |_| true)
            .unwrap();
        assert_eq!(
            fs::read_to_string(server_dir.join("server.properties")).unwrap(),
            "motd=Test Server"
        );
        assert_eq!(
            fs::read_to_string(server_dir.join("config").join("config.yml")).unwrap(),
            "test: value"
        );
    }

    #[test]
    fn test_restore_legacy_metadata_without_server_path() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-legacy-metadata";

        let backup = manager
            .create_backup(
                create_request(server_id, vec![BackupContentType::Core], BackupFormat::Zip),
                &server_dir,
                |_| true,
            )
            .unwrap();
        let metadata_path = temp_dir
            .path()
            .join("backups")
            .join(server_id)
            .join(format!("{}.json", backup.id));
        fs::write(&metadata_path, serde_json::to_vec_pretty(&backup).unwrap()).unwrap();

        fs::write(server_dir.join("server.properties"), "motd=Modified").unwrap();
        manager
            .restore_backup(&backup.id, server_id, &server_dir, |_| true)
            .unwrap();
        assert_eq!(
            fs::read_to_string(server_dir.join("server.properties")).unwrap(),
            "motd=Test Server"
        );
    }

    #[test]
    fn test_restore_legacy_zip_with_tar_gz_metadata() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-legacy-tar";

        let backup = manager
            .create_backup(
                create_request(server_id, vec![BackupContentType::Core], BackupFormat::Zip),
                &server_dir,
                |_| true,
            )
            .unwrap();
        let backup_dir = temp_dir.path().join("backups").join(server_id);
        let zip_path = backup_dir.join(format!("{}.zip", backup.id));
        let legacy_path = backup_dir.join(format!("{}.tar.gz", backup.id));
        fs::rename(&zip_path, &legacy_path).unwrap();

        let metadata_path = backup_dir.join(format!("{}.json", backup.id));
        let mut metadata: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&metadata_path).unwrap()).unwrap();
        metadata["format"] = serde_json::json!("tar.gz");
        fs::write(&metadata_path, serde_json::to_vec_pretty(&metadata).unwrap()).unwrap();

        fs::write(server_dir.join("server.properties"), "motd=Modified").unwrap();
        manager
            .restore_backup(&backup.id, server_id, &server_dir, |_| true)
            .unwrap();
        assert_eq!(
            fs::read_to_string(server_dir.join("server.properties")).unwrap(),
            "motd=Test Server"
        );
    }

    #[test]
    fn test_restore_after_server_directory_move() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "server-before-move");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-moved";

        let backup = manager
            .create_backup(
                create_request(server_id, vec![BackupContentType::Core], BackupFormat::Zip),
                &server_dir,
                |_| true,
            )
            .unwrap();
        let moved_server_dir = temp_dir.path().join("server-after-move");
        fs::rename(&server_dir, &moved_server_dir).unwrap();
        fs::write(moved_server_dir.join("server.properties"), "motd=Modified").unwrap();

        manager
            .restore_backup(&backup.id, server_id, &moved_server_dir, |_| true)
            .unwrap();
        assert_eq!(
            fs::read_to_string(moved_server_dir.join("server.properties")).unwrap(),
            "motd=Test Server"
        );
    }

    #[test]
    fn test_create_rejects_archive_over_restore_limits() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        fs::write(server_dir.join("repeated.bin"), vec![0_u8; 1024 * 1024]).unwrap();
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-compression-limit";

        let result = manager.create_backup(
            create_request(server_id, vec![BackupContentType::Core], BackupFormat::Zip),
            &server_dir,
            |_| true,
        );

        assert!(result.is_err());
        assert!(manager.get_backup_list(server_id).unwrap().is_empty());
        assert!(
            fs::read_dir(temp_dir.path().join("backups").join(server_id))
                .unwrap()
                .next()
                .is_none()
        );
    }

    #[test]
    fn test_tar_gz_rejects_data_after_end_of_archive_marker() {
        let temp_dir = tempdir().unwrap();
        let archive_path = temp_dir.path().join("trailing.tar.gz");
        let file = File::create(&archive_path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        // 两个全零块构成 tar 结束标记。
        encoder.write_all(&[0_u8; 1024]).unwrap();
        // 其后追加非零数据表示归档被拼接过。全零尾部是 GNU tar 记录对齐的
        // 正常产物（默认补齐到 10240 字节），因此判定依据是非零而非有无数据。
        let mut trailing = [0_u8; 512];
        trailing[0] = b'x';
        encoder.write_all(&trailing).unwrap();
        encoder.finish().unwrap();

        let destination = temp_dir.path().join("extracted");
        let result = archive::extract_archive(&archive_path, &destination, BackupFormat::TarGz);

        assert!(matches!(result, Err(BackupError::Archive(_))));
        assert!(!destination.exists());
    }

    #[test]
    fn test_tar_gz_accepts_record_alignment_padding() {
        let temp_dir = tempdir().unwrap();
        let archive_path = temp_dir.path().join("padded.tar.gz");
        let file = File::create(&archive_path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(&[0_u8; 1024]).unwrap();
        // GNU tar 默认 blocking factor 为 20，补齐到 10240 字节。
        encoder.write_all(&[0_u8; 10240 - 1024]).unwrap();
        encoder.finish().unwrap();

        let destination = temp_dir.path().join("extracted");
        archive::extract_archive(&archive_path, &destination, BackupFormat::TarGz).unwrap();

        assert!(destination.is_dir());
    }

    #[test]
    fn test_restore_rejects_different_server_directory() {
        let temp_dir = tempdir().unwrap();
        let server_a = create_test_server_dir(temp_dir.path(), "server-a");
        let server_b = create_test_server_dir(temp_dir.path(), "server-b");
        let manager = create_manager(temp_dir.path());
        let backup = manager
            .create_backup(
                create_request("server-a", vec![BackupContentType::Core], BackupFormat::Zip),
                &server_a,
                |_| true,
            )
            .unwrap();

        let before = fs::read_to_string(server_b.join("server.properties")).unwrap();
        assert!(matches!(
            manager.restore_backup(&backup.id, "server-b", &server_b, |_| true),
            Err(BackupError::Validation(_))
        ));
        assert_eq!(fs::read_to_string(server_b.join("server.properties")).unwrap(), before);
    }

    #[test]
    fn test_rejects_path_traversal_identifiers() {
        let temp_dir = tempdir().unwrap();
        let manager = create_manager(temp_dir.path());
        assert!(matches!(manager.get_backup_list("../outside"), Err(BackupError::Validation(_))));
        assert!(matches!(
            manager.delete_backup("../outside"),
            Err(BackupError::InvalidBackupId(_))
        ));
        assert!(!temp_dir.path().join("outside.json").exists());
    }

    #[test]
    fn test_malformed_metadata_is_kept_for_diagnosis() {
        let temp_dir = tempdir().unwrap();
        let manager = create_manager(temp_dir.path());
        let metadata_path = temp_dir
            .path()
            .join("backups")
            .join("server")
            .join("broken.json");
        fs::create_dir_all(metadata_path.parent().unwrap()).unwrap();
        fs::write(&metadata_path, b"{not-json").unwrap();

        assert!(manager.get_backup_list("server").unwrap().is_empty());
        assert!(metadata_path.exists());
    }

    #[test]
    fn test_server_running_check() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-running";

        let result = manager.create_backup(
            create_request(server_id, vec![BackupContentType::Core], BackupFormat::Zip),
            &server_dir,
            |_| false,
        );

        assert!(matches!(
            result,
            Err(BackupError::ServerRunning(id)) if id == server_id
        ));
    }

    #[test]
    fn test_cleanup_old_backups() {
        let temp_dir = tempdir().unwrap();
        let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
        let manager = create_manager(temp_dir.path());
        let server_id = "test-server-cleanup";

        for index in 0..3 {
            manager
                .create_backup(
                    CreateBackupRequest {
                        server_id: server_id.to_string(),
                        contents: vec![BackupContentType::Core],
                        format: BackupFormat::Zip,
                        compression_level: if index == 0 {
                            CompressionLevel::Low
                        } else {
                            CompressionLevel::High
                        },
                        name: Some(format!("backup-{index}")),
                    },
                    &server_dir,
                    |_| true,
                )
                .unwrap();
        }

        assert_eq!(manager.get_backup_list(server_id).unwrap().len(), 3);
        assert_eq!(manager.cleanup_old_backups(server_id, 2).unwrap().len(), 1);
        assert_eq!(manager.get_backup_list(server_id).unwrap().len(), 2);
    }

    proptest! {
        #![proptest_config(proptest_config())]

        #[test]
        fn prop_archive_round_trip(data in prop::collection::vec(any::<u8>(), 0..=128)) {
            let temp_dir = tempdir().unwrap();
            let source = temp_dir.path().join("source");
            fs::create_dir_all(source.join("nested")).unwrap();
            fs::write(source.join("root.bin"), &data).unwrap();
            let reversed: Vec<u8> = data.iter().rev().copied().collect();
            fs::write(source.join("nested").join("nested.bin"), &reversed).unwrap();

            for (format_index, format) in
                [BackupFormat::Zip, BackupFormat::TarGz].into_iter().enumerate()
            {
                for (level_index, compression_level) in [
                    CompressionLevel::Low,
                    CompressionLevel::Medium,
                    CompressionLevel::High,
                ]
                .into_iter()
                .enumerate()
                {
                    let archive_path = temp_dir.path().join(format!(
                        "archive-{format_index}-{level_index}.{}",
                        format.extension()
                    ));
                    archive::create_archive(
                        &source,
                        &archive_path,
                        format,
                        compression_level,
                    )
                    .unwrap();

                    let destination = temp_dir
                        .path()
                        .join(format!("extracted-{format_index}-{level_index}"));
                    archive::extract_archive(&archive_path, &destination, format).unwrap();
                    prop_assert_eq!(
                        fs::read(destination.join("root.bin")).unwrap(),
                        data.clone()
                    );
                    prop_assert_eq!(
                        fs::read(destination.join("nested").join("nested.bin")).unwrap(),
                        reversed.clone()
                    );
                }
            }
        }

        #[test]
        fn prop_content_selection_is_normalized(
            contents in prop::collection::vec(content_type_strategy(), 1..=8)
        ) {
            let temp_dir = tempdir().unwrap();
            let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
            let manager = create_manager(temp_dir.path());
            let server_id = "test-server-normalized";

            let mut expected = Vec::new();
            if contents.contains(&BackupContentType::Core) {
                expected.push(BackupContentType::Core);
            }
            for content in &contents {
                if *content != BackupContentType::Core && !expected.contains(content) {
                    expected.push(*content);
                }
            }

            let backup = manager
                .create_backup(
                    create_request(server_id, contents, BackupFormat::Zip),
                    &server_dir,
                    |_| true,
                )
                .unwrap();
            prop_assert_eq!(backup.contents, expected);
        }

        #[test]
        fn prop_backup_lifecycle_matches_model(
            operations in prop::collection::vec(any::<bool>(), 1..=8)
        ) {
            let temp_dir = tempdir().unwrap();
            let server_dir = create_test_server_dir(temp_dir.path(), "test-server");
            let manager = create_manager(temp_dir.path());
            let server_id = "test-server-lifecycle";
            let mut expected = HashSet::new();

            for should_create in operations {
                if should_create {
                    let backup = manager
                        .create_backup(
                            create_request(
                                server_id,
                                vec![BackupContentType::Core],
                                BackupFormat::Zip,
                            ),
                            &server_dir,
                            |_| true,
                        )
                        .unwrap();
                    expected.insert(backup.id);
                } else if let Some(id) = expected.iter().next().cloned() {
                    manager.delete_backup(&id).unwrap();
                    expected.remove(&id);
                }

                let actual: HashSet<String> = manager
                    .get_backup_list(server_id)
                    .unwrap()
                    .into_iter()
                    .map(|backup| backup.id)
                    .collect();
                prop_assert_eq!(&actual, &expected);
            }
        }
    }
}
