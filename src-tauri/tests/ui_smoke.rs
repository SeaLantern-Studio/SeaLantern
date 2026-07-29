// 测试用的 UI 事件处理器
// 确保在测试之间不会互相干扰

use mlua::Result as LuaResult;
use sea_lantern_lib::plugins::api::UiEventHandler;
use sea_lantern_lib::plugins::api::{new_api_registry, set_ui_event_handler};
use sea_lantern_lib::plugins::runtime::PluginRuntime;
use std::env;
use std::fs as std_fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

fn create_test_runtime_with_permissions(
    plugin_id: &str,
    permissions: Vec<&str>,
) -> (PluginRuntime, PathBuf) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let temp_dir =
        env::temp_dir().join(format!("sl_test_ui_{}_{}_{}", plugin_id, std::process::id(), now));
    let data_dir = temp_dir.join("data");
    let server_dir = temp_dir.join("servers");
    let global_dir = temp_dir.join("global");
    let api_registry = new_api_registry();

    std_fs::create_dir_all(&data_dir).unwrap();
    std_fs::create_dir_all(&server_dir).unwrap();
    std_fs::create_dir_all(&global_dir).unwrap();

    let perms = permissions
        .into_iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    let runtime = PluginRuntime::new(
        plugin_id,
        &temp_dir,
        &data_dir,
        &server_dir,
        &global_dir,
        api_registry,
        perms,
    )
    .unwrap();

    (runtime, temp_dir)
}

fn cleanup(temp_dir: &std::path::Path) {
    let _ = std_fs::remove_dir_all(temp_dir);
}

static UI_HANDLER_LOCK: std::sync::OnceLock<Mutex<()>> = std::sync::OnceLock::new();
fn lock_ui_handler() -> std::sync::MutexGuard<'static, ()> {
    UI_HANDLER_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap()
}

#[test]
fn ui_insert_invalid_placement_returns_error() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions("ui-insert-invalid", vec!["ui"]);

    let res: LuaResult<bool> =
        runtime.lua_eval(r##"return sl.ui.insert("bad", "#id", "<div></div>")"##);

    assert!(res.is_err(), "invalid placement should return Lua error");

    cleanup(&temp_dir);
}

#[test]
fn ui_hide_returns_true_when_no_handler() {
    let (runtime, temp_dir) = create_test_runtime_with_permissions("ui-hide-ok", vec!["ui"]);

    let res: LuaResult<bool> = runtime.lua_eval(r#"return sl.ui.hide("body")"#);

    assert!(res.is_ok());
    assert!(res.unwrap());

    cleanup(&temp_dir);
}

#[test]
fn ui_infra_failure_compat_returns_false() {
    let _guard = lock_ui_handler();
    // 注入仅对特定插件ID返回 Err 的处理器，避免影响其他测试
    let handler: UiEventHandler = Arc::new(|plugin_id, _action, _element_id, _html| {
        if plugin_id == "ui-err" {
            Err("boom".to_string())
        } else {
            Ok(())
        }
    });
    set_ui_event_handler(handler);

    let (runtime, temp_dir) = create_test_runtime_with_permissions("ui-err", vec!["ui"]);

    let res: LuaResult<bool> = runtime.lua_eval(r##"return sl.ui.hide("#x")"##);

    assert!(res.is_ok());
    assert!(!res.unwrap(), "compat 模式下应返回 false");

    cleanup(&temp_dir);

    // 恢复一个总是 Ok 的处理器，避免影响其他测试
    let handler_ok: UiEventHandler = Arc::new(|_plugin_id, _action, _element_id, _html| Ok(()));
    set_ui_event_handler(handler_ok);
}

#[test]
fn ui_infra_failure_strict_returns_error() {
    let _guard = lock_ui_handler();
    // 对特定插件ID返回 Err
    let handler: UiEventHandler = Arc::new(|plugin_id, _action, _element_id, _html| {
        if plugin_id == "ui-err-strict" {
            Err("boom".to_string())
        } else {
            Ok(())
        }
    });
    set_ui_event_handler(handler);

    let (runtime, temp_dir) = create_test_runtime_with_permissions("ui-err-strict", vec!["ui"]);

    // 切换严格模式
    let _: LuaResult<bool> = runtime.lua_eval(r#"return sl.ui.set_error_mode("strict")"#);

    let res: LuaResult<bool> = runtime.lua_eval(r##"return sl.ui.hide("#y")"##);

    assert!(res.is_err(), "strict 模式下应抛出错误");

    cleanup(&temp_dir);

    // 恢复 Ok 处理器
    let handler_ok: UiEventHandler = Arc::new(|_plugin_id, _action, _element_id, _html| Ok(()));
    set_ui_event_handler(handler_ok);
}
