use super::common::{
    check_fs_permission, copy_dir_recursive, emit_permission_log_api, get_base_dir_for_permission,
    validate_fs_path, FsContext,
};
use mlua::{Function, Lua};
use std::fs;

pub(super) fn copy(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (src, dst): (String, String)| {
        let (base_dir, perm) = get_base_dir_for_permission(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
        )?;
        check_fs_permission(&ctx.permissions, &perm)?;
        let src_path = validate_fs_path(&base_dir, &src)?;
        let dst_path = validate_fs_path(&base_dir, &dst)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.copy", &src);

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)
                .map_err(|e| mlua::Error::runtime(format!("Failed to copy directory: {}", e)))
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| mlua::Error::runtime(format!("Failed to copy file: {}", e)))
                .map(|_| ())
        }
    })
    .map_err(|e| format!("Failed to create fs.copy: {}", e))
}

pub(super) fn move_entry(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (src, dst): (String, String)| {
        let (base_dir, perm) = get_base_dir_for_permission(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
        )?;
        check_fs_permission(&ctx.permissions, &perm)?;
        let src_path = validate_fs_path(&base_dir, &src)?;
        let dst_path = validate_fs_path(&base_dir, &dst)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.move", &src);

        fs::rename(&src_path, &dst_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to move file/directory: {}", e)))
    })
    .map_err(|e| format!("Failed to create fs.move: {}", e))
}

pub(super) fn rename_entry(lua: &Lua, ctx: &FsContext) -> Result<Function, String> {
    let ctx = ctx.clone();
    lua.create_function(move |_, (old_path, new_path): (String, String)| {
        let (base_dir, perm) = get_base_dir_for_permission(
            &ctx.data_dir,
            &ctx.server_dir,
            &ctx.global_dir,
            &ctx.permissions,
        )?;
        check_fs_permission(&ctx.permissions, &perm)?;
        let old_full_path = validate_fs_path(&base_dir, &old_path)?;
        let new_full_path = validate_fs_path(&base_dir, &new_path)?;

        emit_permission_log_api(&ctx.plugin_id, "sl.fs.rename", &old_path);

        fs::rename(&old_full_path, &new_full_path)
            .map_err(|e| mlua::Error::runtime(format!("Failed to rename file/directory: {}", e)))
    })
    .map_err(|e| format!("Failed to create fs.rename: {}", e))
}
