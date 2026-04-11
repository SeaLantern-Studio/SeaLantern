use super::{
    common::{emit_plugins_log, resolve_plugin_path, PluginsContext},
    fs, MAX_FILE_SIZE,
};
use crate::utils::logger::log_warn;
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

        let full_path = resolve_plugin_path(&ctx.plugins_root, &target_id, &relative_path)?;
        if relative_path.eq_ignore_ascii_case("manifest.json") {
            return Err(mlua::Error::runtime("Writing manifest.json is not allowed"));
        }

        if full_path.exists() {
            let metadata = fs::symlink_metadata(&full_path).map_err(|e| {
                mlua::Error::runtime(format!("Failed to inspect destination: {}", e))
            })?;
            if metadata.file_type().is_symlink() {
                return Err(mlua::Error::runtime("Writing to symlink is not allowed"));
            }
            if metadata.is_dir() {
                return Err(mlua::Error::runtime("Cannot write to a directory path"));
            }
        }

        if let Some(parent) = full_path.parent() {
            if parent.exists() {
                let metadata = fs::symlink_metadata(parent).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to inspect parent directory: {}", e))
                })?;
                if metadata.file_type().is_symlink() {
                    return Err(mlua::Error::runtime(
                        "Writing through symlink directory is not allowed",
                    ));
                }
                if !metadata.is_dir() {
                    return Err(mlua::Error::runtime("Parent path is not a directory"));
                }
            } else {
                fs::create_dir_all(parent).map_err(|e| {
                    mlua::Error::runtime(format!("Failed to create directory: {}", e))
                })?;
            }
        }

        fs::write(&full_path, content)
            .map_err(|e| mlua::Error::runtime(format!("Failed to write file: {}", e)))?;

        log_warn(&format!(
            "[plugins.write_file] Plugin '{}' wrote file '{}' in plugin '{}'",
            ctx.plugin_id, relative_path, target_id
        ));

        Ok(true)
    })
    .map_err(|e| format!("Failed to create plugins.write_file: {}", e))
}
