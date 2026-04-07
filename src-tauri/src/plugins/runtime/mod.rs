mod api_bridge;
mod console;
pub(crate) mod core;
mod element;
mod filesystem;
#[cfg(test)]
mod filesystem_test;
mod http;
mod i18n;
mod log;
mod plugins_api;
mod process;
#[cfg(test)]
mod security_test;
mod server;
pub(crate) mod shared;
mod storage;
mod system;
mod ui;

pub use core::PluginRuntime;
pub use process::{kill_all_processes, new_process_registry, ProcessRegistry};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::api::new_api_registry;
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
        let api_registry = new_api_registry();

        let runtime = PluginRuntime::new(
            "test-plugin",
            &temp_dir,
            &data_dir,
            &server_dir,
            &global_dir,
            api_registry,
            vec![],
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
        let api_registry = new_api_registry();

        let runtime = PluginRuntime::new(
            "test-sandbox",
            &temp_dir,
            &data_dir,
            &server_dir,
            &global_dir,
            api_registry,
            vec![],
        )
        .unwrap();

        let result: LuaResult<Value> = runtime.lua().load("return os").eval();
        assert!(matches!(result, Ok(Value::Nil)));

        let result: LuaResult<Value> = runtime.lua().load("return io").eval();
        assert!(matches!(result, Ok(Value::Nil)));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
