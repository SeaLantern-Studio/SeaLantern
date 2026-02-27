use super::PluginRuntime;
use crate::plugins::api::new_api_registry;
use mlua::Result as LuaResult;
use std::env;
use std::fs as std_fs;

fn create_test_runtime_with_permissions(permissions: Vec<String>) -> PluginRuntime {
    let temp_dir = env::temp_dir().join(format!("sl_test_fs_{}", std::process::id()));
    let data_dir = temp_dir.join("data");
    let server_dir = temp_dir.join("servers");
    let global_dir = temp_dir.join("global");
    let api_registry = new_api_registry();

    let _ = std_fs::remove_dir_all(&temp_dir);
    std_fs::create_dir_all(&data_dir).unwrap();
    std_fs::create_dir_all(&server_dir).unwrap();
    std_fs::create_dir_all(&global_dir).unwrap();

    PluginRuntime::new(
        "test-fs-plugin",
        &temp_dir,
        &data_dir,
        &server_dir,
        &global_dir,
        api_registry,
        permissions,
    )
    .unwrap()
}

fn cleanup_test_runtime() {
    let temp_dir = env::temp_dir().join(format!("sl_test_fs_{}", std::process::id()));
    let _ = std_fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_fs_write_and_read() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Hello, World!")"#)
        .eval();
    assert!(result.is_ok());

    let result: LuaResult<String> = runtime.lua.load(r#"return sl.fs.read("test.txt")"#).eval();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, World!");

    cleanup_test_runtime();
}

#[test]
fn test_fs_write_binary_and_read_binary() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.write("test.bin", "Binary Data")"#)
        .eval();
    assert!(result.is_ok());

    let result: LuaResult<String> = runtime
        .lua
        .load(r#"return sl.fs.read_binary("test.bin")"#)
        .eval();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "QmluYXJ5IERhdGE=");

    cleanup_test_runtime();
}

#[test]
fn test_fs_exists() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("test.txt")"#)
        .eval();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);

    runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Content")"#)
        .eval::<()>()
        .unwrap();

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("test.txt")"#)
        .eval();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);

    cleanup_test_runtime();
}

