use super::{
    common::{emit_plugins_log, plugin_dir, PluginsContext},
    fs, validate_path_static, MAX_FILE_SIZE,
};
use mlua::{Lua, Value};

pub(super) fn list(lua: &Lua, ctx: &PluginsContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, ()| {
        emit_plugins_log(&ctx.plugin_id, "sl.plugins.list", "");

        let result = lua.create_table()?;
        let mut i = 1;

        let entries = fs::read_dir(&ctx.plugins_root).map_err(|e| {
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
    .map_err(|e| format!("Failed to create plugins.list: {}", e))
}

pub(super) fn get_manifest(lua: &Lua, ctx: &PluginsContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, target_id: String| {
        emit_plugins_log(&ctx.plugin_id, "sl.plugins.get_manifest", &target_id);

        let target_dir = plugin_dir(&ctx.plugins_root, &target_id)?;
        let manifest_path = target_dir.join("manifest.json");

        if !manifest_path.exists() {
            return Ok(Value::Nil);
        }

        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to read manifest: {}", e)))?;

        let manifest: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| mlua::Error::runtime(format!("Failed to parse manifest: {}", e)))?;

        crate::plugins::runtime::shared::lua_value_from_json(lua, &manifest, 0)
    })
    .map_err(|e| format!("Failed to create plugins.get_manifest: {}", e))
}

pub(super) fn read_file(lua: &Lua, ctx: &PluginsContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (target_id, relative_path): (String, String)| {
        emit_plugins_log(
            &ctx.plugin_id,
            "sl.plugins.read_file",
            &format!("{}/{}", target_id, relative_path),
        );

        let target_dir = plugin_dir(&ctx.plugins_root, &target_id)?;
        if !target_dir.exists() {
            return Ok(None);
        }

        let full_path = validate_path_static(&target_dir, &relative_path)?;
        if !full_path.exists() || full_path.is_dir() {
            return Ok(None);
        }

        let metadata = fs::metadata(&full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to get file metadata: {}", e)))?;
        if metadata.len() > MAX_FILE_SIZE {
            return Err(mlua::Error::runtime("File too large (max 10MB)"));
        }

        match fs::read_to_string(&full_path) {
            Ok(content) => Ok(Some(content)),
            Err(e) => Err(mlua::Error::runtime(format!("Failed to read file: {}", e))),
        }
    })
    .map_err(|e| format!("Failed to create plugins.read_file: {}", e))
}

pub(super) fn file_exists(lua: &Lua, ctx: &PluginsContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (target_id, relative_path): (String, String)| {
        emit_plugins_log(
            &ctx.plugin_id,
            "sl.plugins.file_exists",
            &format!("{}/{}", target_id, relative_path),
        );

        let target_dir = ctx.plugins_root.join(&target_id);
        if !target_dir.exists() {
            return Ok(false);
        }

        match validate_path_static(&target_dir, &relative_path) {
            Ok(full_path) => Ok(full_path.exists()),
            Err(_) => Ok(false),
        }
    })
    .map_err(|e| format!("Failed to create plugins.file_exists: {}", e))
}

pub(super) fn list_files(lua: &Lua, ctx: &PluginsContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |lua, (target_id, relative_path): (String, String)| {
        emit_plugins_log(
            &ctx.plugin_id,
            "sl.plugins.list_files",
            &format!("{}/{}", target_id, relative_path),
        );

        let target_dir = ctx.plugins_root.join(&target_id);
        if !target_dir.exists() {
            return Err(mlua::Error::runtime(format!("Plugin directory not found: {}", target_id)));
        }

        let full_path = validate_path_static(&target_dir, &relative_path)?;
        let entries = fs::read_dir(&full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to read directory: {}", e)))?;

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
    .map_err(|e| format!("Failed to create plugins.list_files: {}", e))
}
