use super::helpers::validate_path_static;
use super::PluginRuntime;
use mlua::Table;
use std::fs;

impl PluginRuntime {
    pub(super) fn setup_plugins_namespace(&self, sl: &Table) -> Result<(), String> {
        use crate::plugins::api::emit_permission_log;

        let plugins_table = self
            .lua
            .create_table()
            .map_err(|e| format!("Failed to create plugins table: {}", e))?;

        let plugin_dir = self.plugin_dir.clone();
        let plugin_id = self.plugin_id.clone();

        let plugins_root = plugin_dir
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| plugin_dir.clone());

        const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

        let root = plugins_root.clone();
        let pid = plugin_id.clone();
        let list_fn = self
            .lua
            .create_function(move |lua, ()| {
                let _ = emit_permission_log(&pid, "api_call", "sl.plugins.list", "");

                let result = lua.create_table()?;
                let mut i = 1;

                let entries = fs::read_dir(&root).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to read plugins directory: {}", e))
                })?;

                for entry in entries {
                    let entry = match entry {
                        Ok(e) => e,
                        Err(_) => continue,
                    };
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    let manifest_path = path.join("manifest.json");
                    if !manifest_path.exists() {
                        continue;
                    }

                    let content = match fs::read_to_string(&manifest_path) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    let manifest: serde_json::Value = match serde_json::from_str(&content) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    let item = lua.create_table()?;
                    if let Some(id) = manifest.get("id").and_then(|v| v.as_str()) {
                        item.set("id", id.to_string())?;
                    }
                    if let Some(name) = manifest.get("name").and_then(|v| v.as_str()) {
                        item.set("name", name.to_string())?;
                    }
                    if let Some(version) = manifest.get("version").and_then(|v| v.as_str()) {
                        item.set("version", version.to_string())?;
                    }
                    item.set("installed", true)?;

                    result.set(i, item)?;
                    i += 1;
                }

