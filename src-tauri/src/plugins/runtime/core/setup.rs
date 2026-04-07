use super::runtime::PluginRuntime;
use crate::plugins::runtime::filesystem::has_any_fs_permission;
use mlua::Table;
use std::path::Path;

impl PluginRuntime {
    pub fn new(
        plugin_id: &str,
        plugin_dir: &Path,
        data_dir: &Path,
        server_dir: &Path,
        global_dir: &Path,
        api_registry: crate::plugins::api::ApiRegistry,
        permissions: Vec<String>,
    ) -> Result<Self, String> {
        let lua = mlua::Lua::new_with(
            mlua::StdLib::TABLE
                | mlua::StdLib::STRING
                | mlua::StdLib::MATH
                | mlua::StdLib::UTF8
                | mlua::StdLib::COROUTINE,
            mlua::LuaOptions::default(),
        )
        .map_err(|e| format!("Failed to create Lua instance: {}", e))?;

        std::fs::create_dir_all(data_dir)
            .map_err(|e| format!("Failed to create data dir: {}", e))?;

        let normalized_permissions = permissions
            .into_iter()
            .map(|p| if p == "fs" { "fs.data".to_string() } else { p })
            .collect::<Vec<_>>();

        let runtime = Self {
            lua,
            plugin_id: plugin_id.to_string(),
            plugin_dir: plugin_dir.to_path_buf(),
            data_dir: data_dir.to_path_buf(),
            server_dir: server_dir.to_path_buf(),
            global_dir: global_dir.to_path_buf(),
            loaded: std::sync::atomic::AtomicBool::new(false),
            permissions: normalized_permissions,
            api_registry,
            storage_lock: std::sync::Arc::new(std::sync::Mutex::new(())),
            process_registry: crate::plugins::runtime::process::new_process_registry(),
            element_callbacks: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
        };

        runtime.setup_sandbox(plugin_id, plugin_dir)?;
        runtime.setup_sl_namespace()?;

        Ok(runtime)
    }

    pub(super) fn setup_sl_namespace(&self) -> Result<(), String> {
        let globals = self.lua.globals();

        let sl = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create sl table: {}", e))?;

        let has_log_permission = self.permissions.iter().any(|p| p == "log");
        self.setup_log_namespace(&sl, has_log_permission)?;

        if self.permissions.iter().any(|p| p == "storage") {
            self.setup_storage_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "storage")?;
        }

        if has_any_fs_permission(&self.permissions) {
            self.setup_fs_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "fs")?;
        }

        if self.permissions.iter().any(|p| p == "api") {
            self.setup_api_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "api")?;
        }

        if self.permissions.iter().any(|p| p == "ui") {
            self.setup_ui_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "ui")?;
        }

        if self.permissions.iter().any(|p| p == "element") {
            self.setup_element_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "element")?;
        }

        if self.permissions.iter().any(|p| p == "server") {
            self.setup_server_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "server")?;
        }

        if self.permissions.iter().any(|p| p == "console") {
            self.setup_console_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "console")?;
        }

        if self.permissions.iter().any(|p| p == "system") {
            self.setup_system_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "system")?;
        }

        if self.permissions.iter().any(|p| p == "network") {
            self.setup_http_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "network")?;
        }

        if self.permissions.iter().any(|p| p == "execute_program") {
            self.setup_process_namespace(&sl, std::sync::Arc::clone(&self.process_registry))?;
        } else {
            self.setup_permission_denied_module(&sl, "execute_program")?;
        }

        if self.permissions.iter().any(|p| p == "plugin_folder_access") {
            self.setup_plugins_namespace(&sl)?;
        } else {
            self.setup_permission_denied_module(&sl, "plugin_folder_access")?;
        }

        self.setup_i18n_namespace(&sl)?;

        globals
            .set("sl", sl)
            .map_err(|e| format!("Failed to set sl global: {}", e))?;

        Ok(())
    }

    pub(super) fn globals_table(&self) -> Table {
        self.lua.globals()
    }
}
