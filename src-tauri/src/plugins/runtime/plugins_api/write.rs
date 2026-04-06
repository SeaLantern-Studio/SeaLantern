use super::{
    common::{emit_plugins_log, plugin_dir, PluginsContext},
    fs, validate_path_static, MAX_FILE_SIZE,
};
use mlua::Lua;

pub(super) fn write_file(lua: &Lua, ctx: &PluginsContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (target_id, relative_path, content): (String, String, String)| {
        emit_plugins_log(
            &ctx.plugin_id,
            "sl.plugins.write_file",
            &format!("{}/{}", target_id, relative_path),
        );

        if content.len() > MAX_FILE_SIZE as usize {
            return Err(mlua::Error::runtime("Content too large (max 10MB)"));
        }

        let target_dir = plugin_dir(&ctx.plugins_root, &target_id)?;
        if !target_dir.exists() {
            return Err(mlua::Error::runtime(format!("Plugin directory not found: {}", target_id)));
        }

        let full_path = validate_path_static(&target_dir, &relative_path)?;
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| mlua::Error::runtime(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(&full_path, content)
            .map_err(|e| mlua::Error::runtime(format!("Failed to write file: {}", e)))?;

        Ok(true)
    })
    .map_err(|e| format!("Failed to create plugins.write_file: {}", e))
}