                Ok(result)
            })
            .map_err(|e| format!("Failed to create plugins.list: {}", e))?;
        plugins_table
            .set("list", list_fn)
            .map_err(|e| format!("Failed to set plugins.list: {}", e))?;

        let root = plugins_root.clone();
        let pid = plugin_id.clone();
        let get_manifest_fn = self
            .lua
            .create_function(move |lua, target_id: String| {
                let _ =
                    emit_permission_log(&pid, "api_call", "sl.plugins.get_manifest", &target_id);

                let _ = validate_path_static(&root, &target_id)?;
                let target_dir = root.join(&target_id);
                let manifest_path = target_dir.join("manifest.json");

                if !manifest_path.exists() {
                    return Ok(mlua::Value::Nil);
                }

                let content = fs::read_to_string(&manifest_path)
                    .map_err(|e| mlua::Error::runtime(format!("Failed to read manifest: {}", e)))?;

                let manifest: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to parse manifest: {}", e))
                })?;

                super::helpers::lua_value_from_json(lua, &manifest, 0)
            })
            .map_err(|e| format!("Failed to create plugins.get_manifest: {}", e))?;
        plugins_table
            .set("get_manifest", get_manifest_fn)
            .map_err(|e| format!("Failed to set plugins.get_manifest: {}", e))?;

        let root = plugins_root.clone();
        let pid = plugin_id.clone();
        let read_file_fn = self
            .lua
            .create_function(move |_, (target_id, relative_path): (String, String)| {
                let _ = emit_permission_log(
                    &pid,
                    "api_call",
                    "sl.plugins.read_file",
                    &format!("{}/{}", target_id, relative_path),
                );

                let _ = validate_path_static(&root, &target_id)?;
                let target_dir = root.join(&target_id);
                if !target_dir.exists() {
                    return Ok(None);
                }

                let full_path = validate_path_static(&target_dir, &relative_path)?;

                if !full_path.exists() || full_path.is_dir() {
                    return Ok(None);
                }

                let metadata = fs::metadata(&full_path).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to get file metadata: {}", e))
                })?;
                if metadata.len() > MAX_FILE_SIZE {
                    return Err(mlua::Error::runtime("File too large (max 10MB)"));
                }

                match fs::read_to_string(&full_path) {
                    Ok(content) => Ok(Some(content)),
                    Err(e) => Err(mlua::Error::runtime(format!("Failed to read file: {}", e))),
                }
            })
            .map_err(|e| format!("Failed to create plugins.read_file: {}", e))?;
        plugins_table
            .set("read_file", read_file_fn)
            .map_err(|e| format!("Failed to set plugins.read_file: {}", e))?;

        let root = plugins_root.clone();
        let pid = plugin_id.clone();
        let write_file_fn = self
            .lua
            .create_function(
                move |_, (target_id, relative_path, content): (String, String, String)| {
                    let _ = emit_permission_log(
                        &pid,
                        "api_call",
                        "sl.plugins.write_file",
                        &format!("{}/{}", target_id, relative_path),
                    );

                    if content.len() > MAX_FILE_SIZE as usize {
                        return Err(mlua::Error::runtime("Content too large (max 10MB)"));
                    }

                    let _ = validate_path_static(&root, &target_id)?;
                    let target_dir = root.join(&target_id);
                    if !target_dir.exists() {
                        return Err(mlua::Error::runtime(format!(
                            "Plugin directory not found: {}",
                            target_id
                        )));
                    }

                    let full_path = validate_path_static(&target_dir, &relative_path)?;

                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            mlua::Error::runtime(format!("Failed to create directory: {}", e))
                        })?;
                    }

                    fs::write(&full_path, content).map_err(|e| {
                        mlua::Error::runtime(format!("Failed to write file: {}", e))
                    })?;

                    Ok(true)
                },
            )
            .map_err(|e| format!("Failed to create plugins.write_file: {}", e))?;
        plugins_table
            .set("write_file", write_file_fn)
            .map_err(|e| format!("Failed to set plugins.write_file: {}", e))?;

        let root = plugins_root.clone();
        let pid = plugin_id.clone();
        let file_exists_fn = self
            .lua
            .create_function(move |_, (target_id, relative_path): (String, String)| {
                let _ = emit_permission_log(
                    &pid,
                    "api_call",
                    "sl.plugins.file_exists",
                    &format!("{}/{}", target_id, relative_path),
                );

                let target_dir = root.join(&target_id);
                if !target_dir.exists() {
                    return Ok(false);
                }

                match validate_path_static(&target_dir, &relative_path) {
                    Ok(full_path) => Ok(full_path.exists()),
                    Err(_) => Ok(false),
                }
            })
            .map_err(|e| format!("Failed to create plugins.file_exists: {}", e))?;
        plugins_table
            .set("file_exists", file_exists_fn)
            .map_err(|e| format!("Failed to set plugins.file_exists: {}", e))?;

        let root = plugins_root.clone();
        let pid = plugin_id.clone();
        let list_files_fn = self
            .lua
            .create_function(move |lua, (target_id, relative_path): (String, String)| {
                let _ = emit_permission_log(
                    &pid,
                    "api_call",
                    "sl.plugins.list_files",
                    &format!("{}/{}", target_id, relative_path),
                );

                let target_dir = root.join(&target_id);
                if !target_dir.exists() {
                    return Err(mlua::Error::runtime(format!(
                        "Plugin directory not found: {}",
                        target_id
                    )));
                }

                let full_path = validate_path_static(&target_dir, &relative_path)?;

                let entries = fs::read_dir(&full_path).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to read directory: {}", e))
                })?;

                let table = lua.create_table()?;
                let mut i = 1;
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        table.set(i, name.to_string())?;
                        i += 1;
                    }
                }
                Ok(table)
            })
            .map_err(|e| format!("Failed to create plugins.list_files: {}", e))?;
        plugins_table
            .set("list_files", list_files_fn)
            .map_err(|e| format!("Failed to set plugins.list_files: {}", e))?;

        sl.set("plugins", plugins_table)
            .map_err(|e| format!("Failed to set sl.plugins: {}", e))?;

        Ok(())
    }
}
