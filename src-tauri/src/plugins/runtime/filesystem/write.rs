use super::common::{
    check_fs_permission, emit_permission_log_api, get_base_dir_for_permission, validate_fs_path,
    FsContext,
};
use mlua::{Function, Lua};
use std::fs;

pub(super) fn write(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (path, content): (String, String)| {
        let (base_dir, perm) = get_base_dir_for_permission(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
        )?;
        check_fs_permission(&ctx.permissions, &perm)?;
        let full_path = validate_fs_path(&base_dir, &path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.write", &path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| mlua::Error::runtime(format!("Failed to create directory: {}", e)))?;
        }

        fs::write(&full_path, content)
            .map_err(|e| mlua::Error::runtime(format!("Failed to write file: {}", e)))
    })
    .map_err(|e| format!("Failed to create fs.write: {}", e))
}

pub(super) fn mkdir(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, path: String| {
        let (base_dir, perm) = get_base_dir_for_permission(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
        )?;
        check_fs_permission(&ctx.permissions, &perm)?;
        let full_path = validate_fs_path(&base_dir, &path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.mkdir", &path);

        fs::create_dir_all(&full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to create directory: {}", e)))
    })
    .map_err(|e| format!("Failed to create fs.mkdir: {}", e))
}

pub(super) fn remove(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, path: String| {
        let (base_dir, perm) = get_base_dir_for_permission(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
        )?;
        check_fs_permission(&ctx.permissions, &perm)?;
        let full_path = validate_fs_path(&base_dir, &path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.remove", &path);

        if full_path.is_dir() {
            fs::remove_dir_all(&full_path)
                .map_err(|e| mlua::Error::runtime(format!("Failed to remove directory: {}", e)))
        } else {
            fs::remove_file(&full_path)
                .map_err(|e| mlua::Error::runtime(format!("Failed to remove file: {}", e)))
        }
    })
    .map_err(|e| format!("Failed to create fs.remove: {}", e))
}
