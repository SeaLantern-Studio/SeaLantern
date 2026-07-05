#[cfg(feature = "plugin-local-runtime")]
mod api_bridge;
#[cfg(feature = "plugin-local-runtime")]
mod console;
#[cfg(feature = "plugin-local-runtime")]
pub(crate) mod core;
#[cfg(feature = "plugin-local-runtime")]
mod element;
#[cfg(feature = "plugin-local-runtime")]
mod filesystem;
#[cfg(all(test, feature = "plugin-local-runtime"))]
#[path = "../../../tests/unit/plugins_runtime_filesystem_test.rs"]
mod filesystem_test;
#[cfg(feature = "plugin-local-runtime")]
mod host;
#[cfg(feature = "plugin-local-runtime")]
mod host_api;
#[cfg(feature = "plugin-local-runtime")]
mod http;
#[cfg(feature = "plugin-local-runtime")]
mod i18n;
#[cfg(feature = "plugin-local-runtime")]
mod log;
pub(crate) mod permissions;
#[cfg(feature = "plugin-local-runtime")]
mod plugins_api;
#[cfg(feature = "plugin-local-runtime")]
mod process;
#[cfg(all(test, feature = "plugin-local-runtime"))]
#[path = "../../../tests/unit/plugins_runtime_security_test.rs"]
mod security_test;
#[cfg(feature = "plugin-local-runtime")]
mod server;
#[cfg(feature = "plugin-local-runtime")]
pub(crate) mod shared;
#[cfg(feature = "plugin-local-runtime")]
mod storage;
#[cfg(not(feature = "plugin-local-runtime"))]
mod stub;
#[cfg(feature = "plugin-local-runtime")]
mod system;
#[cfg(feature = "plugin-local-runtime")]
mod ui;

#[cfg(feature = "plugin-local-runtime")]
pub use core::PluginRuntime;
#[cfg(feature = "plugin-local-runtime")]
pub use process::{kill_all_processes, new_process_registry};
#[cfg(not(feature = "plugin-local-runtime"))]
pub use stub::{kill_all_processes, new_process_registry, PluginRuntime};

#[cfg(all(test, feature = "plugin-local-runtime"))]
mod tests {
    use super::*;
    use crate::plugins::api::PluginApiRegistry;
    use mlua::Result as LuaResult;
    use mlua::Value;
    use std::env;
    use std::fs;

    #[test]
    fn test_runtime_creation() {
        let temp_dir = env::temp_dir().join("sl_test_plugin");
        let data_dir = temp_dir.join("data");
        let server_dir = temp_dir.join("servers");
        let global_dir = temp_dir.join("global");
        let api_registry = PluginApiRegistry::new();

        let runtime = PluginRuntime::new(
            "test-plugin",
            &temp_dir,
            &data_dir,
            &server_dir,
            &global_dir,
            api_registry,
            vec![],
            vec![],
            std::collections::HashMap::new(),
        );
        assert!(runtime.is_ok());

        let runtime = runtime.unwrap();
        assert!(!runtime.is_loaded());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sandbox_restrictions() {
        let temp_dir = env::temp_dir().join("sl_test_sandbox");
        let data_dir = temp_dir.join("data");
        let server_dir = temp_dir.join("servers");
        let global_dir = temp_dir.join("global");
        let api_registry = PluginApiRegistry::new();

        let runtime = PluginRuntime::new(
            "test-sandbox",
            &temp_dir,
            &data_dir,
            &server_dir,
            &global_dir,
            api_registry,
            vec![],
            vec![],
            std::collections::HashMap::new(),
        )
        .unwrap();

        let result: LuaResult<Value> = runtime.lua().load("return os").eval();
        assert!(matches!(result, Ok(Value::Nil)));

        let result: LuaResult<Value> = runtime.lua().load("return io").eval();
        assert!(matches!(result, Ok(Value::Nil)));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
