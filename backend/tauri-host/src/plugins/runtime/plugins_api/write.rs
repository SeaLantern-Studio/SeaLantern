use super::{
    common::{
        emit_plugins_log, plugins_t, plugins_t1, plugins_t2, resolve_plugin_path, PluginsContext,
    },
    fs, MAX_FILE_SIZE,
};
use crate::hardcode_data::app_files::PLUGIN_MANIFEST_FILE_NAME;
use crate::hardcode_data::plugin_manifest::writing_manifest_not_allowed_message;
use crate::plugins::runtime::permissions::{
    PLUGINS_WRITE_PERMISSION, PLUGIN_FOLDER_ACCESS_PERMISSION,
};
use crate::utils::logger::log_warn_ctx;
use mlua::Lua;

fn require_plugins_write(ctx: &PluginsContext) -> mlua::Result<()> {
    if ctx.has_any_permission(&[PLUGINS_WRITE_PERMISSION, PLUGIN_FOLDER_ACCESS_PERMISSION]) {
        Ok(())
    } else {
        Err(mlua::Error::runtime(plugins_t1(
            "plugins.runtime.permissions.permission_required",
            format!("{} | {}", PLUGINS_WRITE_PERMISSION, PLUGIN_FOLDER_ACCESS_PERMISSION),
        )))
    }
}

pub(super) fn write_file(lua: &Lua, ctx: &PluginsContext) -> Result<mlua::Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (target_id, relative_path, content): (String, String, String)| {
        require_plugins_write(&ctx)?;
        emit_plugins_log(
            &ctx.plugin_id,
            "sl.plugins.write_file",
            &format!("{}/{}", target_id, relative_path),
        );

        if content.len() > MAX_FILE_SIZE as usize {
            return Err(mlua::Error::runtime(plugins_t(
                "plugins.runtime.plugins_api.content_too_large",
            )));
        }

        let full_path = resolve_plugin_path(&ctx.plugins_root, &target_id, &relative_path)?;
        if relative_path.eq_ignore_ascii_case(PLUGIN_MANIFEST_FILE_NAME) {
            return Err(mlua::Error::runtime(writing_manifest_not_allowed_message()));
        }

        if full_path.exists() {
            let metadata = fs::symlink_metadata(&full_path).map_err(|e| {
                mlua::Error::runtime(plugins_t1(
                    "plugins.runtime.plugins_api.inspect_destination_failed",
                    e.to_string(),
                ))
            })?;
            if metadata.file_type().is_symlink() {
                return Err(mlua::Error::runtime(plugins_t(
                    "plugins.runtime.plugins_api.write_symlink_forbidden",
                )));
            }
            if metadata.is_dir() {
                return Err(mlua::Error::runtime(plugins_t(
                    "plugins.runtime.plugins_api.write_directory_path_forbidden",
                )));
            }
        }

        if let Some(parent) = full_path.parent() {
            if parent.exists() {
                let metadata = fs::symlink_metadata(parent).map_err(|e| {
                    mlua::Error::runtime(plugins_t1(
                        "plugins.runtime.plugins_api.inspect_parent_dir_failed",
                        e.to_string(),
                    ))
                })?;
                if metadata.file_type().is_symlink() {
                    return Err(mlua::Error::runtime(plugins_t(
                        "plugins.runtime.plugins_api.write_parent_symlink_forbidden",
                    )));
                }
                if !metadata.is_dir() {
                    return Err(mlua::Error::runtime(plugins_t(
                        "plugins.runtime.plugins_api.write_parent_not_dir",
                    )));
                }
            } else {
                fs::create_dir_all(parent).map_err(|e| {
                    mlua::Error::runtime(plugins_t1(
                        "plugins.runtime.plugins_api.create_directory_failed",
                        e.to_string(),
                    ))
                })?;
            }
        }

        fs::write(&full_path, content).map_err(|e| {
            mlua::Error::runtime(plugins_t1(
                "plugins.runtime.plugins_api.write_file_failed",
                e.to_string(),
            ))
        })?;

        log_warn_ctx(
            "plugins.runtime.plugins_api.write",
            "write_file",
            &format!(
                "plugin '{}' wrote file '{}' in plugin '{}'",
                ctx.plugin_id, relative_path, target_id
            ),
        );

        Ok(true)
    })
    .map_err(|e| {
        plugins_t2(
            "plugins.runtime.plugins_api.create_api_failed",
            "plugins.write_file",
            e.to_string(),
        )
    })
}
