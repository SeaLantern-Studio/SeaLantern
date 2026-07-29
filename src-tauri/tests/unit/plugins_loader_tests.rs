use super::*;
use crate::models::plugin::PluginAuthor;
use std::fs;

fn make_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sealantern_test_{}_{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn sample_manifest_json() -> &'static str {
    r#"{
        "id": "com.example.test",
        "name": "Test Plugin",
        "version": "1.0.0",
        "description": "A test plugin",
        "author": { "name": "Tester" },
        "main": "main.lua"
    }"#
}

#[test]
fn test_discover_plugins_finds_valid_dirs() {
    let tmp = make_temp_dir("discover");
    let plugin_a = tmp.join("plugin-a");
    fs::create_dir(&plugin_a).unwrap();
    fs::write(plugin_a.join("manifest.json"), "{}").unwrap();

    let no_manifest = tmp.join("no-manifest");
    fs::create_dir(&no_manifest).unwrap();

    let result = PluginLoader::discover_plugins(&tmp).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], plugin_a);

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_discover_plugins_empty_on_missing_dir() {
    let result =
        PluginLoader::discover_plugins(Path::new("/nonexistent/sealantern_test_path")).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_load_manifest_success() {
    let tmp = make_temp_dir("load");
    fs::write(tmp.join("manifest.json"), sample_manifest_json()).unwrap();

    let manifest = PluginLoader::load_manifest(&tmp).unwrap();
    assert_eq!(manifest.id, "com.example.test");
    assert_eq!(manifest.name, "Test Plugin");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.main, "main.lua");

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_load_manifest_file_not_found() {
    let tmp = make_temp_dir("load_missing");
    let result = PluginLoader::load_manifest(&tmp);
    assert!(result.is_err());

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_validate_manifest_ok() {
    let manifest = PluginManifest {
        id: "com.example.test".into(),
        name: "Test".into(),
        version: "1.0.0".into(),
        description: "desc".into(),
        author: PluginAuthor {
            name: "Dev".into(),
            email: None,
            url: None,
        },
        main: "main.lua".into(),
        license: None,
        homepage: None,
        repository: None,
        engines: None,
        permissions: vec![],
        ui: None,
        events: vec![],
        commands: vec![],
        dependencies: Default::default(),
        optional_dependencies: Default::default(),
        icon: None,
        settings: None,
        sidebar: None,
        locales: None,
        include: vec![],
    };
    assert!(PluginLoader::validate_manifest(&manifest).is_ok());
}

#[test]
fn test_validate_manifest_empty_id() {
    let manifest = PluginManifest {
        id: "".into(),
        name: "Test".into(),
        version: "1.0.0".into(),
        description: "desc".into(),
        author: PluginAuthor {
            name: "Dev".into(),
            email: None,
            url: None,
        },
        main: "main.lua".into(),
        license: None,
        homepage: None,
        repository: None,
        engines: None,
        permissions: vec![],
        ui: None,
        events: vec![],
        commands: vec![],
        dependencies: Default::default(),
        optional_dependencies: Default::default(),
        icon: None,
        settings: None,
        sidebar: None,
        locales: None,
        include: vec![],
    };
    let err = PluginLoader::validate_manifest(&manifest).unwrap_err();
    assert!(err.contains("id"));
}
