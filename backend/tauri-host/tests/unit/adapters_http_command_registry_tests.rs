use super::*;
use runtime::DispatchResult;
use serde_json::json;

#[test]
fn command_registry_includes_tunnel_commands() {
    let registry = CommandRegistry::new();
    let commands = registry.list_commands();

    assert!(commands.contains(&"tunnel_host".to_string()));
    assert!(commands.contains(&"tunnel_join".to_string()));
    assert!(commands.contains(&"tunnel_stop".to_string()));
    assert!(commands.contains(&"tunnel_status".to_string()));
    assert!(commands.contains(&"tunnel_copy_ticket".to_string()));
    assert!(commands.contains(&"tunnel_regenerate_ticket".to_string()));
    assert!(commands.contains(&"tunnel_generate_ticket".to_string()));
}

#[test]
fn command_registry_includes_preview_from_source_command() {
    let registry = CommandRegistry::new();
    let commands = registry.list_commands();

    assert!(commands.contains(&"preview_server_properties_write_from_source".to_string()));
}

#[test]
fn command_registry_includes_parse_server_core_key_and_compat_alias() {
    let registry = CommandRegistry::new();
    let commands = registry.list_commands();

    assert!(commands.contains(&"parse_server_core_key".to_string()));
    assert!(commands.contains(&"parse_server_core_type".to_string()));
}

#[test]
fn command_registry_includes_wrapped_settings_contract_commands() {
    let registry = CommandRegistry::new();
    let commands = registry.list_commands();

    assert!(commands.contains(&"change_data_dir".to_string()));
    assert!(commands.contains(&"change_plugin_dir".to_string()));
    assert!(commands.contains(&"get_web_settings".to_string()));
    assert!(commands.contains(&"save_web_settings".to_string()));
    assert!(commands.contains(&"update_web_settings_partial".to_string()));
    assert!(commands.contains(&"import_web_settings".to_string()));
    assert!(!commands.contains(&"get_settings".to_string()));
    assert!(!commands.contains(&"save_settings_with_diff".to_string()));
    assert!(!commands.contains(&"import_settings".to_string()));
}

#[tokio::test]
async fn parse_server_core_type_compat_alias_preserves_legacy_display_semantics() {
    let dir = tempfile::tempdir().expect("temp dir should exist");
    let jar_path = dir.path().join("paper-server.jar");
    let file = std::fs::File::create(&jar_path).expect("jar should create");
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default();
    zip.start_file("META-INF/MANIFEST.MF", options)
        .expect("manifest entry should start");
    use std::io::Write;
    zip.write_all(b"Manifest-Version: 1.0\r\nMain-Class: io.papermc.paperclip.Main\r\n\r\n")
        .expect("manifest should write");
    zip.finish().expect("zip should finish");

    let registry = CommandRegistry::new();

    let parsed_key = registry
        .dispatch(
            "parse_server_core_key",
            json!({ "sourcePath": jar_path.to_string_lossy().to_string() }),
        )
        .await;
    let parsed_type = registry
        .dispatch(
            "parse_server_core_type",
            json!({ "sourcePath": jar_path.to_string_lossy().to_string() }),
        )
        .await;

    match parsed_key {
        DispatchResult::Success(value) => {
            assert_eq!(value.get("core_type").and_then(|item| item.as_str()), Some("paper"));
        }
        DispatchResult::InvalidRequest(message) => {
            panic!("parse_server_core_key unexpectedly rejected request: {message}")
        }
        DispatchResult::Failure(message) => panic!("parse_server_core_key failed: {message}"),
        DispatchResult::NotFound(message) => {
            panic!("parse_server_core_key unexpectedly missing: {message}")
        }
    }

    match parsed_type {
        DispatchResult::Success(value) => {
            assert_eq!(value.get("core_type").and_then(|item| item.as_str()), Some("Paper"));
        }
        DispatchResult::InvalidRequest(message) => {
            panic!("parse_server_core_type unexpectedly rejected request: {message}")
        }
        DispatchResult::Failure(message) => panic!("parse_server_core_type failed: {message}"),
        DispatchResult::NotFound(message) => {
            panic!("parse_server_core_type unexpectedly missing: {message}")
        }
    }
}
