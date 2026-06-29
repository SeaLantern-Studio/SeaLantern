use super::runtime::PluginRuntime;
use crate::plugins::runtime::filesystem::has_any_fs_permission;
use crate::plugins::runtime::permissions::{
    has_plugin_folder_access_permission, normalize_permissions, PLUGIN_FOLDER_ACCESS_PERMISSION,
};
use crate::services::events::ServerEventSubscription;
use crate::services::global::i18n_service;
use mlua::Table;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

fn core_t1(key: &str, a: impl Into<String>) -> String {
    let mut m = HashMap::new();
    m.insert("0".to_string(), a.into());
    i18n_service().t_with_options(key, &m)
}

impl PluginRuntime {
    fn normalize_declared_programs(
        plugin_dir: &Path,
        allowed_programs: Vec<String>,
    ) -> Result<HashSet<PathBuf>, String> {
        let mut normalized = HashSet::new();

        for program in allowed_programs {
            let path = crate::plugins::runtime::shared::safe_canonicalize_check(
                plugin_dir,
                &plugin_dir.join(&program),
            )?;
            normalized.insert(path);
        }

        Ok(normalized)
    }

    pub fn new(
        plugin_id: &str,
        plugin_dir: &Path,
        data_dir: &Path,
        server_dir: &Path,
        global_dir: &Path,
        api_registry: Arc<
            Mutex<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
        >,
        permissions: Vec<String>,
        allowed_programs: Vec<String>,
        server_event_subscriptions: HashMap<String, ServerEventSubscription>,
    ) -> Result<Self, String> {
        let lua = mlua::Lua::new_with(
            mlua::StdLib::TABLE
                | mlua::StdLib::STRING
                | mlua::StdLib::MATH
                | mlua::StdLib::UTF8
                | mlua::StdLib::COROUTINE,
            mlua::LuaOptions::default(),
        )
        .map_err(|e| core_t1("plugins.runtime.core.create_lua_instance_failed", e.to_string()))?;

        std::fs::create_dir_all(data_dir)
            .map_err(|e| core_t1("plugins.runtime.core.create_data_dir_failed", e.to_string()))?;

        let normalized_permissions = normalize_permissions(permissions);
        let normalized_allowed_programs =
            Self::normalize_declared_programs(plugin_dir, allowed_programs)?;

        let runtime = Self {
            lua,
            plugin_id: plugin_id.to_string(),
            plugin_dir: plugin_dir.to_path_buf(),
            data_dir: data_dir.to_path_buf(),
            server_dir: server_dir.to_path_buf(),
            global_dir: global_dir.to_path_buf(),
            loaded: std::sync::atomic::AtomicBool::new(false),
            permissions: normalized_permissions,
            allowed_programs: normalized_allowed_programs,
            api_registry,
            storage_lock: std::sync::Arc::new(std::sync::Mutex::new(())),
            process_registry: crate::plugins::runtime::process::new_process_registry(),
            server_event_subscriptions,
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
            .map_err(|e| core_t1("plugins.runtime.core.create_sl_table_failed", e.to_string()))?;

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

        if has_plugin_folder_access_permission(&self.permissions) {
            self.setup_plugins_namespace(&sl)?;
        } else {
            self.setup_permission_denied_namespace(
                &sl,
                "plugins",
                PLUGIN_FOLDER_ACCESS_PERMISSION,
            )?;
        }

        self.setup_i18n_namespace(&sl)?;

        globals
            .set("sl", sl)
            .map_err(|e| core_t1("plugins.runtime.core.set_sl_global_failed", e.to_string()))?;

        Ok(())
    }

    pub(super) fn globals_table(&self) -> Table {
        self.lua.globals()
    }
}
