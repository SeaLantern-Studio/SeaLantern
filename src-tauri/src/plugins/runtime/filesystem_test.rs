use super::PluginRuntime;
use crate::plugins::api::new_api_registry;
use mlua::Result as LuaResult;
use std::env;
use std::fs as std_fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn create_test_runtime_with_permissions(
    permissions: Vec<String>,
) -> (PluginRuntime, std::path::PathBuf) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let temp_dir = env::temp_dir().join(format!("sl_test_fs_{}_{}", std::process::id(), now));
    let data_dir = temp_dir.join("data");
    let server_dir = temp_dir.join("servers");
    let global_dir = temp_dir.join("global");
    let api_registry = new_api_registry();

    std_fs::create_dir_all(&data_dir).unwrap();
    std_fs::create_dir_all(&server_dir).unwrap();
    std_fs::create_dir_all(&global_dir).unwrap();

    let runtime = PluginRuntime::new(
        "test-fs-plugin",
        &temp_dir,
        &data_dir,
        &server_dir,
        &global_dir,
        api_registry,
        permissions,
    )
    .unwrap();

    (runtime, temp_dir)
}

fn cleanup_test_runtime(temp_dir: &std::path::Path) {
    let _ = std_fs::remove_dir_all(temp_dir);
}

#[test]
fn test_fs_write_and_read() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Hello, World!")"#)
        .eval();
    assert!(result.is_ok());

    let result: LuaResult<String> = runtime.lua.load(r#"return sl.fs.read("test.txt")"#).eval();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, World!");

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_write_binary_and_read_binary() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

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

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_exists() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

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

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_mkdir_and_list() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

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

    let result: LuaResult<mlua::Table> =
        runtime.lua.load(r#"return sl.fs.list("test_dir")"#).eval();
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

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_remove_file() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

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

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_remove_directory() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

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

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_info() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Hello, World!")"#)
        .eval::<()>()
        .unwrap();

    let result: LuaResult<mlua::Table> =
        runtime.lua.load(r#"return sl.fs.info("test.txt")"#).eval();
    assert!(result.is_ok());

    let info = result.unwrap();
    let size: u64 = info.get("size").unwrap();
    let is_dir: bool = info.get("is_dir").unwrap();

    assert_eq!(size, 13);
    assert_eq!(is_dir, false);

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_copy_file() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

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

    let result: LuaResult<String> = runtime.lua.load(r#"return sl.fs.read("dest.txt")"#).eval();
    assert_eq!(result.unwrap(), "Content");

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_copy_directory() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

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

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_move() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

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

    let result: LuaResult<String> = runtime.lua.load(r#"return sl.fs.read("dest.txt")"#).eval();
    assert_eq!(result.unwrap(), "Content");

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_rename() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

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

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_get_path_data() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let result: LuaResult<String> = runtime.lua.load(r#"return sl.fs.get_path("data")"#).eval();
    assert!(result.is_ok());

    let path = result.unwrap();
    assert!(path.contains("data"));

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_get_path_server() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.server".to_string()]);

    let result: LuaResult<String> = runtime
        .lua
        .load(r#"return sl.fs.get_path("server")"#)
        .eval();
    assert!(result.is_ok());

    let path = result.unwrap();
    assert!(path.contains("servers"));

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_get_path_global() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.global".to_string()]);

    let result: LuaResult<String> = runtime
        .lua
        .load(r#"return sl.fs.get_path("global")"#)
        .eval();
    assert!(result.is_ok());

    let path = result.unwrap();
    assert!(path.contains("global"));

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_permission_denied() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec![]);

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Content")"#)
        .eval();
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("权限不足")
            || error_msg.contains("Permission denied")
            || error_msg.contains("permission")
            || error_msg.contains("required")
    );

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_data_permission_only() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Content")"#)
        .eval();
    assert!(result.is_ok());

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_server_permission() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.server".to_string()]);

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Content")"#)
        .eval();
    assert!(result.is_ok());

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_global_permission() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.global".to_string()]);

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Content")"#)
        .eval();
    assert!(result.is_ok());

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_invalid_scope() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs.data".to_string()]);

    let result: LuaResult<String> = runtime
        .lua
        .load(r#"return sl.fs.get_path("invalid")"#)
        .eval();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid scope"));

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_backward_compatibility() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions(vec!["fs".to_string()]);

    let result: LuaResult<()> = runtime
        .lua
        .load(r#"sl.fs.write("test.txt", "Hello from fs permission!")"#)
        .eval();
    assert!(result.is_ok());

    let result: LuaResult<String> = runtime.lua.load(r#"return sl.fs.read("test.txt")"#).eval();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello from fs permission!");

    cleanup_test_runtime(&temp_dir);
}

#[test]
fn test_fs_path_validation_edge_cases() {
    // 覆盖：绝对路径、不同位置的 `..`，以及多个 scope
    let scopes = vec!["fs.data".to_string(), "fs.server".to_string(), "fs.global".to_string()];

    // 这些路径都应该被 validate_path / validate_path_static 拒绝
    let invalid_paths = vec![
        // 绝对 *nix 风格
        "/etc/passwd",
        "/tmp/escape.txt",
        // 绝对 Windows 风格
        "C:/Windows/system32",
        "C:\\Windows\\system32",
        // 简单的 `..`
        "../escape.txt",
        // 中间包含 `..`
        "dir/../escape.txt",
        "nested/.././escape.txt",
        // 末尾 `..`
        "escape/..",
        // 混合分隔符（Windows 风格的 `..` 模式）
        "..\\escape.txt",
        "dir\\..\\escape.txt",
    ];

    // 所有文件系统函数都应遵守相同的路径规则
    let functions = vec![
        ("write", r#"sl.fs.write("{path}", "test-content")"#),
        ("read", r#"sl.fs.read("{path}")"#),
        ("read_binary", r#"sl.fs.read_binary("{path}")"#),
        ("exists", r#"sl.fs.exists("{path}")"#),
        ("list", r#"sl.fs.list("{path}")"#),
        ("mkdir", r#"sl.fs.mkdir("{path}")"#),
        ("remove", r#"sl.fs.remove("{path}")"#),
        ("copy", r#"sl.fs.copy("{path}", "valid/target.txt")"#),
        ("move", r#"sl.fs.move("{path}", "valid/target.txt")"#),
        ("rename", r#"sl.fs.rename("{path}", "valid/target.txt")"#),
    ];

    for scope in scopes {
        let (runtime, temp_dir) = create_test_runtime_with_permissions(vec![scope.clone()]);

        for path in &invalid_paths {
            for (func_name, func_template) in &functions {
                let lua_code = func_template.replace("{path}", path);

                let result: LuaResult<()> = runtime.lua.load(&lua_code).eval();

                // 所有非法路径都必须以路径校验错误失败
                assert!(
                    result.is_err(),
                    "Expected {} with scope {} and path '{}' to fail, but it succeeded",
                    func_name,
                    scope,
                    path
                );

                // 对于需要两个路径的函数，也测试目标路径
                if func_name == &"copy" || func_name == &"move" || func_name == &"rename" {
                    let lua_code_dest = func_template
                        .replace("{path}", "valid/source.txt")
                        .replace("valid/target.txt", path);
                    let result_dest: LuaResult<()> = runtime.lua.load(&lua_code_dest).eval();

                    assert!(
                        result_dest.is_err(),
                        "Expected {} with scope {} and dest path '{}' to fail, but it succeeded",
                        func_name,
                        scope,
                        path
                    );
                }
            }
        }

        cleanup_test_runtime(&temp_dir);
    }
}
