use super::{
    common::{
        emit_plugins_log, plugin_dir, read_manifest_json, resolve_plugin_path, PluginsContext,
    },
    fs, MAX_FILE_SIZE,
};
use crate::utils::logger::log_warn;
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
                Err(e) => {
                    log_warn(&format!(
                        "[plugins.list] Failed to read plugin entry in '{}': {}",
                        ctx.plugins_root.display(),
                        e
                    ));
                    continue;
                }
            };
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest = match read_manifest_json(&path, Some("plugins.list"))? {
                Some(manifest) => manifest,
                None => continue,
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
            if let Some(description) = manifest.get("description").and_then(|v| v.as_str()) {
                item.set("description", description.to_string())?;
            }
            if let Some(author) = manifest.get("author").and_then(|v| v.as_str()) {
                item.set("author", author.to_string())?;
            }
            if let Some(status) = manifest.get("status").and_then(|v| v.as_str()) {
                item.set("status", status.to_string())?;
            }
            item.set("installed", true)?;
            item.set("has_manifest", true)?;

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
        let Some(manifest) = read_manifest_json(&target_dir, None)? else {
            return Ok(Value::Nil);
        };

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

        let full_path = match resolve_plugin_path(&ctx.plugins_root, &target_id, &relative_path) {
            Ok(path) => path,
            Err(_) => return Ok(None),
        };
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

        match resolve_plugin_path(&ctx.plugins_root, &target_id, &relative_path) {
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

        let full_path = resolve_plugin_path(&ctx.plugins_root, &target_id, &relative_path)?;
        let entries = fs::read_dir(&full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to read directory: {}", e)))?;

        let mut items = Vec::new();
        for entry in entries.flatten() {
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = metadata.is_dir();
            let size = metadata.len();
            let modified_at = metadata
                .modified()
                .ok()
                .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs());

            items.push((name, is_dir, size, modified_at));
        }

        items.sort_by(|a, b| {
            a.1.cmp(&b.1)
                .reverse()
                .then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase()))
                .then_with(|| a.0.cmp(&b.0))
        });

        let table = lua.create_table()?;
        for (i, (name, is_dir, size, modified_at)) in items.into_iter().enumerate() {
            let item = lua.create_table()?;
            item.set("name", name)?;
            item.set("is_dir", is_dir)?;
            item.set("size", size)?;
            if let Some(modified_at) = modified_at {
                item.set("modified_at", modified_at)?;
            }

            table.set(i + 1, item)?;
        }
        Ok(table)
    })
    .map_err(|e| format!("Failed to create plugins.list_files: {}", e))
}