#[test]
fn test_fs_mkdir_and_list() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let result: LuaResult<()> = runtime.lua.load(r#"sl.fs.mkdir("test_dir")"#).eval();
    assert!(result.is_ok());

    runtime
        .lua
        .load(r#"sl.fs.write("test_dir/file1.txt", "File 1")"#)
        .eval::<()>()
        .unwrap();
    runtime
        .lua
        .load(r#"sl.fs.write("test_dir/file2.txt", "File 2")"#)
        .eval::<()>()
        .unwrap();

    let result: LuaResult<mlua::Table> = runtime
        .lua
        .load(r#"return sl.fs.list("test_dir")"#)
        .eval();
    assert!(result.is_ok());

    let table = result.unwrap();
    let mut files = Vec::new();
    for pair in table.pairs::<mlua::Integer, String>() {
        if let Ok((_, file)) = pair {
            files.push(file);
        }
    }
    assert_eq!(files.len(), 2);
    assert!(files.contains(&"file1.txt".to_string()));
    assert!(files.contains(&"file2.txt".to_string()));

    cleanup_test_runtime();
}

#[test]
fn test_fs_remove_file() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Content")"#)
        .eval::<()>()
        .unwrap();

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("test.txt")"#)
        .eval();
    assert_eq!(result.unwrap(), true);

    let result: LuaResult<()> = runtime.lua.load(r#"sl.fs.remove("test.txt")"#).eval();
    assert!(result.is_ok());

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("test.txt")"#)
        .eval();
    assert_eq!(result.unwrap(), false);

    cleanup_test_runtime();
}

#[test]
fn test_fs_remove_directory() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    runtime
        .lua
        .load(r#"sl.fs.mkdir("test_dir")"#)
        .eval::<()>()
        .unwrap();
    runtime
        .lua
        .load(r#"sl.fs.write("test_dir/file.txt", "Content")"#)
        .eval::<()>()
        .unwrap();

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("test_dir")"#)
        .eval();
    assert_eq!(result.unwrap(), true);

    let result: LuaResult<()> = runtime.lua.load(r#"sl.fs.remove("test_dir")"#).eval();
    assert!(result.is_ok());

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("test_dir")"#)
        .eval();
    assert_eq!(result.unwrap(), false);

    cleanup_test_runtime();
}

#[test]
fn test_fs_info() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Hello, World!")"#)
        .eval::<()>()
        .unwrap();

    let result: LuaResult<mlua::Table> = runtime
        .lua
        .load(r#"return sl.fs.info("test.txt")"#)
        .eval();
    assert!(result.is_ok());

    let info = result.unwrap();
    let size: u64 = info.get("size").unwrap();
    let is_dir: bool = info.get("is_dir").unwrap();

    assert_eq!(size, 13);
    assert_eq!(is_dir, false);

    cleanup_test_runtime();
}

#[test]
fn test_fs_copy_file() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    runtime
        .lua
        .load(r#"sl.fs.write("source.txt", "Content")"#)
        .eval::<()>()
        .unwrap();

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.copy("source.txt", "dest.txt")"#)
        .eval();
    assert!(result.is_ok());

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("source.txt")"#)
        .eval();
    assert_eq!(result.unwrap(), true);

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("dest.txt")"#)
        .eval();
    assert_eq!(result.unwrap(), true);

    let result: LuaResult<String> = runtime
        .lua
        .load(r#"return sl.fs.read("dest.txt")"#)
        .eval();
    assert_eq!(result.unwrap(), "Content");

    cleanup_test_runtime();
}

#[test]
fn test_fs_copy_directory() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    runtime
        .lua
        .load(r#"sl.fs.mkdir("source_dir")"#)
        .eval::<()>()
        .unwrap();
    runtime
        .lua
        .load(r#"sl.fs.write("source_dir/file.txt", "Content")"#)
        .eval::<()>()
        .unwrap();

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.copy("source_dir", "dest_dir")"#)
        .eval();
    assert!(result.is_ok());

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("source_dir")"#)
        .eval();
    assert_eq!(result.unwrap(), true);

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("dest_dir")"#)
        .eval();
    assert_eq!(result.unwrap(), true);

    let result: LuaResult<String> = runtime
        .lua
        .load(r#"return sl.fs.read("dest_dir/file.txt")"#)
        .eval();
    assert_eq!(result.unwrap(), "Content");

    cleanup_test_runtime();
}

#[test]
fn test_fs_move() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    runtime
        .lua
        .load(r#"sl.fs.write("source.txt", "Content")"#)
        .eval::<()>()
        .unwrap();

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.move("source.txt", "dest.txt")"#)
        .eval();
    assert!(result.is_ok());

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("source.txt")"#)
        .eval();
    assert_eq!(result.unwrap(), false);

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("dest.txt")"#)
        .eval();
    assert_eq!(result.unwrap(), true);

    let result: LuaResult<String> = runtime
        .lua
        .load(r#"return sl.fs.read("dest.txt")"#)
        .eval();
    assert_eq!(result.unwrap(), "Content");

    cleanup_test_runtime();
}

#[test]
fn test_fs_rename() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    runtime
        .lua
        .load(r#"sl.fs.write("old_name.txt", "Content")"#)
        .eval::<()>()
        .unwrap();

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.rename("old_name.txt", "new_name.txt")"#)
        .eval();
    assert!(result.is_ok());

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("old_name.txt")"#)
        .eval();
    assert_eq!(result.unwrap(), false);

    let result: LuaResult<bool> = runtime
        .lua
        .load(r#"return sl.fs.exists("new_name.txt")"#)
        .eval();
    assert_eq!(result.unwrap(), true);

    let result: LuaResult<String> = runtime
        .lua
        .load(r#"return sl.fs.read("new_name.txt")"#)
        .eval();
    assert_eq!(result.unwrap(), "Content");

    cleanup_test_runtime();
}

#[test]
fn test_fs_get_path_data() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let result: LuaResult<String> = runtime.lua.load(r#"return sl.fs.get_path("data")"#).eval();
    assert!(result.is_ok());

    let path = result.unwrap();
    assert!(path.contains("data"));

    cleanup_test_runtime();
}

#[test]
fn test_fs_get_path_server() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.server".to_string()]);

    let result: LuaResult<String> = runtime
        .lua
        .load(r#"return sl.fs.get_path("server")"#)
        .eval();
    assert!(result.is_ok());

    let path = result.unwrap();
    assert!(path.contains("servers"));

    cleanup_test_runtime();
}

#[test]
fn test_fs_get_path_global() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.global".to_string()]);

    let result: LuaResult<String> = runtime
        .lua
        .load(r#"return sl.fs.get_path("global")"#)
        .eval();
    assert!(result.is_ok());

    let path = result.unwrap();
    assert!(path.contains("global"));

    cleanup_test_runtime();
}

#[test]
fn test_fs_permission_denied() {
    let runtime = create_test_runtime_with_permissions(vec![]);

    let result: LuaResult<()> = runtime.lua.load(r#"sl.fs.write("test.txt", "Content")"#).eval();
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("权限不足") || error_msg.contains("Permission denied") || error_msg.contains("permission") || error_msg.contains("required"));

    cleanup_test_runtime();
}

#[test]
fn test_fs_data_permission_only() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Content")"#)
        .eval();
    assert!(result.is_ok());

    cleanup_test_runtime();
}

#[test]
fn test_fs_server_permission() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.server".to_string()]);

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Content")"#)
        .eval();
    assert!(result.is_ok());

    cleanup_test_runtime();
}

#[test]
fn test_fs_global_permission() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.global".to_string()]);

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Content")"#)
        .eval();
    assert!(result.is_ok());

    cleanup_test_runtime();
}

#[test]
fn test_fs_invalid_scope() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let result: LuaResult<String> = runtime.lua.load(r#"return sl.fs.get_path("invalid")"#).eval();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid scope"));

    cleanup_test_runtime();
}

#[test]
fn test_fs_write_large_file() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let large_content = "a".repeat(11 * 1024 * 1024);
    let result: LuaResult<()> = runtime
        .lua
        .load(format!(r#"sl.fs.write("large.txt", "{}")"#, large_content))
        .eval();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too large"));

    cleanup_test_runtime();
}

#[test]
fn test_fs_read_large_file() {
    let runtime = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let large_content = "a".repeat(11 * 1024 * 1024);
    std_fs::write(
        env::temp_dir()
            .join(format!("sl_test_fs_{}", std::process::id()))
            .join("data")
            .join("large.txt"),
        large_content,
    )
    .unwrap();

    let result: LuaResult<String> = runtime.lua.load(r#"return sl.fs.read("large.txt")"#).eval();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too large"));

    cleanup_test_runtime();
}

#[test]
fn test_fs_backward_compatibility() {
    let runtime = create_test_runtime_with_permissions(vec!["fs".to_string()]);

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Hello from fs permission!")"#)
        .eval();
    assert!(result.is_ok());

    let result: LuaResult<String> = runtime.lua.load(r#"return sl.fs.read("test.txt")"#).eval();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello from fs permission!");

    cleanup_test_runtime();
}
