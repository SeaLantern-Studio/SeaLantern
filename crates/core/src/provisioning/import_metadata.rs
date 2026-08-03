use crate::instance::InstanceSpec;

use super::server_inspection::{
    Detected, DiagnosticSeverity, InspectionDiagnostic, ServerInspectionError,
    ServerInspectionReport,
};

/// 将主机已经完成的服务端检查结果投影到实例导入元数据。
///
/// 该函数只处理报告，不读取路径、不执行脚本。检查失败、字段缺失或候选冲突时，
/// 保留调用方已有的值，并返回可供导入界面展示的诊断。
pub fn apply_server_inspection(
    instance: &mut InstanceSpec,
    inspection: Result<&ServerInspectionReport, &ServerInspectionError>,
) -> Vec<InspectionDiagnostic> {
    let report = match inspection {
        Ok(report) => report,
        Err(error) => {
            return vec![InspectionDiagnostic {
                severity: DiagnosticSeverity::Error,
                code: "server_inspection_failed".to_string(),
                message: format!(
                    "server inspection failed ({error}); existing import metadata was preserved; retry inspection or choose values manually"
                ),
                evidence: Vec::new(),
            }]
        }
    };

    let mut diagnostics = report.diagnostics.clone();
    if let Some(product) = report
        .identity
        .implementation
        .value
        .as_ref()
        .filter(|product| !product.key.trim().is_empty())
    {
        instance.core_type = product.key.trim().to_string();
    } else {
        diagnostics.push(unresolved_diagnostic(
            "server implementation",
            &report.identity.implementation,
            |product| product.key.clone(),
        ));
    }

    if let Some(version) = report
        .identity
        .version
        .value
        .as_ref()
        .filter(|version| !version.trim().is_empty())
    {
        instance.core_version = version.trim().to_string();
    } else {
        diagnostics.push(unresolved_diagnostic(
            "server implementation version",
            &report.identity.version,
            |version| version.clone(),
        ));
    }

    match report.minecraft.as_ref() {
        Some(minecraft) => {
            if let Some(version) = minecraft
                .version
                .value
                .as_ref()
                .filter(|version| !version.trim().is_empty())
            {
                instance.game_version = version.trim().to_string();
            } else {
                diagnostics.push(unresolved_diagnostic(
                    "Minecraft version",
                    &minecraft.version,
                    |version| version.clone(),
                ));
            }
        }
        None => diagnostics.push(missing_diagnostic("Minecraft version")),
    }

    diagnostics
}

fn unresolved_diagnostic<T, F>(
    field: &str,
    detected: &Detected<T>,
    format_candidate: F,
) -> InspectionDiagnostic
where
    F: Fn(&T) -> String,
{
    if detected.alternatives.is_empty() {
        return missing_diagnostic(field);
    }

    let candidates = detected
        .alternatives
        .iter()
        .map(|candidate| {
            format!("{} (confidence {})", format_candidate(&candidate.value), candidate.confidence)
        })
        .collect::<Vec<_>>()
        .join(", ");
    InspectionDiagnostic {
        severity: DiagnosticSeverity::Warning,
        code: format!(
            "server_inspection_{}_ambiguous",
            field.replace(' ', "_").to_ascii_lowercase()
        ),
        message: format!(
            "{field} is ambiguous ({candidates}); existing import metadata was preserved; choose a value manually or provide stronger metadata"
        ),
        evidence: detected.evidence.clone(),
    }
}

