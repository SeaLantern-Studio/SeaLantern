use super::PluginRuntime;
use mlua::Table;

impl PluginRuntime {
    pub(super) fn setup_system_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::plugins::api::emit_permission_log;
        use sysinfo::System;

        let system_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create system table: {}", e))?;

        let plugin_id = self.plugin_id.clone();

        let pid = plugin_id.clone();
        let get_os_fn = self
            .lua
            .create_function(move |_, ()| {
                let _ = emit_permission_log(&pid, "api_call", "sl.system.get_os", "");
                let os = std::env::consts::OS;
                Ok(os.to_string())
            })
            .map_err(|e| format!("Failed to create system.get_os: {}", e))?;
        system_table
            .set("get_os", get_os_fn)
            .map_err(|e| format!("Failed to set system.get_os: {}", e))?;

        let pid = plugin_id.clone();
        let get_arch_fn = self
            .lua
            .create_function(move |_, ()| {
                let _ = emit_permission_log(&pid, "api_call", "sl.system.get_arch", "");
                Ok(std::env::consts::ARCH.to_string())
            })
            .map_err(|e| format!("Failed to create system.get_arch: {}", e))?;
        system_table
            .set("get_arch", get_arch_fn)
            .map_err(|e| format!("Failed to set system.get_arch: {}", e))?;

        let pid = plugin_id.clone();
        let get_app_version_fn = self
            .lua
            .create_function(move |_, ()| {
                let _ = emit_permission_log(&pid, "api_call", "sl.system.get_app_version", "");
                Ok(env!("CARGO_PKG_VERSION").to_string())
            })
            .map_err(|e| format!("Failed to create system.get_app_version: {}", e))?;
        system_table
            .set("get_app_version", get_app_version_fn)
            .map_err(|e| format!("Failed to set system.get_app_version: {}", e))?;

        let pid = plugin_id.clone();
        let get_memory_fn = self
            .lua
            .create_function(move |lua, ()| {
                let _ = emit_permission_log(&pid, "api_call", "sl.system.get_memory", "");
                let mut sys = System::new();
                sys.refresh_memory();

                let total = sys.total_memory();
                let used = sys.used_memory();
                let free = total.saturating_sub(used);

                let mem_table = lua.create_table()?;
                mem_table.set("total", total)?;
                mem_table.set("used", used)?;
                mem_table.set("free", free)?;
                Ok(mem_table)
            })
            .map_err(|e| format!("Failed to create system.get_memory: {}", e))?;
        system_table
            .set("get_memory", get_memory_fn)
            .map_err(|e| format!("Failed to set system.get_memory: {}", e))?;

        let pid = plugin_id.clone();
        let get_cpu_fn = self
            .lua
            .create_function(move |lua, ()| {
                let _ = emit_permission_log(&pid, "api_call", "sl.system.get_cpu", "");
                let mut sys = System::new();
                sys.refresh_cpu_all();

                std::thread::sleep(std::time::Duration::from_millis(100));
                sys.refresh_cpu_all();

                let cpus = sys.cpus();
                let cpu_name = cpus
                    .first()
                    .map(|c| c.brand().to_string())
                    .unwrap_or_default();
                let cores = cpus.len() as u64;
                let usage: f64 = if cores > 0 {
                    cpus.iter().map(|c| c.cpu_usage() as f64).sum::<f64>() / cores as f64
                } else {
                    0.0
                };

                let usage = (usage * 100.0).round() / 100.0;

                let cpu_table = lua.create_table()?;
                cpu_table.set("name", cpu_name)?;
                cpu_table.set("cores", cores)?;
                cpu_table.set("usage", usage)?;
                Ok(cpu_table)
            })
            .map_err(|e| format!("Failed to create system.get_cpu: {}", e))?;
        system_table
            .set("get_cpu", get_cpu_fn)
            .map_err(|e| format!("Failed to set system.get_cpu: {}", e))?;

        sl.set("system", system_table)
            .map_err(|e| format!("Failed to set sl.system: {}", e))?;

        Ok(())
    }
}