fn missing_diagnostic(field: &str) -> InspectionDiagnostic {
    InspectionDiagnostic {
        severity: DiagnosticSeverity::Warning,
        code: format!(
            "server_inspection_{}_unresolved",
            field.replace(' ', "_").to_ascii_lowercase()
        ),
        message: format!(
            "{field} was not identified; existing import metadata was preserved; choose a value manually or provide stronger metadata"
        ),
        evidence: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::apply_server_inspection;
    use crate::instance::{InstanceId, InstanceSpec, LocalLaunch, StartupMode};
    use crate::provisioning::{inspect_server_artifact, InspectionOptions};
    use zip::write::FileOptions;

    fn instance_spec() -> InstanceSpec {
        InstanceSpec {
            id: InstanceId::new("imported").expect("instance ID should be valid"),
            name: "Imported".to_string(),
            aliases: Vec::new(),
            core_type: "paper".to_string(),
            core_version: "old-core".to_string(),
            game_version: "old-game".to_string(),
            directory: PathBuf::from("managed/imported"),
            port: 25565,
            max_memory_mib: 4096,
            min_memory_mib: 1024,
            created_at_unix_secs: 100,
            last_started_at_unix_secs: None,
            launch: LocalLaunch {
                startup_mode: StartupMode::Jar,
                startup_target: Some(PathBuf::from("imports/server.jar")),
                custom_command: None,
                custom_executable: None,
                custom_arguments: Vec::new(),
                java_executable: None,
                jvm_arguments: Vec::new(),
            },
        }
    }

    fn write_test_jar(filename: &str, entries: &[(&str, &str)]) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "sealantern-import-metadata-{}-{timestamp}-{filename}",
            std::process::id()
        ));
        let file = File::create(&path).expect("create test JAR");
        let mut archive = zip::ZipWriter::new(file);
        for (name, content) in entries {
            archive
                .start_file(*name, FileOptions::<()>::default())
                .expect("create metadata entry");
            archive
                .write_all(content.as_bytes())
                .expect("write metadata entry");
        }
        archive.finish().expect("finish test JAR");
        path
    }

    #[test]
    fn projects_unknown_paperclip_product_and_preserves_open_product_key() {
        let mut instance = instance_spec();
        let path = write_test_jar(
            "custom-fork.jar",
            &[
                (
                    "META-INF/MANIFEST.MF",
                    "Main-Class: io.papermc.paperclip.Main\r\n\r\n",
                ),
                (
                    "META-INF/versions.list",
                    "hash\t26.2\t26.2/custom-fork-26.2.jar\n",
                ),
                (
                    "META-INF/libraries.list",
                    "hash\texample.server:custom-fork-api:26.2.build.1-stable\tcustom-fork-api.jar\n",
                ),
            ],
        );
        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect unknown Paperclip fixture");
        std::fs::remove_file(&path).expect("remove unknown Paperclip fixture");

        let diagnostics = apply_server_inspection(&mut instance, Ok(&report));

        assert!(diagnostics.is_empty());
        assert_eq!(instance.core_type, "custom-fork");
        assert_eq!(instance.core_version, "26.2.build.1-stable");
        assert_eq!(instance.game_version, "26.2");
    }

    #[test]
    fn projects_paperclip_derived_product_and_build_version() {
        let mut instance = instance_spec();
        let path = write_test_jar(
            "purpur.jar",
            &[
                ("META-INF/MANIFEST.MF", "Main-Class: io.papermc.paperclip.Main\r\n\r\n"),
                ("META-INF/versions.list", "hash\t26.2\t26.2/purpur-26.2.jar\n"),
                (
                    "META-INF/libraries.list",
                    "hash\torg.purpurmc.purpur:purpur-api:26.2.build.2618-stable\tpurpur-api.jar\n",
                ),
            ],
        );
        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect Paperclip fixture");
        std::fs::remove_file(&path).expect("remove Paperclip fixture");

        let diagnostics = apply_server_inspection(&mut instance, Ok(&report));

        assert!(diagnostics.is_empty());
        assert_eq!(instance.core_type, "purpur");
        assert_eq!(instance.core_version, "26.2.build.2618-stable");
        assert_eq!(instance.game_version, "26.2");
    }

    #[test]
    fn projects_fabric_loader_version_instead_of_installer_version() {
        let mut instance = instance_spec();
        let path = write_test_jar(
            "fabric.jar",
            &[
                (
                    "META-INF/MANIFEST.MF",
                    "Implementation-Title: FabricInstaller\r\nImplementation-Version: 1.1.1\r\nMain-Class: net.fabricmc.installer.ServerLauncher\r\n\r\n",
                ),
                (
                    "install.properties",
                    "fabric-loader-version=0.19.3\ngame-version=26.2\n",
                ),
            ],
        );
        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect Fabric fixture");
        std::fs::remove_file(&path).expect("remove Fabric fixture");

        let diagnostics = apply_server_inspection(&mut instance, Ok(&report));

        assert!(diagnostics.is_empty());
        assert_eq!(instance.core_type, "fabric");
        assert_eq!(instance.core_version, "0.19.3");
        assert_ne!(instance.core_version, "1.1.1");
    }

    #[test]
    fn preserves_conflicting_fields_and_applies_selected_minecraft_version() {
        let mut instance = instance_spec();
        let path = write_test_jar(
            "conflicting.jar",
            &[
                ("META-INF/MANIFEST.MF", "Main-Class: io.papermc.paperclip.Main\r\n\r\n"),
                ("META-INF/versions.list", "hash\t26.2\t26.2/purpur-26.2.jar\n"),
                (
                    "META-INF/libraries.list",
                    "hash\tio.papermc.paper:paper-api:26.2.build.87-stable\tpaper-api.jar\n",
                ),
            ],
        );
        let report = inspect_server_artifact(&path, &InspectionOptions::default())
            .expect("inspect conflicting Paperclip fixture");
        std::fs::remove_file(&path).expect("remove conflicting Paperclip fixture");

        let diagnostics = apply_server_inspection(&mut instance, Ok(&report));

        assert_eq!(instance.core_type, "paper");
        assert_eq!(instance.core_version, "old-core");
        assert_eq!(instance.game_version, "26.2");
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "server_inspection_server_implementation_ambiguous"
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "server_inspection_server_implementation_version_unresolved"
        }));
    }

    #[test]
    fn preserves_existing_values_when_inspection_fails() {
        let mut instance = instance_spec();
        let path = std::env::temp_dir()
            .join(format!("sealantern-import-metadata-{}-invalid.jar", std::process::id()));
        std::fs::write(&path, b"not a ZIP archive").expect("write invalid JAR");
        let inspection = inspect_server_artifact(&path, &InspectionOptions::default());
        std::fs::remove_file(&path).expect("remove invalid JAR");

        let diagnostics = apply_server_inspection(&mut instance, inspection.as_ref());

        assert!(inspection.is_err());
        assert_eq!(instance.core_type, "paper");
        assert_eq!(instance.core_version, "old-core");
        assert_eq!(instance.game_version, "old-game");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "server_inspection_failed");
        assert!(diagnostics[0]
            .message
            .contains("existing import metadata was preserved"));
    }
}
